# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在本仓库中工作时提供指导。

## 项目概述

terminal-serial 是一个用 Rust 编写的跨平台终端串口通信工具。它连接串口（Windows 上为 COM，macOS/Linux 上为 /dev/tty*）并提供交互式终端会话。按 `Ctrl + ]` 退出会话。

## 构建与运行命令

```shell
cargo build                    # 调试构建
cargo build --release          # 发布构建（启用 LTO + strip）
cargo run                      # 以调试模式运行
cargo run -- -l                # 列出可用串口
cargo run -- -p COM3 -b 115200 # 连接指定串口
cargo run -- log event.jsonl   # 查看事件日志
cargo install --path .         # 本地安装
```

测试：`cargo test`

## 架构

二进制入口是一个薄包装层（`src/main.rs`），解析命令行参数后根据子命令分发：`Connect` 委托给 `TerminalSerial::run()`，`Log` 委托给 `log_reader::run()`。

**模块职责：**

- **`cmd.rs`** — 通过 `clap`（derive 模式）解析命令行参数。返回 `Command` 枚举（`Connect(AppConfig)` 或 `Log(LogConfig)`）。`OutputFormat` 使用 `ValueEnum` derive 确保 CLI 只接受有效格式（text/json/md）。串口枚举因平台而异：Windows 从注册表读取，macOS 过滤 `/dev/cu.*`（跳过蓝牙），Linux 扫描 `/dev/ttyUSB*`。未指定 `-p` 时，提示用户从枚举列表中选择。

- **`lib.rs`** — 纯模块声明汇总，聚合所有子模块（`cmd`、`error`、`event_log`、`getch`、`input`、`keyboard`、`log_reader`、`mcp`、`mcp_http`、`serial_manager`、`task`、`telnet`、`util`）。`Input`/`InputMessage` 已迁移至 `input.rs`，hex 工具函数已迁移至 `util.rs`。

- **`keyboard.rs`** — 使用树结构进行平台相关的键码转换。Windows 使用 `0xe0` 前缀扫描码；Unix 使用 `0x1b`（ESC）前缀序列。`get_input()` 返回 `Option<Key<KeyInfo>>`，整个键盘输入链路通过 `Option` 传播错误而非 panic。将方向键、导航键和功能键映射为 ANSI 转义序列（`dst_value`），用于通过串口发送。

- **`getch.rs`** — 原始单字节键盘输入。Windows 使用 libc 的 `_getch()`。Unix 通过 `termios` 将终端设为 raw 模式（非规范模式、无回显、无信号），在 `Drop` 时恢复原始设置。`getch()` 返回 `Result<u8>`，不会 panic。

- **`input.rs`** — 定义 `Input` 和 `InputMessage`。`Input` 封装 `Keyboard`，将原始按键事件映射为 `InputMessage` 变体（`Quit`、`Data(Vec<u8>)`、`ClearBuffer`）。`get_message()` 返回 `Option<InputMessage>`，stdin 关闭时返回 `None`。`Ctrl + ]`（字节 `0x1d`）触发 `Quit`，`Ctrl + K`（字节 `0x0b`）触发 `ClearBuffer`；方向键、导航键等映射为 `Data`，承载来自 `keyboard` 模块的 ANSI 转义序列（`dst_value`）。

- **`serial_manager.rs`** — 串口共享管理器（`SerialManager`）与协议抽象（`SerialOps` trait）。`SerialManager` 封装串口句柄、MCP 读取缓冲区（`Arc<(Mutex<VecDeque<u8>>, Condvar)>`，上限 64KB）和退出标志（`Arc<AtomicBool>`）。`SerialOps` trait 定义协议层依赖的串口操作接口（`send`、`drain_buffer`、`clear_buffer`、`grep_buffer`、`grep_buffer_bytes`、`status`），`SerialManager` 实现该 trait，并额外提供 `peek_buffer`、`port`、`read_buffer`、`quit_flag` 等方法。错误通过 `SerialError`（thiserror）类型传播。`mcp.rs` 通过泛型 `<S: SerialOps>` 依赖此 trait，实现协议层与硬件层的解耦，便于用 mock 替换硬件后端进行单元测试。

- **`task.rs`** — 核心运行时（`TerminalSerial`）。通过 `SerialManager` 打开并配置串口，然后启动线程：输入线程（键盘→串口）、读取线程（串口→终端+MCP 缓冲区+Telnet 广播）。MCP/Telnet 服务器绑定失败时优雅退出（返回 `eprintln + return`，不 `process::exit`）。Event log 打开失败时降级为不记录日志。

- **`mcp.rs`** — MCP 协议处理（与传输层解耦的纯函数层）。实现 JSON-RPC 2.0 的 `initialize`、`tools/list`、`tools/call`、`prompts/list`、`prompts/get` 方法。暴露五个工具：`serial_send`（发送数据，可选等待响应）、`serial_read`（读取缓冲区数据）、`serial_status`（连接状态）、`serial_grep`（非破坏性正则搜索）、`serial_clear`（清空缓冲区），以及一个 prompt `serial_usage_guide`（使用指南）。`handle_request` 及各 `tool_*` 函数均以泛型 `<S: SerialOps>` 依赖串口抽象，便于用 mock 替换硬件后端进行单元测试。hex 解析复用 `util` 模块的 `hex_decode`。

- **`mcp_http.rs`** — MCP 的 HTTP 传输层（`McpHttpServer`）。基于 `TcpListener` 处理 `POST /mcp` 端点，将请求转发给 `mcp.rs` 处理。连接设置 30s 读写超时防止挂起。`start()` 返回 `Result<(), String>`，绑定失败向上传播。绑定地址和端口通过 `--mcp-host`（默认 `0.0.0.0`）和 `--mcp-port`（默认 `8765`）配置。

- **`telnet.rs`** — Telnet 服务器（`TelnetServer`）。支持多客户端连接、telnet 协议协商（IAC/WILL/WONT/DO/DONT）、串口数据广播。客户端输入经过 NUL 字节剥离和 telnet 命令过滤后写入串口。`start()` 返回 `Result<(), String>`。

- **`event_log.rs`** — 事件日志写入器（`EventLogWriter`），线程安全（`Mutex<File>`）。记录 startup/shutdown/tx/rx/error/client_connected/client_disconnected 事件为 JSONL 格式。tx/rx 事件的字节数据通过 `util` 模块的 `hex_encode` 编码为 hex 字符串。

- **`log_reader.rs`** — 日志查看器。解析 JSONL 事件日志，支持按事件类型、来源、正则内容过滤，输出格式 text/json/md，会话摘要统计。

- **`util.rs`** — 通用工具函数。提供 `hex_encode`（字节切片→大写 hex 字符串，使用 `{:02X}`）和 `hex_decode`（hex 字符串→字节切片，奇数长度返回 `None`）。被 `event_log.rs`、`log_reader.rs`、`mcp.rs` 复用。

- **`error.rs`** — 定义 `SerialError` 枚举（thiserror）：`PortOpen`、`Write`、`Read`、`Regex`。

## 金丝雀规则

**语言要求**：所有生成的文档（包括 openspec 规范开发流程生成的文档）、向用户提出的问题、代码注释，都必须使用中文。仅在确实必要时才使用英文，例如：代码标识符、命令行参数、技术专有名词等本身为英文的内容。

## 关键设计细节

- **线程模型**：输入线程（键盘→串口）和读取线程（串口→终端+MCP 缓冲区+Telnet 广播），`-M` 模式下额外启动 MCP HTTP 服务器线程，`-T` 模式下额外启动 Telnet 服务器线程（接受线程+广播线程+每客户端读取线程）。通过 `SerialManager` 统一管理共享状态。
- **退出机制**：退出标志使用 `Arc<AtomicBool>`，输入线程设置标志后退出，读取线程和服务线程通过 `load(Ordering::Relaxed)` 检查退出。
- **MCP 集成**：`-M` 启用 MCP 服务器。Claude Code 通过 HTTP MCP 协议（`http://{mcp-host}:{mcp-port}/mcp`）接入。工具暴露为 `serial_send`、`serial_read`、`serial_status`、`serial_grep`、`serial_clear`。
- **Telnet 集成**：`-T` 启用 Telnet 服务器。支持多客户端同时连接，串口数据通过 `mpsc` 通道广播到所有客户端。
- **快捷键**：`Ctrl + ]` 退出，`Ctrl + K` 清空 MCP 读缓冲区（`-M` 模式下）。
- **编码处理**：`#[cfg(windows)]` 下输入字节先以 GBK 解码再编码为 UTF-8 发送到串口；非 Windows 平台直接透传 UTF-8。`encoding_rs` 仅作为 Windows 条件依赖。
- **串口超时**：设置为 1ms；读取线程每 10ms 轮询一次。
- **命令行配置**：使用 clap derive 模式（`Parser`、`Subcommand`、`ValueEnum`）。
- **平台条件编译**：大量使用 `#[cfg(windows)]` / `#[cfg(not(windows))]`。Windows 特有依赖（`libc`、`encoding_rs`）；Unix 特有依赖（`termios`）。macOS 与 Linux 的串口发现使用嵌套的 `#[cfg(target_os)]`。
- **错误处理**：库代码使用 `SerialError`（thiserror）类型，不使用 `process::exit`。`process::exit` 仅出现在 CLI 入口层（`cmd.rs`、`main.rs`）。
- **事件日志**：`--event-log <path>` 启用 JSONL 格式的事件记录。`log` 子命令用于查看、过滤和分析日志。

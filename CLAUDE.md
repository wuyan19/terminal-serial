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

- **`cmd.rs`** — 通过 `clap`（derive 模式）解析命令行参数。返回 `Command` 枚举（`Connect(AppConfig)` 或 `Log(LogConfig)`）。`OutputFormat` 使用 `ValueEnum` derive 确保 CLI 只接受有效格式（text/json/md）。串口枚举因平台而异：Windows 从注册表读取，macOS 过滤 `/dev/cu.*`（跳过蓝牙），Linux 扫描 `/dev/ttyUSB*`。未指定 `-p` 时，提示用户从枚举列表中选择。`--config <path>` 加载统一工具配置文件（宏定义等）。

- **`lib.rs`** — 纯模块声明汇总，聚合所有子模块（`cmd`、`config`、`error`、`event_log`、`getch`、`input`、`keyboard`、`log_reader`、`macro_runner`、`mcp`、`mcp_http`、`serial_manager`、`task`、`telnet`、`util`）。`Input`/`InputMessage` 已迁移至 `input.rs`，hex 工具函数已迁移至 `util.rs`。

- **`config.rs`** — terminal-serial 的统一工具配置（`Config`）。本版本仅填充 `macros` 字段（`BTreeMap<String, Macro>`，用 BTreeMap 保证 Alt+数字索引顺序稳定）。顶层字段全部 `#[serde(default)]`，未来扩展（默认串口参数、日志策略、快捷键自定义等）新增字段时对老配置透明。`Macro` 包含可选 `description` 与 `steps` 序列。`MacroStep` 使用 `#[serde(tag = "type", rename_all = "lowercase")]` 内部标签枚举，支持 `send`/`delay`/`expect`/`clear` 四种步骤。`Config::load(path)` 读文件 + serde 反序列化，`Config::empty()` 为无 `--config` 时的默认值，`macro_at_index(idx)` 按 BTreeMap 顺序返回第 N 个宏用于 Alt+数字索引。

- **`macro_runner.rs`** — 宏执行引擎。`run_macro<S: SerialOps>(name, mac, manager, event_log, source)` 按序执行宏的所有步骤：`Send` 通过 `SerialOps::send` 发送数据（支持 text/hex/raw 三种 `format`，`auto_newline` 默认 true 追加 `\r\n`），`Delay` 通过 `thread::sleep` 阻塞，`Expect` 调用 `SerialOps::grep_buffer` 等待设备返回（超时返回错误），`Clear` 调用 `SerialOps::clear_buffer`。事件日志只在整体调用前后各记录一次 `run_macro` action（不细到每步，避免噪音）。复用 `util::hex_decode` 解析 hex 数据。包含 MockSerial（`RefCell<Vec<Vec<u8>>>` 记录 send 调用）用于单元测试。

- **`keyboard.rs`** — 使用树结构进行平台相关的键码转换。Windows 使用 `0xe0` 前缀扫描码；Unix 使用 `0x1b`（ESC）前缀序列。`get_input()` 返回 `Option<Key<KeyInfo>>`，整个键盘输入链路通过 `Option` 传播错误而非 panic。将方向键、导航键和功能键映射为 ANSI 转义序列（`dst_value`），用于通过串口发送。

- **`getch.rs`** — 原始单字节键盘输入。Windows 使用 libc 的 `_getch()`。Unix 通过 `termios` 将终端设为 raw 模式（非规范模式、无回显、无信号），在 `Drop` 时恢复原始设置。`getch()` 返回 `Result<u8>`，不会 panic。

- **`input.rs`** — 定义 `Input` 和 `InputMessage`。`Input` 封装 `Keyboard`，内部维护两状态机（`Normal` / `MenuSelect`）。`InputMessage` 变体：`Quit`（`Ctrl + ]`，字节 `0x1d`）、`ClearBuffer`（`Ctrl + K`，字节 `0x0b`）、`ShowMacroMenu`（`Ctrl + O`，字节 `0x0f`，切换到 MenuSelect）、`RunMacro(usize)`（MenuSelect 模式下数字键 `1`-`9`，1-indexed）、`Data(Vec<u8>)`（普通数据，含方向键的 ANSI 转义序列）。`get_message(&mut self)` 返回 `Option<InputMessage>`：Normal 模式下 Ctrl+O 进入菜单，MenuSelect 模式下数字键执行宏、其他键吞掉并回到 Normal。Ctrl+] / Ctrl+K 在任何模式下都立即生效。`transition(menu_mode, raw, dst)` 抽成关联函数便于单元测试。

- **`serial_manager.rs`** — 串口共享管理器（`SerialManager`）与协议抽象（`SerialOps` trait）。`SerialManager` 封装串口句柄、**接收缓冲区（RX buffer，`Arc<(Mutex<VecDeque<u8>>, Condvar)>`，上限 64KB，常量 `RX_BUFFER_MAX`）**和退出标志（`Arc<AtomicBool>`）。接收缓冲区累积设备返回的所有数据，供 MCP 工具（`serial_read`/`serial_grep`）和宏 `expect` 步骤共享使用。`SerialOps` trait 定义协议层依赖的串口操作接口（`send`、`drain_buffer`、`clear_buffer`、`grep_buffer`、`grep_buffer_bytes`、`status`），`SerialManager` 实现该 trait，并额外提供 `peek_buffer`、`port`、`read_buffer`、`quit_flag` 等方法。错误通过 `SerialError`（thiserror）类型传播。`mcp.rs` 与 `macro_runner.rs` 通过泛型 `<S: SerialOps>` 依赖此 trait，实现协议层与硬件层的解耦，便于用 mock 替换硬件后端进行单元测试。

- **`task.rs`** — 核心运行时（`TerminalSerial`）。通过 `SerialManager` 打开并配置串口，加载 `Config`（宏配置，`Arc<Config>` 共享给输入线程），然后启动线程：输入线程（键盘→串口，并处理 `ShowMacroMenu`/`RunMacro` 控制消息）、读取线程（串口→终端+接收缓冲区+Telnet 广播）。MCP/Telnet 服务器绑定失败时优雅退出（返回 `eprintln + return`，不 `process::exit`）。Event log 创建在 MCP/Telnet 之前（startup 事件先记录，且服务器共享此 writer）。Event log 打开失败时降级为不记录日志。所有用户面向的工具反馈通过 `ui` 模块统一风格（青色 ▶ 开始、绿色 ✓ 成功、红色 ✗ 失败、黄色次要提示）。启动时若有宏，打印一行紧凑提示 `Macros: [1]name ... (Ctrl+O for menu)`。

- **`mcp.rs`** — MCP 协议处理（与传输层解耦的纯函数层）。实现 JSON-RPC 2.0 的 `initialize`、`tools/list`、`tools/call`、`prompts/list`、`prompts/get` 方法。暴露五个工具：`serial_send`（发送数据，可选等待响应）、`serial_read`（读取缓冲区数据）、`serial_status`（连接状态）、`serial_grep`（非破坏性正则搜索）、`serial_clear`（清空缓冲区），以及一个 prompt `serial_usage_guide`（使用指南）。`handle_request` 及各 `tool_*` 函数均以泛型 `<S: SerialOps>` 依赖串口抽象，便于用 mock 替换硬件后端进行单元测试。hex 解析复用 `util` 模块的 `hex_decode`。

- **`mcp_http.rs`** — MCP 的 HTTP 传输层（`McpHttpServer`）。基于 `TcpListener` 处理 `POST /mcp` 端点，将请求转发给 `mcp.rs` 处理。连接设置 30s 读写超时防止挂起。accept 错误通过 `EventLogWriter` 记录（不向终端 `eprintln`）。`new()` 接受 `Option<Arc<EventLogWriter>>` 参数；`start()` 返回 `Result<(), String>`，绑定失败向上传播。绑定地址和端口通过 `--mcp-host`（默认 `0.0.0.0`）和 `--mcp-port`（默认 `8765`）配置。

- **`telnet.rs`** — Telnet 服务器（`TelnetServer`）。支持多客户端连接、telnet 协议协商（IAC/WILL/WONT/DO/DONT）、串口数据广播。客户端输入经过 NUL 字节剥离和 telnet 命令过滤后写入串口。客户端连接/断开/accept 错误通过 `EventLogWriter` 记录（不向终端 `eprintln`，避免污染串口会话）。`new()` 接受 `Option<Arc<EventLogWriter>>` 参数；`start()` 返回 `Result<(), String>`。

- **`event_log.rs`** — 事件日志写入器（`EventLogWriter`），线程安全（`Mutex<File>`）。记录 startup/shutdown/tx/rx/error/client_connected/client_disconnected/action 事件为 JSONL 格式。tx/rx 事件的字节数据通过 `util` 模块的 `hex_encode` 编码为 hex 字符串。`log_action(source, action, name)` 用于记录宏执行、缓冲区清空等控制动作，`source` 字段区分来源（当前仅 `"local"`，为后续 MCP/Telnet 触发预留）。

- **`log_reader.rs`** — 日志查看器。解析 JSONL 事件日志，支持按事件类型（含 `action`）、来源、正则内容过滤，输出格式 text/json/md，会话摘要统计（含 action 计数）。`LogEvent` 包含 `action`/`name` 字段用于渲染 action 事件。

- **`util.rs`** — 通用工具函数。提供 `hex_encode`（字节切片→大写 hex 字符串，使用 `{:02X}`）和 `hex_decode`（hex 字符串→字节切片，奇数长度返回 `None`）。被 `event_log.rs`、`log_reader.rs`、`mcp.rs`、`macro_runner.rs` 复用。

- **`ui.rs`** — 终端反馈风格统一模块。所有用户面向的工具提示通过这里发出，保证 ANSI 颜色、符号、括号与换行风格一致。提供 5 个函数：`success`（绿色 ✓ + 方括号，动作完成）、`info`（青色 ▶ + 方括号，动作开始）、`fail`（红色 ✗ + 方括号，动作失败）、`hint`（黄色 + 圆括号，次要提示）、`heading`（青色 + === 边框，章节标题）。**所有函数默认输出独立视觉块**（前导 `\r\n` 与设备输出隔开 + 反馈 + 后导 `\r\n`），调用方不再写任何 `print!("\r\n")`，避免未来引入 quiet 开关时残留空行。每个函数内部 `flush` stdout。仅用于串口会话期间的反馈；启动提示、`cmd.rs` 端口选择、`log_reader.rs` 子命令输出等不经过此模块。

- **`error.rs`** — 定义 `SerialError` 枚举（thiserror）：`PortOpen`、`Write`、`Read`、`Regex`。

## 金丝雀规则

**语言要求**：所有生成的文档（包括 openspec 规范开发流程生成的文档）、向用户提出的问题、代码注释，都必须使用中文。仅在确实必要时才使用英文，例如：代码标识符、命令行参数、技术专有名词等本身为英文的内容。

## 关键设计细节

- **线程模型**：输入线程（键盘→串口）和读取线程（串口→终端+接收缓冲区+Telnet 广播），`-M` 模式下额外启动 MCP HTTP 服务器线程，`-T` 模式下额外启动 Telnet 服务器线程（接受线程+广播线程+每客户端读取线程）。通过 `SerialManager` 统一管理共享状态。
- **退出机制**：退出标志使用 `Arc<AtomicBool>`，输入线程设置标志后退出，读取线程和服务线程通过 `load(Ordering::Relaxed)` 检查退出。
- **MCP 集成**：`-M` 启用 MCP 服务器。Claude Code 通过 HTTP MCP 协议（`http://{mcp-host}:{mcp-port}/mcp`）接入。工具暴露为 `serial_send`、`serial_read`、`serial_status`、`serial_grep`、`serial_clear`。
- **Telnet 集成**：`-T` 启用 Telnet 服务器。支持多客户端同时连接，串口数据通过 `mpsc` 通道广播到所有客户端。
- **快捷键**：`Ctrl + ]` 退出；`Ctrl + K` 清空接收缓冲区；`Ctrl + O` 进入宏菜单选择模式（再按 `1`-`9` 执行对应宏，其他键退出菜单）。`Ctrl+]`/`Ctrl+K` 在菜单模式下仍立即生效。控制动作（清缓冲区、执行宏）通过 `log_action` 记录到事件日志，终端反馈通过 `ui` 模块统一风格（不与设备数据混淆）。
- **宏配置**：`--config <path>` 加载 JSON 格式的统一工具配置文件。当前仅 `macros` section 生效，schema 设计为可扩展（顶层字段全部 `#[serde(default)]`，新增字段对老配置透明）。宏定义示例：`{"macros":{"init":{"description":"设备初始化","steps":[{"type":"send","data":"ATZ","auto_newline":true},{"type":"delay","ms":500},{"type":"expect","pattern":"OK","timeout_ms":3000},{"type":"clear"}]}}}`。
- **编码处理**：`#[cfg(windows)]` 下输入字节先以 GBK 解码再编码为 UTF-8 发送到串口；非 Windows 平台直接透传 UTF-8。`encoding_rs` 仅作为 Windows 条件依赖。
- **串口超时**：设置为 1ms；读取线程每 10ms 轮询一次。
- **命令行配置**：使用 clap derive 模式（`Parser`、`Subcommand`、`ValueEnum`）。
- **平台条件编译**：大量使用 `#[cfg(windows)]` / `#[cfg(not(windows))]`。Windows 特有依赖（`libc`、`encoding_rs`）；Unix 特有依赖（`termios`）。macOS 与 Linux 的串口发现使用嵌套的 `#[cfg(target_os)]`。
- **错误处理**：库代码使用 `SerialError`（thiserror）类型，不使用 `process::exit`。`process::exit` 仅出现在 CLI 入口层（`cmd.rs`、`main.rs`）。运行时（非启动期）的异步事件错误（Telnet/MCP accept 错误等）通过 `event_log` 记录，不向终端 `eprintln`（避免污染串口会话）；启动期错误（端口绑定失败、配置加载失败等）才用 `eprintln`。
- **事件日志**：`--event-log <path>` 启用 JSONL 格式的事件记录。`log` 子命令用于查看、过滤和分析日志。

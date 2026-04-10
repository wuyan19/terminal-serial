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
cargo install --path .         # 本地安装
```

本项目目前没有测试。

## 架构

二进制入口是一个薄包装层（`src/main.rs`），解析命令行参数后委托给 `TerminalSerial::run()`。

**模块职责：**

- **`cmd.rs`** — 通过 `clap` 解析命令行参数（YAML 配置在 `src/cmd.yml`）。返回 `AppConfig` 结构体（包含 `port`、`setting`、`serve`、`mcp_host`、`mcp_port`）。串口枚举因平台而异：Windows 从注册表读取（`HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP\SERIALCOMM`），macOS 扫描 `/dev/tty.usb*`，Linux 扫描 `/dev/ttyUSB*`。未指定 `-p` 时，提示用户从枚举列表中选择（从 1 开始编号）。

- **`lib.rs`** — 定义 `Input` 和 `InputMessage`。`Input` 封装 `Keyboard`，将原始按键事件映射为 `InputMessage` 变体（`Quit`、`Data`、`ClearBuffer`、`None`）。`Ctrl + ]`（字节 `0x1d`）触发 `Quit`，`Ctrl + K`（字节 `0x0b`）触发 `ClearBuffer`。

- **`keyboard.rs`** — 使用树结构进行平台相关的键码转换。Windows 使用 `0xe0` 前缀扫描码；Unix 使用 `0x1b`（ESC）前缀序列。将方向键、导航键和功能键映射为 ANSI 转义序列（`dst_value`），用于通过串口发送。

- **`getch.rs`** — 原始单字节键盘输入。Windows 使用 libc 的 `_getch()`。Unix 通过 `termios` 将终端设为 raw 模式（非规范模式、无回显、无信号），在 `Drop` 时恢复原始设置。

- **`serial_manager.rs`** — 串口共享管理器（`SerialManager`）。封装串口句柄、MCP 读取缓冲区（`VecDeque<u8>`，上限 64KB）和退出标志，全部通过 `Arc<Mutex<...>>` 跨线程共享。提供 `send`、`drain_buffer`、`clear_buffer`、`status` 等方法。

- **`task.rs`** — 核心运行时（`TerminalSerial`）。通过 `SerialManager` 打开并配置串口，然后启动线程：输入线程（键盘→串口，带 GBK 到 UTF-8 编码转换）、读取线程（串口→终端+MCP 缓冲区）。`--serve` 模式下还会启动 MCP HTTP 服务器。

- **`mcp.rs`** — MCP 协议处理。实现 JSON-RPC 2.0 的 `initialize`、`tools/list`、`tools/call` 方法。暴露三个工具：`serial_send`（发送数据，可选等待响应）、`serial_read`（读取缓冲区数据）、`serial_status`（连接状态）。

- **`server.rs`** — HTTP 服务器（`McpServer`）。基于 `TcpListener` 处理 `POST /mcp` 端点，将请求转发给 `mcp.rs` 处理。绑定地址和端口通过 `--mcp-host`（默认 `0.0.0.0`）和 `--mcp-port`（默认 `8765`）配置。

## 金丝雀规则

**语言要求**：所有生成的文档（包括 openspec 规范开发流程生成的文档）、向用户提出的问题、代码注释，都必须使用中文。仅在确实必要时才使用英文，例如：代码标识符、命令行参数、技术专有名词等本身为英文的内容。

## 关键设计细节

- **线程模型**：输入线程（键盘→串口）和读取线程（串口→终端+MCP 缓冲区），`--serve` 模式下额外启动 MCP HTTP 服务器线程。通过 `SerialManager` 统一管理共享状态（`Arc<Mutex<Box<dyn SerialPort + Send>>>`）。
- **MCP 集成**：`--serve` 启用 MCP 服务器。Claude Code 通过 HTTP MCP 协议（`http://{mcp-host}:{mcp-port}/mcp`）接入。工具暴露为 `serial_send`、`serial_read`、`serial_status`。
- **快捷键**：`Ctrl + ]` 退出，`Ctrl + K` 清空 MCP 读缓冲区（`--serve` 模式下）。
- **编码处理**：输入字节先以 GBK 解码，再重新编码为 UTF-8 后发送到串口（支持 Windows 下的中文输入）。
- **串口超时**：设置为 1ms；读取线程每 2ms 轮询一次。
- **命令行配置**：`src/cmd.yml` 定义了通过 `load_yaml!` 加载的 clap 参数模式。
- **平台条件编译**：大量使用 `#[cfg(windows)]` / `#[cfg(not(windows))]`。Windows 特有依赖（`winreg`、`libc`）；Unix 特有依赖（`termios`）。macOS 与 Linux 的串口发现使用嵌套的 `#[cfg(target_os)]`。

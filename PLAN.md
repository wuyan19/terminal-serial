# terminal-serial 进化路线图

## 一、MCP 能力增强（高价值）

当前只暴露了 3 个工具（`serial_send`、`serial_read`、`serial_status`），对 Claude Code 来说还有常用场景未覆盖：

| 新工具 | 用途 |
|--------|------|
| `serial_enumerate` | 列出可用串口，Claude Code 可自行发现端口 |
| `serial_configure` | 运行时修改波特率等配置（调试中切换速率很常见） |
| `serial_signals` | 查询/控制 DTR、RTS 信号线（嵌入式开发刚需） |
| `serial_break` | 发送 Break 信号（某些设备唤醒/重置需要） |

另外 MCP 协议本身还支持 `prompts/list`、`resources/list`、`logging/setLevel`，当前未实现。这些可以让 Claude Code 获取串口使用提示、查看历史数据等。

## 二、终端交互体验（中价值）

当前终端模式功能比较基础：

- **时间戳显示** — `--timestamp` 选项，每行数据前加时间戳，调试时非常有用
- **Hex 显示模式** — `--hex` 选项，终端中同时显示十六进制和 ASCII，嵌入式开发必备
- **数据记录到文件** — `--log file.log` 自动将收发数据写入文件，用于事后分析
- **行尾配置** — 当前文本模式固定 `\r\n`，有些设备只要 `\r` 或 `\n`，应可配置
- **退出时的状态信息** — 当前 `Ctrl+]` 后静默退出，应显示连接时长、收发字节数统计

## 三、工程质量（中价值）

### 测试

当前零测试。可以加的：

- MCP JSON-RPC 处理的单元测试（纯逻辑，不需要真实串口）
- `cmd.rs` 参数解析测试
- `decode_gbk_input()` 编码测试
- `keyboard.rs` 按键映射测试

### 错误处理统一化

当前混用 `Result<_, String>`、`process::exit()`、`unwrap()`。可以引入 `thiserror` 定义统一错误类型。

## 四、安全性（低价值但值得注意）

- MCP 服务器默认绑定 `0.0.0.0`，意味着对局域网开放串口控制。可以在文档中强调风险，或加 `--mcp-auth <token>` 简单认证
- 当前 HTTP 无请求速率限制，恶意客户端可快速消耗线程

## 五、CI/CD 完善（低价值）

- 添加 ARM Linux 目标（树莓派常用）
- 生成 SHA256 校验和
- 发布前自动运行 `cargo clippy` 和 `cargo test`

## 建议优先级

如果主要使用场景是配合 Claude Code 做嵌入式调试，**MCP 能力增强**（特别是 `serial_enumerate` 和 `serial_signals`）投入产出比最高——实现简单（每个工具 20-30 行），但能显著提升 Claude Code 的操控能力。

use crate::serial_manager::SerialOps;
use crate::util::hex_decode;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2024-11-05";

pub fn handle_request<S: SerialOps>(body: &str, manager: &S) -> Value {
    let request: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": format!("Parse error: {}", e)
                }
            });
        }
    };

    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = match request.get("method").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: missing method"
                }
            });
        }
    };

    let params = request.get("params").cloned().unwrap_or(json!({}));

    let result = match method {
        "initialize" => handle_initialize(&params),
        "notifications/initialized" => {
            // 通知，不需要返回 result
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            });
        }
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(&params, manager),
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(&params),
        "ping" => json!({}),
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            });
        }
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn handle_initialize(_params: &Value) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "terminal-serial",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "serial_send",
                "description": "发送数据到串口并可选等待设备响应。设置 timeout_ms > 0 时，会自动等待并返回设备的响应数据，无需再调用 serial_read。支持文本和十六进制模式。text 模式下默认自动在末尾追加 \\r\\n，只需发送命令内容即可，例如 \"showsysinfo\"、\"AT\"。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "string",
                            "description": "要发送的数据。text 模式下会自动追加换行符，无需手动添加 \\r\\n"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["text", "hex"],
                            "default": "text",
                            "description": "数据格式：text 为文本（自动追加 \\r\\n），hex 为十六进制原始字节（如 '48656C6C6F'，不追加换行）"
                        },
                        "auto_newline": {
                            "type": "boolean",
                            "default": true,
                            "description": "是否自动在 text 模式数据末尾追加 \\r\\n。设为 false 则发送原始数据，不追加换行"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": 0,
                            "description": "发送后等待设备响应的超时时间（毫秒）。设为 0（默认）则不等待响应立即返回。设为大于 0 的值则会等待设备返回数据并在结果中包含响应内容，这是获取命令响应的推荐方式，无需再额外调用 serial_read。"
                        }
                    },
                    "required": ["data"]
                }
            },
            {
                "name": "serial_read",
                "description": "读取串口接收缓冲区中的数据。返回自上次读取以来的所有新数据。如果缓冲区为空，会等待直到有数据到达或超时。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "enum": ["text", "hex"],
                            "default": "text",
                            "description": "返回数据的格式：text 为文本（UTF-8），hex 为十六进制字节（如 '4F4B0D0A'）"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": 100,
                            "description": "当缓冲区为空时，等待新数据到达的超时时间（毫秒）。设为 0 则立即返回当前缓冲区内容（可能为空）。建议使用 serial_send 的 timeout_ms 参数代替轮询读取。"
                        }
                    }
                }
            },
            {
                "name": "serial_status",
                "description": "获取当前串口连接状态和配置信息。",
                "inputSchema": {
                    "type": "object"
                }
            },
            {
                "name": "serial_grep",
                "description": "搜索串口接收缓冲区中匹配指定模式的数据行，不清空缓冲区。支持正则表达式。可用于轮询等待特定输出模式。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "搜索模式。text 模式下支持正则表达式（如 'OK'、'ERROR.*timeout'）；hex 模式下为十六进制字节序列（如 'AA55'、'0D 0A'）"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["text", "hex"],
                            "default": "text",
                            "description": "搜索和返回格式：text 为正则匹配文本行，hex 为字节序列匹配原始缓冲区"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "default": 1000,
                            "description": "等待匹配的超时时间（毫秒）。设为 0 则立即搜索当前缓冲区内容。"
                        }
                    },
                    "required": ["pattern"]
                }
            },
            {
                "name": "serial_clear",
                "description": "清空串口接收缓冲区中的所有数据，不返回任何内容。",
                "inputSchema": {
                    "type": "object"
                }
            }
        ]
    })
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = hex.replace(" ", "").replace("0x", "").replace(",", "");
    hex_decode(&cleaned).ok_or_else(|| "Invalid hex string".to_string())
}

fn handle_tools_call<S: SerialOps>(params: &Value, manager: &S) -> Value {
    let name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => {
            return json!({
                "isError": true,
                "content": [{"type": "text", "text": "Missing tool name"}]
            });
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
        "serial_send" => tool_serial_send(&arguments, manager),
        "serial_read" => tool_serial_read(&arguments, manager),
        "serial_status" => tool_serial_status(manager),
        "serial_grep" => tool_serial_grep(&arguments, manager),
        "serial_clear" => tool_serial_clear(manager),
        _ => json!({
            "isError": true,
            "content": [{"type": "text", "text": format!("Unknown tool: {}", name)}]
        }),
    }
}

fn tool_serial_send<S: SerialOps>(args: &Value, manager: &S) -> Value {
    let data_str = match args.get("data").and_then(|d| d.as_str()) {
        Some(d) => d,
        None => {
            return json!({
                "isError": true,
                "content": [{"type": "text", "text": "Missing required parameter: data"}]
            });
        }
    };

    let format = args
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("text");

    let auto_newline = args
        .get("auto_newline")
        .and_then(|a| a.as_bool())
        .unwrap_or(true);

    let mut bytes = match format {
        "hex" => match hex_to_bytes(data_str) {
            Ok(b) => b,
            Err(e) => {
                return json!({
                    "isError": true,
                    "content": [{"type": "text", "text": format!("Invalid hex data: {}", e)}]
                });
            }
        },
        _ => data_str.as_bytes().to_vec(),
    };

    // text 模式下默认自动追加 \r\n
    if format != "hex" && auto_newline {
        bytes.push(b'\r');
        bytes.push(b'\n');
    }

    if bytes.is_empty() {
        return json!({
            "isError": true,
            "content": [{"type": "text", "text": "No data to send"}]
        });
    }

    match manager.send(&bytes) {
        Ok(n) => {
            let timeout_ms = args
                .get("timeout_ms")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32;

            if timeout_ms > 0 {
                // 先等待一小段时间让设备处理
                std::thread::sleep(std::time::Duration::from_millis(50));
                let response_data = manager.drain_buffer(timeout_ms);
                let response_text = String::from_utf8_lossy(&response_data);

                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Sent {} bytes. Response: {}", n, response_text)
                    }]
                })
            } else {
                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Sent {} bytes", n)
                    }]
                })
            }
        }
        Err(e) => json!({
            "isError": true,
            "content": [{"type": "text", "text": format!("Send failed: {}", e)}]
        }),
    }
}

fn tool_serial_read<S: SerialOps>(args: &Value, manager: &S) -> Value {
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(100) as u32;

    let format = args
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("text");

    let data = manager.drain_buffer(timeout_ms);

    let output = match format {
        "hex" => data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(""),
        _ => String::from_utf8_lossy(&data).to_string(),
    };

    json!({
        "content": [{
            "type": "text",
            "text": output
        }]
    })
}

fn tool_serial_status<S: SerialOps>(manager: &S) -> Value {
    let status = manager.status();
    json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Port: {}\nBaud rate: {}\nData bits: {}\nParity: {}\nStop bits: {}\nFlow control: {}\nStatus: {}",
                status.port_name,
                status.baud_rate,
                status.char_size,
                status.parity,
                status.stop_bits,
                status.flow_control,
                if status.is_open { "connected" } else { "disconnected" }
            )
        }]
    })
}

fn tool_serial_grep<S: SerialOps>(args: &Value, manager: &S) -> Value {
    let pattern = match args.get("pattern").and_then(|p| p.as_str()) {
        Some(p) => p,
        None => {
            return json!({
                "isError": true,
                "content": [{"type": "text", "text": "Missing required parameter: pattern"}]
            });
        }
    };

    let timeout_ms = args
        .get("timeout_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(1000) as u32;

    let format = args
        .get("format")
        .and_then(|f| f.as_str())
        .unwrap_or("text");

    if format == "hex" {
        // hex 模式：pattern 作为十六进制字节序列，在原始缓冲区中搜索
        let pattern_bytes = match hex_to_bytes(pattern) {
            Ok(b) => b,
            Err(e) => {
                return json!({
                    "isError": true,
                    "content": [{"type": "text", "text": format!("Invalid hex pattern: {}", e)}]
                });
            }
        };

        if pattern_bytes.is_empty() {
            return json!({
                "isError": true,
                "content": [{"type": "text", "text": "Pattern is empty"}]
            });
        }

        let matches = manager.grep_buffer_bytes(&pattern_bytes, timeout_ms);
        if matches.is_empty() {
            json!({
                "content": [{
                    "type": "text",
                    "text": "No match found (timeout)"
                }]
            })
        } else {
            let output: Vec<String> = matches
                .iter()
                .map(|(pos, context)| {
                    let hex: String = context
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("offset {}: {}", pos, hex)
                })
                .collect();
            json!({
                "content": [{
                    "type": "text",
                    "text": output.join("\n")
                }]
            })
        }
    } else {
        // text 模式：pattern 作为正则表达式，按行匹配文本
        match manager.grep_buffer(pattern, timeout_ms) {
            Ok(lines) => {
                if lines.is_empty() {
                    json!({
                        "content": [{
                            "type": "text",
                            "text": "No match found (timeout)"
                        }]
                    })
                } else {
                    json!({
                        "content": [{
                            "type": "text",
                            "text": lines.join("\n")
                        }]
                    })
                }
            }
            Err(e) => json!({
                "isError": true,
                "content": [{"type": "text", "text": format!("Grep failed: {}", e)}]
            }),
        }
    }
}

fn tool_serial_clear<S: SerialOps>(manager: &S) -> Value {
    manager.clear_buffer();
    json!({
        "content": [{
            "type": "text",
            "text": "Buffer cleared"
        }]
    })
}

// ==================== Prompts ====================

fn handle_prompts_list() -> Value {
    json!({
        "prompts": [
            {
                "name": "serial_usage_guide",
                "description": "串口工具工作流指南：核心概念、推荐使用模式和常见陷阱",
                "arguments": []
            }
        ]
    })
}

fn handle_prompts_get(params: &Value) -> Value {
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");

    match name {
        "serial_usage_guide" => json!({
            "description": "串口工具工作流指南",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": SERIAL_USAGE_GUIDE
                    }
                }
            ]
        }),
        _ => json!({
            "isError": true,
            "content": [{"type": "text", "text": format!("Unknown prompt: {}", name)}]
        })
    }
}

const SERIAL_USAGE_GUIDE: &str = r#"# 串口工具工作流指南

## 核心概念

**MCP 读缓冲区**是一个 64KB 的 FIFO 缓冲区，持续接收串口设备输出的数据。缓冲区满时最旧数据被丢弃。

关键区别：
- `serial_read` 是**破坏性读取**——调用后缓冲区被清空，数据不可恢复。
- `serial_grep` 是**非破坏性搜索**——只读不删，可反复搜索。

## 推荐工作流

### 简单命令-响应

直接使用 `serial_send` 的 `timeout_ms` 参数：

```
serial_send(data="AT", timeout_ms=1000)
```

### 等待特定输出（设备重启、长耗时操作）

使用 `serial_grep` + `serial_clear` 组合：

```
1. serial_send(data="reboot")
2. serial_grep(pattern="Kernel started", timeout_ms=5000)
   → 匹配到：返回匹配行
   → 未匹配：数据仍在缓冲区，可再次 grep 或转用 serial_read
3. serial_clear()  // 清空缓冲区，为下一步准备
```

不要用 `serial_read` 轮询——每次调用都会清空缓冲区，如果目标输出还没出现数据就丢失了。

### 获取大量输出

```
serial_send(data="showsysinfo")
serial_read(timeout_ms=2000)
```

## 常见陷阱

1. **不要手动添加换行**：`serial_send` 的 text 模式默认自动追加 `\r\n`，设置 `auto_newline=false` 才会发送原始数据。
2. **serial_grep 不会阻塞数据流**：等待期间新数据正常进入缓冲区。
3. **缓冲区 64KB 限制**：长时间运行的设备输出会覆盖旧数据，重要信息及时读取。"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SerialError;
    use std::cell::RefCell;

    /// Mock SerialOps 实现，用于测试协议层逻辑
    struct MockSerial {
        sent_data: RefCell<Vec<Vec<u8>>>,
        read_data: Vec<u8>,
        clear_count: RefCell<usize>,
    }

    impl MockSerial {
        fn new(read_data: Vec<u8>) -> Self {
            MockSerial {
                sent_data: RefCell::new(vec![]),
                read_data,
                clear_count: RefCell::new(0),
            }
        }
    }

    impl SerialOps for MockSerial {
        fn send(&self, data: &[u8]) -> Result<usize, SerialError> {
            self.sent_data.borrow_mut().push(data.to_vec());
            Ok(data.len())
        }

        fn drain_buffer(&self, _timeout_ms: u32) -> Vec<u8> {
            self.read_data.clone()
        }

        fn clear_buffer(&self) {
            *self.clear_count.borrow_mut() += 1;
        }

        fn grep_buffer(
            &self,
            _pattern: &str,
            _timeout_ms: u32,
        ) -> Result<Vec<String>, SerialError> {
            Ok(vec![])
        }

        fn grep_buffer_bytes(
            &self,
            _pattern: &[u8],
            _timeout_ms: u32,
        ) -> Vec<(usize, Vec<u8>)> {
            vec![]
        }

        fn status(&self) -> crate::serial_manager::SerialStatus {
            crate::serial_manager::SerialStatus {
                port_name: "MOCK".into(),
                baud_rate: 9600,
                char_size: "8".into(),
                parity: "None".into(),
                stop_bits: "1".into(),
                flow_control: "None".into(),
                is_open: true,
            }
        }
    }

    // ==================== 协议层测试 ====================

    #[test]
    fn parse_error_invalid_json() {
        let mock = MockSerial::new(vec![]);
        let resp = handle_request("not json", &mock);
        assert_eq!(resp["error"]["code"], -32700);
    }

    #[test]
    fn invalid_request_missing_method() {
        let mock = MockSerial::new(vec![]);
        let resp = handle_request(r#"{"jsonrpc":"2.0","id":1,"params":{}}"#, &mock);
        assert_eq!(resp["error"]["code"], -32600);
    }

    #[test]
    fn method_not_found() {
        let mock = MockSerial::new(vec![]);
        let resp = handle_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"foo/bar"}"#,
            &mock,
        );
        assert_eq!(resp["error"]["code"], -32601);
    }

    #[test]
    fn initialize_response() {
        let mock = MockSerial::new(vec![]);
        let resp = handle_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            &mock,
        );
        assert_eq!(resp["result"]["serverInfo"]["name"], "terminal-serial");
        assert!(resp["result"]["protocolVersion"].is_string());
    }

    #[test]
    fn tools_list_contains_all_tools() {
        let mock = MockSerial::new(vec![]);
        let resp = handle_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
            &mock,
        );
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"serial_send"));
        assert!(names.contains(&"serial_read"));
        assert!(names.contains(&"serial_status"));
        assert!(names.contains(&"serial_grep"));
        assert!(names.contains(&"serial_clear"));
    }

    // ==================== 工具调用测试 ====================

    #[test]
    fn serial_send_text_adds_newline() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_send","arguments":{"data":"AT"}}}"#;
        let resp = handle_request(req, &mock);

        assert!(!resp["result"]["isError"].as_bool().unwrap_or(false));
        let sent = mock.sent_data.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], b"AT\r\n");
    }

    #[test]
    fn serial_send_hex_no_newline() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_send","arguments":{"data":"0D0A","format":"hex"}}}"#;
        let resp = handle_request(req, &mock);

        assert!(!resp["result"]["isError"].as_bool().unwrap_or(false));
        let sent = mock.sent_data.borrow();
        assert_eq!(sent[0], b"\r\n");
    }

    #[test]
    fn serial_send_no_newline_flag() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_send","arguments":{"data":"raw","auto_newline":false}}}"#;
        handle_request(req, &mock);

        let sent = mock.sent_data.borrow();
        assert_eq!(sent[0], b"raw");
    }

    #[test]
    fn serial_send_with_timeout_returns_response() {
        let mock = MockSerial::new(b"OK\r\n".to_vec());
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_send","arguments":{"data":"AT","timeout_ms":100}}}"#;
        let resp = handle_request(req, &mock);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Response:"));
        assert!(text.contains("OK"));
    }

    #[test]
    fn serial_send_missing_data_param() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_send","arguments":{}}}"#;
        let resp = handle_request(req, &mock);
        assert_eq!(resp["result"]["isError"], true);
    }

    #[test]
    fn serial_read_returns_buffer_content() {
        let mock = MockSerial::new(b"Hello World".to_vec());
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_read","arguments":{}}}"#;
        let resp = handle_request(req, &mock);
        assert_eq!(resp["result"]["content"][0]["text"].as_str().unwrap(), "Hello World");
    }

    #[test]
    fn serial_read_hex_format() {
        let mock = MockSerial::new(vec![0x0D, 0x0A]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_read","arguments":{"format":"hex"}}}"#;
        let resp = handle_request(req, &mock);
        assert_eq!(resp["result"]["content"][0]["text"].as_str().unwrap(), "0D0A");
    }

    #[test]
    fn serial_status_returns_config() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_status","arguments":{}}}"#;
        let resp = handle_request(req, &mock);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Port: MOCK"));
        assert!(text.contains("Baud rate: 9600"));
    }

    #[test]
    fn serial_clear_invokes_clear_buffer() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"serial_clear","arguments":{}}}"#;
        handle_request(req, &mock);
        assert_eq!(*mock.clear_count.borrow(), 1);
    }

    #[test]
    fn unknown_tool_returns_error() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"foo","arguments":{}}}"#;
        let resp = handle_request(req, &mock);
        assert_eq!(resp["result"]["isError"], true);
    }

    #[test]
    fn missing_tool_name() {
        let mock = MockSerial::new(vec![]);
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"arguments":{}}}"#;
        let resp = handle_request(req, &mock);
        assert_eq!(resp["result"]["isError"], true);
    }

    // ==================== hex 工具测试 ====================

    #[test]
    fn hex_to_bytes_valid() {
        assert_eq!(hex_to_bytes("48656C6C6F").unwrap(), b"Hello");
    }

    #[test]
    fn hex_to_bytes_with_spaces() {
        assert_eq!(hex_to_bytes("48 65 6C 6C 6F").unwrap(), b"Hello");
    }

    #[test]
    fn hex_to_bytes_with_0x_prefix() {
        assert_eq!(hex_to_bytes("0x0D0x0A").unwrap(), b"\r\n");
    }

    #[test]
    fn hex_to_bytes_odd_length() {
        assert!(hex_to_bytes("ABC").is_err());
    }

    #[test]
    fn hex_to_bytes_empty() {
        assert_eq!(hex_to_bytes("").unwrap(), b"");
    }
}

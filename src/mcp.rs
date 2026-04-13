use crate::serial_manager::SerialManager;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2024-11-05";

pub fn handle_request(body: &str, manager: &SerialManager) -> Value {
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
                "description": "发送数据到串口。支持文本和十六进制模式。text 模式下默认自动在末尾追加 \\r\\n，只需发送命令内容即可，例如 \"showsysinfo\"、\"AT\"。",
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
                            "description": "发送后等待设备响应的超时时间（毫秒）。设为 0（默认）则不等待响应立即返回，设为大于 0 的值则等待设备返回数据。"
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

fn handle_tools_call(params: &Value, manager: &SerialManager) -> Value {
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

fn tool_serial_send(args: &Value, manager: &SerialManager) -> Value {
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

fn tool_serial_read(args: &Value, manager: &SerialManager) -> Value {
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

fn tool_serial_status(manager: &SerialManager) -> Value {
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

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.replace(" ", "").replace("0x", "").replace(",", "");
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
}

fn tool_serial_grep(args: &Value, manager: &SerialManager) -> Value {
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

fn tool_serial_clear(manager: &SerialManager) -> Value {
    manager.clear_buffer();
    json!({
        "content": [{
            "type": "text",
            "text": "Buffer cleared"
        }]
    })
}

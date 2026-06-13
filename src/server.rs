use crate::mcp;
use crate::serial_manager::SerialManager;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub struct McpServer {
    manager: SerialManager,
    quit: Arc<AtomicBool>,
}

impl McpServer {
    pub fn new(manager: &SerialManager) -> Self {
        McpServer {
            manager: manager.clone(),
            quit: manager.quit_flag(),
        }
    }

    pub fn start(&self, host: &str, port: u16) -> Result<(), String> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).map_err(|e| {
            format!("Failed to bind MCP server on {}: {}", addr, e)
        })?;

        let manager = self.manager.clone();
        let quit = Arc::clone(&self.quit);

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if quit.load(Ordering::Relaxed) {
                    break;
                }

                match stream {
                    Ok(stream) => {
                        let manager = manager.clone();
                        std::thread::spawn(move || {
                            handle_connection(stream, manager);
                        });
                    }
                    Err(e) => {
                        eprintln!("Connection failed: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

fn handle_connection(stream: std::net::TcpStream, manager: SerialManager) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

    let writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream);
    let mut writer = writer;

    // 读取 HTTP 请求行
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 3 {
        send_http_error(&mut writer, 400, "Bad Request");
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if path != "/mcp" {
        send_http_error(&mut writer, 404, "Not Found");
        return;
    }

    // CORS 预检请求
    if method == "OPTIONS" {
        let response = "HTTP/1.1 204 No Content\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Access-Control-Max-Age: 86400\r\n\
            Content-Length: 0\r\n\r\n";
        let _ = writer.write_all(response.as_bytes());
        let _ = writer.flush();
        return;
    }

    if method != "POST" {
        send_http_error(&mut writer, 405, "Method Not Allowed");
        return;
    }

    // 读取请求头
    let mut content_length: usize = 0;
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).is_err() {
            return;
        }
        if header_line.trim().is_empty() {
            break;
        }
        if let Some(pos) = header_line.to_lowercase().find("content-length:") {
            let value = header_line[pos + 15..].trim();
            content_length = value.parse().unwrap_or(0);
        }
    }

    // 请求体大小限制
    if content_length > MAX_BODY_SIZE {
        send_http_error(&mut writer, 413, "Payload Too Large");
        return;
    }

    // 读取请求体
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        if reader.read_exact(&mut body).is_err() {
            send_http_error(&mut writer, 400, "Failed to read body");
            return;
        }
    }

    let body_str = String::from_utf8_lossy(&body);

    let response = mcp::handle_request(&body_str, &manager);
    let response_body = serde_json::to_string(&response).unwrap_or_default();

    // 发送 HTTP 响应
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    let _ = writer.write_all(http_response.as_bytes());
    let _ = writer.flush();
}

fn send_http_error(writer: &mut std::net::TcpStream, code: u16, message: &str) {
    let body = format!("{{\"error\": {{\"code\": {}, \"message\": \"{}\"}}}}", code, message);
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        code,
        message,
        body.len(),
        body
    );
    let _ = writer.write_all(response.as_bytes());
    let _ = writer.flush();
}

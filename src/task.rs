use crate::cmd::AppConfig;
use crate::event_log::EventLogWriter;
use crate::input::{Input, InputMessage};
use crate::serial_manager::SerialManager;
use crate::server::McpServer;
use crate::telnet::TelnetServer;
#[cfg(windows)]
use encoding_rs::GBK;
use std::io::{prelude::*, stdout};
use std::sync::mpsc;
use std::sync::Arc;
use std::{thread, time::Duration};

pub struct TerminalSerial<'a> {
    config: &'a AppConfig,
}

impl<'a> TerminalSerial<'a> {
    pub fn new(config: &'a AppConfig) -> TerminalSerial<'a> {
        TerminalSerial { config }
    }

    pub fn run(&self) {
        let manager = match SerialManager::open(
            &self.config.port,
            self.config.baud_rate,
            self.config.data_bits,
            self.config.parity,
            self.config.stop_bits,
            self.config.flow_control,
        ) {
            Ok(m) => m,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        println!(
            "{} is connected. Press 'Ctrl + ]' to quit.",
            manager.port_name()
        );

        if self.config.mcp {
            let mcp_server = McpServer::new(&manager);
            match mcp_server.start(&self.config.mcp_host, self.config.mcp_port) {
                Ok(()) => {
                    println!(
                        "MCP server listening on {}:{}",
                        self.config.mcp_host, self.config.mcp_port
                    );
                    println!("Press 'Ctrl + K' to clear MCP read buffer.");
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        }

        // Telnet 广播通道
        let telnet_tx = if self.config.telnet {
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            let telnet_server = TelnetServer::new(&manager);
            match telnet_server.start(&self.config.telnet_host, self.config.telnet_port, rx) {
                Ok(()) => {
                    println!(
                        "Telnet server listening on {}:{}",
                        self.config.telnet_host, self.config.telnet_port
                    );
                    Some(tx)
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        } else {
            None
        };

        let event_log: Option<Arc<EventLogWriter>> =
            self.config.event_log.as_ref().and_then(|path| {
                match EventLogWriter::new(path) {
                    Ok(w) => {
                        println!("Event logging to: {}", path);
                        w.log_startup(manager.port_name());
                        Some(Arc::new(w))
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to open event log file '{}': {}", path, e);
                        None
                    }
                }
            });

        let quit1 = manager.quit_flag();
        let quit2 = manager.quit_flag();
        let serial_port1 = manager.port();
        let serial_port2 = manager.port();
        let read_buffer = manager.read_buffer();
        let read_buffer2 = manager.read_buffer();
        let event_log_tx = event_log.clone();
        let event_log_rx = event_log.clone();
        let mut handles = vec![];

        // 输入线程：键盘 -> 串口
        handles.push(thread::spawn(move || {
            let input = Input::new();
            let mut gbk_pending: Vec<u8> = vec![];
            loop {
                if let Some(msg) = input.get_message() {
                    match msg {
                        InputMessage::Quit => {
                            quit1.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                        InputMessage::ClearBuffer => {
                            let (lock, _) = &*read_buffer2;
                            let mut buffer = lock.lock().unwrap();
                            buffer.clear();
                            drop(buffer);
                            print!("[Buffer cleared]\r\n");
                            let _ = stdout().flush();
                        }
                        InputMessage::Data(msg) => {
                            if let Some(text) = decode_gbk_input(&msg, &mut gbk_pending) {
                                if let Some(ref lw) = event_log_tx {
                                    lw.log_tx("local", text.as_bytes());
                                }
                                let mut serial_port = serial_port1.lock().unwrap();
                                let _ = serial_port.write_all(text.as_bytes());
                            }
                        }
                    }
                }
            }
        }));

        // 读取线程：串口 -> 终端 + MCP 缓冲区 + Telnet 广播
        handles.push(thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 2048];
            loop {
                thread::sleep(Duration::from_millis(10));

                let data = {
                    let mut serial_port = serial_port2.lock().unwrap();
                    match serial_port.read(&mut buf[..]) {
                        Ok(n) if n > 0 => Some(buf[0..n].to_vec()),
                        _ => None,
                    }
                };

                if let Some(data) = data {
                    print!("{}", String::from_utf8_lossy(&data));
                    let _ = stdout().flush();

                    if let Some(ref lw) = event_log_rx {
                        lw.log_rx("serial", &data);
                    }

                    // MCP 缓冲区写入
                    let (lock, cvar) = &*read_buffer;
                    let mut buffer = lock.lock().unwrap();
                    for &b in &data {
                        if buffer.len() >= 65536 {
                            buffer.pop_front();
                        }
                        buffer.push_back(b);
                    }
                    cvar.notify_one();

                    // Telnet 广播
                    if let Some(ref tx) = telnet_tx {
                        let _ = tx.send(data);
                    }
                }

                if quit2.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
            }
        }));

        handles.into_iter().for_each(|handle| {
            handle.join().unwrap();
        });

        if let Some(ref lw) = event_log {
            lw.log_shutdown();
        }
    }
}

#[cfg(windows)]
fn decode_gbk_input(msg: &[u8], pending: &mut Vec<u8>) -> Option<String> {
    let (utf8_string, _, _) = GBK.decode(msg);
    if !msg.is_empty() && msg[0] >= 0x81 && msg[0] <= 0xFE {
        pending.push(msg[0]);
        if pending.len() % 2 == 1 {
            return None;
        }
        let result = GBK.decode(pending).0.to_string();
        pending.clear();
        return Some(result);
    }
    pending.clear();
    Some(utf8_string.to_string())
}

#[cfg(not(windows))]
fn decode_gbk_input(msg: &[u8], _pending: &mut Vec<u8>) -> Option<String> {
    if msg.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(msg).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_gbk_input_ascii() {
        let mut pending = vec![];
        let result = decode_gbk_input(b"Hello", &mut pending);
        assert_eq!(result.as_deref(), Some("Hello"));
    }

    #[cfg(windows)]
    #[test]
    fn decode_gbk_input_empty_returns_some() {
        let mut pending = vec![];
        let result = decode_gbk_input(b"", &mut pending);
        assert_eq!(result.as_deref(), Some(""));
    }

    #[cfg(not(windows))]
    #[test]
    fn decode_gbk_input_empty_returns_none() {
        let mut pending = vec![];
        let result = decode_gbk_input(b"", &mut pending);
        assert!(result.is_none());
    }

    #[cfg(windows)]
    #[test]
    fn decode_gbk_input_chinese_two_calls() {
        let mut pending = vec![];
        // "中" in GBK = 0xD6 0xD0，需要逐字节处理
        let result1 = decode_gbk_input(&[0xD6], &mut pending);
        assert!(result1.is_none());
        assert_eq!(pending, vec![0xD6]);

        let result2 = decode_gbk_input(&[0xD0], &mut pending);
        assert!(result2.is_some());
        assert!(result2.unwrap().contains("中"));
        assert!(pending.is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn decode_gbk_input_utf8_passthrough() {
        let mut pending = vec![];
        let result = decode_gbk_input("你好".as_bytes(), &mut pending);
        assert_eq!(result.as_deref(), Some("你好"));
    }
}

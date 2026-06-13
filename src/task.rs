use crate::cmd::AppConfig;
use crate::event_log::EventLogWriter;
use crate::serial_manager::SerialManager;
use crate::server::McpServer;
use crate::telnet::TelnetServer;
use crate::{Input, InputMessage};
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
            println!(
                "MCP server listening on {}:{}",
                self.config.mcp_host, self.config.mcp_port
            );
            println!("Press 'Ctrl + K' to clear MCP read buffer.");
            let mcp_server = McpServer::new(&manager);
            mcp_server.start(&self.config.mcp_host, self.config.mcp_port);
        }

        // Telnet 广播通道
        let telnet_tx = if self.config.telnet {
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            let telnet_server = TelnetServer::new(&manager);
            telnet_server.start(&self.config.telnet_host, self.config.telnet_port, rx);
            println!(
                "Telnet server listening on {}:{}",
                self.config.telnet_host, self.config.telnet_port
            );
            Some(tx)
        } else {
            None
        };

        let event_log: Option<Arc<EventLogWriter>> =
            self.config.event_log.as_ref().map(|path| {
                match EventLogWriter::new(path) {
                    Ok(w) => {
                        println!("Event logging to: {}", path);
                        w.log_startup(manager.port_name());
                        Arc::new(w)
                    }
                    Err(e) => {
                        println!("Failed to open event log file '{}': {}", path, e);
                        std::process::exit(1);
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
                match input.get_message() {
                    InputMessage::Quit => {
                        let mut quit = quit1.lock().unwrap();
                        *quit = true;
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
                    _ => (),
                }
            }
        }));

        // 读取线程：串口 -> 终端 + MCP 缓冲区 + Telnet 广播
        handles.push(thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 2048];
            loop {
                thread::sleep(Duration::from_millis(2));

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

                let quit = quit2.lock().unwrap();
                if *quit {
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

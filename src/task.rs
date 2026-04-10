extern crate serial;

use crate::serial_manager::SerialManager;
use crate::server::McpServer;
use crate::{Input, InputMessage};
use encoding_rs::GBK;
use std::io::{prelude::*, stdout};
use std::{thread, time::Duration};

pub struct TerminalSerial<'a> {
    port: &'a str,
    setting: serial::PortSettings,
}

impl<'a> TerminalSerial<'a> {
    pub fn new(port: &'a str, setting: serial::PortSettings) -> TerminalSerial<'a> {
        TerminalSerial { port, setting }
    }

    pub fn run(&self, serve: bool, mcp_host: &str, mcp_port: u16) {
        let manager = match SerialManager::open(self.port, self.setting) {
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

        if serve {
            println!(
                "MCP server listening on {}:{}",
                mcp_host, mcp_port
            );
            println!("Press 'Ctrl + K' to clear MCP read buffer.");
            let mcp_server = McpServer::new(manager.read_buffer(), manager.port(), manager.quit_flag(), manager.port_name().to_string());
            mcp_server.start(mcp_host, mcp_port);
        }

        let quit1 = manager.quit_flag();
        let quit2 = manager.quit_flag();
        let serial_port1 = manager.port();
        let serial_port2 = manager.port();
        let read_buffer = manager.read_buffer();
        let read_buffer2 = manager.read_buffer();
        let mut handles = vec![];

        // 输入线程：键盘 -> 串口
        handles.push(thread::spawn(move || {
            let input = Input::new();
            let mut gbk_bytes: Vec<u8> = vec![];
            loop {
                match input.get_message() {
                    InputMessage::Quit => {
                        let mut quit = quit1.lock().unwrap();
                        *quit = true;
                        break;
                    }
                    InputMessage::ClearBuffer => {
                        let mut buffer = read_buffer2.lock().unwrap();
                        buffer.clear();
                        drop(buffer);
                        print!("[Buffer cleared]\r\n");
                        if let Ok(_) = stdout().flush() {}
                    }
                    InputMessage::Data(msg) => {
                        let (mut utf8_string, _, _) = GBK.decode(&msg);
                        if !msg.is_empty() && msg[0] > 0x80 && msg[0] < 0xFF {
                            gbk_bytes.push(msg[0]);
                            if gbk_bytes.len() % 2 == 1 {
                                continue;
                            } else {
                                (utf8_string, _, _) = GBK.decode(&gbk_bytes);
                            }
                        }
                        let mut serial_port = serial_port1.lock().unwrap();
                        if let Ok(_n) = serial_port.write(&utf8_string.as_bytes()) {}
                        gbk_bytes.clear();
                    }
                    _ => (),
                }
            }
        }));

        // 读取线程：串口 -> 终端 + MCP 缓冲区
        handles.push(thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 2048];
            loop {
                thread::sleep(Duration::from_millis(2));
                let mut serial_port = serial_port2.lock().unwrap();
                if let Ok(n) = serial_port.read(&mut buf[..]) {
                    if n > 0 {
                        print!("{}", String::from_utf8_lossy(&buf[0..n]));
                        if let Ok(_) = stdout().flush() {}

                        // 同时填充 MCP 读取缓冲区
                        {
                            let mut buffer = read_buffer.lock().unwrap();
                            for &b in &buf[0..n] {
                                if buffer.len() >= 65536 {
                                    buffer.pop_front();
                                }
                                buffer.push_back(b);
                            }
                        }
                    }
                };
                let quit = quit2.lock().unwrap();
                if *quit {
                    break;
                }
            }
        }));

        handles.into_iter().for_each(|handle| {
            handle.join().unwrap();
        });
    }
}

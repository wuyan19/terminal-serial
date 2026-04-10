use crate::serial_manager::SerialManager;
use crate::server::McpServer;
use crate::{Input, InputMessage};
use encoding_rs::GBK;
use std::io::{prelude::*, stdout};
use std::{thread, time::Duration};

pub struct TerminalSerial<'a> {
    port: &'a str,
    baud_rate: u32,
    data_bits: serialport::DataBits,
    parity: serialport::Parity,
    stop_bits: serialport::StopBits,
    flow_control: serialport::FlowControl,
}

impl<'a> TerminalSerial<'a> {
    pub fn new(
        port: &'a str,
        baud_rate: u32,
        data_bits: serialport::DataBits,
        parity: serialport::Parity,
        stop_bits: serialport::StopBits,
        flow_control: serialport::FlowControl,
    ) -> TerminalSerial<'a> {
        TerminalSerial {
            port,
            baud_rate,
            data_bits,
            parity,
            stop_bits,
            flow_control,
        }
    }

    pub fn run(&self, serve: bool, mcp_host: &str, mcp_port: u16) {
        let manager = match SerialManager::open(
            self.port,
            self.baud_rate,
            self.data_bits,
            self.parity,
            self.stop_bits,
            self.flow_control,
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

        if serve {
            println!(
                "MCP server listening on {}:{}",
                mcp_host, mcp_port
            );
            println!("Press 'Ctrl + K' to clear MCP read buffer.");
            let mcp_server = McpServer::new(
                manager.read_buffer(),
                manager.port(),
                manager.quit_flag(),
                manager.port_name().to_string(),
            );
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
                        if let Ok(_) = stdout().flush() {}
                    }
                    InputMessage::Data(msg) => {
                        if let Some(text) = decode_gbk_input(&msg, &mut gbk_pending) {
                            let mut serial_port = serial_port1.lock().unwrap();
                            if let Ok(_) = serial_port.write_all(text.as_bytes()) {}
                        }
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
                            let (lock, cvar) = &*read_buffer;
                            let mut buffer = lock.lock().unwrap();
                            for &b in &buf[0..n] {
                                if buffer.len() >= 65536 {
                                    buffer.pop_front();
                                }
                                buffer.push_back(b);
                            }
                            cvar.notify_one();
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

fn decode_gbk_input(msg: &[u8], pending: &mut Vec<u8>) -> Option<String> {
    let (utf8_string, _, _) = GBK.decode(msg);
    if !msg.is_empty() && msg[0] > 0x80 && msg[0] < 0xFF {
        pending.push(msg[0]);
        if pending.len() % 2 == 1 {
            return None; // 等待第二个字节
        }
        let result = GBK.decode(pending).0.to_string();
        pending.clear();
        return Some(result);
    }
    pending.clear();
    Some(utf8_string.to_string())
}

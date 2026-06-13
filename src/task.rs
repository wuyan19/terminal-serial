use crate::cmd::AppConfig;
use crate::config::Config;
use crate::event_log::EventLogWriter;
use crate::input::{Input, InputMessage};
use crate::macro_runner;
use crate::mcp_http::McpHttpServer;
use crate::serial_manager::SerialManager;
use crate::telnet::TelnetServer;
use crate::ui;
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

        // 先创建 event_log：startup 事件需先记录，MCP/Telnet 服务器也共享此 writer
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

        if self.config.mcp {
            let mcp_http_server = McpHttpServer::new(&manager, event_log.clone());
            match mcp_http_server.start(&self.config.mcp_host, self.config.mcp_port) {
                Ok(()) => {
                    println!(
                        "MCP server listening on {}:{}",
                        self.config.mcp_host, self.config.mcp_port
                    );
                    println!("Press 'Ctrl + K' to clear RX buffer.");
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
            let telnet_server = TelnetServer::new(&manager, event_log.clone());
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

        let macro_config: Arc<Config> = match self.config.config_file.as_ref() {
            Some(path) => match Config::load(path) {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    eprintln!("Warning: Failed to load config '{}': {}", path, e);
                    Arc::new(Config::empty())
                }
            },
            None => Arc::new(Config::empty()),
        };

        if !macro_config.macros.is_empty() {
            print_macro_hint(&macro_config);
        }

        let quit1 = manager.quit_flag();
        let quit2 = manager.quit_flag();
        let serial_port1 = manager.port();
        let serial_port2 = manager.port();
        let read_buffer = manager.read_buffer();
        let read_buffer2 = manager.read_buffer();
        let event_log_tx = event_log.clone();
        let event_log_rx = event_log.clone();
        let macro_manager = manager.clone();
        let macro_cfg = Arc::clone(&macro_config);
        let mut handles = vec![];

        // 输入线程：键盘 -> 串口
        handles.push(thread::spawn(move || {
            let mut input = Input::new();
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
                            if let Some(ref lw) = event_log_tx {
                                lw.log_action("local", "clear_buffer", None);
                            }
                            ui::success("Buffer cleared");
                        }
                        InputMessage::ShowMacroMenu => {
                            if macro_cfg.macros.is_empty() {
                                ui::hint("No macros defined");
                                input.cancel_menu_mode();
                            } else {
                                print_macro_menu(&macro_cfg);
                            }
                        }
                        InputMessage::RunMacro(idx) => {
                            let log_ref = event_log_tx.as_ref().map(|arc| arc.as_ref());
                            run_macro_by_index(idx, &macro_cfg, &macro_manager, log_ref);
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

        // 读取线程：串口 -> 终端 + 接收缓冲区 + Telnet 广播
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

                    // 接收缓冲区写入
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

/// 启动时若配置含宏，print 一行紧凑提示（不超过 9 个）。
fn print_macro_hint(config: &Config) {
    let names: Vec<String> = config
        .macros
        .iter()
        .take(9)
        .enumerate()
        .map(|(i, (name, _))| format!("[{}]{}", i + 1, name))
        .collect();
    if names.is_empty() {
        return;
    }
    println!("Macros: {} (Ctrl+O for menu)", names.join(" "));
}

/// Ctrl+O 触发的完整宏菜单（青色 ANSI），进入 MenuSelect 模式等待数字选择。
fn print_macro_menu(config: &Config) {
    ui::heading("Macros");
    for (i, (name, mac)) in config.macros.iter().take(9).enumerate() {
        let desc = mac
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();
        println!("[{}] {}{}", i + 1, name, desc);
    }
    ui::hint("Press 1-9 to run, any key to exit");
}

/// 按 1-indexed 序号执行宏。结果以 ANSI 颜色反馈：青色 ▶ 开始，绿色 ✓ 成功，红色 ✗ 失败。
fn run_macro_by_index<S: crate::serial_manager::SerialOps>(
    idx: usize,
    config: &Config,
    manager: &S,
    event_log: Option<&EventLogWriter>,
) {
    if idx == 0 || idx > 9 {
        ui::fail(&format!("macro index {} out of range", idx));
        return;
    }
    match config.macro_at_index(idx - 1) {
        Some((name, mac)) => {
            ui::info(&format!("macro: {}", name));
            match macro_runner::run_macro(name, mac, manager, event_log, "local") {
                Ok(()) => {
                    ui::success(&format!("macro: {} done", name));
                }
                Err(e) => {
                    ui::fail(&format!("macro: {} failed: {}", name, e));
                }
            }
        }
        None => {
            ui::fail(&format!("macro index {} out of range", idx));
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

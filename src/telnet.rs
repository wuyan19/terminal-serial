use crate::event_log::EventLogWriter;
use crate::serial_manager::SerialManager;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// Telnet 协议常量
const IAC: u8 = 255;
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;
const SB: u8 = 250;
const SE: u8 = 240;

const OPT_ECHO: u8 = 1;
const OPT_SUPPRESS_GA: u8 = 3;

// 连接握手：WONT ECHO + WILL SUPPRESS-GA
// WILL ECHO: 告诉客户端服务器负责回显，禁止本地回显（串口对端设备会回显）
// WILL SUPPRESS-GA: 抑制 Go-Ahead，避免不必要的延迟
const HANDSHAKE: &[u8] = &[IAC, WILL, OPT_ECHO, IAC, WILL, OPT_SUPPRESS_GA];

pub struct TelnetServer {
    clients: Arc<Mutex<Vec<TcpStream>>>,
    serial_port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    quit: Arc<AtomicBool>,
    event_log: Option<Arc<EventLogWriter>>,
}

impl TelnetServer {
    pub fn new(
        manager: &SerialManager,
        event_log: Option<Arc<EventLogWriter>>,
    ) -> Self {
        TelnetServer {
            clients: Arc::new(Mutex::new(Vec::new())),
            serial_port: manager.port(),
            quit: manager.quit_flag(),
            event_log,
        }
    }

    pub fn start(&self, host: &str, port: u16, broadcast_rx: mpsc::Receiver<Vec<u8>>) -> Result<(), String> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).map_err(|e| {
            format!("Failed to bind telnet server on {}: {}", addr, e)
        })?;
        listener
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");

        // 广播线程：串口数据 → 所有 telnet 客户端
        let clients_bc = Arc::clone(&self.clients);
        let quit_bc = Arc::clone(&self.quit);
        thread::spawn(move || {
            loop {
                match broadcast_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    Ok(data) => {
                        let mut clients = clients_bc.lock().unwrap();
                        clients.retain(|mut stream| stream.write_all(&data).is_ok());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                if quit_bc.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        // 接受线程：监听新连接
        let clients_acc = Arc::clone(&self.clients);
        let serial_port_acc = Arc::clone(&self.serial_port);
        let quit_acc = Arc::clone(&self.quit);
        let event_log_acc = self.event_log.clone();
        thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.write_all(HANDSHAKE);
                        let _ = stream.flush();

                        // 先 clone 给读取线程，再放入 clients 列表
                        let client_stream = match stream.try_clone() {
                            Ok(s) => s,
                            Err(e) => {
                                if let Some(ref log) = event_log_acc {
                                    log.log_error(&format!("Telnet clone failed: {}", e));
                                }
                                continue;
                            }
                        };

                        {
                            let mut clients = clients_acc.lock().unwrap();
                            clients.push(stream);
                        }
                        if let Some(ref log) = event_log_acc {
                            log.log_client_connected("telnet", &addr.to_string());
                        }

                        let serial_port = Arc::clone(&serial_port_acc);
                        let quit = Arc::clone(&quit_acc);
                        let event_log_reader = event_log_acc.clone();
                        let client_addr = addr.to_string();
                        thread::spawn(move || {
                            client_reader(client_stream, serial_port, quit);
                            if let Some(ref log) = event_log_reader {
                                log.log_client_disconnected("telnet", &client_addr);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(e) => {
                        if let Some(ref log) = event_log_acc {
                            log.log_error(&format!("Telnet accept error: {}", e));
                        }
                    }
                }

                if quit_acc.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Ok(())
    }
}

/// 客户端读取线程：过滤 telnet 协议命令后将有效数据写入串口
fn client_reader(
    mut stream: TcpStream,
    serial_port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    quit: Arc<AtomicBool>,
) {
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let (payload, negotiation) = filter_telnet_commands(&buf[..n]);
                if !negotiation.is_empty() {
                    let _ = stream.write_all(&negotiation);
                    let _ = stream.flush();
                }
                if !payload.is_empty() {
                    // 剥离 NUL：telnet 回车发送 \r\0（CR NUL 表示纯回车），
                    // 不剥离会导致 \0 残留在 tty 缓冲区，污染 getty/login 阶段的下一行输入
                    let filtered: Vec<u8> =
                        payload.into_iter().filter(|&b| b != 0).collect();
                    if !filtered.is_empty() {
                        if let Ok(mut port) = serial_port.lock() {
                            let _ = port.write_all(&filtered);
                        }
                    }
                }
            }
            Err(_) => break,
        }

        if quit.load(Ordering::Relaxed) {
            break;
        }
    }
}

/// 过滤 telnet 协议命令，返回 (纯数据, 协商响应)
fn filter_telnet_commands(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut payload = Vec::new();
    let mut response = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] == IAC {
            if i + 1 >= data.len() {
                break;
            }
            match data[i + 1] {
                IAC => {
                    payload.push(IAC);
                    i += 2;
                }
                WILL | WONT => {
                    if i + 2 < data.len() {
                        response.extend_from_slice(&[IAC, DO, data[i + 2]]);
                    }
                    i += 3;
                }
                DO => {
                    if i + 2 < data.len() {
                        if data[i + 2] == OPT_ECHO || data[i + 2] == OPT_SUPPRESS_GA {
                            // 我们主动 WILL 过的选项，客户端同意了，不需要额外回复
                        } else {
                            response.extend_from_slice(&[IAC, WONT, data[i + 2]]);
                        }
                    }
                    i += 3;
                }
                DONT => {
                    if i + 2 < data.len() {
                        // 客户端要求我们停止某选项，确认 WONT
                        response.extend_from_slice(&[IAC, WONT, data[i + 2]]);
                    }
                    i += 3;
                }
                SB => {
                    i += 2;
                    while i + 1 < data.len() {
                        if data[i] == IAC && data[i + 1] == SE {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {
                    i += 2;
                }
            }
        } else {
            payload.push(data[i]);
            i += 1;
        }
    }

    (payload, response)
}

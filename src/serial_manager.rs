use crate::error::SerialError;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const MCP_BUFFER_MAX: usize = 65536;

pub struct SerialStatus {
    pub port_name: String,
    pub baud_rate: u32,
    pub char_size: String,
    pub parity: String,
    pub stop_bits: String,
    pub flow_control: String,
    pub is_open: bool,
}

pub struct SerialManager {
    port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    port_name: String,
    read_buffer: Arc<(Mutex<VecDeque<u8>>, Condvar)>,
    quit: Arc<AtomicBool>,
}

impl SerialManager {
    pub fn from_parts(
        port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
        read_buffer: Arc<(Mutex<VecDeque<u8>>, Condvar)>,
        port_name: String,
    ) -> SerialManager {
        SerialManager {
            port,
            port_name,
            read_buffer,
            quit: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn open(
        port_name: &str,
        baud_rate: u32,
        data_bits: serialport::DataBits,
        parity: serialport::Parity,
        stop_bits: serialport::StopBits,
        flow_control: serialport::FlowControl,
    ) -> Result<SerialManager, SerialError> {
        let port = serialport::new(port_name, baud_rate)
            .data_bits(data_bits)
            .parity(parity)
            .stop_bits(stop_bits)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(1))
            .open()
            .map_err(|e| SerialError::PortOpen {
                port: port_name.to_string(),
                source: e,
            })?;

        Ok(SerialManager {
            port: Arc::new(Mutex::new(port)),
            port_name: port_name.to_string(),
            read_buffer: Arc::new((
                Mutex::new(VecDeque::with_capacity(MCP_BUFFER_MAX)),
                Condvar::new(),
            )),
            quit: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn port(&self) -> Arc<Mutex<Box<dyn serialport::SerialPort>>> {
        Arc::clone(&self.port)
    }

    pub fn read_buffer(&self) -> Arc<(Mutex<VecDeque<u8>>, Condvar)> {
        Arc::clone(&self.read_buffer)
    }

    pub fn quit_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.quit)
    }

    pub fn send(&self, data: &[u8]) -> Result<usize, SerialError> {
        let mut port = self.port.lock().unwrap();
        port.write_all(data).map_err(SerialError::Write)?;
        Ok(data.len())
    }

    pub fn read_serial(&self, buf: &mut [u8]) -> Result<usize, SerialError> {
        let mut port = self.port.lock().unwrap();
        port.read(buf).map_err(SerialError::Read)
    }

    pub fn push_to_buffer(&self, data: &[u8]) {
        let (lock, cvar) = &*self.read_buffer;
        let mut buffer = lock.lock().unwrap();
        for &b in data {
            if buffer.len() >= MCP_BUFFER_MAX {
                buffer.pop_front();
            }
            buffer.push_back(b);
        }
        cvar.notify_one();
    }

    pub fn drain_buffer(&self, timeout_ms: u32) -> Vec<u8> {
        let (lock, cvar) = &*self.read_buffer;
        let mut buffer = lock.lock().unwrap();
        if timeout_ms > 0 && buffer.is_empty() {
            let guard = cvar
                .wait_timeout(buffer, Duration::from_millis(timeout_ms as u64))
                .unwrap_or_else(|e| e.into_inner());
            buffer = guard.0;
        }
        buffer.drain(..).collect()
    }

    pub fn clear_buffer(&self) {
        let (lock, _) = &*self.read_buffer;
        let mut buffer = lock.lock().unwrap();
        buffer.clear();
    }

    /// 非破坏性读取：返回缓冲区内容的副本，不修改缓冲区
    pub fn peek_buffer(&self) -> Vec<u8> {
        let (lock, _) = &*self.read_buffer;
        let buffer = lock.lock().unwrap();
        buffer.iter().copied().collect()
    }

    /// 在缓冲区中搜索匹配模式的数据行，不修改缓冲区。
    /// timeout_ms > 0 时等待直到匹配或超时；timeout_ms = 0 时立即返回当前结果。
    pub fn grep_buffer(
        &self,
        pattern: &str,
        timeout_ms: u32,
    ) -> Result<Vec<String>, SerialError> {
        let re = regex::Regex::new(pattern).map_err(SerialError::Regex)?;
        let (lock, cvar) = &*self.read_buffer;

        let deadline =
            std::time::Instant::now() + Duration::from_millis(timeout_ms as u64);

        loop {
            let buffer = lock.lock().unwrap();
            let bytes: Vec<u8> = buffer.iter().copied().collect();
            let text = String::from_utf8_lossy(&bytes);
            let lines: Vec<String> = text
                .lines()
                .filter(|line| re.is_match(line))
                .map(|s| s.to_string())
                .collect();

            if !lines.is_empty() {
                return Ok(lines);
            }

            if timeout_ms == 0 {
                return Ok(vec![]);
            }

            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Ok(vec![]);
            }

            let guard = cvar
                .wait_timeout(buffer, remaining)
                .unwrap_or_else(|e| e.into_inner());
            drop(guard.0);
        }
    }

    /// 在缓冲区中搜索字节序列，不修改缓冲区。
    /// timeout_ms > 0 时等待直到匹配或超时；timeout_ms = 0 时立即返回当前结果。
    /// 返回所有匹配位置及其上下文（前后各 16 字节）。
    pub fn grep_buffer_bytes(
        &self,
        pattern: &[u8],
        timeout_ms: u32,
    ) -> Vec<(usize, Vec<u8>)> {
        let (lock, cvar) = &*self.read_buffer;
        let deadline =
            std::time::Instant::now() + Duration::from_millis(timeout_ms as u64);
        const CONTEXT: usize = 16;

        loop {
            let buffer = lock.lock().unwrap();
            let bytes: Vec<u8> = buffer.iter().copied().collect();

            let matches: Vec<(usize, Vec<u8>)> = bytes
                .windows(pattern.len())
                .enumerate()
                .filter(|(_, window)| *window == pattern)
                .map(|(pos, _)| {
                    let start = pos.saturating_sub(CONTEXT);
                    let end = (pos + pattern.len() + CONTEXT).min(bytes.len());
                    (pos, bytes[start..end].to_vec())
                })
                .collect();

            if !matches.is_empty() {
                return matches;
            }

            if timeout_ms == 0 {
                return vec![];
            }

            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return vec![];
            }

            let guard = cvar
                .wait_timeout(buffer, remaining)
                .unwrap_or_else(|e| e.into_inner());
            drop(guard.0);
        }
    }

    pub fn status(&self) -> SerialStatus {
        let port = self.port.lock().unwrap();

        let baud_rate = port.baud_rate().unwrap_or(0);
        let char_size = match port.data_bits() {
            Ok(serialport::DataBits::Five) => "5",
            Ok(serialport::DataBits::Six) => "6",
            Ok(serialport::DataBits::Seven) => "7",
            _ => "8",
        };
        let parity = match port.parity() {
            Ok(serialport::Parity::None) => "None",
            Ok(serialport::Parity::Odd) => "Odd",
            Ok(serialport::Parity::Even) => "Even",
            _ => "None",
        };
        let stop_bits = match port.stop_bits() {
            Ok(serialport::StopBits::One) => "1",
            Ok(serialport::StopBits::Two) => "2",
            _ => "1",
        };
        let flow_control = match port.flow_control() {
            Ok(serialport::FlowControl::None) => "None",
            Ok(serialport::FlowControl::Software) => "Software",
            Ok(serialport::FlowControl::Hardware) => "Hardware",
            _ => "None",
        };

        SerialStatus {
            port_name: self.port_name.clone(),
            baud_rate,
            char_size: char_size.to_string(),
            parity: parity.to_string(),
            stop_bits: stop_bits.to_string(),
            flow_control: flow_control.to_string(),
            is_open: !self.is_quit(),
        }
    }

    pub fn is_quit(&self) -> bool {
        self.quit.load(Ordering::Relaxed)
    }

    pub fn set_quit(&self) {
        self.quit.store(true, Ordering::Relaxed);
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

extern crate serial;

use serial::SerialPort;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MCP_BUFFER_MAX: usize = 65536;

pub struct SerialStatus {
    pub port_name: String,
    pub baud_rate: usize,
    pub char_size: String,
    pub parity: String,
    pub stop_bits: String,
    pub flow_control: String,
    pub is_open: bool,
}

pub struct SerialManager {
    port: Arc<Mutex<Box<dyn serial::SerialPort + Send>>>,
    port_name: String,
    setting: serial::PortSettings,
    read_buffer: Arc<Mutex<VecDeque<u8>>>,
    quit: Arc<Mutex<bool>>,
}

impl SerialManager {
    pub fn from_parts(
        port: Arc<Mutex<Box<dyn serial::SerialPort + Send>>>,
        read_buffer: Arc<Mutex<VecDeque<u8>>>,
        port_name: String,
    ) -> SerialManager {
        SerialManager {
            port,
            port_name,
            setting: serial::PortSettings {
                baud_rate: serial::Baud115200,
                char_size: serial::Bits8,
                parity: serial::ParityNone,
                stop_bits: serial::Stop1,
                flow_control: serial::FlowNone,
            },
            read_buffer,
            quit: Arc::new(Mutex::new(false)),
        }
    }

    pub fn open(
        port_name: &str,
        setting: serial::PortSettings,
    ) -> Result<SerialManager, String> {
        let mut com_port =
            serial::open(port_name).map_err(|e| format!("Failed to open {}: {}", port_name, e))?;

        com_port
            .configure(&setting)
            .map_err(|e| format!("Failed to configure {}: {}", port_name, e))?;

        com_port
            .set_timeout(Duration::from_millis(1))
            .map_err(|e| format!("Failed to set timeout for {}: {}", port_name, e))?;

        Ok(SerialManager {
            port: Arc::new(Mutex::new(Box::new(com_port))),
            port_name: port_name.to_string(),
            setting,
            read_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MCP_BUFFER_MAX))),
            quit: Arc::new(Mutex::new(false)),
        })
    }

    pub fn port(&self) -> Arc<Mutex<Box<dyn serial::SerialPort + Send>>> {
        Arc::clone(&self.port)
    }

    pub fn read_buffer(&self) -> Arc<Mutex<VecDeque<u8>>> {
        Arc::clone(&self.read_buffer)
    }

    pub fn quit_flag(&self) -> Arc<Mutex<bool>> {
        Arc::clone(&self.quit)
    }

    pub fn send(&self, data: &[u8]) -> Result<usize, String> {
        let mut port = self.port.lock().unwrap();
        port.write(data).map_err(|e| format!("Write error: {}", e))
    }

    pub fn read_serial(&self, buf: &mut [u8]) -> Result<usize, String> {
        let mut port = self.port.lock().unwrap();
        port.read(buf).map_err(|e| format!("Read error: {}", e))
    }

    pub fn push_to_buffer(&self, data: &[u8]) {
        let mut buffer = self.read_buffer.lock().unwrap();
        for &b in data {
            if buffer.len() >= MCP_BUFFER_MAX {
                buffer.pop_front();
            }
            buffer.push_back(b);
        }
    }

    pub fn drain_buffer(&self, timeout_ms: u32) -> Vec<u8> {
        if timeout_ms > 0 {
            let start = std::time::Instant::now();
            loop {
                {
                    let buffer = self.read_buffer.lock().unwrap();
                    if !buffer.is_empty() {
                        break;
                    }
                }
                if start.elapsed().as_millis() as u32 >= timeout_ms {
                    break;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }

        let mut buffer = self.read_buffer.lock().unwrap();
        buffer.drain(..).collect()
    }

    pub fn clear_buffer(&self) {
        let mut buffer = self.read_buffer.lock().unwrap();
        buffer.clear();
    }

    pub fn status(&self) -> SerialStatus {
        let char_size = match self.setting.char_size {
            serial::CharSize::Bits5 => "5",
            serial::CharSize::Bits6 => "6",
            serial::CharSize::Bits7 => "7",
            serial::CharSize::Bits8 => "8",
        };
        let parity = match self.setting.parity {
            serial::Parity::ParityNone => "None",
            serial::Parity::ParityOdd => "Odd",
            serial::Parity::ParityEven => "Even",
        };
        let stop_bits = match self.setting.stop_bits {
            serial::StopBits::Stop1 => "1",
            serial::StopBits::Stop2 => "2",
        };
        let flow_control = match self.setting.flow_control {
            serial::FlowControl::FlowNone => "None",
            serial::FlowControl::FlowSoftware => "Software",
            serial::FlowControl::FlowHardware => "Hardware",
        };

        SerialStatus {
            port_name: self.port_name.clone(),
            baud_rate: self.setting.baud_rate.speed(),
            char_size: char_size.to_string(),
            parity: parity.to_string(),
            stop_bits: stop_bits.to_string(),
            flow_control: flow_control.to_string(),
            is_open: !self.is_quit(),
        }
    }

    pub fn is_quit(&self) -> bool {
        *self.quit.lock().unwrap()
    }

    pub fn set_quit(&self) {
        *self.quit.lock().unwrap() = true;
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

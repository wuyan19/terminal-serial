use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub enum SessionLogDirection {
    Tx,
    Rx,
}

pub struct SessionLogWriter {
    file: Mutex<std::fs::File>,
}

impl SessionLogWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(SessionLogWriter {
            file: Mutex::new(file),
        })
    }

    pub fn log(&self, direction: SessionLogDirection, data: &[u8]) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let dir_str = match direction {
            SessionLogDirection::Tx => "TX",
            SessionLogDirection::Rx => "RX",
        };
        let hex: String = data.iter().map(|b| format!("{:02X} ", b)).collect();
        let text = String::from_utf8_lossy(data);
        let line = format!("[{}] {}: {}| {}\n", timestamp, dir_str, hex.trim_end(), text);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

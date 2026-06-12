use chrono::Utc;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct EventLogWriter {
    file: Mutex<std::fs::File>,
}

impl EventLogWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(EventLogWriter {
            file: Mutex::new(file),
        })
    }

    fn write_event(&self, event: serde_json::Value) {
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file, "{}", event);
            let _ = file.flush();
        }
    }

    pub fn log_startup(&self, port: &str) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "startup",
            "port": port,
        }));
    }

    pub fn log_shutdown(&self) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "shutdown",
        }));
    }

    pub fn log_client_connected(&self, source: &str, client: &str) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "client_connected",
            "source": source,
            "client": client,
        }));
    }

    pub fn log_client_disconnected(&self, source: &str, client: &str) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "client_disconnected",
            "source": source,
            "client": client,
        }));
    }

    pub fn log_tx(&self, source: &str, data: &[u8]) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "tx",
            "source": source,
            "data": hex_encode(data),
        }));
    }

    pub fn log_rx(&self, data: &[u8]) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "rx",
            "data": hex_encode(data),
        }));
    }

    pub fn log_error(&self, message: &str) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "error",
            "message": message,
        }));
    }
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

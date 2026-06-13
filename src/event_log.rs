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

    pub fn log_rx(&self, source: &str, data: &[u8]) {
        self.write_event(json!({
            "ts": Utc::now().to_rfc3339(),
            "event": "rx",
            "source": source,
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

pub(crate) fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

pub(crate) fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() {
        return Some(Vec::new());
    }
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn hex_encode_single_byte() {
        assert_eq!(hex_encode(&[0x41]), "41");
    }

    #[test]
    fn hex_encode_multi_byte() {
        assert_eq!(hex_encode(&[0x0D, 0x0A, 0xFF]), "0D0AFF");
    }

    #[test]
    fn hex_decode_empty() {
        assert_eq!(hex_decode(""), Some(vec![]));
    }

    #[test]
    fn hex_decode_valid() {
        assert_eq!(hex_decode("48656C6C6F"), Some(b"Hello".to_vec()));
    }

    #[test]
    fn hex_decode_odd_length() {
        assert_eq!(hex_decode("ABC"), None);
    }

    #[test]
    fn hex_decode_mixed_case() {
        assert_eq!(hex_decode("0d0a"), Some(vec![0x0D, 0x0A]));
    }
}

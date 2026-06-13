use crate::config::{Macro, MacroStep};
use crate::event_log::EventLogWriter;
use crate::serial_manager::SerialOps;
use crate::util::hex_decode;
use std::thread;
use std::time::Duration;

/// 按序执行宏的所有步骤。任何一步失败立即返回错误。
/// 成功执行后返回 Ok(())。事件日志只记录一次整体的 run_macro 调用，
/// 不细到每步（避免日志噪音；后续如需步骤级追踪再加 log_action_step）。
pub fn run_macro<S: SerialOps>(
    name: &str,
    mac: &Macro,
    manager: &S,
    event_log: Option<&EventLogWriter>,
    source: &str,
) -> Result<(), String> {
    if let Some(log) = event_log {
        log.log_action(source, "run_macro", Some(name));
    }

    for (i, step) in mac.steps.iter().enumerate() {
        match step {
            MacroStep::Send {
                data,
                format,
                auto_newline,
            } => {
                let mut bytes = encode_data(data, format.as_deref())?;
                if auto_newline.unwrap_or(true) {
                    bytes.extend_from_slice(b"\r\n");
                }
                manager
                    .send(&bytes)
                    .map_err(|e| format!("step {}: send failed: {}", i + 1, e))?;
            }
            MacroStep::Delay { ms } => {
                thread::sleep(Duration::from_millis(*ms));
            }
            MacroStep::Expect {
                pattern,
                timeout_ms,
            } => {
                let matched = manager
                    .grep_buffer(pattern, *timeout_ms)
                    .map_err(|e| format!("step {}: expect regex error: {}", i + 1, e))?;
                if matched.is_empty() {
                    return Err(format!("step {}: expect '{}' timeout", i + 1, pattern));
                }
            }
            MacroStep::Clear => {
                manager.clear_buffer();
            }
        }
    }
    Ok(())
}

fn encode_data(data: &str, format: Option<&str>) -> Result<Vec<u8>, String> {
    match format.unwrap_or("text") {
        "hex" => hex_decode(data).ok_or_else(|| format!("invalid hex: {}", data)),
        "raw" => Ok(data.bytes().collect()),
        "text" => Ok(data.as_bytes().to_vec()),
        other => Err(format!("unknown format: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial_manager::SerialStatus;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct MockSerial {
        sent: RefCell<Vec<Vec<u8>>>,
        clear_count: Cell<usize>,
        grep_result: RefCell<Vec<String>>,
    }

    impl MockSerial {
        fn with_grep(lines: Vec<&str>) -> Self {
            MockSerial {
                grep_result: RefCell::new(lines.into_iter().map(String::from).collect()),
                ..Default::default()
            }
        }
    }

    impl SerialOps for MockSerial {
        fn send(&self, data: &[u8]) -> Result<usize, crate::error::SerialError> {
            self.sent.borrow_mut().push(data.to_vec());
            Ok(data.len())
        }
        fn drain_buffer(&self, _timeout_ms: u32) -> Vec<u8> {
            vec![]
        }
        fn clear_buffer(&self) {
            self.clear_count.set(self.clear_count.get() + 1);
        }
        fn grep_buffer(
            &self,
            _pattern: &str,
            _timeout_ms: u32,
        ) -> Result<Vec<String>, crate::error::SerialError> {
            Ok(self.grep_result.borrow().clone())
        }
        fn grep_buffer_bytes(
            &self,
            _pattern: &[u8],
            _timeout_ms: u32,
        ) -> Vec<(usize, Vec<u8>)> {
            vec![]
        }
        fn status(&self) -> SerialStatus {
            SerialStatus {
                port_name: String::new(),
                baud_rate: 0,
                char_size: "8".into(),
                parity: "None".into(),
                stop_bits: "1".into(),
                flow_control: "None".into(),
                is_open: true,
            }
        }
    }

    fn macro_with_steps(steps: Vec<MacroStep>) -> Macro {
        Macro {
            description: None,
            steps,
        }
    }

    #[test]
    fn run_empty_macro_returns_ok() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![]);
        assert!(run_macro("test", &mac, &m, None, "local").is_ok());
    }

    #[test]
    fn run_send_text_step() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "AT".into(),
            format: None,
            auto_newline: Some(false),
        }]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow()[0], b"AT");
    }

    #[test]
    fn run_send_hex_step() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "0D0A".into(),
            format: Some("hex".into()),
            auto_newline: Some(false),
        }]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow()[0], vec![0x0D, 0x0A]);
    }

    #[test]
    fn run_send_raw_step() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "AB".into(),
            format: Some("raw".into()),
            auto_newline: Some(false),
        }]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow()[0], vec![b'A', b'B']);
    }

    #[test]
    fn run_send_auto_newline_default_true() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "AT".into(),
            format: None,
            auto_newline: None,
        }]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow()[0], b"AT\r\n");
    }

    #[test]
    fn run_send_auto_newline_false() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "AT".into(),
            format: None,
            auto_newline: Some(false),
        }]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow()[0], b"AT");
    }

    #[test]
    fn run_send_invalid_hex_returns_err() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "XYZ".into(),
            format: Some("hex".into()),
            auto_newline: Some(false),
        }]);
        assert!(run_macro("test", &mac, &m, None, "local").is_err());
    }

    #[test]
    fn run_send_unknown_format_returns_err() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Send {
            data: "AT".into(),
            format: Some("binary".into()),
            auto_newline: Some(false),
        }]);
        assert!(run_macro("test", &mac, &m, None, "local").is_err());
    }

    #[test]
    fn run_delay_step() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Delay { ms: 5 }]);
        let start = std::time::Instant::now();
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert!(start.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn run_expect_match() {
        let m = MockSerial::with_grep(vec!["OK"]);
        let mac = macro_with_steps(vec![MacroStep::Expect {
            pattern: "OK".into(),
            timeout_ms: 100,
        }]);
        assert!(run_macro("test", &mac, &m, None, "local").is_ok());
    }

    #[test]
    fn run_expect_timeout_returns_err() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Expect {
            pattern: "OK".into(),
            timeout_ms: 100,
        }]);
        let result = run_macro("test", &mac, &m, None, "local");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }

    #[test]
    fn run_clear_step() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![MacroStep::Clear]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.clear_count.get(), 1);
    }

    #[test]
    fn run_multiple_steps_in_order() {
        let m = MockSerial::default();
        let mac = macro_with_steps(vec![
            MacroStep::Send {
                data: "A".into(),
                format: None,
                auto_newline: Some(false),
            },
            MacroStep::Clear,
            MacroStep::Send {
                data: "B".into(),
                format: None,
                auto_newline: Some(false),
            },
        ]);
        run_macro("test", &mac, &m, None, "local").unwrap();
        assert_eq!(m.sent.borrow().len(), 2);
        assert_eq!(m.sent.borrow()[0], b"A");
        assert_eq!(m.sent.borrow()[1], b"B");
        assert_eq!(m.clear_count.get(), 1);
    }
}

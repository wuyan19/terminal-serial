use crate::cmd::{LogConfig, OutputFormat};
use crate::util::{hex_decode, hex_encode};
use chrono::DateTime;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::io::Write;

#[derive(Clone)]
pub struct LogEvent {
    pub timestamp: String,
    pub event_type: String,
    pub source: Option<String>,
    pub data: Option<Vec<u8>>,
    pub message: Option<String>,
    pub port: Option<String>,
    pub client: Option<String>,
    pub action: Option<String>,
    pub name: Option<String>,
    pub raw_json: Value,
}

struct SessionSummary {
    start_time: Option<String>,
    end_time: Option<String>,
    duration: Option<String>,
    port: Option<String>,
    total_events: usize,
    tx_count: usize,
    rx_count: usize,
    tx_bytes: usize,
    rx_bytes: usize,
    error_count: usize,
    action_count: usize,
    clients: Vec<String>,
}

fn parse_log_event(line: &str) -> Option<LogEvent> {
    let value: Value = serde_json::from_str(line).ok()?;

    let event_type = value.get("event")?.as_str()?.to_string();
    let timestamp = value
        .get("ts")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let source = value.get("source").and_then(|v| v.as_str()).map(String::from);
    let data = value
        .get("data")
        .and_then(|v| v.as_str())
        .and_then(hex_decode);
    let message = value
        .get("message")
        .and_then(|v| v.as_str())
        .map(String::from);
    let port = value
        .get("port")
        .and_then(|v| v.as_str())
        .map(String::from);
    let client = value
        .get("client")
        .and_then(|v| v.as_str())
        .map(String::from);
    let action = value
        .get("action")
        .and_then(|v| v.as_str())
        .map(String::from);
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(LogEvent {
        timestamp,
        event_type,
        source,
        data,
        message,
        port,
        client,
        action,
        name,
        raw_json: value,
    })
}

fn read_log_file(path: &str) -> Result<Vec<LogEvent>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("无法读取文件 '{}': {}", path, e))?;
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_log_event(line))
        .collect())
}

fn decode_event_text(event: &LogEvent) -> String {
    if let Some(ref data) = event.data {
        match std::str::from_utf8(data) {
            Ok(s) => {
                let printable: String = s
                    .chars()
                    .map(|c| if c.is_control() && c != '\n' && c != '\r' { ' ' } else { c })
                    .collect();
                printable.trim_end().to_string()
            }
            Err(_) => format!("<binary: {} bytes>", data.len()),
        }
    } else if event.event_type == "action" {
        let action = event.action.as_deref().unwrap_or("");
        match event.name.as_deref() {
            Some(n) if !n.is_empty() => format!("{} {}", action, n),
            _ => action.to_string(),
        }
    } else {
        event
            .message
            .as_deref()
            .or(event.port.as_deref())
            .or(event.client.as_deref())
            .unwrap_or("")
            .to_string()
    }
}

fn filter_events(events: &[LogEvent], config: &LogConfig) -> Vec<LogEvent> {
    let grep_re = config.grep.as_ref().and_then(|p| Regex::new(p).ok());

    events
        .iter()
        .filter(|e| {
            if let Some(ref event_type) = config.event {
                if e.event_type != *event_type {
                    return false;
                }
            }
            if let Some(ref source) = config.source {
                if e.source.as_deref() != Some(source.as_str()) {
                    return false;
                }
            }
            if let Some(ref pattern) = grep_re {
                let text = decode_event_text(e);
                if !pattern.is_match(&text) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

fn format_timestamp(ts: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        dt.format("%H:%M:%S%.3f").to_string()
    } else {
        ts.to_string()
    }
}

fn format_text(events: &[LogEvent], raw: bool) -> String {
    let mut out = String::new();
    for e in events {
        match e.event_type.as_str() {
            "tx" | "rx" => {
                if raw {
                    if let Some(ref d) = e.data {
                        out.push_str(&hex_encode(d));
                    }
                } else {
                    out.push_str(&decode_event_text(e));
                }
            }
            _ => {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                let ts = format_timestamp(&e.timestamp);
                let src = e.source.as_deref().unwrap_or("");
                let line = match e.event_type.as_str() {
                    "startup" => format!(
                        "[{}] {:<4} port={}",
                        ts,
                        e.event_type,
                        e.port.as_deref().unwrap_or("")
                    ),
                    "shutdown" => format!("[{}] {}", ts, e.event_type),
                    "error" => format!(
                        "[{}] {:<4} {}",
                        ts,
                        e.event_type,
                        e.message.as_deref().unwrap_or("")
                    ),
                    "client_connected" | "client_disconnected" => format!(
                        "[{}] {:<4} {:<8} client={}",
                        ts,
                        e.event_type,
                        src,
                        e.client.as_deref().unwrap_or("")
                    ),
                    "action" => format!(
                        "[{}] {:<4} {:<8} {:<10} {}",
                        ts,
                        e.event_type,
                        src,
                        e.action.as_deref().unwrap_or(""),
                        e.name.as_deref().unwrap_or("")
                    ),
                    _ => format!("[{}] {}", ts, e.event_type),
                };
                out.push_str(&line);
                out.push('\n');
            }
        }
    }
    out
}

fn format_json(events: &[LogEvent]) -> String {
    events
        .iter()
        .map(|e| serde_json::to_string(&e.raw_json).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_markdown(events: &[LogEvent]) -> String {
    let mut md = String::from("# Session Log\n\n");
    md.push_str("| Time | Event | Source | Data |\n");
    md.push_str("|------|-------|--------|------|\n");

    for e in events {
        let ts = format_timestamp(&e.timestamp);
        let src = e.source.as_deref().unwrap_or("");
        let data_str = decode_event_text(e).replace('\\', "\\\\").replace('|', "\\|").replace('\n', " ");

        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            ts, e.event_type, src, data_str
        ));
    }

    md
}

fn compute_summary(events: &[LogEvent]) -> SessionSummary {
    let mut clients: Vec<String> = Vec::new();
    let mut tx_count = 0usize;
    let mut rx_count = 0usize;
    let mut tx_bytes = 0usize;
    let mut rx_bytes = 0usize;
    let mut error_count = 0usize;
    let mut action_count = 0usize;
    let mut start_time = None;
    let mut end_time = None;
    let mut port = None;

    for e in events {
        match e.event_type.as_str() {
            "startup" => {
                start_time = Some(e.timestamp.clone());
                port = e.port.clone();
            }
            "shutdown" => {
                end_time = Some(e.timestamp.clone());
            }
            "tx" => {
                tx_count += 1;
                tx_bytes += e.data.as_ref().map(|d| d.len()).unwrap_or(0);
            }
            "rx" => {
                rx_count += 1;
                rx_bytes += e.data.as_ref().map(|d| d.len()).unwrap_or(0);
            }
            "error" => {
                error_count += 1;
            }
            "action" => {
                action_count += 1;
            }
            "client_connected" => {
                if let Some(ref c) = e.client {
                    if !clients.contains(c) {
                        clients.push(c.clone());
                    }
                }
            }
            _ => {}
        }
    }

    let duration = match (&start_time, &end_time) {
        (Some(s), Some(e)) => {
            let start = DateTime::parse_from_rfc3339(s).ok();
            let end = DateTime::parse_from_rfc3339(e).ok();
            match (start, end) {
                (Some(s), Some(e)) => {
                    let diff = e.signed_duration_since(s);
                    let secs = diff.num_seconds();
                    let mins = secs / 60;
                    let secs = secs % 60;
                    Some(if mins > 0 {
                        format!("{}m {}s", mins, secs)
                    } else {
                        format!("{}s", secs)
                    })
                }
                _ => None,
            }
        }
        _ => None,
    };

    SessionSummary {
        start_time,
        end_time,
        duration,
        port,
        total_events: events.len(),
        tx_count,
        rx_count,
        tx_bytes,
        rx_bytes,
        error_count,
        action_count,
        clients,
    }
}

fn format_summary_text(summary: &SessionSummary) -> String {
    let mut out = String::from("=== Session Summary ===\n");
    if let Some(ref port) = summary.port {
        out.push_str(&format!("Port:      {}\n", port));
    }
    if let Some(ref t) = summary.start_time {
        out.push_str(&format!("Start:     {}\n", format_timestamp(t)));
    }
    if let Some(ref t) = summary.end_time {
        out.push_str(&format!("End:       {}\n", format_timestamp(t)));
    }
    if let Some(ref d) = summary.duration {
        out.push_str(&format!("Duration:  {}\n", d));
    }
    out.push_str(&format!("Events:    {} total\n", summary.total_events));
    out.push_str(&format!(
        "  tx:        {:>6}  ({:>10} bytes)\n",
        summary.tx_count, summary.tx_bytes
    ));
    out.push_str(&format!(
        "  rx:        {:>6}  ({:>10} bytes)\n",
        summary.rx_count, summary.rx_bytes
    ));
    if summary.error_count > 0 {
        out.push_str(&format!("  error:     {:>6}\n", summary.error_count));
    }
    if summary.action_count > 0 {
        out.push_str(&format!("  action:    {:>6}\n", summary.action_count));
    }
    if !summary.clients.is_empty() {
        out.push_str(&format!(
            "Clients:   {}\n",
            summary.clients.join(", ")
        ));
    }
    out
}

fn write_output(content: &str, output_path: &Option<String>) -> Result<(), String> {
    match output_path {
        Some(path) => {
            let mut file =
                fs::File::create(path).map_err(|e| format!("无法写入文件 '{}': {}", path, e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("写入失败: {}", e))?;
        }
        None => {
            print!("{}", content);
            let _ = std::io::stdout().flush();
        }
    }
    Ok(())
}

pub fn run(config: &LogConfig) -> Result<(), String> {
    let events = read_log_file(&config.file)?;
    if events.is_empty() {
        println!("日志文件为空或无有效事件。");
        return Ok(());
    }

    if config.summary {
        let summary = compute_summary(&events);
        let output = format_summary_text(&summary);
        write_output(&output, &config.output)?;
        return Ok(());
    }

    let filtered = filter_events(&events, config);
    if filtered.is_empty() {
        println!("没有匹配的事件。");
        return Ok(());
    }

    let output = match config.format {
        OutputFormat::Text => format_text(&filtered, config.raw),
        OutputFormat::Json => format_json(&filtered),
        OutputFormat::Md => format_markdown(&filtered),
    };

    write_output(&output, &config.output)?;

    if config.output.is_none() && !output.ends_with('\n') {
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_event_valid() {
        let line = r#"{"ts":"2025-01-01T00:00:00Z","event":"startup","port":"COM3"}"#;
        let e = parse_log_event(line).unwrap();
        assert_eq!(e.event_type, "startup");
        assert_eq!(e.port.as_deref(), Some("COM3"));
    }

    #[test]
    fn parse_log_event_with_data() {
        let line = r#"{"ts":"2025-01-01T00:00:00Z","event":"tx","source":"local","data":"48656C6C6F"}"#;
        let e = parse_log_event(line).unwrap();
        assert_eq!(e.event_type, "tx");
        assert_eq!(e.data.as_deref(), Some(b"Hello".as_slice()));
        assert_eq!(e.source.as_deref(), Some("local"));
    }

    #[test]
    fn parse_log_event_action() {
        let line = r#"{"ts":"2025-01-01T00:00:00Z","event":"action","source":"local","action":"run_macro","name":"init"}"#;
        let e = parse_log_event(line).unwrap();
        assert_eq!(e.event_type, "action");
        assert_eq!(e.action.as_deref(), Some("run_macro"));
        assert_eq!(e.name.as_deref(), Some("init"));
    }

    #[test]
    fn parse_log_event_invalid_json() {
        assert!(parse_log_event("not json").is_none());
    }

    #[test]
    fn parse_log_event_missing_event_field() {
        assert!(parse_log_event(r#"{"ts":"2025-01-01T00:00:00Z"}"#).is_none());
    }

    fn make_event(event_type: &str, data: Option<Vec<u8>>) -> LogEvent {
        LogEvent {
            timestamp: String::new(),
            event_type: event_type.into(),
            source: None,
            data,
            message: None,
            port: None,
            client: None,
            action: None,
            name: None,
            raw_json: Value::Null,
        }
    }

    #[test]
    fn decode_event_text_ascii() {
        let e = make_event("tx", Some(b"Hello\r\n".to_vec()));
        assert_eq!(decode_event_text(&e), "Hello");
    }

    #[test]
    fn decode_event_text_binary() {
        let e = make_event("rx", Some(vec![0xFF, 0xFE]));
        assert_eq!(decode_event_text(&e), "<binary: 2 bytes>");
    }

    #[test]
    fn decode_event_text_action_with_name() {
        let mut e = make_event("action", None);
        e.action = Some("run_macro".into());
        e.name = Some("init".into());
        assert_eq!(decode_event_text(&e), "run_macro init");
    }

    #[test]
    fn decode_event_text_action_without_name() {
        let mut e = make_event("action", None);
        e.action = Some("clear_buffer".into());
        assert_eq!(decode_event_text(&e), "clear_buffer");
    }

    #[test]
    fn format_timestamp_rfc3339() {
        assert_eq!(
            format_timestamp("2025-06-13T12:34:56.789+00:00"),
            "12:34:56.789"
        );
    }

    #[test]
    fn format_timestamp_fallback() {
        assert_eq!(format_timestamp("raw-text"), "raw-text");
    }

    #[test]
    fn compute_summary_counts_action() {
        let events = vec![
            make_event("action", None),
            make_event("action", None),
            make_event("tx", Some(b"x".to_vec())),
        ];
        let summary = compute_summary(&events);
        assert_eq!(summary.action_count, 2);
        assert_eq!(summary.tx_count, 1);
    }
}

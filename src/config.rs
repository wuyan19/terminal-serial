use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

/// terminal-serial 的统一工具配置。本版本只填充 `macros` 字段，
/// 未来扩展（默认串口参数、日志策略、telnet/mcp 服务配置、快捷键自定义等）
/// 都加到同一文件里。顶层字段全部 `#[serde(default)]`，新增字段对老配置透明。
#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub macros: BTreeMap<String, Macro>,
}

#[derive(Deserialize)]
pub struct Macro {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub steps: Vec<MacroStep>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MacroStep {
    Send {
        data: String,
        #[serde(default)]
        format: Option<String>,
        #[serde(default)]
        auto_newline: Option<bool>,
    },
    Delay { ms: u64 },
    Expect {
        pattern: String,
        timeout_ms: u32,
    },
    Clear,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("无法读取配置文件 '{}': {}", path, e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("配置文件 '{}' 解析失败: {}", path, e))
    }

    pub fn empty() -> Self {
        Self::default()
    }

    /// 按 BTreeMap 顺序（字母序）返回第 idx 个宏，用于 Alt+数字 索引。
    pub fn macro_at_index(&self, idx: usize) -> Option<(&String, &Macro)> {
        self.macros.iter().nth(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_config() {
        let json = r#"{
            "macros": {
                "init": {
                    "description": "设备初始化",
                    "steps": [
                        {"type": "send", "data": "ATZ", "auto_newline": true},
                        {"type": "delay", "ms": 500},
                        {"type": "expect", "pattern": "OK", "timeout_ms": 3000},
                        {"type": "clear"}
                    ]
                }
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.macros.len(), 1);
        let init = config.macros.get("init").unwrap();
        assert_eq!(init.description.as_deref(), Some("设备初始化"));
        assert_eq!(init.steps.len(), 4);
    }

    #[test]
    fn parse_empty_config() {
        let json = "{}";
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.macros.is_empty());
    }

    #[test]
    fn parse_unknown_top_level_field() {
        let json = r#"{"future_section": {"foo": 1}, "macros": {}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.macros.is_empty());
    }

    #[test]
    fn parse_invalid_json_returns_err() {
        let json = "not json";
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_unknown_step_type_returns_err() {
        let json = r#"{"macros":{"m":{"steps":[{"type":"unknown","data":"x"}]}}}"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn macro_at_index_ordering() {
        let json = r#"{"macros":{"beta":{"steps":[]},"alpha":{"steps":[]}}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.macro_at_index(0).unwrap().0, "alpha");
        assert_eq!(config.macro_at_index(1).unwrap().0, "beta");
        assert!(config.macro_at_index(2).is_none());
    }

    #[test]
    fn macro_default_steps_empty() {
        let json = r#"{"macros":{"m":{}}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.macros.get("m").unwrap().steps.is_empty());
    }

    #[test]
    fn macro_step_send_optional_fields() {
        let json = r#"{"macros":{"m":{"steps":[{"type":"send","data":"AT"}]}}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        match &config.macros["m"].steps[0] {
            MacroStep::Send { data, format, auto_newline } => {
                assert_eq!(data, "AT");
                assert!(format.is_none());
                assert!(auto_newline.is_none());
            }
            _ => panic!("expected Send step"),
        }
    }
}

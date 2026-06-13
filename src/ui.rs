//! 终端反馈风格统一模块。所有用户面向的工具提示通过这里发出，
//! 保证 ANSI 颜色、符号、括号与换行风格一致。
//!
//! 所有函数默认输出**独立视觉块**：前导 `\r\n`（与设备输出隔开）+ 反馈 + 后导 `\r\n`。
//! 调用方不再需要写任何 `print!("\r\n")`，避免未来引入 quiet 开关时残留空行。
//!
//! 约定：
//! - 绿色 ✓：成功类（清缓冲区、宏执行成功）
//! - 青色 ▶：信息类（宏开始执行、章节标题）
//! - 红色 ✗：失败类（宏执行错误、索引越界）
//! - 黄色：次要提示（无宏定义、操作引导）

use std::io::{self, Write};

/// 成功反馈：绿色 ✓ + 方括号。
pub fn success(msg: &str) {
    print!("\r\n\x1b[32m✓ [{}]\x1b[0m\r\n", msg);
    let _ = io::stdout().flush();
}

/// 信息反馈：青色 ▶ + 方括号。
pub fn info(msg: &str) {
    print!("\r\n\x1b[36m▶ [{}]\x1b[0m\r\n", msg);
    let _ = io::stdout().flush();
}

/// 失败反馈：红色 ✗ + 方括号。
pub fn fail(msg: &str) {
    print!("\r\n\x1b[31m✗ [{}]\x1b[0m\r\n", msg);
    let _ = io::stdout().flush();
}

/// 次要提示：黄色 + 圆括号。
pub fn hint(msg: &str) {
    print!("\r\n\x1b[33m({})\x1b[0m\r\n", msg);
    let _ = io::stdout().flush();
}

/// 章节标题：青色 + === 边框。
pub fn heading(msg: &str) {
    print!("\r\n\x1b[36m=== {} ===\x1b[0m\r\n", msg);
    let _ = io::stdout().flush();
}

use crate::keyboard::{Key, Keyboard};

const CTRL_RIGHT_BRACKET: u8 = 0x1d; // Ctrl + ] 退出
const CTRL_K: u8 = 0x0b; // Ctrl + K 清空接收缓冲区
const CTRL_O: u8 = 0x0f; // Ctrl + O 进入宏菜单选择模式

#[derive(Debug, PartialEq)]
pub enum InputMessage {
    Quit,
    Data(Vec<u8>),
    ClearBuffer,
    ShowMacroMenu,
    RunMacro(usize),
}

/// 键盘输入处理器。包含两状态机：
/// - Normal：普通输入模式，Ctrl+O 切换到 MenuSelect
/// - MenuSelect：等待 '1'-'9' 选择宏，其他键吞掉并回到 Normal
///
/// Ctrl+] / Ctrl+K 在任何模式下都立即生效（并退出菜单）。
pub struct Input {
    keyboard: Keyboard,
    menu_mode: bool,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keyboard: Keyboard::new(),
            menu_mode: false,
        }
    }

    pub fn get_message(&mut self) -> Option<InputMessage> {
        let input = self.keyboard.get_input()?;
        let (raw, dst): (Vec<u8>, Vec<u8>) = match input {
            Key::Other(x) => (x.raw_value, x.dst_value),
            Key::Up(x) | Key::Down(x) | Key::Right(x) | Key::Left(x)
            | Key::Insert(x) | Key::Delete(x) | Key::Home(x) | Key::End(x)
            | Key::PageUp(x) | Key::PageDown(x) => (x.raw_value, x.dst_value),
            // F1-F12 等不识别的按键：菜单模式下退出菜单，Normal 模式下无操作
            _ => {
                self.menu_mode = false;
                return None;
            }
        };
        Self::transition(&mut self.menu_mode, &raw, dst)
    }

    /// 撤销菜单选择模式。用于上层在 ShowMacroMenu 后判断宏列表为空时
    /// 主动退出菜单，避免用户输入数字时被吞掉。
    pub fn cancel_menu_mode(&mut self) {
        self.menu_mode = false;
    }

    /// 状态机转移：根据当前 menu_mode 与按键决定下一个 InputMessage。
    /// 抽成关联函数便于单元测试（不依赖 Keyboard 初始化）。
    fn transition(menu_mode: &mut bool, raw: &[u8], dst: Vec<u8>) -> Option<InputMessage> {
        // Ctrl+] / Ctrl+K 在任何模式下都立即生效，并强制退出菜单
        if raw.len() == 1 {
            match raw[0] {
                CTRL_RIGHT_BRACKET => {
                    *menu_mode = false;
                    return Some(InputMessage::Quit);
                }
                CTRL_K => {
                    *menu_mode = false;
                    return Some(InputMessage::ClearBuffer);
                }
                _ => {}
            }
        }

        if *menu_mode {
            // 菜单选择模式：'1'-'9' 执行对应宏，其他键吞掉并退出菜单
            if raw.len() == 1 && raw[0] >= b'1' && raw[0] <= b'9' {
                let idx = (raw[0] - b'1' + 1) as usize;
                *menu_mode = false;
                return Some(InputMessage::RunMacro(idx));
            }
            *menu_mode = false;
            return None;
        }

        // Normal 模式：Ctrl+O 进入菜单
        if raw.len() == 1 && raw[0] == CTRL_O {
            *menu_mode = true;
            return Some(InputMessage::ShowMacroMenu);
        }

        Some(InputMessage::Data(dst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：从指定初始 menu_mode 触发一次转移，返回 (消息, 转移后 menu_mode)
    fn transition_from(menu_mode: bool, raw: &[u8], dst: Vec<u8>) -> (Option<InputMessage>, bool) {
        let mut mode = menu_mode;
        let msg = Input::transition(&mut mode, raw, dst);
        (msg, mode)
    }

    // ----- Normal 模式 -----

    #[test]
    fn normal_ctrl_o_enters_menu() {
        let (msg, mode) = transition_from(false, &[CTRL_O], vec![CTRL_O]);
        assert_eq!(msg, Some(InputMessage::ShowMacroMenu));
        assert!(mode, "Ctrl+O 后应进入 menu_mode");
    }

    #[test]
    fn normal_regular_byte_returns_data() {
        let (msg, mode) = transition_from(false, &[b'A'], vec![b'A']);
        assert_eq!(msg, Some(InputMessage::Data(vec![b'A'])));
        assert!(!mode);
    }

    #[test]
    fn normal_digit_returns_data() {
        let (msg, _) = transition_from(false, &[b'1'], vec![b'1']);
        assert_eq!(msg, Some(InputMessage::Data(vec![b'1'])));
    }

    #[test]
    fn normal_ctrl_k_clears() {
        let (msg, mode) = transition_from(false, &[CTRL_K], vec![CTRL_K]);
        assert_eq!(msg, Some(InputMessage::ClearBuffer));
        assert!(!mode);
    }

    #[test]
    fn normal_ctrl_right_bracket_quits() {
        let (msg, mode) = transition_from(false, &[CTRL_RIGHT_BRACKET], vec![CTRL_RIGHT_BRACKET]);
        assert_eq!(msg, Some(InputMessage::Quit));
        assert!(!mode);
    }

    // ----- MenuSelect 模式 -----

    #[test]
    fn menu_digit_runs_macro() {
        let (msg, mode) = transition_from(true, &[b'3'], vec![b'3']);
        assert_eq!(msg, Some(InputMessage::RunMacro(3)));
        assert!(!mode, "执行宏后应退出 menu_mode");
    }

    #[test]
    fn menu_digit_one_runs_first_macro() {
        let (msg, _) = transition_from(true, &[b'1'], vec![b'1']);
        assert_eq!(msg, Some(InputMessage::RunMacro(1)));
    }

    #[test]
    fn menu_digit_nine_runs_last_macro() {
        let (msg, _) = transition_from(true, &[b'9'], vec![b'9']);
        assert_eq!(msg, Some(InputMessage::RunMacro(9)));
    }

    #[test]
    fn menu_zero_exits_without_running() {
        let (msg, mode) = transition_from(true, &[b'0'], vec![b'0']);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    #[test]
    fn menu_letter_exits_without_running() {
        let (msg, mode) = transition_from(true, &[b'a'], vec![b'a']);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    #[test]
    fn menu_enter_exits_without_running() {
        let (msg, mode) = transition_from(true, &[b'\r'], vec![b'\r']);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    #[test]
    fn menu_escape_exits_without_running() {
        let (msg, mode) = transition_from(true, &[0x1b], vec![0x1b]);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    #[test]
    fn menu_ctrl_o_exits_menu() {
        // 在菜单模式下再按 Ctrl+O：吞掉，退出菜单（不刷新菜单，避免循环）
        let (msg, mode) = transition_from(true, &[CTRL_O], vec![CTRL_O]);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    #[test]
    fn menu_multi_byte_exits_without_running() {
        // 方向键等多字节序列在菜单模式下被吞掉
        let (msg, mode) = transition_from(true, &[0xe0, b'H'], vec![0x1b, 0x5b, b'A']);
        assert_eq!(msg, None);
        assert!(!mode);
    }

    // ----- 控制键在任何模式下立即生效 -----

    #[test]
    fn menu_ctrl_k_still_clears() {
        let (msg, mode) = transition_from(true, &[CTRL_K], vec![CTRL_K]);
        assert_eq!(msg, Some(InputMessage::ClearBuffer));
        assert!(!mode);
    }

    #[test]
    fn menu_ctrl_right_bracket_still_quits() {
        let (msg, mode) = transition_from(true, &[CTRL_RIGHT_BRACKET], vec![CTRL_RIGHT_BRACKET]);
        assert_eq!(msg, Some(InputMessage::Quit));
        assert!(!mode);
    }
}

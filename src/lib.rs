pub mod cmd;
pub mod error;
pub mod event_log;
pub mod getch;
pub mod keyboard;
pub mod mcp;
pub mod serial_manager;
pub mod server;
pub mod telnet;
pub mod task;

use keyboard::Key;
use keyboard::Keyboard;

const CTRL_RIGHT_BRACKET: u8 = 0x1d; // Ctrl + ] 退出
const CTRL_K: u8 = 0x0b;             // Ctrl + K 清空 MCP 读缓冲区

pub enum InputMessage {
    Quit,
    Data(Vec<u8>),
    ClearBuffer,
    None,
}

pub struct Input {
    keyboard: Keyboard,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keyboard: Keyboard::new(),
        }
    }

    pub fn get_message(&self) -> InputMessage {
        let input = self.keyboard.get_input();
        match input {
            Key::Other(x) => {
                if x.raw_value.len() == 1 && x.raw_value[0] == CTRL_RIGHT_BRACKET {
                    InputMessage::Quit
                } else if x.raw_value.len() == 1 && x.raw_value[0] == CTRL_K {
                    InputMessage::ClearBuffer
                } else {
                    InputMessage::Data(x.dst_value)
                }
            }
            Key::Up(x) | Key::Down(x) | Key::Right(x) | Key::Left(x)
            | Key::Insert(x) | Key::Delete(x) | Key::Home(x) | Key::End(x)
            | Key::PageUp(x) | Key::PageDown(x) => InputMessage::Data(x.dst_value),
            _ => InputMessage::None,
        }
    }
}

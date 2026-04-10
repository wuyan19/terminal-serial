pub mod cmd;
pub mod getch;
pub mod keyboard;
pub mod mcp;
pub mod serial_manager;
pub mod server;
pub mod task;

use keyboard::Key;
use keyboard::Keyboard;

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
                if x.raw_value.len() == 1 && x.raw_value[0] == 0x1d {
                    // Ctrl + ]
                    return InputMessage::Quit;
                } else if x.raw_value.len() == 1 && x.raw_value[0] == 0x0b {
                    // Ctrl + K
                    return InputMessage::ClearBuffer;
                } else {
                    return InputMessage::Data(x.dst_value);
                }
            }
            Key::Up(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Down(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Right(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Left(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Insert(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Delete(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::Home(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::End(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::PageUp(x) => {
                return InputMessage::Data(x.dst_value);
            }
            Key::PageDown(x) => {
                return InputMessage::Data(x.dst_value);
            }
            _ => return InputMessage::None,
        }
    }
}

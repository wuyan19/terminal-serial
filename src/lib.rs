pub mod cmd;
pub mod getch;
pub mod keyboard;
pub mod task;

pub mod serial_port {
    pub fn get_list() -> Vec<String> {
        let mut port_list: Vec<String> = Vec::new();

        #[cfg(windows)]
        {
            use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
            let serial_comms =
                RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("HARDWARE\\DEVICEMAP\\SERIALCOMM");
            if let Ok(serial) = serial_comms {
                for (_name, value) in serial.enum_values().map(|x| x.unwrap()) {
                    if let Ok(com) = String::from_utf8(value.bytes) {
                        let mut tmp = String::new();
                        for val in com.as_bytes().iter() {
                            if *val != 0 {
                                tmp.push(*val as char);
                            }
                        }
                        port_list.push(tmp);
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            let items = std::fs::read_dir("/dev/").unwrap();

            for item in items {
                let file = item.unwrap().path().display().to_string();
                if file.contains("tty.usb") {
                    port_list.push(file);
                }
            }
        }

        port_list
    }
}

use keyboard::Key;
use keyboard::Keyboard;

pub enum InputMessage {
    Quit,
    Data(Vec<u8>),
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

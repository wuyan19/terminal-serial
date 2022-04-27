extern crate libc;

use libc::c_int;
extern "C" {
    fn _getch() -> c_int;
}

fn getch() -> Result<u8, std::io::Error> {
    loop {
        unsafe {
            let k = _getch();
            return Ok(k as u8);
        }
    }
}

pub enum InputMessage {
    Quit,
    Data(Vec<u8>),
    None,
}

impl InputMessage {
    pub fn get_message() -> InputMessage {
        if let Ok(x) = getch() {
            match x {
                0x1d => {
                    // Ctrl + ']'
                    return InputMessage::Quit;
                }
                0xe0 | 0x00 => {
                    if let Ok(x) = getch() {
                        // 'H':Up  'P':Down  'K':Left  'M':Right
                        // 'I':PageUp  'Q':PageDown  'G':Home  'O':End
                        // 'R':Insert  'S':Delete
                        // ';':F1  '<':F2  '=':F3  '>':F4  '?':F5
                        // '@':F6  'A':F7  'B':F8  'C':F9  'D':F10
                        //  134:F12
                        const F12: char = 134 as char;
                        match x as char {
                            'H' => {
                                println!("Up");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'A' as u8]);
                            }
                            'P' => {
                                println!("Down");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'B' as u8]);
                            }
                            'K' => {
                                println!("Left");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'D' as u8]);
                            }
                            'M' => {
                                println!("Right");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'C' as u8]);
                            }
                            'G' => {
                                println!("Home");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'H' as u8]);
                            }
                            'O' => {
                                println!("End");
                                return InputMessage::Data(vec![0x1b, 0x5b, 'F' as u8]);
                            }
                            'R' => {
                                println!("Insert");
                                return InputMessage::Data(vec![0x1b, 0x5b, '2' as u8, '~' as u8]);
                            }
                            'S' => {
                                println!("Delete");
                                return InputMessage::Data(vec![0x1b, 0x5b, '3' as u8, '~' as u8]);
                            }
                            'I' => {
                                println!("PageUp");
                                return InputMessage::Data(vec![0x1b, 0x5b, '5' as u8, '~' as u8]);
                            }
                            'Q' => {
                                println!("PageDown");
                                return InputMessage::Data(vec![0x1b, 0x5b, '6' as u8, '~' as u8]);
                            }
                            ';' => {
                                println!("F1");
                                return InputMessage::None;
                            }
                            '<' => {
                                println!("F2");
                                return InputMessage::None;
                            }
                            '=' => {
                                println!("F3");
                                return InputMessage::None;
                            }
                            '>' => {
                                println!("F4");
                                return InputMessage::None;
                            }
                            '?' => {
                                println!("F5");
                                return InputMessage::None;
                            }
                            '@' => {
                                println!("F6");
                                return InputMessage::None;
                            }
                            'A' => {
                                println!("F7");
                                return InputMessage::None;
                            }
                            'B' => {
                                println!("F8");
                                return InputMessage::None;
                            }
                            'C' => {
                                println!("F9");
                                return InputMessage::None;
                            }
                            'D' => {
                                println!("F10");
                                return InputMessage::None;
                            }
                            F12 => {
                                println!("F12");
                                return InputMessage::None;
                            }
                            c => {
                                println!("{}", c as u8);
                                return InputMessage::None;
                            }
                        }
                    }
                }
                _ => {
                    if x > 31 && x < 127 {
                        println!("{}", x as char);
                    } else {
                        println!("{}", x as u8);
                    }
                    return InputMessage::Data(vec![x]);
                }
            }
        }

        return InputMessage::None;
    }
}

use crate::getch;

pub enum InputMessage {
    Quit,
    Data(Vec<u8>),
    None,
}

impl InputMessage {
    pub fn get_message() -> InputMessage {
        let gc = getch::Getch::new();
        if let Ok(x) = gc.getch() {
            match x {
                0x1d => {
                    // Ctrl + ']'
                    return InputMessage::Quit;
                }
                #[cfg(windows)]
                0xe0 | 0x00 => {
                    if let Ok(x) = gc.getch() {
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
                #[cfg(not(windows))]
                0x1b => {
                    if let Ok(x) = gc.getch() {
                        match x {
                            0x5b => {
                                if let Ok(x) = gc.getch() {
                                    match x as char {
                                        'A' => {
                                            //println!("Up");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'A' as u8]);
                                        }
                                        'B' => {
                                            //println!("Down");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'B' as u8]);
                                        }
                                        'D' => {
                                            //println!("Left");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'D' as u8]);
                                        }
                                        'C' => {
                                            //println!("Right");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'C' as u8]);
                                        }
                                        'H' => {
                                            //println!("Home");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'H' as u8]);
                                        }
                                        'F' => {
                                            //println!("End");
                                            return InputMessage::Data(vec![0x1b, 0x5b, 'F' as u8]);
                                        }
                                        '1' => {
                                            if let Ok(x) = gc.getch() {
                                                match x as char {
                                                    '5' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F5");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '7' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F6");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '8' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F7");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '9' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F8");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _c => {
                                                        //println!("{}: {}", line!(), c);
                                                        return InputMessage::None;
                                                    }
                                                }
                                            }
                                        }
                                        '2' => {
                                            if let Ok(x) = gc.getch() {
                                                match x as char {
                                                    '~' => {
                                                        //println!("Insert");
                                                        return InputMessage::Data(vec![0x1b, 0x5b, '2' as u8, '~' as u8]);
                                                    }
                                                    '0' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F9");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), _c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '1' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F10");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '3' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F11");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    '4' => {
                                                        if let Ok(x) = gc.getch() {
                                                            match x as char {
                                                                '~' => {
                                                                    //println!("F12");
                                                                    return InputMessage::None;
                                                                }
                                                                _c => {
                                                                    //println!("{}: {}", line!(), c);
                                                                    return InputMessage::None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _c => {
                                                        //println!("{}: {}", line!(), c);
                                                        return InputMessage::None;
                                                    }
                                                }
                                            }
                                        }
                                        '3' => {
                                            if let Ok(x) = gc.getch() {
                                                match x as char {
                                                    '~' => {
                                                        //println!("Delete");
                                                        return InputMessage::Data(vec![0x1b, 0x5b, '3' as u8, '~' as u8]);
                                                    }
                                                    _c => {
                                                        //println!("{}: {}", line!(), c);
                                                        return InputMessage::None;
                                                    }
                                                }
                                            }
                                        }
                                        _c => {
                                            //println!("{}: {}", line!(), c);
                                            return InputMessage::None;
                                        }
                                    }
                                }
                            }
                            0x4f => {
                                if let Ok(x) = gc.getch() {
                                    match x as char {
                                        'P' => {
                                            //println!("F1");
                                            return InputMessage::None;
                                        }
                                        'Q' => {
                                            //println!("F2");
                                            return InputMessage::None;
                                        }
                                        'R' => {
                                            //println!("F3");
                                            return InputMessage::None;
                                        }
                                        'S' => {
                                            //println!("F4");
                                            return InputMessage::None;
                                        }
                                        _c => {
                                            //println!("{}: {}", line!(), c);
                                            return InputMessage::None;
                                        }
                                    }
                                }
                            }
                            // 0x32 => {
                            //     println!("Insert");
                            //     return InputMessage::Data(vec![0x1b, 0x5b, '2' as u8, '~' as u8]);
                            // }
                            _c => {
                                //println!("{}: {:x}", line!(), c);
                                return InputMessage::None;
                            }
                        }
                    }
                }
                _ => {
                    // if x > 31 && x < 127 {
                    //     println!("{}: {}", line!(), x as char);
                    // } else {
                    //     println!("{}: {}", line!(), x as u8);
                    // }
                    if x < 128 {
                        return InputMessage::Data(vec![x]);
                    }
                }
            }
        }

        return InputMessage::None;
    }
}

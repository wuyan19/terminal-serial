use crate::getch::Getch;

#[derive(Debug, Clone)]
pub enum Key<T> {
    F1(T),
    F2(T),
    F3(T),
    F4(T),
    F5(T),
    F6(T),
    F7(T),
    F8(T),
    F9(T),
    F10(T),
    F11(T),
    F12(T),
    Up(T),
    Down(T),
    Right(T),
    Left(T),
    PageUp(T),
    PageDown(T),
    Home(T),
    End(T),
    Insert(T),
    Delete(T),
    Other(T),
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub raw_value: Vec<u8>,
    pub dst_value: Vec<u8>,
}

pub struct Keyboard {
    getch: Getch,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            getch: Getch::new(),
        }
    }

    fn getch(&self) -> u8 {
        self.getch.getch().unwrap()
    }

    pub fn get_input(&self) -> Key<KeyInfo> {
        let first = self.getch();

        #[cfg(windows)]
        if first == 0xe0 || first == 0x00 {
            let second = self.getch();
            return self.match_windows_key(first, second);
        }

        #[cfg(not(windows))]
        if first == 0x1b {
            return self.match_unix_escape();
        }

        Key::Other(KeyInfo {
            raw_value: vec![first],
            dst_value: vec![first],
        })
    }

    #[cfg(windows)]
    fn match_windows_key(&self, prefix: u8, code: u8) -> Key<KeyInfo> {
        match (prefix, code) {
            (0xe0, b'H') => Key::Up(KeyInfo {
                raw_value: vec![0xe0, b'H'],
                dst_value: vec![0x1b, 0x5b, b'A'],
            }),
            (0xe0, b'P') => Key::Down(KeyInfo {
                raw_value: vec![0xe0, b'P'],
                dst_value: vec![0x1b, 0x5b, b'B'],
            }),
            (0xe0, b'M') => Key::Right(KeyInfo {
                raw_value: vec![0xe0, b'M'],
                dst_value: vec![0x1b, 0x5b, b'C'],
            }),
            (0xe0, b'K') => Key::Left(KeyInfo {
                raw_value: vec![0xe0, b'K'],
                dst_value: vec![0x1b, 0x5b, b'D'],
            }),
            (0xe0, b'G') => Key::Home(KeyInfo {
                raw_value: vec![0xe0, b'G'],
                dst_value: vec![0x1b, 0x5b, b'H'],
            }),
            (0xe0, b'O') => Key::End(KeyInfo {
                raw_value: vec![0xe0, b'O'],
                dst_value: vec![0x1b, 0x5b, b'F'],
            }),
            (0xe0, b'R') => Key::Insert(KeyInfo {
                raw_value: vec![0xe0, b'R'],
                dst_value: vec![0x1b, 0x5b, b'2', b'~'],
            }),
            (0xe0, b'S') => Key::Delete(KeyInfo {
                raw_value: vec![0xe0, b'S'],
                dst_value: vec![0x1b, 0x5b, b'3', b'~'],
            }),
            (0xe0, b'I') => Key::PageUp(KeyInfo {
                raw_value: vec![0xe0, b'I'],
                dst_value: vec![0x1b, 0x5b, b'5', b'~'],
            }),
            (0xe0, b'Q') => Key::PageDown(KeyInfo {
                raw_value: vec![0xe0, b'Q'],
                dst_value: vec![0x1b, 0x5b, b'6', b'~'],
            }),
            // Windows F-keys (0x00 prefix)
            (0x00, b';') => Key::F1(KeyInfo { raw_value: vec![0x00, b';'], dst_value: vec![] }),
            (0x00, b'<') => Key::F2(KeyInfo { raw_value: vec![0x00, b'<'], dst_value: vec![] }),
            (0x00, b'=') => Key::F3(KeyInfo { raw_value: vec![0x00, b'='], dst_value: vec![] }),
            (0x00, b'>') => Key::F4(KeyInfo { raw_value: vec![0x00, b'>'], dst_value: vec![] }),
            (0x00, b'?') => Key::F5(KeyInfo { raw_value: vec![0x00, b'?'], dst_value: vec![] }),
            (0x00, b'@') => Key::F6(KeyInfo { raw_value: vec![0x00, b'@'], dst_value: vec![] }),
            (0x00, b'A') => Key::F7(KeyInfo { raw_value: vec![0x00, b'A'], dst_value: vec![] }),
            (0x00, b'B') => Key::F8(KeyInfo { raw_value: vec![0x00, b'B'], dst_value: vec![] }),
            (0x00, b'C') => Key::F9(KeyInfo { raw_value: vec![0x00, b'C'], dst_value: vec![] }),
            (0x00, b'D') => Key::F10(KeyInfo { raw_value: vec![0x00, b'D'], dst_value: vec![] }),
            // F12 (0xe0, 0x86)
            (0xe0, 0x86) => Key::F12(KeyInfo { raw_value: vec![0xe0, 0x86], dst_value: vec![] }),
            _ => Key::Other(KeyInfo {
                raw_value: vec![prefix, code],
                dst_value: vec![prefix, code],
            }),
        }
    }

    #[cfg(not(windows))]
    fn match_unix_escape(&self) -> Key<KeyInfo> {
        let second = self.getch();

        if second == 0x5b {
            // CSI 序列: ESC [
            let third = self.getch();
            match third {
                b'A' => Key::Up(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'A'],
                    dst_value: vec![0x1b, 0x5b, b'A'],
                }),
                b'B' => Key::Down(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'B'],
                    dst_value: vec![0x1b, 0x5b, b'B'],
                }),
                b'C' => Key::Right(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'C'],
                    dst_value: vec![0x1b, 0x5b, b'C'],
                }),
                b'D' => Key::Left(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'D'],
                    dst_value: vec![0x1b, 0x5b, b'D'],
                }),
                b'H' => Key::Home(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'H'],
                    dst_value: vec![0x1b, 0x5b, b'H'],
                }),
                b'F' => Key::End(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, b'F'],
                    dst_value: vec![0x1b, 0x5b, b'F'],
                }),
                b'1' => {
                    let fourth = self.getch();
                    match fourth {
                        b'5' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'1', b'5'],
                            Key::F5,
                        ),
                        b'7' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'1', b'7'],
                            Key::F6,
                        ),
                        b'8' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'1', b'8'],
                            Key::F7,
                        ),
                        b'9' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'1', b'9'],
                            Key::F8,
                        ),
                        _ => Key::Other(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'1', fourth],
                            dst_value: vec![0x1b, 0x5b, b'1', fourth],
                        }),
                    }
                }
                b'2' => {
                    let fourth = self.getch();
                    match fourth {
                        b'~' => Key::Insert(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'2', b'~'],
                            dst_value: vec![0x1b, 0x5b, b'2', b'~'],
                        }),
                        b'0' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'2', b'0'],
                            Key::F9,
                        ),
                        b'1' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'2', b'1'],
                            Key::F10,
                        ),
                        b'3' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'2', b'3'],
                            Key::F11,
                        ),
                        b'4' => self.expect_terminator_and_key(
                            &[0x1b, 0x5b, b'2', b'4'],
                            Key::F12,
                        ),
                        _ => Key::Other(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'2', fourth],
                            dst_value: vec![0x1b, 0x5b, b'2', fourth],
                        }),
                    }
                }
                b'3' => {
                    let fourth = self.getch();
                    if fourth == b'~' {
                        Key::Delete(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'3', b'~'],
                            dst_value: vec![0x1b, 0x5b, b'3', b'~'],
                        })
                    } else {
                        Key::Other(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'3', fourth],
                            dst_value: vec![0x1b, 0x5b, b'3', fourth],
                        })
                    }
                }
                b'5' => {
                    let fourth = self.getch();
                    if fourth == b'~' {
                        Key::PageUp(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'5', b'~'],
                            dst_value: vec![0x1b, 0x5b, b'5', b'~'],
                        })
                    } else {
                        Key::Other(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'5', fourth],
                            dst_value: vec![0x1b, 0x5b, b'5', fourth],
                        })
                    }
                }
                b'6' => {
                    let fourth = self.getch();
                    if fourth == b'~' {
                        Key::PageDown(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'6', b'~'],
                            dst_value: vec![0x1b, 0x5b, b'6', b'~'],
                        })
                    } else {
                        Key::Other(KeyInfo {
                            raw_value: vec![0x1b, 0x5b, b'6', fourth],
                            dst_value: vec![0x1b, 0x5b, b'6', fourth],
                        })
                    }
                }
                _ => Key::Other(KeyInfo {
                    raw_value: vec![0x1b, 0x5b, third],
                    dst_value: vec![0x1b, 0x5b, third],
                }),
            }
        } else if second == 0x4f {
            // SS3 序列: ESC O
            let third = self.getch();
            match third {
                b'P' => Key::F1(KeyInfo { raw_value: vec![0x1b, 0x4f, b'P'], dst_value: vec![] }),
                b'Q' => Key::F2(KeyInfo { raw_value: vec![0x1b, 0x4f, b'Q'], dst_value: vec![] }),
                b'R' => Key::F3(KeyInfo { raw_value: vec![0x1b, 0x4f, b'R'], dst_value: vec![] }),
                b'S' => Key::F4(KeyInfo { raw_value: vec![0x1b, 0x4f, b'S'], dst_value: vec![] }),
                _ => Key::Other(KeyInfo {
                    raw_value: vec![0x1b, 0x4f, third],
                    dst_value: vec![0x1b, 0x4f, third],
                }),
            }
        } else {
            Key::Other(KeyInfo {
                raw_value: vec![0x1b, second],
                dst_value: vec![0x1b, second],
            })
        }
    }

    /// Unix 下读取期望的 '~' 终止符，然后返回指定的 F-key
    #[cfg(not(windows))]
    fn expect_terminator_and_key(
        &self,
        prefix: &[u8],
        key_fn: fn(KeyInfo) -> Key<KeyInfo>,
    ) -> Key<KeyInfo> {
        let terminator = self.getch();
        let mut raw = prefix.to_vec();
        raw.push(terminator);
        if terminator == b'~' {
            key_fn(KeyInfo { raw_value: raw, dst_value: vec![] })
        } else {
            Key::Other(KeyInfo {
                raw_value: raw.clone(),
                dst_value: raw,
            })
        }
    }
}

use crate::getch;

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

#[derive(Debug)]
enum TreeChild<T> {
    Childs(T),
    Value(Key<KeyInfo>),
}

#[derive(Debug)]
struct TreeNode {
    value: Vec<u8>,
    childs: TreeChild<Vec<Box<TreeNode>>>,
}

type TreeRoot = TreeNode;

pub struct Keyboard {
    key_tree: TreeRoot,
}

impl Keyboard {
    fn getch(&self) -> u8 {
        let gc = getch::Getch::new();
        let c = gc.getch().unwrap();
        // println!("getch: code 0x{:<02x}({0}), char {}", c, c as char);
        return c;
    }

    fn search(&self, node: &TreeNode, v: u8) -> Option<Key<KeyInfo>> {
        if node.value.contains(&v) {
            match &node.childs {
                TreeChild::Childs(childs) => {
                    let c = self.getch();
                    for child in childs {
                        if let Some(x) = &self.search(&child, c) {
                            return Some(x.clone());
                        }
                    }
                    return None;
                }
                TreeChild::Value(x) => return Some(x.clone()),
            }
        }
        return None;
    }

    pub fn get_input(&self) -> Key<KeyInfo> {
        let c = self.getch();
        if let Some(x) = self.search(&self.key_tree, c) {
            // println!("{:?}", x);
            return x;
        } else {
            return Key::Other(KeyInfo {
                raw_value: vec![c],
                dst_value: vec![c],
            });
        }
    }
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            #[cfg(windows)]
            key_tree: TreeRoot {
                value: vec![0xe0, 0x00],
                childs: TreeChild::Childs(vec![
                    Box::new(TreeNode {
                        value: vec!['H' as u8], // Up
                        childs: TreeChild::Value(Key::Up(KeyInfo {
                            raw_value: vec![0xe0, 'H' as u8],
                            dst_value: vec![0x1b, 0x5b, 'A' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['P' as u8], // Down
                        childs: TreeChild::Value(Key::Down(KeyInfo {
                            raw_value: vec![0xe0, 'P' as u8],
                            dst_value: vec![0x1b, 0x5b, 'B' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['M' as u8], // Right
                        childs: TreeChild::Value(Key::Right(KeyInfo {
                            raw_value: vec![0xe0, 'M' as u8],
                            dst_value: vec![0x1b, 0x5b, 'C' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['K' as u8], // Left
                        childs: TreeChild::Value(Key::Left(KeyInfo {
                            raw_value: vec![0xe0, 'K' as u8],
                            dst_value: vec![0x1b, 0x5b, 'D' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['G' as u8], // Home
                        childs: TreeChild::Value(Key::Home(KeyInfo {
                            raw_value: vec![0xe0, 'G' as u8],
                            dst_value: vec![0x1b, 0x5b, 'H' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['O' as u8], // End
                        childs: TreeChild::Value(Key::End(KeyInfo {
                            raw_value: vec![0xe0, 'O' as u8],
                            dst_value: vec![0x1b, 0x5b, 'F' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['R' as u8], // Insert
                        childs: TreeChild::Value(Key::Insert(KeyInfo {
                            raw_value: vec![0xe0, 'R' as u8],
                            dst_value: vec![0x1b, 0x5b, '2' as u8, '~' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['S' as u8], // Delete
                        childs: TreeChild::Value(Key::Delete(KeyInfo {
                            raw_value: vec![0xe0, 'S' as u8],
                            dst_value: vec![0x1b, 0x5b, '3' as u8, '~' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['I' as u8], // PageUp
                        childs: TreeChild::Value(Key::PageUp(KeyInfo {
                            raw_value: vec![0xe0, 'I' as u8],
                            dst_value: vec![0x1b, 0x5b, '5' as u8, '~' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['Q' as u8], // PageDown
                        childs: TreeChild::Value(Key::PageDown(KeyInfo {
                            raw_value: vec![0xe0, 'Q' as u8],
                            dst_value: vec![0x1b, 0x5b, '6' as u8, '~' as u8],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec![';' as u8], // F1
                        childs: TreeChild::Value(Key::F1(KeyInfo {
                            raw_value: vec![0x00, ';' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['<' as u8], // F2
                        childs: TreeChild::Value(Key::F2(KeyInfo {
                            raw_value: vec![0x00, '<' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['=' as u8], // F3
                        childs: TreeChild::Value(Key::F3(KeyInfo {
                            raw_value: vec![0x00, '=' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['>' as u8], // F4
                        childs: TreeChild::Value(Key::F4(KeyInfo {
                            raw_value: vec![0x00, '>' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['?' as u8], // F5
                        childs: TreeChild::Value(Key::F5(KeyInfo {
                            raw_value: vec![0x00, '?' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['@' as u8], // F6
                        childs: TreeChild::Value(Key::F6(KeyInfo {
                            raw_value: vec![0x00, '@' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['A' as u8], // F7
                        childs: TreeChild::Value(Key::F7(KeyInfo {
                            raw_value: vec![0x00, 'A' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['B' as u8], // F8
                        childs: TreeChild::Value(Key::F8(KeyInfo {
                            raw_value: vec![0x00, 'B' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['C' as u8], // F9
                        childs: TreeChild::Value(Key::F9(KeyInfo {
                            raw_value: vec![0x00, 'C' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec!['D' as u8], // F10
                        childs: TreeChild::Value(Key::F10(KeyInfo {
                            raw_value: vec![0x00, 'D' as u8],
                            dst_value: vec![],
                        })),
                    }),
                    Box::new(TreeNode {
                        value: vec![0x86], // F12
                        childs: TreeChild::Value(Key::F12(KeyInfo {
                            raw_value: vec![0xe0, 0x86],
                            dst_value: vec![],
                        })),
                    }),
                ]),
            },
            #[cfg(not(windows))]
            key_tree: TreeRoot {
                value: vec![0x1b],
                childs: TreeChild::Childs(vec![
                    Box::new(TreeNode {
                        value: vec![0x5b],
                        childs: TreeChild::Childs(vec![
                            Box::new(TreeNode {
                                value: vec!['A' as u8], // Up
                                childs: TreeChild::Value(Key::Up(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'A' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'A' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['B' as u8], // Down
                                childs: TreeChild::Value(Key::Down(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'B' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'B' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['C' as u8], // Right
                                childs: TreeChild::Value(Key::Right(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'C' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'C' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['D' as u8], // Left
                                childs: TreeChild::Value(Key::Left(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'D' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'D' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['H' as u8], // Home
                                childs: TreeChild::Value(Key::Home(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'H' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'H' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['F' as u8], // End
                                childs: TreeChild::Value(Key::End(KeyInfo {
                                    raw_value: vec![0x1b, 0x5b, 'F' as u8],
                                    dst_value: vec![0x1b, 0x5b, 'F' as u8],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['1' as u8],
                                childs: TreeChild::Childs(vec![
                                    Box::new(TreeNode {
                                        value: vec!['5' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F5
                                            childs: TreeChild::Value(Key::F5(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '1' as u8, '5' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['7' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F6
                                            childs: TreeChild::Value(Key::F6(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '1' as u8, '7' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['8' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F7
                                            childs: TreeChild::Value(Key::F7(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '1' as u8, '8' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['9' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F8
                                            childs: TreeChild::Value(Key::F8(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '1' as u8, '9' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                ]),
                            }),
                            Box::new(TreeNode {
                                value: vec!['2' as u8],
                                childs: TreeChild::Childs(vec![
                                    Box::new(TreeNode {
                                        value: vec!['~' as u8], // Insert
                                        childs: TreeChild::Value(Key::Insert(KeyInfo {
                                            raw_value: vec![0x1b, 0x5b, '2' as u8, '~' as u8],
                                            dst_value: vec![0x1b, 0x5b, '2' as u8, '~' as u8],
                                        })),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['0' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F9
                                            childs: TreeChild::Value(Key::F9(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '2' as u8, '0' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['1' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F10
                                            childs: TreeChild::Value(Key::F10(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '2' as u8, '1' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['3' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F11
                                            childs: TreeChild::Value(Key::F11(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '2' as u8, '3' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                    Box::new(TreeNode {
                                        value: vec!['4' as u8],
                                        childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                            value: vec!['~' as u8], // F12
                                            childs: TreeChild::Value(Key::F12(KeyInfo {
                                                raw_value: vec![
                                                    0x1b, 0x5b, '2' as u8, '4' as u8, '~' as u8,
                                                ],
                                                dst_value: vec![],
                                            })),
                                        })]),
                                    }),
                                ]),
                            }),
                            Box::new(TreeNode {
                                value: vec!['3' as u8],
                                childs: TreeChild::Childs(vec![Box::new(TreeNode {
                                    value: vec!['~' as u8], // Delete
                                    childs: TreeChild::Value(Key::Delete(KeyInfo {
                                        raw_value: vec![0x1b, 0x5b, '3' as u8, '~' as u8],
                                        dst_value: vec![0x1b, 0x5b, '3' as u8, '~' as u8],
                                    })),
                                })]),
                            }),
                        ]),
                    }),
                    Box::new(TreeNode {
                        value: vec![0x4f],
                        childs: TreeChild::Childs(vec![
                            Box::new(TreeNode {
                                value: vec!['P' as u8], // F1
                                childs: TreeChild::Value(Key::F1(KeyInfo {
                                    raw_value: vec![0x1b, 0x4f, 'P' as u8],
                                    dst_value: vec![],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['Q' as u8], // F2
                                childs: TreeChild::Value(Key::F2(KeyInfo {
                                    raw_value: vec![0x1b, 0x4f, 'Q' as u8],
                                    dst_value: vec![],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['R' as u8], // F3
                                childs: TreeChild::Value(Key::F3(KeyInfo {
                                    raw_value: vec![0x1b, 0x4f, 'R' as u8],
                                    dst_value: vec![],
                                })),
                            }),
                            Box::new(TreeNode {
                                value: vec!['S' as u8], // F4
                                childs: TreeChild::Value(Key::F4(KeyInfo {
                                    raw_value: vec![0x1b, 0x4f, 'S' as u8],
                                    dst_value: vec![],
                                })),
                            }),
                        ]),
                    }),
                ]),
            },
        }
    }
}

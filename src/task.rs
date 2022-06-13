extern crate serial;

use crate::cmd::SerialPortInfo;
use crate::{Input, InputMessage};
use serial::prelude::*;
use std::io::{prelude::*, stdout};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

pub struct TerminalSerial;

impl TerminalSerial {
    pub fn new() -> TerminalSerial {
        TerminalSerial {}
    }

    pub fn run(&self) {
        let mut handles = vec![];
        let (port, setting) = SerialPortInfo::new().get_info();

        let mut com_port = serial::open(port.as_str()).unwrap();
        com_port.configure(&setting).unwrap();
        com_port.set_timeout(Duration::from_millis(1)).unwrap();

        let quit = Arc::new(Mutex::new(false));
        let serial_port = Arc::new(Mutex::new(com_port));

        println!("{} is connected. Press 'Ctrl + ]' to quit.", port);

        let serial_port1 = Arc::clone(&serial_port);
        let quit1 = Arc::clone(&quit);
        handles.push(thread::spawn(move || {
            let input = Input::new();
            loop {
                match input.get_message() {
                    InputMessage::Quit => {
                        let mut quit = quit1.lock().unwrap();
                        *quit = true;
                        break;
                    }
                    InputMessage::Data(msg) => {
                        //println!("converted: {:?}", msg);
                        let mut serial_port = serial_port1.lock().unwrap();
                        if let Ok(_n) = serial_port.write(&msg) {
                            //println!("write {} bytes.", _n);
                        }
                    }
                    _ => (), // Ignored
                }
            }
        }));

        let serial_port2 = Arc::clone(&serial_port);
        let quit2 = Arc::clone(&quit);
        handles.push(thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 2048];
            loop {
                thread::sleep(Duration::from_millis(2));
                let mut serial_port = serial_port2.lock().unwrap();
                if let Ok(n) = serial_port.read(&mut buf[..]) {
                    if let Ok(_) = stdout().write(&buf[0..n]) { /* Ignored */ };
                    if let Ok(_) = stdout().flush() { /* Ignored */ };
                };
                let quit = quit2.lock().unwrap();
                if *quit {
                    break;
                }
            }
        }));

        handles.into_iter().for_each(|handle| {
            handle.join().unwrap();
        });
    }
}

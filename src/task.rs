extern crate serial;

use serial::prelude::*;
use std::io::{self, prelude::*};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use terminal_serial::input::InputMessage;
use terminal_serial::parse::SerialPortInfo;

pub struct TerminalSerial;

impl TerminalSerial {
    pub fn new() -> TerminalSerial {
        TerminalSerial
    }

    pub fn run(&self) {
        let mut handles = Vec::with_capacity(3);
        let (port, setting) = SerialPortInfo::get_info();
        let mut serial_port = serial::open(port.as_str()).unwrap();

        serial_port.configure(&setting).unwrap();
        serial_port.set_timeout(Duration::from_millis(1)).unwrap();

        println!("{} is connected. Press 'Ctrl + ]' to quit.", port);

        let quit = Arc::new(Mutex::new(false));
        let sp = Arc::new(Mutex::new(serial_port));

        let serial_port1 = Arc::clone(&sp);
        let quit1 = Arc::clone(&quit);
        handles.push(thread::spawn(move || loop {
            match InputMessage::get_message() {
                InputMessage::Quit => {
                    let mut quit = quit1.lock().unwrap();
                    *quit = true;
                    break;
                }
                InputMessage::Data(msg) => {
                    //println!("converted: {:?}", msg);
                    let mut serial_port = serial_port1.lock().unwrap();
                    if let Ok(_n) = serial_port.write(&msg) {
                        //println!("write {} bytes.", n);
                    };
                }
                _ => (), // Ignored
            }
        }));

        let serial_port2 = Arc::clone(&sp);
        let quit2 = Arc::clone(&quit);
        handles.push(thread::spawn(move || {
            let mut buf: Vec<u8> = vec![0; 2048];
            loop {
                thread::sleep(Duration::from_millis(2));
                let mut serial_port = serial_port2.lock().unwrap();
                if let Ok(n) = serial_port.read(&mut buf[..]) {
                    if let Ok(_) = io::stdout().write(&buf[0..n]) { /* Ignored */ };
                    if let Ok(_) = io::stdout().flush() { /* Ignored */ };
                };
                //thread::sleep(Duration::from_millis(1000));
                let quit = quit2.lock().unwrap();
                if *quit {
                    break;
                }
            }
        }));

        handles.into_iter().for_each(|handle| {
            handle.join().unwrap();
        })
    }
}
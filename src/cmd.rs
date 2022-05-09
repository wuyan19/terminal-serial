extern crate clap;
extern crate serial;

use clap::{load_yaml, App};
use std::io::{stdin, stdout, Write};
use std::process;

pub struct SerialPortInfo {
    port_list: Vec<String>,
    valid: bool,
}

impl SerialPortInfo {
    pub fn new() -> SerialPortInfo {
        SerialPortInfo { port_list: vec![], valid: false }
    }

    pub fn get_info(&mut self) -> (String, serial::PortSettings) {
        let cmd = load_yaml!("cmd.yml");
        let arg_matches = App::from_yaml(cmd).get_matches();
        let mut setting: serial::PortSettings = serial::PortSettings {
            baud_rate: serial::Baud115200,
            char_size: serial::Bits8,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
            flow_control: serial::FlowNone,
        };
        let mut port = String::new();

        if arg_matches.is_present("list") {
            self.show_port_list();
            process::exit(0);
        }
        match arg_matches.value_of("port") {
            Some(p) => {
                port.push_str(p);
            }
            None => {
                let mut line = String::new();
                self.show_port_list();
                loop {
                    if self.port_list.len() == 0 {
                        println!("There is no serial port to open.");
                        process::exit(0);
                    } else {
                        print!("Select <0~{}>: ", self.port_list.len() - 1);
                        if let Ok(()) = stdout().flush() {};
                        line.clear();
                        match stdin().read_line(&mut line) {
                            Ok(_n) => {
                                if let Ok(idx) = line.trim().parse::<usize>() {
                                    if let Some(com) = self.port_list.get(idx) {
                                        port.push_str(com);
                                        break;
                                    }
                                }
                            }
                            Err(error) => {
                                println!("Read line from stdin error({})", error);
                                process::exit(0);
                            }
                        }
                        println!("The input is invalid.");
                    }
                }
            }
        }

        if let Some(baudrate) = arg_matches.value_of("baudrate") {
            if let Ok(value) = baudrate.to_string().trim().parse::<usize>() {
                setting.baud_rate = serial::BaudRate::from_speed(value);
            } else {
                println!("Baudrate setting error.");
                process::exit(0);
            }
        }

        if let Some(parity) = arg_matches.value_of("parity") {
            match parity {
                "N" | "n" => setting.parity = serial::Parity::ParityNone,
                "O" | "o" => setting.parity = serial::Parity::ParityOdd,
                "E" | "e" => setting.parity = serial::Parity::ParityEven,
                _ => {
                    println!("Parity setting error.");
                    process::exit(0);
                }
            }
        }

        if let Some(datasize) = arg_matches.value_of("datasize") {
            match datasize {
                "5" => setting.char_size = serial::CharSize::Bits5,
                "6" => setting.char_size = serial::CharSize::Bits6,
                "7" => setting.char_size = serial::CharSize::Bits7,
                "8" => setting.char_size = serial::CharSize::Bits8,
                _ => {
                    println!("Datasize setting error.");
                    process::exit(0);
                }
            }
        }

        if let Some(stopbits) = arg_matches.value_of("stopbits") {
            match stopbits {
                "1" => setting.stop_bits = serial::StopBits::Stop1,
                "2" => setting.stop_bits = serial::StopBits::Stop2,
                _ => {
                    println!("Stopbits setting error.");
                    process::exit(0);
                }
            }
        }

        if let Some(flowcontrol) = arg_matches.value_of("flowcontrol") {
            match flowcontrol {
                "N" | "n" => setting.flow_control = serial::FlowControl::FlowNone,
                "S" | "s" => setting.flow_control = serial::FlowControl::FlowSoftware,
                "H" | "h" => setting.flow_control = serial::FlowControl::FlowHardware,
                _ => {
                    println!("Flow control setting error.");
                    process::exit(0);
                }
            }
        }

        (port, setting)
    }

    fn show_port_list(&mut self) {
        if !self.valid {
            self.port_list = crate::serial_port::get_list();
            self.valid = true;
        }

        println!("---------------------------");
        println!("    Serial Port List       ");
        println!("---------------------------");
        for (idx, value) in self.port_list.iter().enumerate() {
            println!("{} - {}", idx, value);
        }
        println!("---------------------------");
    }
}

extern crate clap;
extern crate serial;

use clap::{load_yaml, App};
use std::io::{self, Write};
use std::process;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

struct SerialPorts {
    port_list: Vec<String>,
    valid: bool,
}

impl SerialPorts {
    fn new() -> SerialPorts {
        SerialPorts {
            port_list: vec![],
            valid: false,
        }
    }

    fn get_list(&mut self, show: bool) -> &Vec<String> {
        if self.valid {
            if show {
                self.show_list();
            }
            return &self.port_list;
        }

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
                    self.port_list.push(tmp);
                }
            }
        }
        self.valid = true;

        if show {
            self.show_list();
        }

        &self.port_list
    }

    fn show_list(&self) {
        println!("---------------------------");
        println!("    Serial Port List       ");
        println!("---------------------------");
        for (idx, value) in self.port_list.iter().enumerate() {
            println!("{} - {}", idx, value);
        }
        println!("---------------------------");
    }
}

pub struct SerialPortInfo;

impl SerialPortInfo {
    pub fn get_info() -> (String, serial::PortSettings) {
        let cmd = load_yaml!("cmd.yml");
        let arg_matches = App::from_yaml(cmd).get_matches();
        let mut serial_port = SerialPorts::new();
        let mut setting: serial::PortSettings = serial::PortSettings {
            baud_rate: serial::Baud115200,
            char_size: serial::Bits8,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
            flow_control: serial::FlowNone,
        };
        let mut port = String::new();

        if arg_matches.is_present("list") {
            serial_port.get_list(true);
            process::exit(0);
        }
        match arg_matches.value_of("port") {
            Some(p) => {
                port.push_str(p);
            }
            None => {
                let port_list = serial_port.get_list(true);
                let mut line = String::new();
                loop {
                    if port_list.len() == 0 {
                        println!("There is no serial port to open.");
                        process::exit(0);
                    }
                    print!("Select <0~{}>: ", port_list.len() - 1);
                    if let Ok(()) = io::stdout().flush() {};
                    line.clear();
                    match io::stdin().read_line(&mut line) {
                        Ok(_n) => {
                            if let Ok(idx) = line.trim().parse::<usize>() {
                                if let Some(com) = port_list.get(idx) {
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
                "N" => setting.parity = serial::Parity::ParityNone,
                "O" => setting.parity = serial::Parity::ParityOdd,
                "E" => setting.parity = serial::Parity::ParityEven,
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
                "N" => setting.flow_control = serial::FlowControl::FlowNone,
                "S" => setting.flow_control = serial::FlowControl::FlowSoftware,
                "H" => setting.flow_control = serial::FlowControl::FlowHardware,
                _ => {
                    println!("Flow control setting error.");
                    process::exit(0);
                }
            }
        }

        (port, setting)
    }
}

extern crate clap;
extern crate serial;

use clap::{load_yaml, App};
use std::io::{stdin, stdout, Write};
use std::process;

pub struct AppConfig {
    pub port: String,
    pub setting: serial::PortSettings,
    pub serve: bool,
    pub mcp_host: String,
    pub mcp_port: u16,
}

fn get_serial_port_list() -> Vec<String> {
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
            #[cfg(target_os = "macos")]
            if file.contains("tty.") && file.contains("usb") {
                port_list.push(file);
            }
            #[cfg(target_os = "linux")]
            if file.contains("ttyUSB") {
                port_list.push(file);
            }
        }
    }

    port_list
}

pub fn cmd_parse() -> AppConfig {
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
    let port_list = get_serial_port_list();
    let show_serial_port_list = |port_list: &Vec<String>| {
        println!("---------------------------");
        println!("    Serial Port List       ");
        println!("---------------------------");
        for (idx, value) in port_list.iter().enumerate() {
            println!("{} - {}", idx + 1, value);
        }
        println!("---------------------------");
    };

    if arg_matches.is_present("list") {
        show_serial_port_list(&port_list);
        process::exit(0);
    }

    match arg_matches.value_of("port") {
        Some(p) => {
            port.push_str(p);
        }
        None => {
            let mut line = String::new();
            show_serial_port_list(&port_list);
            loop {
                if port_list.len() == 0 {
                    println!("There is no serial port to open.");
                    process::exit(0);
                } else {
                    print!("Select <1~{}>: ", port_list.len());
                    if let Ok(()) = stdout().flush() {};
                    line.clear();
                    match stdin().read_line(&mut line) {
                        Ok(_n) => {
                            if let Ok(idx) = line.trim().parse::<usize>() {
                                if let Some(com) = port_list.get(idx - 1) {
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

    let serve = arg_matches.is_present("server");

    let mcp_host = match arg_matches.value_of("mcp-host") {
        Some(h) => h.to_string(),
        None => "0.0.0.0".to_string(),
    };

    let mcp_port = match arg_matches.value_of("mcp-port") {
        Some(p) => match p.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                println!("MCP port setting error.");
                process::exit(0);
            }
        },
        None => 8765,
    };

    AppConfig {
        port,
        setting,
        serve,
        mcp_host,
        mcp_port,
    }
}

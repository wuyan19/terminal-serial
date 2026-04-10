extern crate clap;

use clap::{load_yaml, App};
use std::io::{stdin, stdout, Write};
use std::process;

pub struct AppConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: serialport::DataBits,
    pub parity: serialport::Parity,
    pub stop_bits: serialport::StopBits,
    pub flow_control: serialport::FlowControl,
    pub serve: bool,
    pub mcp_host: String,
    pub mcp_port: u16,
}

fn get_serial_port_list() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

pub fn cmd_parse() -> AppConfig {
    let cmd = load_yaml!("cmd.yml");
    let arg_matches = App::from_yaml(cmd).get_matches();

    let mut baud_rate: u32 = 115200;
    let mut data_bits = serialport::DataBits::Eight;
    let mut parity = serialport::Parity::None;
    let mut stop_bits = serialport::StopBits::One;
    let mut flow_control = serialport::FlowControl::None;
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
        match baudrate.trim().parse::<u32>() {
            Ok(v) => baud_rate = v,
            Err(_) => {
                println!("Baudrate setting error.");
                process::exit(0);
            }
        }
    }

    if let Some(p) = arg_matches.value_of("parity") {
        match p {
            "N" | "n" => parity = serialport::Parity::None,
            "O" | "o" => parity = serialport::Parity::Odd,
            "E" | "e" => parity = serialport::Parity::Even,
            _ => {
                println!("Parity setting error.");
                process::exit(0);
            }
        }
    }

    if let Some(d) = arg_matches.value_of("datasize") {
        match d {
            "5" => data_bits = serialport::DataBits::Five,
            "6" => data_bits = serialport::DataBits::Six,
            "7" => data_bits = serialport::DataBits::Seven,
            "8" => data_bits = serialport::DataBits::Eight,
            _ => {
                println!("Datasize setting error.");
                process::exit(0);
            }
        }
    }

    if let Some(s) = arg_matches.value_of("stopbits") {
        match s {
            "1" => stop_bits = serialport::StopBits::One,
            "2" => stop_bits = serialport::StopBits::Two,
            _ => {
                println!("Stopbits setting error.");
                process::exit(0);
            }
        }
    }

    if let Some(f) = arg_matches.value_of("flowcontrol") {
        match f {
            "N" | "n" => flow_control = serialport::FlowControl::None,
            "S" | "s" => flow_control = serialport::FlowControl::Software,
            "H" | "h" => flow_control = serialport::FlowControl::Hardware,
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
        baud_rate,
        data_bits,
        parity,
        stop_bits,
        flow_control,
        serve,
        mcp_host,
        mcp_port,
    }
}

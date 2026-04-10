use clap::Parser;
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

/// 终端串口通信工具
#[derive(Parser, Debug)]
#[command(name = "terminal-serial", version, about)]
struct Cli {
    /// 串口名称
    #[arg(short = 'p', long)]
    port: Option<String>,

    /// 波特率，默认 115200
    #[arg(short = 'b', long, default_value_t = 115200)]
    baud_rate: u32,

    /// 校验位：N|O|E，默认 N
    #[arg(short = 'a', long)]
    parity: Option<String>,

    /// 数据位：5|6|7|8，默认 8
    #[arg(short = 'd', long)]
    datasize: Option<String>,

    /// 停止位：1|2，默认 1
    #[arg(short = 's', long)]
    stopbits: Option<String>,

    /// 流控：N|S|H，默认 N
    #[arg(short = 'f', long)]
    flowcontrol: Option<String>,

    /// 列出可用串口
    #[arg(short = 'l', long)]
    list: bool,

    /// 启用 MCP 服务器模式
    #[arg(short = 'S', long)]
    server: bool,

    /// MCP 服务器端口，默认 8765
    #[arg(short = 'P', long, default_value_t = 8765)]
    mcp_port: u16,

    /// MCP 服务器绑定地址，默认 0.0.0.0
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    mcp_host: String,
}

fn get_serial_port_list() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

pub fn cmd_parse() -> AppConfig {
    let cli = Cli::parse();

    let port_list = get_serial_port_list();

    if cli.list {
        println!("---------------------------");
        println!("    Serial Port List       ");
        println!("---------------------------");
        for (idx, value) in port_list.iter().enumerate() {
            println!("{} - {}", idx + 1, value);
        }
        println!("---------------------------");
        process::exit(0);
    }

    let port = match cli.port {
        Some(p) => p,
        None => {
            let mut line = String::new();
            println!("---------------------------");
            println!("    Serial Port List       ");
            println!("---------------------------");
            for (idx, value) in port_list.iter().enumerate() {
                println!("{} - {}", idx + 1, value);
            }
            println!("---------------------------");

            loop {
                if port_list.is_empty() {
                    println!("There is no serial port to open.");
                    process::exit(1);
                }
                print!("Select <1~{}>: ", port_list.len());
                let _ = stdout().flush();
                line.clear();
                match stdin().read_line(&mut line) {
                    Ok(_n) => {
                        if let Ok(idx) = line.trim().parse::<usize>() {
                            if let Some(com) = port_list.get(idx - 1) {
                                break com.clone();
                            }
                        }
                    }
                    Err(error) => {
                        println!("Read line from stdin error({})", error);
                        process::exit(1);
                    }
                }
                println!("The input is invalid.");
            }
        }
    };

    let parity = match cli.parity.as_deref() {
        None | Some("N") | Some("n") => serialport::Parity::None,
        Some("O") | Some("o") => serialport::Parity::Odd,
        Some("E") | Some("e") => serialport::Parity::Even,
        _ => {
            println!("Parity setting error.");
            process::exit(1);
        }
    };

    let data_bits = match cli.datasize.as_deref() {
        None | Some("8") => serialport::DataBits::Eight,
        Some("5") => serialport::DataBits::Five,
        Some("6") => serialport::DataBits::Six,
        Some("7") => serialport::DataBits::Seven,
        _ => {
            println!("Datasize setting error.");
            process::exit(1);
        }
    };

    let stop_bits = match cli.stopbits.as_deref() {
        None | Some("1") => serialport::StopBits::One,
        Some("2") => serialport::StopBits::Two,
        _ => {
            println!("Stopbits setting error.");
            process::exit(1);
        }
    };

    let flow_control = match cli.flowcontrol.as_deref() {
        None | Some("N") | Some("n") => serialport::FlowControl::None,
        Some("S") | Some("s") => serialport::FlowControl::Software,
        Some("H") | Some("h") => serialport::FlowControl::Hardware,
        _ => {
            println!("Flow control setting error.");
            process::exit(1);
        }
    };

    AppConfig {
        port,
        baud_rate: cli.baud_rate,
        data_bits,
        parity,
        stop_bits,
        flow_control,
        serve: cli.server,
        mcp_host: cli.mcp_host,
        mcp_port: cli.mcp_port,
    }
}

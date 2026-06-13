use terminal_serial::cmd::{cmd_parse, Command};
use terminal_serial::task::TerminalSerial;

fn main() {
    match cmd_parse() {
        Command::Connect(config) => {
            let task = TerminalSerial::new(&config);
            task.run();
        }
        Command::Log(config) => {
            if let Err(e) = terminal_serial::log_reader::run(&config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

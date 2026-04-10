use terminal_serial::task::TerminalSerial;

fn main() {
    let config = terminal_serial::cmd::cmd_parse();
    let task = TerminalSerial::new(&config);
    task.run();
}

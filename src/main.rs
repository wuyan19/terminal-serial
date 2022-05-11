use terminal_serial::task::TerminalSerial;

fn main() {
    let task = TerminalSerial::new();
    task.run();
}

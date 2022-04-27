mod task;

use task::TerminalSerial;

fn main() {
    let ts = TerminalSerial::new();
    ts.run();
}

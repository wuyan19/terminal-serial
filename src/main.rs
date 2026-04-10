use terminal_serial::task::TerminalSerial;

fn main() {
    let config = terminal_serial::cmd::cmd_parse();
    let task = TerminalSerial::new(config.port.as_str(), config.setting);
    task.run(config.serve, &config.mcp_host, config.mcp_port);
}

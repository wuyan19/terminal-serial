use terminal_serial::task::TerminalSerial;

fn main() {
    let config = terminal_serial::cmd::cmd_parse();
    let task = TerminalSerial::new(
        config.port.as_str(),
        config.baud_rate,
        config.data_bits,
        config.parity,
        config.stop_bits,
        config.flow_control,
    );
    task.run(config.serve, &config.mcp_host, config.mcp_port);
}

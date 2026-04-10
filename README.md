# terminal-serial

## Introduce
A terminal serial port tool developed in the **Rust** language.

## Install

```shell
# cd terminal-serial
# cargo install --path .
```

## Instructions
- **Help**
```shell
USAGE:
    terminal-serial.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -l, --list       List serial ports
    -S, --server     Enable MCP server mode
    -V, --version    Prints version information

OPTIONS:
    -b, --baudrate <INTEGER>     Set baud rate, 115200 as default
    -d, --datasize <5|6|7|8>     Set data size, 8 as default
    -f, --flowcontrol <N|S|H>    Set flow control, 'N' as default
    -H, --mcp-host <HOST>        MCP HTTP server bind address, 0.0.0.0 as default
    -P, --mcp-port <PORT>        MCP HTTP server port, 8765 as default
    -a, --parity <N|O|E>         Set parity, 'N' as default
    -p, --port <TEXT>            Serial port name
    -s, --stopbits <1|2>         Set stop bits, 1 as default
```
- **Example**
```shell
# terminal-serial
---------------------------
    Serial Port List
---------------------------
1 - COM3
2 - COM8
---------------------------
Select <1~2>: 1
COM3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p com3 -b 115200 -d 8 -s 1 -a N -f N
com3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p /dev/tty.usbserial -b 115200 -d 8 -s 1 -a N -f N
/dev/tty.usbserial is connected. Press 'Ctrl + ]' to quit.
```

## MCP Server

terminal-serial supports MCP (Model Context Protocol) server mode, allowing AI tools like Claude Code to interact with the serial port via HTTP.

### Quick Start

```shell
# Start with MCP server enabled
terminal-serial --server -p COM3 -b 115200

# Specify MCP server address and port
terminal-serial --server --mcp-host 0.0.0.0 --mcp-port 9000 -p COM3
```

When `--server` is enabled, an HTTP MCP server starts alongside the interactive terminal. You can use the serial port normally while Claude Code is connected.

Keyboard shortcuts in serve mode:
- `Ctrl + ]` — Quit
- `Ctrl + K` — Clear MCP read buffer

### Claude Code Configuration

Add the following to your `.mcp.json`:

```json
{
  "mcpServers": {
    "serial": {
      "type": "http",
      "url": "http://<host>:8765/mcp"
    }
  }
}
```

Replace `<host>` with the IP address of the machine running terminal-serial (e.g., `192.168.20.175`).

### Available Tools

| Tool | Description |
|------|-------------|
| `serial_send` | Send data to serial port. Supports text/hex format. Auto-appends `\r\n` in text mode. Optionally waits for device response. |
| `serial_read` | Read data from serial receive buffer. Supports text/hex format. |
| `serial_status` | Get serial port connection status and configuration. |

### Example: AT Command via MCP

```json
{
  "name": "serial_send",
  "arguments": {
    "data": "AT",
    "wait_response": true,
    "timeout_ms": 2000
  }
}
```

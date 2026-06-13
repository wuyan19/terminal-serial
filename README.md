# terminal-serial

## Introduce
A terminal serial port tool developed in the **Rust** language. Supports local terminal, MCP server, and Telnet server for multi-client remote serial access.

## Install

### From Source

```shell
cd terminal-serial
cargo install --path .
```

### From Release

Download the binary for your platform from the [Releases](https://github.com/wuyan19/terminal-serial/releases) page.

> **macOS Users**: The binary is unsigned. If macOS blocks it with "cannot be opened because it is from an unidentified developer", run:
> ```shell
> xattr -d com.apple.quarantine terminal-serial-*
> ```

## Instructions
- **Help**
```shell
Usage: terminal-serial [OPTIONS]

Options:
  -p, --port <PORT>                Serial port name
  -b, --baud-rate <BAUD_RATE>      Baud rate [default: 115200]
  -a, --parity <PARITY>            Parity: N|O|E
  -d, --datasize <DATASIZE>        Data bits: 5|6|7|8
  -s, --stopbits <STOPBITS>        Stop bits: 1|2
  -f, --flowcontrol <FLOWCONTROL>  Flow control: N|S|H
  -l, --list                       List available serial ports
  -M, --mcp                        Enable MCP server
      --mcp-port <MCP_PORT>        MCP server port [default: 8765]
      --mcp-host <MCP_HOST>        MCP server bind address [default: 0.0.0.0]
  -T, --telnet                     Enable Telnet server
      --telnet-port <TELNET_PORT>  Telnet server port [default: 8766]
      --telnet-host <TELNET_HOST>  Telnet server bind address [default: 0.0.0.0]
      --event-log <EVENT_LOG>      Write events as JSONL
      --config <CONFIG>            Path to tool config file (JSON, currently defines macros)
  -h, --help                       Print help
  -V, --version                    Print version
```
- **Example**
```shell
terminal-serial
# ---------------------------
#     Serial Port List
# ---------------------------
# 1 - COM3
# 2 - COM8
# ---------------------------
# Select <1~2>: 1
# COM3 is connected. Press 'Ctrl + ]' to quit.

terminal-serial -p com3 -b 115200 -d 8 -s 1 -a N -f N
# com3 is connected. Press 'Ctrl + ]' to quit.

terminal-serial -p /dev/tty.usbserial -b 115200 -d 8 -s 1 -a N -f N
# /dev/tty.usbserial is connected. Press 'Ctrl + ]' to quit.
```

## MCP Server

terminal-serial supports MCP (Model Context Protocol) server mode, allowing AI tools like Claude Code to interact with the serial port via HTTP.

### Quick Start

```shell
# Start with MCP server enabled
terminal-serial -M -p COM3 -b 115200

# Specify MCP server address and port
terminal-serial -M --mcp-host 0.0.0.0 --mcp-port 9000 -p COM3
```

When `-M` is enabled, an HTTP MCP server starts alongside the interactive terminal. You can use the serial port normally while Claude Code is connected.

Keyboard shortcuts in serve mode:
- `Ctrl + ]` — Quit
- `Ctrl + K` — Clear RX buffer

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
| `serial_status` | Get serial port connection status and configuration. |
| `serial_send` | Send data to serial port. Supports text/hex format. Auto-appends `\r\n` in text mode. Optionally waits for device response. |
| `serial_read` | Read data from serial receive buffer (destructive — buffer is cleared after read). Supports text/hex format. |
| `serial_grep` | Search receive buffer for matching pattern without clearing it. Supports regex in text mode and byte sequences in hex mode. Useful for waiting for specific output. |
| `serial_clear` | Clear all data in the receive buffer. |

### Example: AT Command via MCP

```json
{
  "name": "serial_send",
  "arguments": {
    "data": "AT",
    "timeout_ms": 2000
  }
}
```

### Example: Wait for Device Ready

```json
{
  "name": "serial_grep",
  "arguments": {
    "pattern": "Kernel started",
    "timeout_ms": 5000
  }
}
```

### Example: Send Hex Data

```json
{
  "name": "serial_send",
  "arguments": {
    "data": "AA55",
    "format": "hex"
  }
}
```

## Telnet Server

terminal-serial supports Telnet server mode, allowing multiple remote clients to access the serial port simultaneously via standard telnet clients.

### Quick Start

```shell
# Local terminal + Telnet server
terminal-serial -T -p COM3 -b 115200

# Local terminal + MCP + Telnet
terminal-serial -M -T -p COM3 -b 115200

# Custom Telnet port
terminal-serial -T --telnet-port 3000 -p COM3
```

When `-T` is enabled, a Telnet server starts on port 8766 (default). Remote clients can connect using any standard telnet client:

```shell
telnet <host> 8766
```

Features:
- **Multi-client**: Multiple telnet clients can connect simultaneously
- **Coexistence**: Local terminal, MCP server, and telnet clients share the same serial port
- **Broadcast**: Serial output is broadcast to all connected clients in real-time
- **Telnet protocol**: Handles IAC negotiation and CR/NUL conventions

## Configuration File

terminal-serial supports a JSON configuration file for defining reusable macros — sequences of `send` / `delay` / `expect` / `clear` steps that automate repetitive serial interactions like device initialization, login, or debug command sequences.

### Quick Start

```shell
terminal-serial -p COM3 -b 115200 --config config.example.json
```

A sample config is provided at [config.example.json](config.example.json). Copy it as a starting point:

```shell
cp config.example.json my-config.json
# Edit my-config.json as needed
terminal-serial -p COM3 --config my-config.json
```

On startup, the tool prints a compact macro list:

```
COM3 is connected. Press 'Ctrl + ]' to quit.
Macros: [1]init [2]login [3]ping [4]reboot (Ctrl+O for menu)
```

### JSON Schema

```json
{
  "macros": {
    "init": {
      "description": "Device initialization",
      "steps": [
        { "type": "send", "data": "ATZ", "format": "text", "auto_newline": true },
        { "type": "delay", "ms": 500 },
        { "type": "send", "data": "ATE0", "auto_newline": true },
        { "type": "expect", "pattern": "OK", "timeout_ms": 3000 },
        { "type": "clear" }
      ]
    },
    "login": {
      "steps": [
        { "type": "send", "data": "0D0A", "format": "hex" }
      ]
    }
  }
}
```

Macro ordering is determined alphabetically by key (BTreeMap), so `[1]` always maps to the first key, `[2]` to the second, and so on.

### Step Types

| Step | Fields | Description |
|------|--------|-------------|
| `send` | `data` (required), `format` (`text` \| `hex` \| `raw`, default `text`), `auto_newline` (default `true`, appends `\r\n`) | Send data to serial port. `hex` format interprets `data` as a hex string (e.g. `"0D0A"`). |
| `delay` | `ms` | Wait `ms` milliseconds before the next step. |
| `expect` | `pattern` (regex), `timeout_ms` | Wait until a line matching `pattern` appears in the RX buffer, or fail on timeout. Use after a `send`+`delay` to verify device response. |
| `clear` | — | Clear the receive buffer. Useful before a fresh command sequence. |

### Triggering Macros

Press `Ctrl + O` to list macros and enter menu selection mode:

```
=== Macros ===
[1] init - Device initialization
[2] login
[3] ping
[4] reboot
(Press 1-9 to run, any key to exit)
```

- Press `1`–`9` to execute the corresponding macro.
- Press any other key (including `Enter`, `Esc`, arrows) to exit the menu without running.
- `Ctrl + ]` (quit) and `Ctrl + K` (clear buffer) still take effect immediately inside the menu.

Macro execution feedback is shown with ANSI colors:

```
▶ [macro: init]        (cyan, start)
<serial output...>
✓ [macro: init done]   (green, success)
```

On failure (e.g. `expect` timeout):

```
✗ [macro: init failed: expect "OK" timeout]   (red)
```

Macro execution is also recorded as `action` events when `--event-log` is enabled (see [Event Log](#event-log)).

## Event Log

Use `--event-log` to record the serial hub event stream as JSONL (one JSON object per line):

```shell
terminal-serial -p COM3 -T --event-log events.jsonl
```

Event types:

| Event | Description |
|-------|-------------|
| `startup` | Hub started, includes port name |
| `shutdown` | Hub stopped |
| `client_connected` | Client connected (source: mcp/telnet) |
| `client_disconnected` | Client disconnected |
| `tx` | Data sent to serial port (source: local/mcp/telnet) |
| `rx` | Data received from serial port |
| `action` | Control action performed (source: local). `action` field is `run_macro` or `clear_buffer`; `name` field carries the macro name when applicable. |
| `error` | Error occurred |

Example output:

```json
{"ts":"2026-06-12T10:00:00Z","event":"startup","port":"COM3"}
{"ts":"2026-06-12T10:00:02Z","event":"client_connected","source":"telnet","client":"192.168.1.100:52344"}
{"ts":"2026-06-12T10:00:03Z","event":"tx","source":"telnet","data":"68656C6C6F0D0A"}
{"ts":"2026-06-12T10:00:04Z","event":"rx","data":"4F4B0D0A"}
{"ts":"2026-06-12T10:00:05Z","event":"action","source":"local","action":"run_macro","name":"init"}
{"ts":"2026-06-12T10:00:06Z","event":"action","source":"local","action":"clear_buffer"}
```

`data` fields are hex-encoded.

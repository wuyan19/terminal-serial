# terminal-serial

## Introduce
A terminal serial port tool developed in the **Rust** language.

## Install

```shell
# cd terminal-serial
# cargo install --path .
```

## Instructions
***Help***
```shell
USAGE:
    terminal-serial [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -l, --list       List serial ports
    -V, --version    Prints version information

OPTIONS:
    -b, --baudrate <INTEGER>     Set baud reate, 115200 as default
    -d, --datasize <5|6|7|8>     Set datasize, 8 as default
    -f, --flowcontrol <N|S|H>    Set flow control, 'N' as default
    -a, --parity <N|O|E>         Set parity, 'N' as default
    -p, --port <TEXT>            Serial port name
    -s, --stopbits <1|2>         Set stop bits, 1 as default
```
***Example***
```shell
# terminal-serial
---------------------------
    Serial Port List
---------------------------
0 - COM3
1 - COM8
---------------------------
Select <0~1>: 0
com3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p com3 -b 115200 -d 8 -s 1 -a N -f N
com3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p /dev/tty.usbserial -b 115200 -d 8 -s 1 -a N -f N
/dev/tty.usbserial is connected. Press 'Ctrl + ]' to quit.
```

## Release
- **Windows**
    - [terminal-serial.zip](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.zip) - 359K
    - [terminal-serial.tar.gz](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.tar.gz) - 370K
    - [terminal-serial.7z](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.7z) - 273K
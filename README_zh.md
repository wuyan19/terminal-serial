# terminal-serial

## 介绍
使用 **Rust** 语言开发的命令行串口工具。

## 安装

```shell
# cd terminal-serial
# cargo install --path .
```

## 使用说明
- **帮助**
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
- **示例**
```shell
# terminal-serial
---------------------------
Serial Port List
---------------------------
0 - COM3
1 - COM8
---------------------------
Select <0~1>: 0
COM3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p com3 -b 115200 -d 8 -s 1 -a N -f N
com3 is connected. Press 'Ctrl + ]' to quit.

# terminal-serial -p /dev/tty.usbserial -b 115200 -d 8 -s 1 -a N -f N
/dev/tty.usbserial is connected. Press 'Ctrl + ]' to quit.
```

## 发布
- **Windows**
    - [terminal-serial.zip (359KB)](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.zip)
    - [terminal-serial.tar.gz (370KB)](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.tar.gz)
    - [terminal-serial.7z (273KB)](https://gitee.com/wuyan19/application/blob/master/terminal-serial/windows/terminal-serial.7z)
- **macOS**
    - [terminal-serial.zip (499KB)](https://gitee.com/wuyan19/application/blob/master/terminal-serial/macos/terminal-serial.zip)
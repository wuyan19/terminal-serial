# terminal-serial

#### 介绍
一款使用 `rust` 开发的终端命令行串口工具。

#### 安装教程

1.  进入源码目录 `cd terminal-serial`
2.  使用 `cargo` 构建应用程序 `cargo install --path .`

#### 使用说明

1.  使用命令 `terminal-serial -V` 查看版本信息
2.  使用命令 `terminal-serial -h` 查看帮助信息
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
3. 在windows下可以使用如下两种方法
    ```shell
    # terminal-serial
    # terminal-serial -p com3 -b 115200 -d 8 -s 1 -a N -f N
    ```
4. 在macOS下可以使用如下两种方法
    ```shell
    # terminal-serial
    # terminal-serial -p /dev/tty.usbserial -b 115200 -d 8 -s 1 -a N -f N
    ```
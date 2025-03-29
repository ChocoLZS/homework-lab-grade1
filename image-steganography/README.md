# 图片信息隐写

bmp，替换图像像素某个字节的位来达到隐藏信息的目的

bmp格式：https://geocld.github.io/2021/03/02/bmp

## 使用说明

```bash
./image-steganography.exe --help                                                                                                                                                                         
Simple program to greet a person

Usage: image-steganography.exe <COMMAND>

Commands:
  hide
  extract
  capacity
  debug
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## 开发说明

安装rust工具链

```bash
cargo run
```
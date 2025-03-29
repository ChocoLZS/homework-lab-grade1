#import "../base.typ": *
#import "../base_info.typ": base_project_info

#show: project.with(
    title: base_project_info.title,
    name: base_project_info.name,
    id: base_project_info.id,
    grade: base_project_info.grade,
    authors: base_project_info.authors,
    department: base_project_info.department,
    date: (2025, 3, 29),
    cover_style: base_project_info.cover_style,
)

#toc()
#pagebreak()

= 基本说明

使用Rust编写的图片隐写工具，支持BMP格式的图片隐写和提取。
该工具可以将文本信息隐藏在BMP格式的图片中，并且可以从图片中提取出隐藏的信息。

- *使用的设计语言：*Rust
- *图片（BMP）文件格式参考：*#link("https://geocld.github.io/2021/03/02/bmp/")[解读BMP图片]

= 分析设计

== 基本功能

- *基本功能*:
    #set enum(numbering: "a)")
    
    + 隐藏命令：`image-steganography.exe hide`
    
    + 提取命令：`image-steganography.exe extract`
    
    + 获取容量命令：`image-steganography.exe capacity`
    
    + 显示图片字节数据命令：`image-steganography.exe debug`

== 分析设计

=== 功能分析

对于图片隐写需求来说，需要基本的隐藏信息和提取信息的功能。

为了让用户了解图片的隐写容量，设计了获取容量的功能。

此外，为了方便调试，此工具还提供了显示图片字节数据的功能。

=== BMP图片分析

#figure(
    image("assets/BMPFileStructure.png",width: 60%,),
    caption: "BMP文件结构",
) <a>


文件头与位图信息头总共占用54个字节，并且位图信息头里有图片数据的起始位置。为了保证图片元信息不受到影响，隐写信息的存储位置只会在图片数据里。

由于BMP格式的图片数据是以行扫描的方式存储的，扫描行的原则需要每行字节数为4的倍数。出于隐藏信息的目的考虑，本工具不会将信息填充至*padding(0x00)*中，这样可以尽可能地减少被发现隐写的概率。

=== 隐写设计分析

由于BMP有多种类型，比如8位、16位、24位等，隐写的设计需要考虑到不同类型的BMP图片。对于每个像素，比如8位的BMP图片每个像素只占用一个字节，而24位的BMP图片每个像素占用3个字节。

#figure(
    image("assets/24位5x5.jpg",width: 80%,),
    caption: "24位BMP图片字节数据",
)

可以看见每个像素占用3个字节，分别是B、G、R三个通道，最后一列00是padding。

#figure(
    image("assets/16位5x5.jpg",width: 80%,),
    caption: "16位BMP图片字节数据",
)

同样地，每个像素占用2个字节，最后一列00是padding。

为了隐写信息，我们可以将信息按顺序依次隐藏在每个像素的最低有效位中。但由于人眼分辨不出这么多的颜色差异，所以我们甚至可以将信息隐藏在每个像素的最低几位中。

即一个字节的数据，我们会将其分为8bit，分别存储至8个字节像素的最低有效位中。
比如一个字节数据`0b01010101`，我们可以将其分为8个bit，分别存储在8个像素的最低有效位中。
比如第一个像素的R值为`0b00000000`，我们可以提取`0b01010101`的最低位，并入R值中，即`0b00000001`。这样就可以将信息存储在图片中。

此外，我们需要记录下隐藏信息的长度。在本实验设计中，给信息长度记录分配了4个字节（32位），分别存储在图片数据的前32个像素字节中。

即图片隐写数据格式为:

$$$
    "4字节信息长度" + "信息数据"
$$$

考虑到padding以及BMP图片的位数，隐写数据的最小存储长度为：

$"像素字节数" = "长" dot "宽" dot "位数" / 3$

$"最小存储长度" = "像素字节数" - "4字节信息长度" dot 8$

如果我们不仅仅在每个像素的最低有效位中存储信息，而是将信息存储在每个像素的最低几位中，那么我们可以将信息存储在更多的像素中。令”最低几位“为_layer_：

$"信息最大存储数量（bit）" = "最小存储长度" dot "layer"$

实际上对于一个bmp图片来说，存储的字节数应该是

$"信息最大存储数量（bit）"\ 
    &= "图片大小" - "图片头大小" - "调色板大小" - "4字节信息长度（32）" - "padding数量"$

= 代码实现 <experiment>

```bash
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

== 简要流程

程序通过子命令读取图片，转为程序方便分析的结构体，根据子命令来进行相关操作。

```Rust
pub struct BmpImage {
    pub header: BmpHeader,
    pub raw_header: Vec<u8>,
    pub raw_body: Vec<u8>,
    pub extra_info: ExtraInfo,
}
```

#pagebreak()

== hide - 隐藏信息

隐藏用户输入的信息到图片中。

```bash
Usage: image-steganography.exe hide [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>
  -o, --output <OUTPUT>
  -m, --message <MESSAGE>  [default: "Hello, world!"]
  -l, --layer <LAYER>      [default: 1]
  -r, --random
  -h, --help               Print help
```

layer 是隐写层数，random是是否自动填满所有的可用隐写空间。

== extract - 提取信息

尝试从用户提供的图片中提取信息。

```bash
Usage: image-steganography.exe extract [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
  -l, --layer <LAYER>  [default: 1]
  -h, --help           Print help
```

== capacity - 获取隐写容量

获取图片的隐写容量。

```bash
Usage: image-steganography.exe capacity [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
  -l, --layer <LAYER>  [default: 1]
  -h, --help           Print help
```

== debug - 显示图片字节数据

显示图片的结构体数据。

= 实验验证

== 正确插入和提取隐藏信息

===  5x5 24位BMP图片 <simple-24bit>

为方便展示程序正确性，以5x5 24位BMP图片为例，测试插入和提取隐藏信息的功能。

==== layer = 1

#figure(
    image("assets/55隐藏信息layer1.jpg",width: 100%,),
    caption: "24位BMP图片字节数据 循环写入一层",
)

==== layer = 2

#figure(
    image("assets/55隐藏信息layer2.jpg",width: 100%,),
    caption: "24位BMP图片字节数据 循环写入两层",
)

==== layer = 3

#figure(
    image("assets/55隐藏信息layer3.jpg",width: 100%,),
    caption: "24位BMP图片字节数据 循环写入三层",
)
此样例展示了如果layer < 3，那么程序会检测前32位记录的信息长度，来判断layer的设置是否符合预期。
当layer > 3时，程序仍然可以正确识别隐写的信息。

==== layer = 4

#figure(
    image("assets/55隐藏信息layer4.jpg",width: 100%,),
    caption: "24位BMP图片字节数据 循环写入四层",
)

==== 写入图片展示

#grid(
    columns: 5,
    gutter: 12pt,
    figure(
        image("assets/5x5-24位原图.jpg",width: 100%,),
        caption: "对角5个像素为白色，其余像素为黑色的原图",
    ),
    figure(
        image("assets/5x5-24位-layer1.jpg",width: 100%,),
        caption: "layer=1的隐写图片",
    ),
    figure(
        image("assets/5x5-24位-layer2.jpg",width: 100%,),
        caption: "layer=2的隐写图片",
    ),
    figure(
        image("assets/5x5-24位-layer3.jpg",width: 100%,),
        caption: "layer=3的隐写图片",
    ),
    figure(
        image("assets/5x5-24位-layer4.jpg",width: 100%,),
        caption: "layer=4的隐写图片",
    ),
) <simple-24bit-change>)

=== 指定图片文件

==== pic-01.bmp

#figure(
    image("assets/pic-01-操作演示.jpg",width: 100%,),
    caption: "操作演示",
) <pic-01>

#figure(
    grid(columns: 2, gutter: 12pt,
        image("assets/pic-01.jpg",width: 100%,),
        image("assets/pic-01-show-layer1.jpg",width: 100%,),
    ),
    caption: "图片效果展示",
)

==== pic-02.bmp

#figure(
    image("assets/pic-02-操作演示.jpg",width: 100%,),
    caption: "操作演示",
) <pic-01>

#figure(
    grid(columns: 2, gutter: 12pt,
        image("assets/pic-02.jpg",width: 100%,),
        image("assets/pic-02-show-layer1.jpg",width: 100%,),
    ),
    caption: "图片效果展示",
)

== 感官差异

从指定的图片来看，如果体积较大的图片，信息量比较小，无法轻易通过肉眼观测出差异。

但是如果体积较大，信息量也较大，需要情况分析。

=== 5x5 24位BMP图片

从较为#link(<simple-24bit-change>)[简单的图片]来看，中间白色像素的颜色分别为

\#ffffff -> \#fefefe -> \#fcfcfe -> \#faf8fb -> \#fceff6

当layer = 4时，有明显的颜色差异

=== pic-01.bmp

#grid(
    columns: 2,
    gutter: 12pt,
    figure(
        image("assets/pic-01.jpg",width: 100%,),
        caption: "pic-01 原图",
    ),
    figure(
        image("assets/pic-01-layer1.jpg",width: 100%,),
        caption: "pic-01 隐写图片 layer = 1",
    ),
    figure(
        image("assets/pic-01-layer2.jpg",width: 100%,),
        caption: "pic-01 隐写图片 layer = 2",
    ),
    figure(
        image("assets/pic-01-layer3.jpg",width: 100%,),
        caption: "pic-01 隐写图片 layer = 3",
    ),
    figure(
        image("assets/pic-01-layer4.jpg",width: 100%,),
        caption: "pic-01 隐写图片 layer = 4",
    ),
)

从layer=3开始，左部分天空有些许的颜色分层现象。从layer=4开始，可以很明显的看见竖状条纹。

#pagebreak()

=== pic-02.bmp

#grid(
    columns: 2,
    gutter: 12pt,
    figure(
        image("assets/pic-02.jpg",width: 100%,),
        caption: "pic-02 原图",
    ),
    figure(
        image("assets/pic-02-layer1.jpg",width: 100%,),
        caption: "pic-02 隐写图片 layer = 1",
    ),
    figure(
        image("assets/pic-02-layer2.jpg",width: 100%,),
        caption: "pic-02 隐写图片 layer = 2",
    ),
    figure(
        image("assets/pic-02-layer3.jpg",width: 100%,),
        caption: "pic-02 隐写图片 layer = 3",
    ),
    figure(
        image("assets/pic-02-layer4.jpg",width: 100%,),
        caption: "pic-02 隐写图片 layer = 4",
    ),
)

从layer=4开始，有很明显的色块区分。

== 最多存储信息

根据上述实验，可以假定layer=3是一个比较合理的隐写层数。即将每个字节的最低3位作为我们的信息存储位。

那么24位bmp图片最多存储的信息量为：

$
    "layer" = 3 \
    "信息最大存储数量（byte）" = ("图片像素数" dot "bytes per pixel" dot "layer") / 8 - 4
$

如 pic-01.bmp

$ "max_bytes" = 0.5273399353 "MB" = 552956 "Bytes" = (1536 * 960 * 3) / 8 - 4 $

#pagebreak()

= 进一步思考

如果要支持其它图片格式，比如png、jpg等，可能需要考虑到图片的压缩算法和存储方式。比如jpg格式的图片是有损压缩的，可能会导致隐写信息的丢失。

从程序设计角度来看，需要考虑代码的可读性和可维护性，需要合理的抽象与封装，让程序更健壮。比如使用枚举类型来表示不同的图片格式，使用结构体来表示图片的元信息和数据。这样可以提高代码的可读性和可维护性。

此外，现在的信息存储算法只是一个循环遍历存放bit的算法。可以考虑使用更复杂的算法，比如使用加密算法来加密信息，或者使用更复杂的隐写算法来隐藏信息。这样可以提高隐写信息的安全性和隐蔽性。
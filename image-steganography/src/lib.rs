use std::convert::{TryFrom, TryInto};
use std::io::{self};
use log::{info, debug};

#[derive(Debug)]
pub enum ImageError {
    InvalidFormat,
    ParseError(String),
    IoError(io::Error),
}

impl From<std::array::TryFromSliceError> for ImageError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        ImageError::ParseError("切片转换失败".to_string())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct FileHeader {
    file_type: [u8; 2],
    file_size: u32,
    reserved: [u8; 4],
    data_offset: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct BMPInformation {
    header_size: u32,
    width: u32,
    height: u32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    image_size: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

#[derive(Debug)]
pub struct BmpHeader {
    file_header: FileHeader,
    information: BMPInformation,
}

#[derive(Debug)]
pub struct ExtraInfo {
    padding: u32,
}

pub struct BmpImage {
    pub header: BmpHeader,
    pub raw_header: Vec<u8>,
    pub raw_body: Vec<u8>,
    pub extra_info: ExtraInfo,
}

impl TryFrom<Vec<u8>> for BmpImage {
    type Error = ImageError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let file_header = FileHeader {
            file_type: value[0..2].try_into()?,
            file_size: u32::from_le_bytes(value[2..6].try_into()?),
            reserved: value[6..10].try_into()?,
            data_offset: u32::from_le_bytes(value[10..14].try_into()?),
        };
        let bmp_information = BMPInformation {
            header_size: u32::from_le_bytes(value[14..18].try_into()?),
            width: u32::from_le_bytes(value[18..22].try_into()?),
            height: u32::from_le_bytes(value[22..26].try_into()?),
            planes: u16::from_le_bytes(value[26..28].try_into()?),
            bits_per_pixel: u16::from_le_bytes(value[28..30].try_into()?),
            compression: u32::from_le_bytes(value[30..34].try_into()?),
            image_size: u32::from_le_bytes(value[34..38].try_into()?),
            x_pixels_per_meter: i32::from_le_bytes(value[38..42].try_into()?),
            y_pixels_per_meter: i32::from_le_bytes(value[42..46].try_into()?),
            colors_used: u32::from_le_bytes(value[46..50].try_into()?),
            colors_important: u32::from_le_bytes(value[50..54].try_into()?),
        };
        let header = BmpHeader {
            file_header,
            information: bmp_information,
        };
        let raw_header = value[..header.file_header.data_offset as usize].to_vec();
        let raw_body = value[header.file_header.data_offset as usize..].to_vec();
        let extra_info = ExtraInfo {
            padding: (4
                - (header.information.width * (header.information.bits_per_pixel / 8) as u32) % 4)
                % 4,
        };
        Ok(BmpImage {
            header,
            raw_header,
            raw_body,
            extra_info: extra_info,
        })
    }
}

impl BmpImage {}

///返回可以存储的字节数
pub fn get_steganography_capacity(img: &BmpImage, layer: u8) -> usize {
    let bpp = img.header.information.bits_per_pixel;
    let bytes_per_pixel = bpp / 8;
    let pixels = img.header.information.width * img.header.information.height;
    let message_size = 4;
    (pixels as usize * bytes_per_pixel as usize / 8  - message_size) * layer as usize
}

/// 前4字节用于存储消息的长度
pub fn hide(mut img: BmpImage, message: &str, _layer: u8) -> Result<BmpImage, ImageError> {
    if message.len() > get_steganography_capacity(&img, _layer) {
        return Err(ImageError::ParseError(format!(
            "消息长度 {} 大于容量 {}",
            message.len(),
            get_steganography_capacity(&img, _layer)
        )));
    }
    let mut bytes_to_append = (message.len() as u32).to_le_bytes().to_vec();
    bytes_to_append.append(&mut message.as_bytes().to_vec());
    let mut raw_body_index = 0;
    info!("消息长度 {}", message.len());
    // 循环层数
    let max_layer: u8 = _layer;
    const MASKING: u8 = 0b11111111;
    let mut layer: u8 = 1;
    // 只存储在每byte的最低位
    for i in 0..bytes_to_append.len() {
        if i >= img.raw_body.len() * max_layer as usize {
            break;
        }
        let mask = MASKING ^ (1 << (layer - 1));
        debug!("---------当前循环------------");
        let current_byte = bytes_to_append[i];
        debug!("{} {:x} {:b}", current_byte, current_byte, current_byte);
        // 小端存储
        // 遍历需要存储的字节，每byte的每bit，存储至body中
        for j in 0..8 {
            let bit = ((current_byte >> j) & 1) << (layer - 1);
            img.raw_body[raw_body_index] = (img.raw_body[raw_body_index] & mask) | bit;
            raw_body_index += 1;
        }

        if i % img.header.information.width as usize == 0 && img.extra_info.padding != 0 {
            debug!(
                "padding: {} body数组下标: {}",
                img.extra_info.padding, raw_body_index
            );
            // 跳过padding
            raw_body_index += img.extra_info.padding as usize - 1;
        }

        if raw_body_index >= img.raw_body.len() {
            // 回溯至长度之后
            raw_body_index = 4 * 8;
            layer += 1;
        }
    }
    Ok(img)
}

pub fn extract(img: &BmpImage, _layer: u8) -> Result<String, ImageError> {
    let mut bytes = Vec::new();
    let mut raw_body_index = 0;
    // 读取前4个字节，获取消息的长度
    let mut length_bytes = Vec::<u8>::new();
    for _ in 0..4 {
        let mut byte = 0;
        for j in 0..8 {
            byte |= (img.raw_body[raw_body_index] & 1) << j;
            raw_body_index += 1;
        }
        length_bytes.push(byte);
    }
    let length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .map_err(|_| ImageError::ParseError("消息长度转换失败".to_string()))?,
    );
    if length > get_steganography_capacity(img, _layer) as u32 {
        return Err(ImageError::ParseError(
            "消息长度超过了图片的容量".to_string(),
        ));
    }

    let _max_layer: u8 = _layer;
    const MASKING: u8 = 0b00000000;
    let mut layer: u8 = 1;
    // 从body中读取消息
    for _ in 0..length {
        let mask = MASKING | (1 << (layer - 1)); 
        debug!("mask 0b{:b} {}", mask, raw_body_index);
        let mut byte = 0;
        for j in 0..8 {
            debug!("{:08b}", img.raw_body[raw_body_index] & mask);
            let bit_we_need = (img.raw_body[raw_body_index] & mask) >> (layer - 1);
            
            byte |= bit_we_need << j;
            raw_body_index += 1;
            debug!(" ");
        }
        debug!(" ");
        debug!("{} ", byte as char);
        if raw_body_index % img.header.information.width as usize == 0
            && img.extra_info.padding != 0
        {
            debug!(
                "padding: {} body数组下标: {}",
                img.extra_info.padding, raw_body_index
            );
            // 跳过padding
            raw_body_index += img.extra_info.padding as usize - 1;
        }
        bytes.push(byte);

        if raw_body_index >= img.raw_body.len() {
            debug!("回溯 {}", raw_body_index);
            // 回溯至长度之后
            raw_body_index = 4 * 8;
            layer += 1;
        }
    }
    let message = String::from_utf8(bytes).unwrap();
    Ok(message)
}

/// 打印bmp像素信息，根据bbp的不同，打印不同的信息，并打印padding
pub fn debug_image(img: &BmpImage) -> Result<(), ImageError> {
    let bpp = img.header.information.bits_per_pixel;
    let bytes_per_pixel = bpp / 8;
    let padding = img.extra_info.padding;
    for row in 0..img.header.information.height {
        for col in 0..img.header.information.width {
            let pixel_index =
                (row * img.header.information.width + col) * bytes_per_pixel as u32 + row * padding;
            let pixel = &img.raw_body
                [pixel_index as usize..(pixel_index + bytes_per_pixel as u32) as usize];
            print!(
                "{}",
                pixel
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            print!("  ");
        }
        print!(" ");
        for i in 0..padding {
            let byte = &img.raw_body[(img.header.information.width + i) as usize];
            print!("{:02x}", byte);
        }
        println!();
    }
    Ok(())
}

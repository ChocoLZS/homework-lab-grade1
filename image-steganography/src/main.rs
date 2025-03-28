
use image_steganography::{extract, get_steganography_capacity, hide, BmpImage};
use std::{fs, path::Path};
use std::process;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Hide {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long, default_value_t = String::from("Hello, world!"))]
        message: String,
    },
    Extract {
        #[arg(short, long)]
        input: String,
    },
    Capacity {
        #[arg(short, long)]
        input: String,
    },
    DebugImage {
        #[arg(short, long)]
        input: String,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Hide { input, output, message } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    match hide(bmp_img, message.as_str()) {
                    Ok(bmp_img) => {
                        fs::write(output, [bmp_img.raw_header, bmp_img.raw_body].concat()).unwrap();
                        println!("隐藏成功");
                    },
                    Err(e) => {
                        eprintln!("错误: {:?}", e);
                        process::exit(1);
                    }
                }
                },
                Err(e) => {
                    eprintln!("错误: {:?}", e);
                    process::exit(1);
                }
            }
            
        },
        Commands::Extract { input } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    let message =  extract(&bmp_img).map_err(|e| {
                        eprintln!("错误: {:?}", e);
                        process::exit(1);
                    }).unwrap();
                    println!("隐藏的信息是：{}", message);
                },
                Err(e) => {
                    eprintln!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        },
        Commands::Capacity { input } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    println!("{:?}", get_steganography_capacity(&bmp_img));
                },
                Err(e) => {
                    eprintln!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        },
        Commands::DebugImage { input } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    println!("{:?}", bmp_img.header);
                    // println!("{:?}", bmp_img.raw_header);
                    // println!("{:?}", bmp_img.raw_body);
                },
                Err(e) => {
                    eprintln!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        }
    }
}

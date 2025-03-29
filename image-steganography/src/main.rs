use image_steganography::{BmpImage, debug_image, extract, get_steganography_capacity, hide};
use log::{error, info};
use env_logger::Env;
use std::fs;
use std::process;
use rand::prelude::*;

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
        #[arg(short, long, default_value_t = 1)]
        layer: u8,
        #[arg(short, long, default_value_t = false)]
        random: bool,
    },
    Extract {
        #[arg(short, long)]
        input: String,
        #[arg(short, long, default_value_t = 1)]
        layer: u8
    },
    Capacity {
        #[arg(short, long)]
        input: String,
        #[arg(short, long, default_value_t = 1)]
        layer: u8
    },
    Debug {
        #[arg(short, long)]
        input: String,
    },
}

static STR_BASE: &'static [u8] = b"01234567890abcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=";
fn main() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
    let cli = Cli::parse();
    match cli.command {
        Commands::Hide {
            input,
            output,
            message,
            layer,
            random
        } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    let real_message = if random {
                        let max_capacity = get_steganography_capacity(&bmp_img, layer);
                        let mut rng = rand::rng();
                        let mut s = Vec::with_capacity(max_capacity);
                        for _ in 0..max_capacity {
                        let idx = rng.random_range(0..STR_BASE.len());
                            s.push(STR_BASE[idx]);
                        }
                        String::from_utf8(s).unwrap()
                    } else {
                        message
                    };
                    info!("随机字符串为：{}", real_message);
                    match hide(bmp_img, &real_message, layer) {
                        Ok(bmp_img) => {
                            fs::write(output, [bmp_img.raw_header, bmp_img.raw_body].concat()).unwrap();
                            info!("隐藏成功");
                        }
                        Err(e) => {
                            error!("错误: {:?}", e);
                            process::exit(1);
                        }
                    }
                },
                Err(e) => {
                    error!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Extract { input, layer } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    let message = extract(&bmp_img, layer)
                        .map_err(|e| {
                            error!("错误: {:?}", e);
                            process::exit(1);
                        })
                        .unwrap();
                    info!("隐藏的信息是：{}", message);
                }
                Err(e) => {
                    error!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Capacity { input, layer } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    info!(
                        "最多可隐藏的字节数为：{}",
                        get_steganography_capacity(&bmp_img, layer)
                    );
                }
                Err(e) => {
                    error!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Debug { input } => {
            let raw_bytes = fs::read(input.as_str()).unwrap();
            let bmp_img = BmpImage::try_from(raw_bytes);
            match bmp_img {
                Ok(bmp_img) => {
                    info!("{:?}", bmp_img.header);
                    // let _ = debug_image(&bmp_img);
                }
                Err(e) => {
                    error!("错误: {:?}", e);
                    process::exit(1);
                }
            }
        }
    }
}

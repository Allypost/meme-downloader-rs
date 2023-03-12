use clap::Parser;
use once_cell::sync::Lazy;
use std::path;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[arg(short, long, default_value = None)]
    pub yt_dlp_path: Option<path::PathBuf>,
    #[arg(short, long, default_value = None)]
    pub memes_directory: Option<path::PathBuf>,
    #[arg(default_value = None)]
    pub download_url: Option<String>,
}

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);

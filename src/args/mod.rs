use clap::Parser;
use once_cell::sync::Lazy;
use std::path;

#[derive(Debug, Clone, Parser)]
/// Media downloader and "fixer"
///
/// Downloads media from various websites.
/// Transcodes and crops files if necessary.
pub struct Args {
    /// Path to the yt-dlp executable
    #[arg(long, default_value = None)]
    pub yt_dlp_path: Option<path::PathBuf>,

    /// The directory to save memes to. If not provided, $HOME/MEMES will be used
    #[arg(short='d', long, default_value = None)]
    pub memes_directory: Option<path::PathBuf>,

    /// The URL to download media from
    #[arg(default_value = None)]
    pub download_url: Option<String>,
}

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);

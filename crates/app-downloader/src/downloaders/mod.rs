use std::{path::PathBuf, result::Result};

pub mod generic;
pub mod imgur;
pub mod instagram;
pub mod mastodon;
pub mod reddit;
pub mod tumblr;
pub mod twitter;
pub mod yt_dlp;

mod common;

type DownloaderReturn = Result<Vec<PathBuf>, String>;

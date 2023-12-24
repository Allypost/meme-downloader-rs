use std::{path::PathBuf, result::Result};

pub mod generic;
pub mod instagram;
pub mod mastodon;
pub mod tumblr;
pub mod twitter;
pub mod yt_dlp;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
                           Chrome/88.0.4324.182 Safari/537.36";

type DownloaderReturn = Result<Vec<PathBuf>, String>;

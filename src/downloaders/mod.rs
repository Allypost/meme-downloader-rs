use crate::config::CONFIG;
use log::{debug, error, info, trace};
use std::{env, fs, path::PathBuf, process::exit, result::Result};

mod instagram;
mod twitter;
mod util;
mod yt_dlp;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36";

type DownloaderReturn = Result<Vec<PathBuf>, String>;

pub fn download_file(url: &str) -> Result<Vec<PathBuf>, String> {
    info!("Downloading url: {}", &url);

    let meme_dir = CONFIG.clone().memes_dir().unwrap_or_else(|e| {
        error!("Error resolving memes directory: {:?}", e);
        exit(1);
    });

    trace!("Meme dir: {:?}", &meme_dir);
    if !meme_dir.exists() {
        info!("Memes directory does not exist. Creating...");
        fs::create_dir_all(&meme_dir).unwrap();
    }
    env::set_current_dir(&meme_dir).unwrap();

    let mut new_file_paths = match url {
        instagram_url if instagram::URL_MATCH.is_match(url) => {
            info!("Found URL is instagram url. Downloading all post media.");
            instagram::download(&meme_dir, instagram_url)?
        }
        twitter_url if twitter::URL_MATCH.is_match(url) => {
            info!("Found URL is twitter status. Trying to download post media...");
            twitter::download(&meme_dir, twitter_url)?
        }
        _ => {
            info!("Trying to download with yt-dlp...");
            yt_dlp::download(&meme_dir, url)?
        }
    };

    debug!("Downloaded files: {:?}", &new_file_paths);

    let new_file_paths = util::fix::fix_files(&mut new_file_paths)?;

    Ok(new_file_paths)
}

use crate::fixers;
use log::{debug, info};
use std::{env, path::PathBuf, result::Result};

mod instagram;
mod twitter;
mod yt_dlp;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36";

type DownloaderReturn = Result<Vec<PathBuf>, String>;

pub fn download_file(url: &str, download_dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    info!("Downloading {url:?} into {download_dir:?}");

    env::set_current_dir(download_dir).unwrap();

    let new_file_paths = match url {
        instagram_url if instagram::URL_MATCH.is_match(url) => {
            info!("Found URL is instagram url. Downloading all post media.");
            instagram::download(download_dir, instagram_url)?
        }
        twitter_url if twitter::URL_MATCH.is_match(url) => {
            info!("Found URL is twitter status. Trying to download post media...");
            twitter::download(download_dir, twitter_url)?
        }
        twitter_media_url if twitter::MEDIA_URL_MATCH.is_match(url) => {
            info!("Found URL is twitter media. Downloading...");
            twitter::download_media_url(download_dir, twitter_media_url)?
        }
        _ => {
            info!("Trying to download with yt-dlp...");
            yt_dlp::download(download_dir, url)?
        }
    };

    debug!("Downloaded files: {:?}", &new_file_paths);

    let new_file_paths = fixers::fix_files(&new_file_paths)?;

    Ok(new_file_paths)
}

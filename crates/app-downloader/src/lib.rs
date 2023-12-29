use std::{env, path::PathBuf};

use app_logger::{debug, info};
use downloaders::{instagram, mastodon, reddit, tumblr, twitter, yt_dlp};

use crate::downloaders::generic;

mod downloaders;

pub fn download_file(url: &str, download_dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    info!("Downloading {url:?} into {download_dir:?}");

    env::set_current_dir(download_dir).map_err(|e| format!("{e:?}"))?;

    let new_file_paths = match url {
        instagram_url if instagram::URL_MATCH.is_match(url) => {
            debug!("Found URL is instagram url. Downloading all post media.");
            instagram::download(download_dir, instagram_url)?
        }
        twitter_url if twitter::URL_MATCH.is_match(url) => {
            debug!("Found URL is twitter status. Trying to download post media...");
            twitter::download(download_dir, twitter_url)?
        }
        twitter_media_url if twitter::MEDIA_URL_MATCH.is_match(url) => {
            debug!("Found URL is twitter media. Downloading...");
            twitter::download_media_url(download_dir, twitter_media_url)?
        }
        mastodon_url if mastodon::is_mastodon_toot(url) => {
            debug!("Found URL is mastodon toot. Downloading...");
            mastodon::screenshot_toot(download_dir, mastodon_url)?
        }
        tumblr_url if tumblr::URL_MATCH.is_match(url) => {
            debug!("Found URL is tumblr post. Downloading...");
            tumblr::download(download_dir, tumblr_url)?
        }
        reddit_image_url if reddit::is_reddit_image_url(url) => {
            debug!("Found URL is reddit image. Downloading...");
            generic::download(download_dir, reddit_image_url)?
        }
        _ => {
            debug!("Trying to download with yt-dlp...");
            yt_dlp::download(download_dir, url)?
        }
    };

    debug!("Downloaded files: {:?}", &new_file_paths);

    let new_file_paths = app_fixers::fix_files(&new_file_paths)?;

    Ok(new_file_paths)
}

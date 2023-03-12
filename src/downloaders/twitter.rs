use super::DownloaderReturn;
use crate::downloaders::yt_dlp;
use log::{debug, info, trace};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

pub static URL_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://(www\.)?twitter\.com/(?P<username>[^/]+)/status/(?P<status_id>[0-9]+)")
        .unwrap()
});

pub fn download(meme_dir: &PathBuf, url: &str) -> DownloaderReturn {
    debug!("Trying to download tweet media from: {:?}", &url);

    yt_dlp::download(meme_dir, url).or_else(|_e| {
        info!("Failed to download with yt-dlp. Trying to screenshot...");

        screenshot_tweet(meme_dir, url)
    })
}

fn screenshot_tweet(meme_dir: &PathBuf, url: &str) -> DownloaderReturn {
    debug!("Trying to screenshot tweet: {:?}", &url);

    let tweet_screenshot_url = format!("https://twitter.igr.ec/{url}");

    trace!("Tweet screenshot URL: {:?}", &tweet_screenshot_url);

    yt_dlp::download(meme_dir, &tweet_screenshot_url)
}

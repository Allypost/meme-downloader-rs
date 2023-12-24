use super::DownloaderReturn;
use crate::downloaders::yt_dlp;
use app_config::CONFIG;
use log::{debug, trace};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

pub static URL_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://(www\.)?twitter\.com/(?P<username>[^/]+)/status/(?P<status_id>[0-9]+)")
        .unwrap()
});

pub static MEDIA_URL_MATCH: Lazy<Regex> = Lazy::new(|| {
    // https://pbs.twimg.com/media/FqPFEWYWYBQ5iG3?format=png&name=small
    Regex::new(r"^https?://pbs\.twimg\.com/media/").unwrap()
});

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    debug!("Trying to download tweet media from: {:?}", &url);

    yt_dlp::download(download_dir, url).or_else(|_e| {
        debug!("Failed to download with yt-dlp. Trying to screenshot...");

        screenshot_tweet(download_dir, url)
    })
}

pub fn download_media_url(download_dir: &PathBuf, twitter_media_url: &str) -> DownloaderReturn {
    let mut parsed = url::Url::parse(twitter_media_url)
        .map_err(|x| format!("Failed to parse twitter media URL: {x:?}"))?;

    let url_without_name = {
        let params = parsed.query_pairs().filter(|(key, _)| key != "name");
        let params = url::form_urlencoded::Serializer::new(String::new())
            .clear()
            .extend_pairs(params)
            .finish();

        parsed.set_query(Some(&params));

        parsed.as_str()
    };

    yt_dlp::download(download_dir, url_without_name)
}

fn screenshot_tweet(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    debug!("Trying to screenshot tweet: {:?}", &url);

    let endpoint = CONFIG.endpoints.twitter_screenshot_base_url();
    let tweet_screenshot_url = format!("{}/{}", endpoint.trim_end_matches('/'), url);

    trace!("Tweet screenshot URL: {:?}", &tweet_screenshot_url);

    yt_dlp::download(download_dir, &tweet_screenshot_url)
}

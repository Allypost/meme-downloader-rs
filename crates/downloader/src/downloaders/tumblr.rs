use super::DownloaderReturn;
use crate::downloaders::twitter;

use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

pub static URL_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://(www\.)?tumblr\.com/(?P<username>[^/]+)/(?P<post_id>[0-9]+)(/|/[^/]+)?")
        .unwrap()
});

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    twitter::download(download_dir, url)
}

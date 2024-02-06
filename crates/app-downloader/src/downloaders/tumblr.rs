use std::path::PathBuf;

use once_cell::sync::Lazy;
use regex::Regex;

use super::DownloaderReturn;
use crate::downloaders::twitter;

pub static URL_MATCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://(www\.)?tumblr\.com/(?P<username>[^/]+)/(?P<post_id>[0-9]+)(/|/[^/]+)?")
        .expect("Invalid regex")
});

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    twitter::download(download_dir, url)
}

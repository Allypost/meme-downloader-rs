use std::{path::PathBuf, time::Duration};

use app_logger::{debug, trace, warn};
use once_cell::sync::Lazy;
use regex::Regex;

use super::{twitter, DownloaderReturn};
use crate::downloaders::common::request::Client;

pub static IS_NUMBERS_ONLY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d+$").expect("Invalid regex"));

pub fn is_mastodon_toot(toot_url: &str) -> bool {
    trace!("Checking whether {toot_url:?} is a Mastodon toot");
    let toot_url = toot_url.trim_end_matches('/');
    let Some(toot_id) = toot_url.split('/').last() else {
        return false;
    };

    if !IS_NUMBERS_ONLY.is_match(toot_id) {
        return false;
    }

    let Ok(toot_url) = url::Url::parse(toot_url) else {
        return false;
    };

    let Some(mastodon_host) = toot_url.host() else {
        return false;
    };

    let api_url = format!("https://{mastodon_host}/api/v1/statuses/{toot_id}");

    trace!("Making request to instance {mastodon_host:?} for status info for {toot_id:?}");

    let client = match Client::default() {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create client: {e:?}");
            return false;
        }
    };

    let result = client
        .get(api_url)
        .timeout(Duration::from_secs(5))
        .send()
        .map_err(|e| format!("Failed to send request to instagram API: {e:?}"))
        .and_then(|x| {
            x.text()
                .map_err(|e| format!("Failed to parse response from instagram API: {e:?}"))
        })
        .and_then(|res_text| {
            serde_json::from_str::<serde_json::Value>(&res_text)
                .map_err(|e| format!("Failed to parse response from instagram API: {e:?}"))
        });

    match result {
        Ok(result) => {
            trace!("Got OK result from api request: {result:?}");
            true
        }
        Err(e) => {
            debug!("Got error from API check: {e:?}");
            false
        }
    }
}

/// Presumes that the URL is of a Mastodon toot
pub fn screenshot_toot(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    twitter::download(download_dir, url)
}

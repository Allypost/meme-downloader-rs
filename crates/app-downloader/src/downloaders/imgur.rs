use std::{path::PathBuf, string::ToString, time::Duration};

use rayon::prelude::*;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;

use super::DownloaderReturn;
use crate::downloaders::{generic, USER_AGENT};

pub fn is_imgur_direct_media_url(url: &str) -> bool {
    url.starts_with("https://i.imgur.com/")
}

pub fn is_imgur_url(url: &str) -> bool {
    url.starts_with("https://imgur.com/") || url.starts_with("http://imgur.com/")
}

#[derive(Debug, Deserialize)]
struct ImgurPostData {
    pub media: Vec<ImgurPostMedia>,
}

#[derive(Debug, Deserialize)]
struct ImgurPostMedia {
    url: String,
}

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    app_logger::info!(
        "Downloading imgur post {:?} media to {:?}",
        url,
        download_dir
    );

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create client: {:?}", e))?;

    let resp = client
        .get(url)
        .send()
        .and_then(Response::text)
        .map_err(|e| format!("Failed to send request to imgur: {:?}", e))?;

    app_logger::trace!("Got response from imgur");

    let dom = tl::parse(&resp, tl::ParserOptions::default())
        .map_err(|e| format!("Failed to parse html from imgur: {:?}", e))?;
    let parser = dom.parser();

    app_logger::trace!("Parsed html from imgur");

    let script_data = dom
        .query_selector("script")
        .expect("Failed parse query selector")
        .filter_map(|x| x.get(parser))
        .filter_map(|x| x.as_tag())
        .find_map(|x| {
            x.inner_text(parser)
                .strip_prefix("window.postDataJSON=")
                .map(ToString::to_string)
        })
        .and_then(|x| serde_json::from_str::<String>(&x).ok())
        .and_then(|x| serde_json::from_str::<ImgurPostData>(&x).ok())
        .ok_or_else(|| "Failed to get script data".to_string())?;

    app_logger::trace!("Got script data from imgur: {:?}", &script_data);

    let (downloaded, failed) = {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(rayon::current_num_threads().min(4))
            .build()
            .map_err(|e| format!("Failed to create thread pool: {:?}", e))?;

        let downloaded_items = thread_pool.install(|| {
            script_data
                .media
                .par_iter()
                .map(|x| (&x.url, generic::download(download_dir, &x.url)))
                .collect::<Vec<_>>()
        });

        app_logger::trace!("Downloaded items from imgur: {:?}", &downloaded_items);

        let (success, failed): (Vec<_>, Vec<_>) =
            downloaded_items.into_iter().partition(|x| x.1.is_ok());

        let success = success
            .into_iter()
            .filter_map(|x| x.1.ok())
            .flatten()
            .collect::<Vec<_>>();

        let failed = failed
            .into_iter()
            .filter_map(|x| x.1.err().map(|e| (x.0, e)))
            .collect::<Vec<_>>();

        (success, failed)
    };

    if !failed.is_empty() {
        app_logger::warn!("Failed to download: {:?}", failed);
    }

    Ok(downloaded)
}

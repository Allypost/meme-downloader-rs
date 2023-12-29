use std::{ffi::OsString, fs::File, path::PathBuf, string::ToString};

use app_helpers::id::time_id;
use reqwest::blocking::Client;
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use super::DownloaderReturn;
use crate::downloaders::USER_AGENT;

pub const MAX_FILENAME_LENGTH: usize = 120;

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    app_logger::info!("Downloading {:?} to {:?}", url, download_dir);

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to create client: {:?}", e))?;

    let mut res = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to send request: {:?}", e))?
        .error_for_status()
        .map_err(|e| format!("Failed to get response: {:?}", e))?;

    let mime_type = res.headers().get("content-type").map(|x| x.to_str());
    app_logger::debug!("Got mime type: {:?}", mime_type);
    let mime_type = match mime_type {
        Some(Ok(mime_type)) => mime_type,
        _ => "",
    };
    let extension = mime_guess::get_mime_extensions_str(mime_type)
        .and_then(|x| x.first())
        .map_or("unknown".to_string(), |x| (*x).to_string());

    let id = time_id().map_err(|e| format!("Failed to get time id: {:?}", e))?;
    let mut file_name = OsString::from(&id);

    let taken_filename_len = id.len() + 1 + extension.len();

    let url_file_name = Url::parse(url)
        .ok()
        .map(|x| PathBuf::from(x.path()))
        .and_then(|x| {
            let stem = x.file_stem()?;

            let trunc = stem
                .to_string_lossy()
                .graphemes(true)
                .filter(|x| !x.chars().all(char::is_control))
                .filter(|x| !x.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']))
                .take(MAX_FILENAME_LENGTH - 1 - taken_filename_len)
                .collect::<String>();

            if trunc.is_empty() {
                None
            } else {
                Some(trunc)
            }
        });

    if let Some(url_file_name) = url_file_name {
        app_logger::trace!("Got url file name: {:?}", url_file_name);
        file_name.push(".");
        file_name.push(url_file_name);
    }

    file_name.push(".");
    file_name.push(extension);

    let file_path = download_dir.join(file_name);
    app_logger::debug!("Writing to file: {:?}", &file_path);
    let mut out_file =
        File::create(&file_path).map_err(|e| format!("Failed to create file: {:?}", e))?;

    res.copy_to(&mut out_file)
        .map_err(|e| format!("Failed to copy response to file: {:?}", e))?;

    Ok(vec![file_path])
}

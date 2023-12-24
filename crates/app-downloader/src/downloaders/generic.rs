use std::{fs::File, path::PathBuf};

use app_helpers::id::time_id;
use reqwest::blocking::Client;

use super::DownloaderReturn;
use crate::downloaders::USER_AGENT;

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

    let file_name = format!(
        "{}.{}",
        time_id().map_err(|e| { format!("Failed to get time id: {:?}", e) })?,
        extension
    );

    let file_path = download_dir.join(file_name);
    app_logger::debug!("Writing to file: {:?}", &file_path);
    let mut out_file =
        File::create(&file_path).map_err(|e| format!("Failed to create file: {:?}", e))?;

    res.copy_to(&mut out_file)
        .map_err(|e| format!("Failed to copy response to file: {:?}", e))?;

    Ok(vec![file_path])
}

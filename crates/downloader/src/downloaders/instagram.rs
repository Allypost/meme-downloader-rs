use super::DownloaderReturn;
use crate::downloaders::yt_dlp;
use log::{debug, info, trace};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use std::{path::PathBuf, result::Result, string};

pub static URL_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?://(www\.)?instagram.com/p/(?P<post_id>[^/?]+)").unwrap());

pub fn download(meme_dir: &PathBuf, url: &str) -> DownloaderReturn {
    let instagram_urls = fetch_instagram_urls(url)?;
    debug!("Instagram URLs: {:?}", &instagram_urls);

    let res: Vec<Result<Vec<PathBuf>, String>> = instagram_urls
        .par_iter()
        .map(|url| yt_dlp::download(meme_dir, url))
        .collect();

    if res.iter().any(Result::is_err) {
        return Err(res
            .iter()
            .filter(|x| x.is_err())
            .map(|x| return x.as_ref().unwrap_err().clone())
            .collect::<Vec<String>>()
            .join(", "));
    }

    let res = res
        .iter()
        .flat_map(|x| return x.as_ref().unwrap().clone())
        .collect::<Vec<PathBuf>>();

    Ok(res)
}

fn fetch_instagram_urls(url: &str) -> Result<Vec<String>, String> {
    fn get_api_response(post_id: &str) -> Result<serde_json::Value, String> {
        let query_hash = "2efa04f61586458cef44441f474eee7c";
        let query_args = serde_json::json!({
            "shortcode": post_id,
            "child_comment_count": 0,
            "fetch_comment_count": 0,
            "parent_comment_count": 0,
            "has_threaded_comments": true,
        });
        trace!("Query args: {:?}", &query_args);
        let query_args =
            serde_json::to_string(&query_args).map_err(|_e| "Failed to stringify json")?;

        let api_url = format!(
            "https://www.instagram.com/graphql/query/?query_hash={}&variables={}",
            &query_hash, &query_args,
        );

        debug!("Fetching from instagram API url: {:?}", &api_url);

        let client = reqwest::blocking::Client::new();
        client
            .get(&api_url)
            .send()
            .map_err(|e| format!("Failed to send request to instagram API: {e:?}"))?
            .text()
            .map_err(|e| format!("Failed to parse response from instagram API: {e:?}"))
            .and_then(|res_text| {
                serde_json::from_str::<serde_json::Value>(&res_text)
                    .map_err(|e| format!("Failed to parse response from instagram API: {e:?}"))
            })
    }

    trace!("Fetching instagram media URLs for: {}", &url);

    let post_id = URL_MATCH
        .captures(url)
        .and_then(|x| x.name("post_id"))
        .map(|x| x.as_str())
        .ok_or_else(|| "URL is not a valid Instagram post".to_string())?;
    debug!("Instagram post ID: {:?}", &post_id);

    let json_response = get_api_response(post_id)?;
    let edges = json_response
        .get("data")
        .and_then(|x| x.get("shortcode_media"))
        .and_then(serde_json::Value::as_object)
        .ok_or("Failed to get edges from response")?;

    if !edges.contains_key("edge_sidecar_to_children") {
        let url = edges
            .get("display_url")
            .and_then(serde_json::Value::as_str)
            .map(string::ToString::to_string)
            .ok_or("Failed to get `display_url' on edges")?;

        debug!("Fetched Instagram media and got single image");

        return Ok(vec![url]);
    }

    debug!("Fetched Instagram media and got multiple images");

    let edges = &edges["edge_sidecar_to_children"]["edges"]
        .as_array()
        .unwrap();

    let urls = edges
        .iter()
        .map(|entry| {
            let node = entry.get("node").and_then(serde_json::Value::as_object)?;

            if node.contains_key("video_url") {
                debug!("Found video in post: {}", node["id"]);
                return node.get("video_url").and_then(serde_json::Value::as_str);
            }

            debug!("Found image in post: {}", node["id"]);
            return node.get("display_url").and_then(serde_json::Value::as_str);
        })
        .filter(std::option::Option::is_some)
        .map(|x| x.unwrap().to_string())
        .collect::<Vec<String>>();

    info!("Found multiple Instagram media");
    Ok(urls)
}

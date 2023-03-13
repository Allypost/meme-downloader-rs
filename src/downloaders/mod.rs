use crate::config::CONFIG;
use log::{debug, error, info, trace};
use std::{env, fs, path::PathBuf, process::exit, result::Result, time};

mod instagram;
mod twitter;
mod yt_dlp;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.182 Safari/537.36";

type DownloaderReturn = Result<Vec<PathBuf>, String>;

pub fn download_file(url: &str) -> Result<Vec<PathBuf>, String> {
    info!("Downloading url: {}", &url);

    let meme_dir = CONFIG.clone().memes_dir().unwrap_or_else(|e| {
        error!("Error resolving memes directory: {:?}", e);
        exit(1);
    });

    trace!("Meme dir: {:?}", &meme_dir);
    env::set_current_dir(&meme_dir).unwrap();

    let mut new_file_paths = match url {
        instagram_url if instagram::URL_MATCH.is_match(url) => {
            info!("Found URL is instagram url. Downloading all post media.");
            instagram::download(&meme_dir, instagram_url)?
        }
        twitter_url if twitter::URL_MATCH.is_match(url) => {
            info!("Found URL is twitter status. Trying to download post media...");
            twitter::download(&meme_dir, twitter_url)?
        }
        _ => {
            info!("Trying to download with yt-dlp...");
            yt_dlp::download(&meme_dir, url)?
        }
    };

    debug!("Downloaded files: {:?}", &new_file_paths);

    let new_file_paths = fix_file_extensions(&mut new_file_paths);

    if new_file_paths.iter().any(Result::is_err) {
        return Err(new_file_paths
            .iter()
            .filter(|x| x.is_err())
            .map(|x| return x.as_ref().unwrap_err().clone())
            .collect::<Vec<String>>()
            .join(", "));
    }

    return Ok(new_file_paths
        .iter()
        .map(|x| return x.as_ref().unwrap().clone())
        .collect());
}

fn fix_file_extensions(new_file_paths: &mut [PathBuf]) -> Vec<Result<PathBuf, String>> {
    new_file_paths
        .iter_mut()
        .map(|file_path| {
            match file_path.extension().and_then(|x| return x.to_str()) {
                Some(ext) if ext == "unknown_video" => {
                    trace!("File extension is `unknown_video'. Trying to infer file extension...");
                }
                None => {
                    return Err(format!("Failed to get extension for file {:?}", &file_path));
                }
                Some(_) => {
                    trace!(
                        "File extension for {:?} is OK. Skipping...",
                        &file_path.file_name().unwrap()
                    );
                    return Ok(file_path.clone());
                }
            }

            debug!("Trying to infer file extension for {:?}", &file_path);

            let file_ext = match infer::get_from_path(&file_path) {
                Ok(Some(ext)) => ext.extension(),
                _ => {
                    return Err(format!("Failed to get extension for file {:?}", &file_path));
                }
            };
            debug!("Inferred file extension: {:?}", file_ext);

            let new_file_path = file_path.with_extension(file_ext);
            match fs::rename(&file_path, &new_file_path) {
                Ok(_) => Ok(new_file_path),
                Err(e) => Err(format!("Failed to rename file: {e:?}")),
            }
        })
        .collect()
}

fn get_output_template<S: Into<PathBuf>>(meme_dir: S) -> PathBuf {
    let now_ns = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_identifier = now_ns.to_string();
    let file_name = format!("{file_identifier}.%(id)s.%(ext)s");

    meme_dir.into().join(file_name)
}

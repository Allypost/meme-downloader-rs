use super::FixerReturn;
use log::{debug, info};
use std::{fs, path::PathBuf};

pub fn fix_file_extension(file_path: &PathBuf) -> FixerReturn {
    info!("Checking file extension for {file_path:?}...");

    let extension = file_path.extension().and_then(|x| return x.to_str());
    match extension {
        Some(ext) if ext == "unknown_video" => {
            debug!("File extension is `unknown_video'. Trying to infer file extension...");
        }
        None => {
            return Err(format!("Failed to get extension for file {:?}", &file_path));
        }
        Some(_) => {
            info!(
                "File extension for {:?} is OK. Skipping...",
                &file_path.file_name().unwrap()
            );
            return Ok(file_path.clone());
        }
    }

    debug!("Trying to infer file extension for {:?}", &file_path);

    let file_ext = match infer::get_from_path(file_path) {
        Ok(Some(ext)) => ext.extension(),
        _ => {
            return Err(format!("Failed to get extension for file {:?}", &file_path));
        }
    };
    debug!("Inferred file extension: {:?}", file_ext);

    let new_file_path = file_path.with_extension(file_ext);

    info!("Renaming file from {file_path:?} to {new_file_path:?}");
    match fs::rename(file_path, &new_file_path) {
        Ok(_) => Ok(new_file_path),
        Err(e) => Err(format!("Failed to rename file: {e:?}")),
    }
}

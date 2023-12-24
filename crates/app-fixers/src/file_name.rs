use std::{fs, path::PathBuf};

use log::{debug, trace};

use super::FixerReturn;

pub fn fix_file_name(file_path: &PathBuf) -> FixerReturn {
    debug!("Checking file name for {file_path:?}...");
    let name = file_path.file_stem().and_then(|x| return x.to_str());

    let new_name = match name {
        Some(name) if !name.is_ascii() => {
            debug!("File name {name:?} contains non-ascii characters. Trying to fix...");
            name.replace(|c: char| !c.is_ascii(), "")
        }
        None => {
            return Err(format!("Failed to get name for file {:?}", &file_path));
        }
        Some(name) => {
            debug!("File name for {name:?} is OK. Skipping...");
            return Ok(file_path.clone());
        }
    };

    let extension = file_path
        .extension()
        .and_then(|x| return x.to_str())
        .ok_or_else(|| {
            format!(
                "Failed to get extension for file {:?}",
                &file_path.file_name()
            )
        })?;

    trace!("New file name: {new_name:?} (extension: {extension:?}) for file {file_path:?}");

    let new_name = format!("{new_name}.{extension}");
    let new_file_path = file_path.with_file_name(new_name);

    debug!("Renaming file from {file_path:?} to {new_file_path:?}");

    match fs::rename(file_path, &new_file_path) {
        Ok(()) => Ok(new_file_path),
        Err(e) => Err(format!("Failed to rename file: {e:?}")),
    }
}

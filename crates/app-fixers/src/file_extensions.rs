use std::{fs, path::PathBuf};

use log::{debug, trace};

use super::FixerReturn;

pub fn fix_file_extension(file_path: &PathBuf) -> FixerReturn {
    debug!("Checking file extension for {file_path:?}...");

    let extension = file_path.extension().and_then(std::ffi::OsStr::to_str);

    let file_ext = match infer::get_from_path(file_path) {
        Ok(Some(ext)) => ext.extension(),
        _ => {
            return Err(format!("Failed to get extension for file {:?}", &file_path));
        }
    };
    debug!("Inferred file extension: {:?}", file_ext);

    if let Some(extension) = extension {
        if extension == file_ext {
            debug!("File extension is correct");
            return Ok(file_path.clone());
        }
    }

    trace!(
        "File extension is incorrect ({:?} vs ({:?}))",
        extension,
        file_ext
    );

    let new_file_path = file_path.with_extension(file_ext);

    debug!("Renaming file from {file_path:?} to {new_file_path:?}");
    match fs::rename(file_path, &new_file_path) {
        Ok(()) => Ok(new_file_path),
        Err(e) => Err(format!("Failed to rename file: {e:?}")),
    }
}

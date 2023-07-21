use std::{fs, path::PathBuf, process};

use config::CONFIG;
use filetime::FileTime;
use helpers::{ffprobe, id::time_thread_id, trash::move_to_trash};
use log::{debug, info, trace};
use phf::phf_map;

use crate::FixerReturn;

pub fn convert_file_into_preferred(file_path: &PathBuf) -> FixerReturn {
    debug!("Checking if {file_path:?} has unwanted formats");

    Ok(file_path).and_then(check_and_fix_file).map(|p| {
        debug!("File {file_path:?} done being converted");
        p
    })
}

const CODEC_TO_EXT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "jpeg" => "jpg",
    "mjpeg" => "jpg",
};

const FORMAT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "png" => "png",
    "jpg" => "jpg",

    "webp" => "png",
};

#[allow(clippy::too_many_lines)]
fn check_and_fix_file(file_path: &PathBuf) -> Result<PathBuf, String> {
    if !file_path.exists() {
        return Err(format!("File {file_path:?} does not exist"));
    }

    let file_format_info = ffprobe::ffprobe(file_path)
        .map_err(|e| format!("Failed to get ffprobe information of {file_path:?}: {e:?}"))?;

    trace!(
        "File format info: {file_format_info:?}",
        file_format_info = file_format_info
    );

    let Some(file_first_stream) = file_format_info.streams.first() else {
        return Err(format!(
            "Failed to get first stream of {file_path:?}"
        ));
    };

    let file_codec_extension = {
        let codec_name = file_first_stream
            .codec_name
            .clone()
            .ok_or_else(|| format!("Failed to get codec name of {file_path:?}"))?;

        CODEC_TO_EXT_MAP
            .get(&codec_name)
            .copied()
            .map_or(codec_name, std::string::ToString::to_string)
    };

    trace!(
        "File codec extension: {file_codec_extension:?}",
        file_codec_extension = file_codec_extension
    );

    let Some(to_extension) = FORMAT_MAP.get(&file_codec_extension) else {
        return Err(format!(
            "Don't know how to convert {file_codec_extension:?}"
        ));
    };

    trace!(
        "File to extension: {to_extension:?}",
        to_extension = to_extension
    );

    if to_extension == &file_codec_extension {
        info!(
            "File {file_path:?} is already in preferred format",
            file_path = file_path
        );
        return Ok(file_path.into());
    }

    let normalized_extension_file_path = if let Some(ext) = file_path.extension() {
        let ext = ext.to_str().unwrap_or_default();
        let ext = CODEC_TO_EXT_MAP.get(ext).copied().unwrap_or(ext);

        file_path.with_extension(ext)
    } else {
        file_path.into()
    };

    trace!(
        "Normalized extension file path: {normalized_extension_file_path:?}",
        normalized_extension_file_path = normalized_extension_file_path
    );
    if let Some(file_ext) = normalized_extension_file_path.extension() {
        trace!(
            "File extension '{file_ext:?}` vs '{file_codec_extension:?}`",
            file_ext = file_ext.to_str().unwrap_or_default()
        );

        if file_codec_extension == file_ext.to_str().unwrap_or_default() {
            info!(
                "File {file_path:?} is already in preferred format",
                file_path = file_path
            );
            return Ok(normalized_extension_file_path);
        }
    } else {
        return Err(format!(
            "Failed to get extension of {normalized_extension_file_path:?}"
        ));
    }

    let cache_folder = CONFIG.cache_dir();

    let cache_file_path = {
        let file_name = time_thread_id()
            .map(|x| PathBuf::from(x).with_extension(file_codec_extension))
            .or_else(|_e| {
                normalized_extension_file_path
                    .file_name()
                    .map(PathBuf::from)
                    .ok_or_else(|| {
                        format!("Failed to get file name of {normalized_extension_file_path:?}")
                    })
            })?;

        cache_folder.join(file_name)
    };

    trace!("Checking if {cache_file_path:?} folder exists");
    if let Some(parent_path) = cache_file_path.parent() {
        if !parent_path.exists() {
            trace!("Creating {parent_path:?}", parent_path = parent_path);
            fs::create_dir_all(parent_path)
                .map_err(|e| format!("Failed to create {parent_path:?}: {e:?}"))?;
        }
    } else {
        return Err(format!("Failed to get parent of {cache_file_path:?}"));
    }

    trace!("Copying {file_path:?} to {cache_file_path:?}");
    fs::copy(file_path, &cache_file_path)
        .map_err(|e| format!("Failed to copy {file_path:?} to {cache_file_path:?}: {e:?}"))?;

    let new_cache_file_path = cache_file_path.with_extension(to_extension);

    trace!(
        "Converting {cache_path:?} to {new_cache_path:?}",
        cache_path = cache_file_path,
        new_cache_path = new_cache_file_path
    );

    let ffmpeg_path = &CONFIG
        .dependencies
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "Failed to get `ffmpeg' path from configuration".to_string())?;
    debug!("`ffmpeg' binary: {ffmpeg_path:?}");
    let mut cmd = process::Command::new(ffmpeg_path);
    let cmd = cmd
        .arg("-y")
        .arg("-hide_banner")
        .args(["-loglevel", "panic"])
        .args(["-i", (cache_file_path.to_str().unwrap())])
        .args(["-max_muxing_queue_size", "1024"])
        .args(["-vf", "scale=ceil(iw/2)*2:ceil(ih/2)*2"])
        .args(["-ab", "320k"])
        .args(["-map_metadata", "-1"])
        .args(["-preset", "slow"])
        .arg(&new_cache_file_path);
    info!("Running `ffmpeg' command: {cmd:?}");

    trace!("Deleting {cache_file_path:?}");

    let cmd_output = cmd.output();
    let _ = fs::remove_file(&cache_file_path);
    match cmd_output {
        Ok(process::Output { status, .. }) if status.success() && new_cache_file_path.exists() => {
            info!("Converted file {file_path:?} to {to_extension}");

            let new_file_path = file_path.with_extension(to_extension);

            trace!(
                "Copying {cache_path:?} to {new_path:?}",
                cache_path = new_cache_file_path,
                new_path = new_file_path
            );
            if let Err(e) = fs::copy(&new_cache_file_path, &new_file_path) {
                return Err(format!(
                    "Failed to copy {new_cache_file_path:?} to {new_file_path:?}: {e:?}",
                ));
            }

            trace!("Deleting {new_cache_file_path:?}");
            let _ = fs::remove_file(&new_cache_file_path);

            match copy_file_times(file_path, &new_file_path) {
                Err(e) => {
                    debug!("Failed to copy file times: {e:?}");
                }
                Ok(_) => {
                    trace!("Copied file times from {file_path:?}");
                }
            }

            if move_to_trash(file_path).is_ok() {
                debug!("Deleted old file {file_path:?}");
            }

            Ok(new_file_path)
        }
        _ => {
            trace!("`ffmpeg' command output: {cmd_output:?}");
            debug!(
                "Deleting new file {file_path:?}",
                file_path = new_cache_file_path
            );
            if let Err(e) = fs::remove_file(&new_cache_file_path) {
                debug!(
                    "Failed to delete new file {file_path:?}: {e:?}",
                    file_path = new_cache_file_path
                );
            }
            Err(format!(
                "Failed transforming {file_path:?} into {to_extension}"
            ))
        }
    }
}

#[allow(clippy::similar_names)]
fn copy_file_times<'s>(old: &PathBuf, new: &'s PathBuf) -> Result<&'s PathBuf, String> {
    trace!(
        "Copying file times from {old:?} to {new:?}",
        old = old,
        new = new
    );
    let old_meta = old
        .metadata()
        .map_err(|e| format!("Failed to get metadata of {old:?}: {e:?}"))?;

    let old_mtime = FileTime::from_last_modification_time(&old_meta);
    let old_atime = FileTime::from_last_access_time(&old_meta);

    filetime::set_file_times(new, old_atime, old_mtime)
        .map_err(|e| format!("Failed to set file times of {new:?}: {e:?}"))?;

    Ok(new)
}

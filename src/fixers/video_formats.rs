use super::FixerReturn;
use crate::{
    config::CONFIGURATION,
    helpers::{ffprobe, results::option_contains, trash::move_to_trash},
};
use filetime::FileTime;
use log::{debug, info, trace};
use std::{env, fs, path::PathBuf, process, time};

pub fn convert_file_into_known(file_path: &PathBuf) -> FixerReturn {
    debug!("Checking if {file_path:?} has unwanted codecs or formatting");

    Ok(file_path)
        .and_then(|p| convert_a_to_b(p, "webm", "mp4"))
        .and_then(|p| convert_a_to_b(&p, "mkv", "mp4"))
        .and_then(|p| convert_a_to_b(&p, "mov", "mp4"))
        .and_then(|p| convert_a_to_b(&p, "webp", "png"))
        .and_then(|p| reencode_dodgy_encodings(&p))
        .map(|p| {
            debug!("File {file_path:?} done being converted");
            p
        })
}

fn convert_a_to_b(
    file_path: &PathBuf,
    from_format: &str,
    to_format: &str,
) -> Result<PathBuf, String> {
    if from_format == to_format {
        return Ok(file_path.into());
    }

    if !file_path.exists() {
        return Err(format!("File {file_path:?} does not exist"));
    }

    if let Some(ext) = file_path.extension() {
        if ext != from_format || ext == to_format {
            return Ok(file_path.into());
        }
    }

    let from_file_path = file_path.with_extension(from_format);
    let to_file_path = file_path.with_extension(to_format);

    info!("Converting file {from_file_path:?} to {to_format}");

    let ffmpeg_path = &CONFIGURATION.ffmpeg_path;
    debug!("`ffmpeg' binary: {ffmpeg_path:?}");
    let mut cmd = process::Command::new(ffmpeg_path);
    let cmd = cmd
        .arg("-y")
        .arg("-hide_banner")
        .args(["-loglevel", "panic"])
        .args(["-i", (from_file_path.to_str().unwrap())])
        .args(["-max_muxing_queue_size", "1024"])
        .args(["-vf", "scale=ceil(iw/2)*2:ceil(ih/2)*2"])
        .args(["-ab", "320k"])
        .args(["-map_metadata", "-1"])
        .args(["-preset", "slow"])
        .arg(&to_file_path);
    info!("Running `ffmpeg' command: {cmd:?}");

    let cmd_output = cmd.output();
    match cmd_output {
        Ok(process::Output { status, .. }) if status.success() && to_file_path.exists() => {
            info!("Converted file {from_file_path:?} to {to_format}");

            match copy_file_times(&from_file_path, &to_file_path) {
                Err(e) => {
                    debug!("Failed to copy file times: {e:?}");
                }
                Ok(_) => {
                    trace!("Copied file times from {from_file_path:?}");
                }
            }

            if move_to_trash(&from_file_path).is_ok() {
                debug!("Deleted old file {from_file_path:?}");
            }

            Ok(to_file_path)
        }
        _ => {
            trace!("`ffmpeg' command output: {cmd_output:?}");
            debug!("Deleting new file {to_file_path:?}");
            if let Err(e) = fs::remove_file(&to_file_path) {
                debug!("Failed to delete new file {to_file_path:?}: {e:?}");
            }
            Err(format!(
                "Failed transforming {from_file_path:?} into {to_format}"
            ))
        }
    }
}

const UNWANTED_CODECS: [&str; 2] = ["av1", "hevc"];

fn reencode_dodgy_encodings(file_path: &PathBuf) -> Result<PathBuf, String> {
    let media_info = ffprobe::ffprobe(file_path)
        .map_err(|e| format!("Failed to get media info of {file_path:?}: {e:?}"))?;
    trace!("`ffprobe' output: {media_info:?}");

    let has_unwanted_codec = media_info
        .streams
        .iter()
        .filter(|s| option_contains(&s.codec_type, &"video".to_string()))
        .find(|s| {
            UNWANTED_CODECS
                .iter()
                .any(|c| option_contains(&s.codec_name, &(*c).to_string()))
        });

    if let Some(codec) = has_unwanted_codec {
        debug!("Has unwanted codec: {codec:?}");
    }

    match has_unwanted_codec {
        Some(_) => reencode_video_file(file_path),
        None => Ok(file_path.into()),
    }
}

#[allow(clippy::similar_names)]
fn reencode_video_file(file_path: &PathBuf) -> Result<PathBuf, String> {
    info!("Reencoding file {file_path:?}");

    let now_ns = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = env::temp_dir().join(format!("tmp.{now_ns}"));
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir {temp_dir:?}: {e:?}"))?;
    let temp_file = temp_dir.join(file_path.file_name().unwrap());

    let old_meta = file_path
        .metadata()
        .map_err(|e| format!("Failed to get metadata of {file_path:?}: {e:?}"))?;
    let old_mtime = FileTime::from_last_modification_time(&old_meta);
    let old_atime = FileTime::from_last_access_time(&old_meta);

    let ffmpeg_path = &CONFIGURATION.ffmpeg_path;
    let mut cmd = process::Command::new(ffmpeg_path);
    let cmd = cmd
        .args(["-i", file_path.to_str().unwrap()])
        .args(["-loglevel", "panic"])
        .args(["-preset", "slow"])
        .arg(&temp_file);
    info!("`ffmpeg' reencode command: {cmd:?}");

    let temp_file = match cmd.status() {
        Ok(status) if status.success() => Ok(temp_file),
        _ => Err(format!("Failed to reencode {file_path:?}")),
    }?;

    let move_options = fs_extra::file::CopyOptions::new().overwrite(true);

    fs_extra::file::move_file(&temp_file, file_path, &move_options)
        .map_err(|e| format!("Failed to move {temp_file:?} to {file_path:?}: {e:?}"))?;

    filetime::set_file_times(file_path, old_atime, old_mtime)
        .map_err(|e| format!("Failed to set file times of {file_path:?}: {e:?}"))?;

    Ok(file_path.into())
}

#[allow(clippy::similar_names)]
fn copy_file_times<'s>(old: &PathBuf, new: &'s PathBuf) -> Result<&'s PathBuf, String> {
    let old_meta = old
        .metadata()
        .map_err(|e| format!("Failed to get metadata of {old:?}: {e:?}"))?;

    let old_mtime = FileTime::from_last_modification_time(&old_meta);
    let old_atime = FileTime::from_last_access_time(&old_meta);

    filetime::set_file_times(new, old_atime, old_mtime)
        .map_err(|e| format!("Failed to set file times of {new:?}: {e:?}"))?;

    Ok(new)
}

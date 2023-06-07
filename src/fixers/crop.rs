use super::FixerReturn;
use crate::{
    config::CONFIGURATION,
    helpers::{ffprobe, results::option_contains, trash::move_to_trash},
};
use log::{debug, info, trace};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{ffi::OsStr, fmt::Display, path::PathBuf, process};

pub fn auto_crop_video(file_path: &PathBuf) -> FixerReturn {
    info!("Auto cropping video {file_path:?}");

    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| format!("Failed to convert {file_path:?} to string"))?;
    let media_info = ffprobe::ffprobe(file_path).map_err(|e| format!("{e:?}"))?;
    let video_stream = media_info
        .streams
        .iter()
        .find(|s| option_contains(&s.codec_type, &"video".to_string()));

    let video_stream = match video_stream {
        Some(s) => {
            trace!("Found video stream");
            s
        }
        None => {
            info!("File does not contain a video stream, skipping");
            return Ok(file_path.into());
        }
    };

    let (w, h) = match (video_stream.width, video_stream.height) {
        (Some(w), Some(h)) => {
            trace!("Video width: {w}, height: {h}");
            (w, h)
        }
        _ => {
            return Err(format!(
                "Failed to get video width and height for {file_path:?}"
            ));
        }
    };

    let crop_filters = vec![BorderColor::White, BorderColor::Black]
        .into_par_iter()
        .map(|color| get_crop_filter(file_path_str, &color))
        .collect::<Result<Option<Vec<_>>, String>>()?;

    let crop_filters = match crop_filters {
        Some(crop_filters) => {
            trace!("Crop filters: {crop_filters:?}");
            crop_filters
        }
        None => {
            info!("No crop filters found, skipping");
            return Ok(file_path.into());
        }
    };

    let mut final_crop_filter = CropFilter {
        width: i64::MAX,
        height: i64::MAX,
        x: i64::MIN,
        y: i64::MIN,
    };
    for crop_filter in crop_filters {
        if crop_filter.width < final_crop_filter.width {
            final_crop_filter.width = crop_filter.width;
        }

        if crop_filter.height < final_crop_filter.height {
            final_crop_filter.height = crop_filter.height;
        }

        if crop_filter.x > final_crop_filter.x {
            final_crop_filter.x = crop_filter.x;
        }

        if crop_filter.y > final_crop_filter.y {
            final_crop_filter.y = crop_filter.y;
        }
    }

    debug!("Final crop filter: {final_crop_filter:?}");

    if final_crop_filter.width >= w && final_crop_filter.height >= h {
        info!("Video is already cropped, skipping");
        return Ok(file_path.into());
    }

    let new_filename = {
        let file_name = file_path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("Failed to get file stem from {file_path:?}"))?;

        let file_extension = file_path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("Failed to get file extension from {file_path:?}"))?;

        file_path.with_file_name(format!("{file_name}.ac.{file_extension}"))
    };

    let mut cmd = process::Command::new(&CONFIGURATION.ffmpeg_path);
    let cmd = cmd
        .arg("-y")
        .args(["-loglevel", "panic"])
        .args(["-i", file_path_str])
        .args(["-vf", &final_crop_filter.to_string()])
        .args(["-preset", "slow"])
        .arg(&new_filename);
    info!("Running command {cmd:?}");

    let cmd_output = cmd
        .output()
        .map_err(|e| format!("Failed to run command {cmd:?}: {e:?}"))?;

    if !cmd_output.status.success() {
        let stderr = String::from_utf8(cmd_output.stderr)
            .map_err(|e| format!("Failed to convert command output to UTF-8: {e:?}"))?;

        return Err(format!(
            "Command {cmd:?} failed with status {status:?} and output {output:?}",
            cmd = cmd,
            status = cmd_output.status,
            output = stderr
        ));
    }

    move_to_trash(file_path).map_err(|e| format!("Failed to move file to trash: {e:?}"))?;

    Ok(new_filename)
}

#[derive(Default, Debug, Clone)]
struct CropFilter {
    width: i64,
    height: i64,
    x: i64,
    y: i64,
}

impl Display for CropFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "crop={width}:{height}:{x}:{y}",
            width = self.width,
            height = self.height,
            x = self.x,
            y = self.y
        )
    }
}

#[derive(Debug, Clone)]
enum BorderColor {
    White,
    Black,
}

fn get_crop_filter(
    file_path: &str,
    border_color: &BorderColor,
) -> Result<Option<CropFilter>, String> {
    let cropdetect_filter = "cropdetect=mode=black:limit=24:round=2:reset=0";

    let mut cmd = process::Command::new(&CONFIGURATION.ffmpeg_path);
    let cmd = cmd
        .arg("-hide_banner")
        .args(["-i", file_path])
        .args([
            "-vf",
            (match border_color {
                BorderColor::White => {
                    format!("negate,{cropdetect_filter}")
                }
                BorderColor::Black => cropdetect_filter.to_string(),
            })
            .as_str(),
        ])
        .args(["-f", "null", "-"]);

    let cmd_output = cmd
        .output()
        .map_err(|e| format!("Failed to run command {cmd:?}: {e:?}"))?;
    let stderr = String::from_utf8(cmd_output.stderr)
        .map_err(|e| format!("Failed to convert command output to UTF-8: {e:?}"))?;

    let mut res = stderr
        .split('\n')
        .filter(|s| s.starts_with("[Parsed_cropdetect") && s.contains("crop="))
        .map(str::trim)
        .map(|s| {
            s.split("crop=")
                .nth(1)
                .ok_or_else(|| format!("Failed to parse cropdetect output from {s:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    res.sort_unstable();
    res.dedup();

    let res = res
        .iter()
        .map(|s| {
            let mut s = s.split(':');
            let mut next_s = || {
                s.next()
                    .and_then(|x| x.to_string().parse::<i64>().ok())
                    .ok_or_else(|| format!("Failed to parse width from {s:?}"))
            };

            Ok(CropFilter {
                width: next_s()?,
                height: next_s()?,
                x: next_s()?,
                y: next_s()?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(get_minmax_crop_filter(res))
}

fn get_minmax_crop_filter(res: Vec<CropFilter>) -> Option<CropFilter> {
    trace!("get_minmax_crop_filter({res:?})");

    if res.is_empty() {
        return None;
    }

    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_w = i64::MIN;
    let mut max_h = i64::MIN;
    for filter in res {
        let x = filter.x;
        let y = filter.y;
        let w = filter.width;
        let h = filter.height;

        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if w > max_w {
            max_w = w;
        }
        if h > max_h {
            max_h = h;
        }
    }

    Some(CropFilter {
        width: max_w,
        height: max_h,
        x: min_x,
        y: min_y,
    })
}

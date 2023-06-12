use super::FixerReturn;
use config::CONFIGURATION;
use helpers::{ffprobe, results::option_contains, trash::move_to_trash};
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

    let (w, h) = {
        let video_stream = if let Some(s) = video_stream {
            trace!("Found video stream");
            s
        } else {
            info!("File does not contain a video stream, skipping");
            return Ok(file_path.into());
        };

        if let (Some(w), Some(h)) = (video_stream.width, video_stream.height) {
            trace!("Video width: {w}, height: {h}");
            (w, h)
        } else {
            return Err(format!(
                "Failed to get video width and height for {file_path:?}"
            ));
        }
    };

    let crop_filters = {
        let crop_filters = vec![BorderColor::White, BorderColor::Black]
            .into_par_iter()
            .filter_map(|color| get_crop_filter(file_path_str, &color).ok())
            .collect::<Option<Vec<_>>>();

        if let Some(fs) = crop_filters {
            trace!("Crop filters: {fs:?}");
            fs
        } else {
            info!("No crop filters found, skipping");
            return Ok(file_path.into());
        }
    };

    let final_crop_filter = CropFilter::intersect_all(crop_filters)
        .ok_or_else(|| "Failed to intersect crop filters".to_string())?;

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

#[derive(Debug, Clone)]
struct CropFilter {
    width: i64,
    height: i64,
    x: i64,
    y: i64,
}

impl CropFilter {
    fn union(&mut self, other: &Self) {
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
        self.x = self.x.min(other.x);
        self.y = self.y.min(other.y);
    }

    fn union_all(filters: Vec<Self>) -> Option<Self> {
        if filters.is_empty() {
            return None;
        }

        let mut res = Self {
            width: i64::MIN,
            height: i64::MIN,
            x: i64::MAX,
            y: i64::MAX,
        };

        for filter in filters {
            res.union(&filter);
        }

        Some(res)
    }

    fn intersect(&mut self, other: &Self) {
        self.width = self.width.min(other.width);
        self.height = self.height.min(other.height);
        self.x = self.x.max(other.x);
        self.y = self.y.max(other.y);
    }

    fn intersect_all(filters: Vec<Self>) -> Option<Self> {
        if filters.is_empty() {
            return None;
        }

        let mut res = Self {
            width: i64::MAX,
            height: i64::MAX,
            x: i64::MIN,
            y: i64::MIN,
        };

        for filter in filters {
            res.intersect(&filter);
        }

        Some(res)
    }
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

    Ok(CropFilter::union_all(res))
}

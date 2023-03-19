use super::FixerReturn;
use crate::{
    config::CONFIG,
    helpers::{ffprobe, results::option_contains, trash::move_to_trash},
};
use log::{debug, info};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process,
};

pub fn auto_crop_video(file_path: &PathBuf) -> FixerReturn {
    let ffmpeg = CONFIG.clone().ffmpeg_path()?;
    let crop_filters = vec![BorderColor::White, BorderColor::Black]
        .into_par_iter()
        .map(|color| get_crop_filters(&ffmpeg, file_path, &color).map(get_minmax_crop_filter))
        .collect::<Result<Vec<_>, String>>()?;

    let mut final_crop_filter = CropFilter {
        width: i32::MAX,
        height: i32::MAX,
        x: i32::MIN,
        y: i32::MIN,
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

    let media_info = ffprobe::ffprobe(file_path).map_err(|e| format!("{e:?}"))?;
    let video_stream = media_info
        .streams
        .iter()
        .find(|s| option_contains(&s.codec_type, &"video".to_string()))
        .ok_or("No video stream found")?;

    match (video_stream.width, video_stream.height) {
        (Some(w), Some(h))
            if i64::from(final_crop_filter.width) >= w
                && i64::from(final_crop_filter.height) >= h =>
        {
            debug!("Video is already cropped, skipping");
            return Ok(file_path.into());
        }
        _ => {}
    }

    let new_filename = {
        let file_name = file_path.file_stem().unwrap().to_str().unwrap();
        let file_extension = file_path.extension().unwrap().to_str().unwrap();

        file_path.with_file_name(format!("{file_name}.ac.{file_extension}"))
    };

    let mut cmd = process::Command::new(ffmpeg);
    let cmd = cmd
        .arg("-y")
        .args(["-loglevel", "panic"])
        .args(["-i", file_path.to_str().unwrap()])
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
    width: i32,
    height: i32,
    x: i32,
    y: i32,
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

fn get_crop_filters(
    ffmpeg_path: &PathBuf,
    file: &Path,
    border_color: &BorderColor,
) -> Result<Vec<CropFilter>, String> {
    let mut cmd = process::Command::new(ffmpeg_path);
    let cmd = cmd
        .arg("-hide_banner")
        .args(["-i", file.to_str().unwrap()])
        .args([
            "-vf",
            match border_color {
                BorderColor::White => "negate,cropdetect=24:2:0,negate",
                BorderColor::Black => "cropdetect=24:2:0",
            },
        ])
        .args(["-f", "null", "-"]);

    let cmd_output = cmd
        .output()
        .map_err(|e| format!("Failed to run command {cmd:?}: {e:?}"))?;
    let stderr = String::from_utf8(cmd_output.stderr)
        .map_err(|e| format!("Failed to convert command output to UTF-8: {e:?}"))?;

    let res = stderr
        .split('\n')
        .filter(|s| s.starts_with("[Parsed_cropdetect") && s.contains("crop="))
        .map(str::trim)
        .map(|s| s.split("crop=").nth(1).unwrap())
        .map(|s| {
            let mut s = s.split(':');

            CropFilter {
                width: s.next().unwrap().to_string().parse::<i32>().unwrap(),
                height: s.next().unwrap().to_string().parse::<i32>().unwrap(),
                x: s.next().unwrap().to_string().parse::<i32>().unwrap(),
                y: s.next().unwrap().to_string().parse::<i32>().unwrap(),
            }
        });

    Ok(res.collect())
}

fn get_minmax_crop_filter(res: Vec<CropFilter>) -> CropFilter {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_w = i32::MIN;
    let mut max_h = i32::MIN;
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

    CropFilter {
        width: max_w,
        height: max_h,
        x: min_x,
        y: min_y,
    }
}

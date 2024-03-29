use std::{path::PathBuf, process};

use app_config::CONFIGURATION;
use app_helpers::id::time_id;
use app_logger::{debug, trace};

use super::DownloaderReturn;
use crate::downloaders::{common::USER_AGENT, generic};

pub fn download(download_dir: &PathBuf, url: &str) -> DownloaderReturn {
    let yt_dlp = &CONFIGURATION.yt_dlp_path;
    trace!("`yt-dlp' binary: {:?}", &yt_dlp);
    let output_template = get_output_template(download_dir);
    debug!("template: {:?}", &output_template);
    let mut cmd = process::Command::new(yt_dlp);
    let cmd = cmd
        .arg("--no-check-certificate")
        .args(["--socket-timeout", "120"])
        .arg("--no-part")
        .arg("--no-mtime")
        .arg("--no-embed-metadata")
        .args([
            "--output",
            output_template
                .to_str()
                .ok_or_else(|| "Failed to convert path to string".to_string())?,
        ])
        .args(["--user-agent", USER_AGENT])
        .args(["--no-simulate", "--print", "after_move:filepath"])
        // .arg("--verbose")
        .arg(url);
    debug!("Running cmd: {:?}", &cmd);
    let cmd_output = cmd.output();
    trace!("Cmd output: {:?}", &cmd_output);
    let mut err = String::new();
    let new_file_path = match cmd_output {
        Ok(process::Output {
            stdout,
            stderr: _,
            status,
        }) if status.success() => {
            let output = String::from_utf8(stdout)
                .map_err(|e| format!("Failed to convert output to UTF-8: {e:?}"))?;
            let output_path = PathBuf::from(output.trim());

            if output_path.exists() {
                debug!("yt-dlp successful download to file: {:?}", output_path);
                Ok(output_path)
            } else {
                Err("yt-dlp finished but file does not exist.")
            }
        }
        Ok(process::Output {
            stdout: _,
            stderr,
            status: _,
        }) if is_image_error(stderr.clone()) => return generic::download(download_dir, url),
        _ => {
            let msg = format!("yt-dlp failed downloading meme: {cmd_output:?}");
            err.push_str(msg.as_str());
            Err(err.as_str())
        }
    }?;

    if !new_file_path.exists() {
        return Err("yt-dlp finished but file does not exist.".to_string());
    }

    Ok(vec![new_file_path])
}

fn get_output_template<S: Into<PathBuf>>(download_dir: S) -> PathBuf {
    let file_identifier = time_id();
    let file_name = format!("{file_identifier}.%(id).64s.%(ext)s");

    download_dir.into().join(file_name)
}

fn is_image_error(output: Vec<u8>) -> bool {
    let output = String::from_utf8(output).unwrap_or_default();
    let output = output.trim();

    trace!("yt-dlp output: {output}");

    output.ends_with(". Maybe an image?")
}

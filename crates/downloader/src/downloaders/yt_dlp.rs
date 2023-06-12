use super::DownloaderReturn;
use crate::downloaders::USER_AGENT;
use config::CONFIGURATION;
use helpers::id::time_id;
use log::{debug, info};
use std::{path::PathBuf, process};

pub fn download(meme_dir: &PathBuf, url: &str) -> DownloaderReturn {
    let yt_dlp = &CONFIGURATION.yt_dlp_path;
    debug!("`yt-dlp' binary: {:#?}", &yt_dlp);
    let output_template = get_output_template(meme_dir);
    debug!("template: {:#?}", &output_template);
    let mut cmd = process::Command::new(yt_dlp);
    let cmd = cmd
        .arg("--no-check-certificate")
        .args(["--socket-timeout", "120"])
        .arg("--no-part")
        .arg("--no-mtime")
        .arg("--no-embed-metadata")
        .args(["--output", output_template.to_str().unwrap()])
        .args(["--user-agent", USER_AGENT])
        .args(["--no-simulate", "--print", "after_move:filepath"])
        // .arg("--verbose")
        .arg(url);
    info!("Running cmd: {:?}", &cmd);
    let cmd_output = cmd.output();
    debug!("Cmd output: {:?}", &cmd_output);
    let mut err = String::new();
    let new_file_path = match cmd_output {
        Ok(process::Output {
            stdout,
            stderr: _,
            status,
        }) if status.success() => {
            let output = String::from_utf8(stdout).unwrap();
            let output_path = PathBuf::from(output.trim());

            if output_path.exists() {
                info!("yt-dlp successful download to file: {:?}", output_path);
                Ok(output_path)
            } else {
                Err("yt-dlp finished but file does not exist.")
            }
        }
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

fn get_output_template<S: Into<PathBuf>>(meme_dir: S) -> PathBuf {
    let file_identifier = time_id().unwrap();
    let file_name = format!("{file_identifier}.%(id).64s.%(ext)s");

    meme_dir.into().join(file_name)
}

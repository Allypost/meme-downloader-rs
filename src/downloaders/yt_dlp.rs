use super::DownloaderReturn;
use crate::{
    args, config,
    downloaders::{get_output_template, USER_AGENT},
};
use log::{debug, error, info};
use std::{path::PathBuf, process::exit};

pub fn download(meme_dir: &PathBuf, url: &str) -> DownloaderReturn {
    let args = args::ARGS.clone();
    let config = config::CONFIG.clone();

    let yt_dlp = args
        .yt_dlp_path
        .or(config.yt_dlp_path)
        .unwrap_or_else(get_yt_dlp_path);
    debug!("`yt-dlp' binary: {:#?}", &yt_dlp);
    let output_template = get_output_template(meme_dir);
    debug!("template: {:#?}", &output_template);
    let mut cmd = std::process::Command::new(yt_dlp);
    let cmd = cmd
        .arg("--no-check-certificate")
        .args(["--socket-timeout", "120"])
        .arg("--no-part")
        .arg("--no-mtime")
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
        Ok(std::process::Output {
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

fn get_yt_dlp_path() -> PathBuf {
    let yt_dlp = which::which("yt-dlp");
    if yt_dlp.is_err() {
        error!("`yt-dlp' is not installed. Please install it first.\nInstructions to install it are available at <https://github.com/yt-dlp/yt-dlp#installation>");
        exit(1);
    }

    yt_dlp.unwrap()
}

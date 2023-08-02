#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::manual_let_else)]

use std::{fs, path::PathBuf, process::exit};

use config::{APPLICATION_NAME, CONFIG};
use log::{error, info, trace};
use logger::LoggerConfig;

#[cfg(feature = "desktop-notifications")]
mod notif;

fn main() {
    dbg!((*CONFIG).clone());

    #[cfg(feature = "telegram-bot")]
    {
        if let Some(config::RunAsBot::Telegram) = CONFIG.run.run_as_bot {
            run_telegram_bot();
        }
    }

    let download_url = if let Ok(url) = get_download_url() {
        url
    } else {
        eprintln!("Failed to get download URL.");
        exit(1);
    };

    if download_url.is_empty() {
        eprintln!("No download URL provided. Please provide one.");
        exit(1);
    }

    if logger::init(
        LoggerConfig::builder()
            .program_name(APPLICATION_NAME)
            .name_suffix(&download_url),
    )
    .is_err()
    {
        eprintln!("Failed to initialize logger.");
        exit(1);
    }

    trace!("Config: {:?}", *CONFIG);

    if CONFIG.run.fix {
        let file_path = PathBuf::from(&download_url);

        info!("Fixing file: {:?}", &file_path);

        fixers::fix_files(&vec![file_path]).unwrap_or_else(|e| {
            error!("Error fixing file: {:?}", e);
            exit(1);
        });

        return;
    }

    let meme_dir = CONFIG.app.memes_directory.clone();
    if !meme_dir.exists() {
        info!("Memes directory does not exist. Creating...");
        fs::create_dir_all(&meme_dir).unwrap_or_else(|e| {
            error!("Error creating memes directory: {:?}", e);
            exit(1);
        });
    }
    trace!("Meme dir: {meme_dir:?}");

    match downloader::download_file(&download_url, &meme_dir) {
        Ok(paths) => {
            info!(
                "Downloaded file(s): {}",
                paths
                    .iter()
                    .map(|x| { return x.to_str().unwrap() })
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

            #[cfg(feature = "desktop-notifications")]
            {
                let notif = notif::send_notification(&notif::NotificationInfo {
                    urgency: notify_rust::Urgency::Low,
                    timeout: notify_rust::Timeout::Milliseconds(5000),
                    icon: "success".to_string(),
                    title: "Download finished".to_string(),
                    message: format!("The meme from {} has finished downloading", &download_url),
                });

                if let Err(e) = notif {
                    error!("Error sending notification: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Error downloading file: {}", e);

            #[cfg(feature = "desktop-notifications")]
            {
                let notif = notif::send_notification(&notif::NotificationInfo {
                    urgency: notify_rust::Urgency::Normal,
                    timeout: notify_rust::Timeout::Milliseconds(10_000),
                    icon: "error".to_string(),
                    title: "Download failed".to_string(),
                    message: format!(
                        "The meme downloader couldn't download the provided page: {}",
                        &download_url
                    ),
                });

                if let Err(e) = notif {
                    error!("Error sending notification: {}", e);
                }
            }
            exit(1);
        }
    }
}

#[cfg(feature = "telegram-bot")]
fn run_telegram_bot() {
    if logger::init(
        LoggerConfig::builder()
            .program_name(APPLICATION_NAME)
            .name_suffix("telegram-bot")
            .file_log_level(log::LevelFilter::Debug)
            .stdout_log_level(if cfg!(debug_assertions) {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Info
            }),
    )
    .is_err()
    {
        eprintln!("Failed to initialize logger.");
        exit(1);
    }

    if CONFIG.bots.telegram.is_none() {
        eprintln!("No Telegram configuration provided. Please provide one.");
        exit(1);
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(bots::bot::telegram::run());

    info!("Telegram bot stopped");
    exit(0);
}

fn get_download_url() -> anyhow::Result<String> {
    let download_url = CONFIG.run.download_url.as_ref();

    if cfg!(feature = "ask-for-url") {
        use std::io;
        use std::io::prelude::*;

        if let Some(download_url) = download_url {
            return Ok(download_url.to_string());
        }

        if atty::isnt(atty::Stream::Stdin) {
            anyhow::bail!("No download URL provided. Please provide one.");
        }

        print!("Download URL: ");
        io::stdout().flush()?;

        let res = io::stdin()
            .lock()
            .lines()
            .next()
            .unwrap_or_else(|| Ok(String::new()))
            .unwrap_or_default();

        Ok(res)
    } else {
        download_url
            .ok_or_else(|| anyhow::anyhow!("No download URL provided. Please provide one."))
            .map(std::string::ToString::to_string)
    }
}

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]

use crate::config::CONFIG;
use log::{error, info, trace};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{
    fs,
    io::{self, Write},
    process::exit,
};

mod args;
mod bot;
mod config;
mod downloaders;
mod fixers;
mod helpers;
mod logger;
mod notif;

extern crate sanitize_filename;

fn main() {
    let args = args::ARGS.clone();

    if args.telegram_run_as_bot {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(bot::telegram::run());
        info!("Bot stopped");
        return;
    }

    let download_url = args.download_url.unwrap_or_else(|| {
        if atty::isnt(atty::Stream::Stdin) {
            error!("No download URL provided. Please provide one.");
            exit(1);
        }

        print!("Download URL: ");
        io::stdout().flush().unwrap();
        return io::stdin().lock().lines().next().unwrap().unwrap();
    });

    logger::init(&download_url);

    trace!("Config: {:?}", config::CONFIG);
    trace!("Args: {:?}", args::ARGS);

    if args.fix {
        let file_path = PathBuf::from(&download_url);

        info!("Fixing file: {:?}", &file_path);

        fixers::fix_files(&vec![file_path]).unwrap_or_else(|e| {
            error!("Error fixing file: {:?}", e);
            exit(1);
        });

        return;
    }

    let meme_dir = CONFIG.clone().memes_dir().unwrap_or_else(|e| {
        error!("Error resolving memes directory: {:?}", e);
        exit(1);
    });
    if !meme_dir.exists() {
        info!("Memes directory does not exist. Creating...");
        fs::create_dir_all(&meme_dir).unwrap_or_else(|e| {
            error!("Error creating memes directory: {:?}", e);
            exit(1);
        });
    }
    trace!("Meme dir: {meme_dir:?}");

    match downloaders::download_file(&download_url, &meme_dir) {
        Ok(paths) => {
            info!(
                "Downloaded file(s): {}",
                paths
                    .iter()
                    .map(|x| { return x.to_str().unwrap() })
                    .collect::<Vec<&str>>()
                    .join(", ")
            );

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
        Err(e) => {
            error!("Error downloading file: {}", e);

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
            exit(1);
        }
    }
}

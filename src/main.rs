#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::config::CONFIG;
use log::{debug, error, info, trace};
use std::io::prelude::*;
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process::exit,
};

mod args;
mod config;
mod downloaders;
mod fixers;
mod helpers;
mod notif;

extern crate sanitize_filename;

fn main() {
    let args = args::ARGS.clone();

    let download_url = args.download_url.unwrap_or_else(|| {
        if atty::isnt(atty::Stream::Stdin) {
            error!("No download URL provided. Please provide one.");
            exit(1);
        }

        print!("Download URL: ");
        io::stdout().flush().unwrap();
        return io::stdin().lock().lines().next().unwrap().unwrap();
    });

    init_logger(&download_url);

    trace!("Config: {:?}", config::CONFIG);
    trace!("Args: {:?}", args::ARGS);

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

fn init_logger<S: AsRef<str>>(download_url: S) {
    let download_url_filename = sanitize_filename::sanitize_with_options(
        download_url.as_ref(),
        sanitize_filename::Options {
            windows: cfg!(target_os = "windows"),
            truncate: true,
            replacement: "^",
        },
    );
    let program_name = get_program_name();
    let mut tmp_file = format!("{program_name}_{download_url_filename}");
    if cfg!(target_os = "windows") {
        tmp_file = format!("{tmp_file}.txt");
    }
    let mut tmp_file = env::temp_dir().join(tmp_file);
    tmp_file.shrink_to_fit();
    fs::create_dir_all(tmp_file.parent().unwrap()).unwrap();

    let stdout = log4rs::append::console::ConsoleAppender::builder()
        .target(log4rs::append::console::Target::Stdout)
        .build();
    let log = log4rs::append::file::FileAppender::builder()
        .build(&tmp_file)
        .unwrap();

    let config = log4rs::config::Config::builder()
        .appender(
            log4rs::config::Appender::builder()
                .filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
                    log::LevelFilter::Info,
                )))
                .build("logfile", Box::new(log)),
        )
        .appender(
            log4rs::config::Appender::builder()
                .filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
                    log::LevelFilter::Trace,
                )))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            log4rs::config::Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(log::LevelFilter::Trace),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    debug!("Logging to {:?}", &tmp_file);
}

fn get_program_name() -> String {
    let args = env::args().collect::<Vec<String>>();
    let program_name = Path::new(args.first().unwrap())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    program_name.to_string()
}

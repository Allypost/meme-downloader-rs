use crate::args;
use crate::args::ARGS;
use log::{debug, error, trace};
use once_cell::sync::Lazy;
use resolve_path::PathResolveExt;
use std::io::{self, prelude::*};
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};
use which::which;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct TelegramBotConfig {
    bot_token: String,
    pub owner_id: Option<u64>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    yt_dlp_path: Option<PathBuf>,
    ffmpeg_path: Option<PathBuf>,
    ffprobe_path: Option<PathBuf>,
    memes_directory: Option<PathBuf>,

    pub telegram: Option<TelegramBotConfig>,
}

impl Config {
    fn parse() -> Self {
        let mut config: Self = Self::default();
        let config_dir = dirs::config_dir().unwrap();
        let config_file = config_dir.join("meme-downloader").join("config.toml");
        if config_file.exists() {
            let config_file = fs::read_to_string(config_file).unwrap();
            config = match toml::from_str(&config_file) {
                Ok(config) => {
                    trace!("Parsed config file successfully");
                    config
                }
                Err(e) => {
                    error!("Error parsing config file: {:?}", e);
                    exit(1);
                }
            };
        } else {
            debug!("Config file not found. Creating one at {:#?}", config_file);
            if let Ok(memes_dir) = config.clone().memes_dir() {
                config.memes_directory = Some(memes_dir);
            }
            let config_dir = config_file.parent().unwrap();
            fs::create_dir_all(config_dir).unwrap();
            let mut config_file = fs::File::create(config_file).unwrap();
            let config = toml::to_string_pretty(&config).unwrap();
            config_file.write_all(config.as_bytes()).unwrap();
        }

        config
    }

    pub fn yt_dlp_path(self) -> Result<PathBuf, String> {
        let args = ARGS.clone();
        let yt_dlp_path = args.yt_dlp_path.or(self.yt_dlp_path);
        config_or_which(&yt_dlp_path, "yt-dlp")
    }

    pub fn ffmpeg_path(self) -> Result<PathBuf, String> {
        let ffmpeg_path = self.ffmpeg_path;
        config_or_which(&ffmpeg_path, "ffmpeg")
    }

    pub fn ffprobe_path(self) -> Result<PathBuf, String> {
        let ffprobe_path = self.ffprobe_path;
        config_or_which(&ffprobe_path, "ffprobe")
    }

    pub fn memes_dir(self) -> Result<PathBuf, io::Error> {
        let args = args::ARGS.clone();

        let raw_path = args
            .memes_directory
            .or(self.memes_directory)
            .unwrap_or_else(|| {
                let home_dir = dirs::home_dir().unwrap();
                home_dir.join("MEMES")
            });

        match raw_path.try_resolve() {
            Ok(path) => Ok(path.into()),
            Err(e) => Err(e),
        }
    }

    pub fn telegram_bot_token(&self) -> Option<String> {
        let args = ARGS.clone();

        args.telegram_bot_token
            .map(|t| {
                debug!("Using bot token from arguments");
                t
            })
            .or_else(|| self.telegram.as_ref().map(|t| &t.bot_token).cloned())
            .map(|t| {
                debug!("Using bot token from config");
                t
            })
            .or_else(|| env::var("MEME_DOWNLOADER_TELEGRAM_TOKEN").ok())
            .map(|t| {
                debug!("Using bot token from environment variable");
                t
            })
    }

    pub fn telegram_owner_id(self) -> Option<u64> {
        if let Some(id) = ARGS.clone().telegram_owner_id {
            return Some(id);
        }

        if let Some(Some(id)) = self.telegram.as_ref().map(|t| t.owner_id) {
            return Some(id);
        }

        if let Ok(id) = env::var("MEME_DOWNLOADER_TELEGRAM_OWNER_ID") {
            return id.parse().ok();
        }

        None
    }
}

fn config_or_which(field: &Option<PathBuf>, program: &str) -> Result<PathBuf, String> {
    field
        .clone()
        .ok_or_else(|| format!("`{program}' path not found in config.toml"))
        .or_else(|_e| which(program))
        .map_err(|e| {
            format!(
                "`{program}' not found in PATH or config. Please install it or specify the path in config.toml. Error: {e}"
            )
        })
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);

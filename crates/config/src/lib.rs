#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_let_else)]

use std::env;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::Parser;
use cli::CliArgs;
use directories::ProjectDirs;
use file::FileConfiguration;
use lazy_static::lazy_static;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use which::which;

use crate::cli::DumpType;

mod cli;
mod common;
mod file;

pub static APPLICATION_NAME: &str = "meme-downloader";
pub static ORGANIZATION_NAME: &str = "allypost";
pub static ORGANIZATION_QUALIFIER: &str = "net";

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
    // pub static ref CONFIGURATION: Configuration = Configuration::new();
    pub static ref CONFIGURATION: Configuration = Configuration::from_new(CONFIG.clone());
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,

    #[serde(skip)]
    pub run: RunConfig,

    pub dependencies: common::ProgramPathConfig,

    pub bots: common::BotConfig,
}

impl Config {
    #[must_use]
    pub fn get_config_dir() -> Option<PathBuf> {
        Self::get_project_dir().map(|x| x.config_dir().into())
    }

    #[must_use]
    pub fn config_dir(&self) -> Option<PathBuf> {
        Self::get_config_dir()
    }

    #[must_use]
    pub fn get_cache_dir() -> PathBuf {
        Self::get_project_dir().map_or_else(
            || env::temp_dir().join(APPLICATION_NAME),
            |x| x.cache_dir().into(),
        )
    }

    #[must_use]
    pub fn cache_dir(&self) -> PathBuf {
        Self::get_cache_dir()
    }

    fn get_project_dir() -> Option<ProjectDirs> {
        ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
    }

    fn new() -> Self {
        let mut config = Self::default();
        let args = CliArgs::parse();
        let file_config = FileConfiguration::new(args.app.config_path.as_deref()).unwrap();

        config.merge_file_config(&file_config);
        config.merge_args(&args);

        {
            if config.dependencies.yt_dlp_path.is_none() {
                config.dependencies.yt_dlp_path = Some(
                    which("yt-dlp")
                        .map_err(|e| anyhow!("yt-dlp not found: {}", e))
                        .unwrap(),
                );
            }

            if config.dependencies.ffmpeg_path.is_none() {
                config.dependencies.ffmpeg_path = Some(
                    which("ffmpeg")
                        .map_err(|e| anyhow!("ffmpeg not found: {}", e))
                        .unwrap(),
                );
            }

            if config.dependencies.ffprobe_path.is_none() {
                config.dependencies.ffprobe_path = Some(
                    which("ffprobe")
                        .map_err(|e| anyhow!("ffprobe not found: {}", e))
                        .unwrap(),
                );
            }

            if config.dependencies.scenedetect_path.is_none() {
                config.dependencies.scenedetect_path = which("scenedetect")
                    .map_err(|e| anyhow!("scenedetect not found: {}", e))
                    .ok();
            }
        }

        {
            if config.app.memes_directory.as_os_str().is_empty() {
                config.app.memes_directory = directories::UserDirs::new()
                    .unwrap()
                    .home_dir()
                    .join("MEMES");
            }
            config.app.memes_directory = config.app.memes_directory.try_resolve().unwrap().into();
        }

        #[cfg(feature = "telegram-bot")]
        {
            config.run.run_as_bot = if args.bots.telegram.run_as_bot {
                Some(RunAsBot::Telegram)
            } else {
                None
            };

            match &config.run.run_as_bot {
                Some(RunAsBot::Telegram) if config.bots.telegram.is_none() => {
                    eprintln!("Telegram bot config not set");
                    std::process::exit(1);
                }

                _ => {}
            }
        }

        if let Some(dump_type) = args.app.dump_config {
            match dump_type.unwrap_or(DumpType::Toml) {
                DumpType::Toml => {
                    println!("{}", toml::to_string_pretty(&config).unwrap());
                }

                DumpType::Json => {
                    println!("{}", serde_json::to_string_pretty(&config).unwrap());
                }
            }
            std::process::exit(0);
        }

        config
    }

    fn merge_args(&mut self, args: &CliArgs) -> &Self {
        args.merge_into_config(self);

        self
    }

    fn merge_file_config(&mut self, file_config: &FileConfiguration) -> &Self {
        file_config.merge_into_config(self);

        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub memes_directory: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunAsBot {
    Telegram,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunConfig {
    pub download_url: Option<String>,
    pub fix: bool,
    pub run_as_bot: Option<RunAsBot>,
}

#[derive(Debug, Default, Serialize)]
pub struct Configuration {
    pub args_download_url: Option<String>,
    pub args_fix: bool,
    pub config_path: PathBuf,

    pub yt_dlp_path: PathBuf,
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: PathBuf,
    pub scenedetect_path: Option<PathBuf>,

    pub memes_directory: PathBuf,

    #[cfg(feature = "telegram-bot")]
    pub telegram: Option<common::TelegramBotConfig>,

    pub bots: Option<common::BotConfig>,
}

#[allow(dead_code)]
impl Configuration {
    fn get_project_dir() -> Option<ProjectDirs> {
        ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
    }

    #[must_use]
    pub fn get_config_dir() -> Option<PathBuf> {
        Self::get_project_dir().map(|x| x.config_dir().into())
    }

    #[must_use]
    pub fn config_dir(&self) -> Option<PathBuf> {
        Self::get_config_dir()
    }

    #[must_use]
    pub fn get_cache_dir() -> PathBuf {
        Self::get_project_dir().map_or_else(
            || env::temp_dir().join(APPLICATION_NAME),
            |x| x.cache_dir().into(),
        )
    }

    #[must_use]
    pub fn cache_dir(&self) -> PathBuf {
        Self::get_cache_dir()
    }

    fn from_new(config: Config) -> Self {
        Self {
            args_download_url: config.run.download_url.clone(),
            args_fix: config.run.fix,
            config_path: config.app.config_path.clone(),

            yt_dlp_path: config.dependencies.yt_dlp_path.unwrap_or_default(),
            ffmpeg_path: config.dependencies.ffmpeg_path.unwrap_or_default(),
            ffprobe_path: config.dependencies.ffprobe_path.unwrap_or_default(),
            scenedetect_path: config.dependencies.scenedetect_path,

            memes_directory: config.app.memes_directory,

            bots: Some(config.bots.clone()),

            #[cfg(feature = "telegram-bot")]
            telegram: config.bots.telegram,
        }
    }

    fn new() -> Self {
        let mut config = Self::default();
        let args = CliArgs::parse();
        let file_config = FileConfiguration::new(args.app.config_path.as_deref()).unwrap();

        config.merge_file_config(&file_config);
        config.merge_args(&args);

        {
            if config.yt_dlp_path.as_os_str().is_empty() {
                config.yt_dlp_path = which("yt-dlp")
                    .map_err(|e| anyhow!("yt-dlp not found: {}", e))
                    .unwrap();
            }
        }

        {
            if config.ffmpeg_path.as_os_str().is_empty() {
                config.ffmpeg_path = which("ffmpeg")
                    .map_err(|e| anyhow!("ffmpeg not found: {}", e))
                    .unwrap();
            }
        }

        {
            if config.ffprobe_path.as_os_str().is_empty() {
                config.ffprobe_path = which("ffprobe")
                    .map_err(|e| anyhow!("ffprobe not found: {}", e))
                    .unwrap();
            }
        }

        {
            if let Some(scenedetect_path) = &config.scenedetect_path {
                if scenedetect_path.as_os_str().is_empty() {
                    config.scenedetect_path = which("scenedetect").ok();
                }
            } else {
                config.scenedetect_path = which("scenedetect").ok();
            }
        }

        {
            if config.memes_directory.as_os_str().is_empty() {
                config.memes_directory = directories::UserDirs::new()
                    .unwrap()
                    .home_dir()
                    .join("MEMES");
            }
            config.memes_directory = config.memes_directory.try_resolve().unwrap().into();
        }

        #[cfg(feature = "telegram-bot")]
        {
            if !args.bots.telegram.run_as_bot {
                config.telegram = None;
            } else if config.telegram.is_none() {
                eprintln!("Telegram bot config not set");
                std::process::exit(1);
            }
        }

        if let Some(dump_type) = args.app.dump_config {
            match dump_type.unwrap_or(DumpType::Toml) {
                DumpType::Toml => {
                    println!("{}", toml::to_string_pretty(&config).unwrap());
                }

                DumpType::Json => {
                    println!("{}", serde_json::to_string_pretty(&config).unwrap());
                }
            }
            std::process::exit(0);
        }

        config
    }

    fn merge_args(&mut self, args: &CliArgs) -> &Self {
        args.merge_into_configuration(self);

        self
    }

    fn merge_file_config(&mut self, file_config: &FileConfiguration) -> &Self {
        file_config.merge_into_configuration(self);

        self
    }
}

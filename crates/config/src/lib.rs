#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::missing_errors_doc)]

use anyhow::{anyhow, bail};
use clap::Parser;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
#[cfg(feature = "telegram-bot")]
use std::process::exit;
use std::{env, fs};
use which::which;

pub static APPLICATION_NAME: &str = "meme-downloader";
pub static ORGANIZATION_NAME: &str = "allypost";
pub static ORGANIZATION_QUALIFIER: &str = "net";

lazy_static! {
    pub static ref CONFIGURATION: Configuration = Configuration::new();
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
/// Configuration for the Telegram bot functionality
pub struct TelegramBotConfig {
    /// The Telegram bot token. <https://core.telegram.org/bots/features#creating-a-new-bot>
    pub bot_token: String,

    /// The Telegram user ID of the owner of the bot.
    /// Used to restrict access to the bot or allow additional commands
    /// By default, also saves media sent by the owner to the memes directory
    pub owner_id: Option<u64>,

    /// The Telegram API URL for the bot to use.
    /// Can be used if a Local API server is in use <https://github.com/tdlib/telegram-bot-api>.
    /// Defaults to the standard https://api.telegram.org
    pub api_url: Option<String>,
}

#[derive(Debug, Default)]
pub struct Configuration {
    pub args_download_url: Option<String>,
    pub args_fix: bool,
    pub config_path: PathBuf,

    pub yt_dlp_path: PathBuf,
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: PathBuf,
    pub scenedetect_path: Option<PathBuf>,

    pub memes_directory: PathBuf,

    pub telegram: Option<TelegramBotConfig>,
}

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

    fn new() -> Self {
        let mut config = Self::default();
        let args = Args::parse();
        let file_config = FileConfiguration::new();

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
            if !args.telegram_run_as_bot {
                config.telegram = None;
            } else if config.telegram.is_none() {
                eprintln!("Telegram bot config not set");
                exit(1);
            }
        }

        config
    }

    fn merge_args(&mut self, args: &Args) -> &Self {
        args.merge_into_config(self);

        self
    }

    fn merge_file_config(&mut self, file_config: &FileConfiguration) -> &Self {
        file_config.merge_into_config(self);

        self
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// The URL to download media from
    #[arg(default_value = None)]
    pub download_url: Option<String>,
    /// Just fix the given file, don't download anything
    #[arg(long)]
    pub fix: bool,
    /// Location of the configuration file.
    /// By default shoud be in the os-appropriate config directory
    /// under the name `meme-downloader/config.toml`
    #[arg(short='c', long, default_value = None, env = "MEME_DOWNLOADER_CONFIG")]
    pub config_path: Option<PathBuf>,

    /// Path to the yt-dlp executable.
    /// If not provided, yt-dlp will be searched for in $PATH
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_YT_DLP")]
    pub yt_dlp_path: Option<PathBuf>,
    /// Path to the ffmpeg executable.
    /// If not provided, ffmpeg will be searched for in $PATH
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_FFMPEG")]
    pub ffmpeg_path: Option<PathBuf>,
    /// Path to the ffprobe executable.
    /// If not provided, ffprobe will be searched for in $PATH
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_FFPROBE")]
    pub ffprobe_path: Option<PathBuf>,
    /// Path to the scenedetect executable.
    /// If not provided, scenedetect will be searched for in $PATH
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_SCENEDETECT")]
    pub scenedetect_path: Option<PathBuf>,

    /// The directory to save memes to.
    /// If not provided, `$HOME/MEMES' will be used
    #[arg(short='d', long, default_value = None, env = "MEME_DOWNLOADER_MEMES_DIR")]
    pub memes_directory: Option<PathBuf>,

    /// Run as a Telegram bot.
    /// Requires setting a bot token in the config under the `[telegram] bot_token` key,
    /// setting the `telegram-bot-token` argument,
    /// or by passing it via the `MEME_DOWNLOADER_TELEGRAM_TOKEN` environment variable
    #[cfg(feature = "telegram-bot")]
    #[cfg_attr(feature = "telegram-bot", arg(long = "as-telegram-bot"))]
    pub telegram_run_as_bot: bool,
    /// The telegram bot token. <https://core.telegram.org/bots/features#botfather>
    #[cfg(feature = "telegram-bot")]
    #[cfg_attr(feature = "telegram-bot", arg(long, default_value = None, value_name = "BOT_TOKEN", env = "MEME_DOWNLOADER_TELEGRAM_TOKEN"))]
    pub telegram_bot_token: Option<String>,
    /// The Telegram user ID of the owner of the bot. Used to restrict access to the bot or allow additional commands
    #[cfg(feature = "telegram-bot")]
    #[cfg_attr(feature = "telegram-bot", arg(long, default_value = None, value_name = "OWNER_ID", env = "MEME_DOWNLOADER_TELEGRAM_OWNER_ID"))]
    pub telegram_owner_id: Option<u64>,
    /// The Telegram API URL for the bot to use.
    /// Can be used if a Local API server is in use <https://github.com/tdlib/telegram-bot-api>.
    /// Defaults to the standard https://api.telegram.org
    #[cfg(feature = "telegram-bot")]
    #[cfg_attr(feature = "telegram-bot", arg(long, default_value = None, value_name = "API_URL", env = "MEME_DOWNLOADER_TELEGRAM_API_URL"))]
    pub telegram_api_url: Option<String>,
}

impl Args {
    fn merge_into_config(&self, config: &mut Configuration) {
        if let Some(yt_dlp_path) = &self.yt_dlp_path {
            println!(
                "Found yt-dlp path from arguments: {:?}",
                yt_dlp_path.display()
            );
            config.yt_dlp_path = yt_dlp_path.into();
        }

        if let Some(memes_directory) = &self.memes_directory {
            println!(
                "Found memes directory from arguments: {:?}",
                memes_directory.display()
            );
            config.memes_directory = memes_directory.into();
        }

        #[cfg(feature = "telegram-bot")]
        if self.telegram_run_as_bot {
            if let Some(telegram_bot_token) = &self.telegram_bot_token {
                config.telegram = Some(TelegramBotConfig {
                    bot_token: telegram_bot_token.into(),
                    owner_id: self.telegram_owner_id,
                    api_url: self.telegram_api_url.clone(),
                });
            }

            if let Some(telegram_api_url) = &self.telegram_api_url {
                config.telegram = config.telegram.as_mut().map(|x| {
                    x.api_url = Some(telegram_api_url.clone());
                    x.clone()
                });
            }
        }

        if let Some(config_path) = &self.config_path {
            config.config_path = config_path.into();
        }

        config.args_download_url = self.download_url.clone();
        config.args_fix = self.fix;
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FileConfiguration {
    /// Path to the yt-dlp executable
    /// If not provided, yt-dlp will be searched for in $PATH
    pub yt_dlp_path: Option<PathBuf>,

    /// Path to the ffmpeg executable
    /// If not provided, ffmpeg will be searched for in $PATH
    pub ffmpeg_path: Option<PathBuf>,

    /// Path to the ffprobe executable
    /// If not provided, ffprobe will be searched for in $PATH
    pub ffprobe_path: Option<PathBuf>,

    /// Path to the scenedetect executable
    /// If not provided, scenedetect will be searched for in $PATH
    pub scenedetect_path: Option<PathBuf>,

    /// The directory to save memes to.
    /// If not provided, $HOME/MEMES will be used
    pub memes_directory: Option<PathBuf>,

    pub telegram: Option<TelegramBotConfig>,
}

impl FileConfiguration {
    fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::unused_self)]
    fn merge_into_config(&self, config: &mut Configuration) {
        if Self::is_default_config_path(&config.config_path) {
            config.config_path = Self::create_default_config_file().unwrap();
        }

        let file_config = Self::load_from_file(&config.config_path).unwrap();

        if let Some(yt_dlp_path) = file_config.yt_dlp_path {
            println!("Found yt-dlp path from config file: {yt_dlp_path:?}");
            config.yt_dlp_path = yt_dlp_path;
        }

        if let Some(ffmpeg_path) = file_config.ffmpeg_path {
            println!("Found ffmpeg path from config file: {ffmpeg_path:?}");
            config.ffmpeg_path = ffmpeg_path;
        }

        if let Some(ffprobe_path) = file_config.ffprobe_path {
            println!("Found ffprobe path from config file: {ffprobe_path:?}");
            config.ffprobe_path = ffprobe_path;
        }

        if let Some(scenedetect_path) = file_config.scenedetect_path {
            println!("Found scenedetect path from config file: {scenedetect_path:?}");
            config.scenedetect_path = Some(scenedetect_path);
        }

        if let Some(memes_directory) = file_config.memes_directory {
            println!("Found memes directory from config file: {memes_directory:?}");
            config.memes_directory = memes_directory;
        }

        if let Some(telegram) = file_config.telegram {
            println!("Found telegram config from config file: {telegram:?}");
            config.telegram = Some(telegram);
        }
    }

    fn load_from_file<P>(path: P) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let p = path.as_ref();

        if !p.is_file() {
            eprintln!("Config file {:?} does not exist or is not a file", &p);
            return None;
        }

        let config_file = fs::read_to_string(p).unwrap();
        match toml::from_str(&config_file) {
            Ok(config) => {
                println!("Parsed config file successfully: {config:?}");
                config
            }
            Err(e) => {
                eprintln!("Error parsing config file: {e:?}");
                None
            }
        }
    }

    fn create_default_config_file() -> anyhow::Result<PathBuf> {
        let file = Self::default_config_path().ok_or_else(|| {
            anyhow!(
                "Failed to get config directory. \
                Please set the MEME_DOWNLOADER_CONFIG_DIR environment variable to a valid directory"
            )
        })?;

        let config_dir: PathBuf = file
            .parent()
            .ok_or_else(|| {
                anyhow!(
                    "Failed to get parent directory of config file. Is the config file in root?"
                )
            })?
            .into();

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        if !file.exists() {
            println!("Config file not found. Creating one at {file:?}");
            let mut f = fs::File::create(&file);
            let res: Result<_, _> = f
                .as_mut()
                .map(|f| f.write_all(include_bytes!("./config.toml")));

            if let Err(e) = res {
                eprintln!("Failed to create config file: {e}");
                bail!("Failed to create config file: {}", e);
            }
        }

        Ok(file)
    }

    fn default_config_path() -> Option<PathBuf> {
        Configuration::get_config_dir().map(|x| x.join("config.toml"))
    }

    fn is_default_config_path<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let p = path.as_ref().as_os_str();

        p.is_empty() || p == Self::default_config_path().unwrap().as_os_str()
    }
}

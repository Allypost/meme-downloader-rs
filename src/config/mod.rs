use anyhow::{anyhow, bail};
use clap::Parser;
use lazy_static::lazy_static;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
#[cfg(feature = "telegram-bot")]
use std::process::exit;
use which::which;

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
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    yt_dlp_path: Option<PathBuf>,
    ffmpeg_path: Option<PathBuf>,
    ffprobe_path: Option<PathBuf>,
    memes_directory: Option<PathBuf>,

    pub telegram: Option<TelegramBotConfig>,
}

#[derive(Debug, Default)]
pub struct Configuration {
    pub args_download_url: Option<String>,
    pub args_fix: bool,
    pub config_path: PathBuf,

    pub yt_dlp_path: PathBuf,
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: PathBuf,

    pub memes_directory: PathBuf,

    pub telegram: Option<TelegramBotConfig>,
}

impl Configuration {
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
            if config.memes_directory.as_os_str().is_empty() {
                config.memes_directory = dirs::home_dir().unwrap().join("MEMES");
            }
            config.memes_directory = config.memes_directory.try_resolve().unwrap().into();
        }

        #[cfg(feature = "telegram-bot")]
        {
            if args.telegram_run_as_bot == false {
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
            eprintln!("Config file {:#?} does not exist or is not a file", &p);
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
        let config_dir: PathBuf = dirs::config_dir().ok_or_else(|| {
            anyhow!(
                "Failed to get config directory. \
                Please set the MEME_DOWNLOADER_CONFIG_DIR environment variable to a valid directory"
            )
        })?;
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let file = config_dir.join("meme-downloader").join("config.toml");

        if !file.exists() {
            println!("Config file not found. Creating one at {file:#?}");
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

    fn default_config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap();

        config_dir.join("meme-downloader").join("config.toml")
    }

    fn is_default_config_path<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let p = path.as_ref().as_os_str();

        p.is_empty() || p == Self::default_config_path().as_os_str()
    }
}

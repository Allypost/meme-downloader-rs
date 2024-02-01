use std::path::PathBuf;

use clap::{Args, ValueHint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
pub struct AppConfig {
    #[arg(short='d', long, default_value = None, env = "MEME_DOWNLOADER_MEMES_DIR", value_hint = ValueHint::DirPath)]
    /// The directory to save memes to.
    ///
    /// If not provided, `$HOME/MEMES' will be used
    pub memes_directory: Option<PathBuf>,
}
impl AppConfig {
    pub(crate) fn merge(&mut self, config: &Self) -> &Self {
        if let Some(memes_directory) = config.memes_directory.as_ref() {
            self.memes_directory = Some(memes_directory.clone());
        }

        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
pub struct BotConfig {
    #[cfg(feature = "telegram-bot")]
    #[command(flatten, next_help_heading = Some("Telegram bot config"))]
    pub telegram: Option<TelegramBotConfig>,
}
impl BotConfig {
    pub(crate) fn merge(&mut self, config: &Self) -> &Self {
        #[cfg(feature = "telegram-bot")]
        {
            if let Some(telegram) = config.telegram.as_ref() {
                let merged = self.telegram.as_mut().map_or_else(
                    || telegram.clone(),
                    |x| {
                        x.merge(telegram);
                        x.clone()
                    },
                );

                self.telegram = Some(merged);
            }
        }

        self
    }
}

#[cfg(feature = "telegram-bot")]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
/// Configuration for the Telegram bot functionality
pub struct TelegramBotConfig {
    #[arg(long = "telegram-bot-token", default_value = None, value_name = "BOT_TOKEN", env = "MEME_DOWNLOADER_TELEGRAM_TOKEN", value_hint = ValueHint::Other)]
    /// The telegram bot token.
    ///
    /// See API docs for more info: <https://core.telegram.org/bots/features#botfather>
    pub bot_token: Option<String>,

    #[arg(long = "telegram-owner-id", default_value = None, value_name = "OWNER_ID", env = "MEME_DOWNLOADER_TELEGRAM_OWNER_ID", value_hint = ValueHint::Other)]
    /// The Telegram user ID of the owner of the bot.
    ///
    /// Used to restrict access to the bot or allow additional commands
    /// By default, also saves media sent by the owner to the memes directory
    pub owner_id: Option<u64>,

    #[arg(long = "telegram-api-url", default_value = None, value_name = "API_URL", env = "MEME_DOWNLOADER_TELEGRAM_API_URL", value_hint = ValueHint::Url)]
    /// The Telegram API URL for the bot to use.
    ///
    /// Can be used if a Local API server is in use <https://github.com/tdlib/telegram-bot-api>.
    /// Defaults to the standard https://api.telegram.org
    pub api_url: Option<String>,
}
#[cfg(feature = "telegram-bot")]
impl TelegramBotConfig {
    pub(crate) fn merge(&mut self, config: &Self) -> &Self {
        if let Some(bot_token) = config.bot_token.as_ref() {
            self.bot_token = Some(bot_token.clone());
        }

        if let Some(owner_id) = config.owner_id.as_ref() {
            self.owner_id = Some(*owner_id);
        }

        if let Some(api_url) = config.api_url.as_ref() {
            self.api_url = Some(api_url.clone());
        }

        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
#[allow(clippy::struct_field_names)]
pub struct ProgramPathConfig {
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_YT_DLP", value_hint = ValueHint::FilePath)]
    /// Path to the yt-dlp executable.
    ///
    /// If not provided, yt-dlp will be searched for in $PATH
    pub yt_dlp_path: Option<PathBuf>,

    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_FFMPEG", value_hint = ValueHint::FilePath)]
    /// Path to the ffmpeg executable.
    ///
    /// If not provided, ffmpeg will be searched for in $PATH
    pub ffmpeg_path: Option<PathBuf>,

    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_FFPROBE", value_hint = ValueHint::FilePath)]
    /// Path to the ffprobe executable.
    ///
    /// If not provided, ffprobe will be searched for in $PATH
    pub ffprobe_path: Option<PathBuf>,

    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_SCENEDETECT", value_hint = ValueHint::FilePath)]
    /// Path to the scenedetect executable.
    ///
    /// If not provided, scenedetect will be searched for in $PATH
    pub scenedetect_path: Option<PathBuf>,
}
impl ProgramPathConfig {
    pub(crate) fn merge(&mut self, config: &Self) -> &Self {
        if let Some(yt_dlp_path) = config.yt_dlp_path.as_ref() {
            self.yt_dlp_path = Some(yt_dlp_path.clone());
        }

        if let Some(ffmpeg_path) = config.ffmpeg_path.as_ref() {
            self.ffmpeg_path = Some(ffmpeg_path.clone());
        }

        if let Some(ffprobe_path) = config.ffprobe_path.as_ref() {
            self.ffprobe_path = Some(ffprobe_path.clone());
        }

        if let Some(scenedetect_path) = config.scenedetect_path.as_ref() {
            self.scenedetect_path = Some(scenedetect_path.clone());
        }

        self
    }
}

const DEFAULT_TWITTER_SCREENSHOT_BASE_URL: &str = "https://twitter.igr.ec";

#[derive(Debug, Clone, Default, Serialize, Deserialize, Args)]
pub struct EndpointConfig {
    #[arg(long, default_value = None, env = "MEME_DOWNLOADER_ENDPOINT_TWITTER_SCREENSHOT", value_hint = ValueHint::Url)]
    /// The base URL for the Twitter screenshot API.
    pub(crate) twitter_screenshot_base_url: Option<String>,
}

impl EndpointConfig {
    pub(crate) fn merge(&mut self, config: &Self) -> &Self {
        if let Some(twitter_screenshot_base_url) = config.twitter_screenshot_base_url.as_ref() {
            self.twitter_screenshot_base_url = Some(twitter_screenshot_base_url.clone());
        }

        self
    }

    pub fn twitter_screenshot_base_url(&self) -> String {
        self.twitter_screenshot_base_url
            .as_ref()
            .cloned()
            .unwrap_or_else(|| DEFAULT_TWITTER_SCREENSHOT_BASE_URL.to_string())
    }
}

use clap::Parser;
use once_cell::sync::Lazy;
use std::path;

#[derive(Debug, Clone, Parser)]
/// Media downloader and "fixer"
///
/// Downloads media from various websites.
/// Transcodes and crops files if necessary.
pub struct Args {
    /// Path to the yt-dlp executable
    #[arg(long, default_value = None)]
    pub yt_dlp_path: Option<path::PathBuf>,

    /// The directory to save memes to. If not provided, $HOME/MEMES will be used
    #[arg(short='d', long, default_value = None)]
    pub memes_directory: Option<path::PathBuf>,

    /// The URL to download media from
    #[arg(default_value = None)]
    pub download_url: Option<String>,

    /// Just fix the given file, don't download anything
    #[arg(long)]
    pub fix: bool,

    /// Run as a Telegram bot.
    #[cfg(feature = "telegram-bot")]
    /// Requires setting a bot token in the config under the `[telegram] bot_token` key,
    /// setting the `telegram-bot-token` argument,
    /// or by passing it via the `MEME_DOWNLOADER_TELEGRAM_TOKEN` environment variable
    #[cfg_attr(feature = "telegram-bot", arg(long = "as-telegram-bot"))]
    pub telegram_run_as_bot: bool,

    #[cfg(feature = "telegram-bot")]
    /// The telegram bot token. <https://core.telegram.org/bots/features#botfather>
    #[cfg_attr(feature = "telegram-bot", arg(long, default_value = None, value_name = "BOT_TOKEN"))]
    pub telegram_bot_token: Option<String>,

    #[cfg(feature = "telegram-bot")]
    /// The Telegram user ID of the owner of the bot. Used to restrict access to the bot or allow additional commands
    #[cfg_attr(feature = "telegram-bot", arg(long, default_value = None, value_name = "OWNER_ID"))]
    pub telegram_owner_id: Option<u64>,
}

pub static ARGS: Lazy<Args> = Lazy::new(Args::parse);

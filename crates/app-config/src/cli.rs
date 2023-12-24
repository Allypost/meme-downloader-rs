use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum, ValueHint};
use serde::{Deserialize, Serialize};

use crate::{
    common::{AppConfig, EndpointConfig, ProgramPathConfig},
    Config, Configuration,
};

#[derive(Debug, Clone, Serialize, Deserialize, Parser)]
pub struct CliArgs {
    #[command(flatten)]
    pub app: AppArgs,

    #[command(flatten, next_help_heading = Some("Program paths"))]
    pub paths: ProgramPathConfig,

    #[cfg(feature = "telegram-bot")]
    #[command(flatten, next_help_heading = Some("Bot config"))]
    pub bots: self::bot::BotArgs,

    #[command(flatten, next_help_heading = Some("Endpoint config"))]
    pub endpoints: EndpointConfig,
}

impl CliArgs {
    pub(crate) fn merge_into_configuration(&self, config: &mut Configuration) {
        if let Some(yt_dlp_path) = &self.paths.yt_dlp_path {
            eprintln!(
                "Found yt-dlp path from arguments: {:?}",
                yt_dlp_path.display()
            );
            config.yt_dlp_path = yt_dlp_path.into();
        }

        if let Some(app_config) = &self.app.app_config {
            if let Some(memes_directory) = &app_config.memes_directory {
                eprintln!(
                    "Found memes directory from arguments: {:?}",
                    memes_directory.display()
                );
                config.memes_directory = memes_directory.into();
            }
        }

        #[cfg(feature = "telegram-bot")]
        {
            if self.bots.telegram.run_as_bot {
                if let Some(telegram_config) = self.bots.telegram.config.as_ref() {
                    if let Some(telegram_bot_token) = telegram_config.bot_token.as_deref() {
                        config.telegram = Some(crate::common::TelegramBotConfig {
                            bot_token: Some(telegram_bot_token.to_owned()),
                            owner_id: telegram_config.owner_id,
                            api_url: telegram_config.api_url.clone(),
                        });
                    }

                    if let Some(telegram_api_url) = telegram_config.api_url.as_deref() {
                        config.telegram = config.telegram.as_mut().map(|x| {
                            x.api_url = Some(telegram_api_url.to_owned());
                            x.clone()
                        });
                    }
                }
            }
        }

        if let Some(config_path) = &self.app.config_path {
            config.config_path = config_path.into();
        }

        config.args_download_url = self.app.download_url.clone();
        config.args_fix = self.app.fix;
        config.endpoints.merge(&self.endpoints);
    }

    pub(crate) fn merge_into_config(&self, config: &mut Config) {
        if let Some(yt_dlp_path) = &self.paths.yt_dlp_path {
            eprintln!(
                "Found yt-dlp path from arguments: {:?}",
                yt_dlp_path.display()
            );
            config.dependencies.yt_dlp_path = Some(yt_dlp_path.into());
        }

        if let Some(app_config) = &self.app.app_config {
            if let Some(memes_directory) = &app_config.memes_directory {
                eprintln!(
                    "Found memes directory from arguments: {:?}",
                    memes_directory.display()
                );
                config.app.memes_directory = memes_directory.into();
            }
        }

        #[cfg(feature = "telegram-bot")]
        {
            use self::bot::{BotArgs, TelegramBotArgs};
            match &self.bots {
                #[cfg(feature = "telegram-bot")]
                BotArgs {
                    telegram:
                        TelegramBotArgs {
                            run_as_bot: true,
                            config: Some(telegram_config),
                        },
                } => {
                    if let Some(telegram_bot_token) = &telegram_config.bot_token {
                        config.bots.telegram = Some(crate::common::TelegramBotConfig {
                            bot_token: Some(telegram_bot_token.into()),
                            owner_id: telegram_config.owner_id,
                            api_url: telegram_config.api_url.clone(),
                        });
                    }

                    if let Some(telegram_api_url) = &telegram_config.api_url {
                        config.bots.telegram = config.bots.telegram.as_mut().map(|x| {
                            x.api_url = Some(telegram_api_url.into());
                            x.clone()
                        });
                    }
                }

                _ => {}
            }
        }

        if let Some(config_path) = &self.app.config_path {
            config.app.config_path = config_path.into();
        }

        config.run.download_url = self.app.download_url.clone();
        config.run.fix = self.app.fix;
        config.endpoints.merge(&self.endpoints);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum DumpType {
    Toml,
    Json,
}
#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct AppArgs {
    #[arg(default_value = None, value_hint = ValueHint::Url)]
    /// The URL to download media from.
    pub download_url: Option<String>,

    #[arg(short, long)]
    /// Just fix the given file, don't download anything.
    pub fix: bool,

    #[arg(short='c', long, default_value = None, env = "MEME_DOWNLOADER_CONFIG", value_hint = ValueHint::FilePath)]
    /// Location of the configuration file.
    ///
    /// By default shoud be in the os-appropriate config directory
    /// under the name `meme-downloader/config.toml`
    pub config_path: Option<PathBuf>,

    #[arg(long, ignore_case = true, value_name = "FORMAT")]
    /// Dump the configuration to stderr and exit.
    ///
    /// Useful for debugging.
    /// When dumped with the `toml` format, can be used as a config file.
    #[allow(clippy::option_option)]
    pub dump_config: Option<Option<DumpType>>,

    #[command(flatten)]
    pub app_config: Option<AppConfig>,
}

#[cfg(feature = "telegram-bot")]
mod bot {
    use clap::Args;
    use serde::{Deserialize, Serialize};

    use crate::common::TelegramBotConfig;

    #[derive(Debug, Clone, Serialize, Deserialize, Args)]
    pub struct BotArgs {
        #[cfg(feature = "telegram-bot")]
        #[command(flatten, next_help_heading = Some("Telegram bot config"))]
        pub telegram: TelegramBotArgs,
    }

    #[cfg(feature = "telegram-bot")]
    #[derive(Debug, Clone, Serialize, Deserialize, Args)]
    pub struct TelegramBotArgs {
        #[arg(long = "as-telegram-bot")]
        /// Run as a Telegram bot.
        ///
        /// Requires setting a bot token in the config under the `[telegram] bot_token` key,
        /// setting the `telegram-bot-token` argument,
        /// or by passing it via the `MEME_DOWNLOADER_TELEGRAM_TOKEN` environment variable
        pub run_as_bot: bool,

        #[command(flatten)]
        pub config: Option<TelegramBotConfig>,
    }
}

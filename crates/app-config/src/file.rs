use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::{
    common::{AppConfig, BotConfig, EndpointConfig, ProgramPathConfig},
    Config, Configuration,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OldFileConfiguration {
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

    #[cfg(feature = "telegram-bot")]
    pub telegram: Option<crate::common::TelegramBotConfig>,
}
impl From<OldFileConfiguration> for FileConfiguration {
    fn from(val: OldFileConfiguration) -> Self {
        Self {
            app: val.memes_directory.map(|x| AppConfig {
                memes_directory: Some(x),
            }),
            dependencies: Some(ProgramPathConfig {
                yt_dlp_path: val.yt_dlp_path,
                ffmpeg_path: val.ffmpeg_path,
                ffprobe_path: val.ffprobe_path,
                scenedetect_path: val.scenedetect_path,
            }),
            bots: Some(BotConfig {
                #[cfg(feature = "telegram-bot")]
                telegram: val.telegram,
            }),
            endpoints: None,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FileConfiguration {
    pub app: Option<AppConfig>,

    pub dependencies: Option<ProgramPathConfig>,

    pub bots: Option<BotConfig>,

    pub endpoints: Option<EndpointConfig>,
}

impl FileConfiguration {
    pub(crate) fn new(config_path: Option<&Path>) -> anyhow::Result<Self> {
        let config_path = if let Some(config_path) = &config_path {
            config_path.into()
        } else {
            Self::create_default_config_file()?
        };

        let config_path = if Self::is_default_config_path(&config_path) {
            Self::create_default_config_file()?
        } else {
            config_path
        };

        Self::load_from_file(config_path)
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn merge_into_configuration(&self, config: &mut Configuration) {
        if Self::is_default_config_path(&config.config_path) {
            config.config_path =
                Self::create_default_config_file().expect("Failed to create config file");
        }

        if let Some(dependencies) = &self.dependencies {
            if let Some(yt_dlp_path) = &dependencies.yt_dlp_path {
                eprintln!("Found yt-dlp path from config file: {yt_dlp_path:?}");
                config.yt_dlp_path = yt_dlp_path.into();
            }

            if let Some(ffmpeg_path) = &dependencies.ffmpeg_path {
                eprintln!("Found ffmpeg path from config file: {ffmpeg_path:?}");
                config.ffmpeg_path = ffmpeg_path.into();
            }

            if let Some(ffprobe_path) = &dependencies.ffprobe_path {
                eprintln!("Found ffprobe path from config file: {ffprobe_path:?}");
                config.ffprobe_path = ffprobe_path.into();
            }

            if let Some(scenedetect_path) = &dependencies.scenedetect_path {
                eprintln!("Found scenedetect path from config file: {scenedetect_path:?}");
                config.scenedetect_path = Some(scenedetect_path.into());
            }
        }

        if let Some(app) = &self.app {
            if let Some(memes_directory) = &app.memes_directory {
                eprintln!("Found memes directory from config file: {memes_directory:?}");
                config.memes_directory = memes_directory.into();
            }
        }

        if let Some(endpoints) = &self.endpoints {
            config.endpoints.merge(endpoints);
        }

        #[cfg(feature = "telegram-bot")]
        {
            if let Some(bots) = &self.bots {
                if let Some(telegram) = &bots.telegram {
                    eprintln!("Found telegram config from config file: {telegram:?}");
                    config.telegram = Some(telegram.clone());
                }
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn merge_into_config(&self, config: &mut Config) {
        if Self::is_default_config_path(&config.app.config_path) {
            config.app.config_path =
                Self::create_default_config_file().expect("Failed to create config file");
        }

        if let Some(dependencies) = &self.dependencies {
            if let Some(yt_dlp_path) = &dependencies.yt_dlp_path {
                eprintln!("Found yt-dlp path from config file: {yt_dlp_path:?}");
                config.dependencies.yt_dlp_path = Some(yt_dlp_path.into());
            }

            if let Some(ffmpeg_path) = &dependencies.ffmpeg_path {
                eprintln!("Found ffmpeg path from config file: {ffmpeg_path:?}");
                config.dependencies.ffmpeg_path = Some(ffmpeg_path.into());
            }

            if let Some(ffprobe_path) = &dependencies.ffprobe_path {
                eprintln!("Found ffprobe path from config file: {ffprobe_path:?}");
                config.dependencies.ffprobe_path = Some(ffprobe_path.into());
            }

            if let Some(scenedetect_path) = &dependencies.scenedetect_path {
                eprintln!("Found scenedetect path from config file: {scenedetect_path:?}");
                config.dependencies.scenedetect_path = Some(scenedetect_path.into());
            }
        }

        if let Some(app) = &self.app {
            if let Some(memes_directory) = &app.memes_directory {
                eprintln!("Found memes directory from config file: {memes_directory:?}");
                config.app.memes_directory = memes_directory.into();
            }
        }

        if let Some(endpoints) = &self.endpoints {
            config.endpoints.merge(endpoints);
        }

        #[cfg(feature = "telegram-bot")]
        {
            if let Some(bots) = &self.bots {
                if let Some(telegram) = &bots.telegram {
                    eprintln!("Found telegram config from config file: {telegram:?}");
                    config.bots.telegram = Some(telegram.clone());
                }
            }
        }
    }

    fn merge(self, other: Self) -> Self {
        let other_app = other.app.unwrap_or_default();
        let app = self
            .app
            .map(|mut app| {
                app.merge(&other_app);

                app.clone()
            })
            .or(Some(other_app));

        let other_dependencies = other.dependencies.unwrap_or_default();
        let dependencies = self
            .dependencies
            .map(|mut dependencies| {
                dependencies.merge(&other_dependencies);

                dependencies.clone()
            })
            .or(Some(other_dependencies));

        let other_bots = other.bots.unwrap_or_default();
        let bots = self
            .bots
            .map(|mut bots| {
                bots.merge(&other_bots);

                bots.clone()
            })
            .or(Some(other_bots));

        let other_endpoint = other.endpoints.unwrap_or_default();
        let endpoints = self
            .endpoints
            .map(|mut endpoints| {
                endpoints.merge(&other_endpoint);

                endpoints.clone()
            })
            .or(Some(other_endpoint));

        Self {
            app,
            dependencies,
            bots,
            endpoints,
        }
    }

    fn load_from_file<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let p = path.as_ref();

        if !p.is_file() {
            anyhow::bail!("Config file {:?} does not exist or is not a file", &p);
        }

        let config = {
            let config_file = fs::read_to_string(p).expect("Failed to read config file");
            let old_config =
                toml::from_str::<OldFileConfiguration>(&config_file).unwrap_or_default();
            let new_config = match toml::from_str::<Self>(&config_file) {
                Ok(config) => config,
                Err(e) => {
                    anyhow::bail!("Error parsing config file: {e}", e = e);
                }
            };

            Self::default().merge(old_config.into()).merge(new_config)
        };

        Ok(config)
    }

    fn create_default_config_file() -> anyhow::Result<PathBuf> {
        let file = Self::default_config_path().ok_or_else(|| {
            anyhow!(
                "Failed to get config directory. Please set the MEME_DOWNLOADER_CONFIG_DIR \
                 environment variable to a valid directory"
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
            eprintln!("Config file not found. Creating one at {file:?}");
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

        p.is_empty()
            || p == Self::default_config_path()
                .expect("MEME_DOWNLOADER_CONFIG_DIR is not set")
                .as_os_str()
    }
}

use crate::args;
use log::debug;
use once_cell::sync::Lazy;
use resolve_path::PathResolveExt;
use std::io::{self, prelude::*};
use std::path::PathBuf;
use std::{fs, path};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub yt_dlp_path: Option<path::PathBuf>,
    pub memes_directory: Option<path::PathBuf>,
}

impl Config {
    fn parse() -> Self {
        let mut config: Self = Self::default();
        let config_dir = dirs::config_dir().unwrap();
        let config_file = config_dir.join("meme-downloader").join("config.toml");
        if config_file.exists() {
            let config_file = fs::read_to_string(config_file).unwrap();
            config = toml::from_str(&config_file).unwrap();
        } else {
            debug!("Config file not found. Creating one at {:#?}", config_file);
            let config_dir = config_file.parent().unwrap();
            fs::create_dir_all(config_dir).unwrap();
            let mut config_file = fs::File::create(config_file).unwrap();
            let config = toml::to_string_pretty(&config).unwrap();
            config_file.write_all(config.as_bytes()).unwrap();
        }

        config
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
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);

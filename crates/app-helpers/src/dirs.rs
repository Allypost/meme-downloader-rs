use std::{fs, path::PathBuf};

use anyhow::anyhow;
use app_config::CONFIGURATION;

use crate::id::time_thread_id;

pub fn create_temp_dir() -> anyhow::Result<PathBuf> {
    let id = time_thread_id().map_err(|e| anyhow!("Error while getting time: {e:?}"))?;
    let temp_dir = CONFIGURATION.cache_dir().join(id);

    fs::create_dir_all(&temp_dir)?;

    Ok(temp_dir)
}

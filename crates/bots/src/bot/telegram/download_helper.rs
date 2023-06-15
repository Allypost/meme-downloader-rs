use config::CONFIGURATION;
use helpers::dirs::create_temp_dir;
use log::trace;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DownloadResult {
    download_dir: PathBuf,
    files: Vec<PathBuf>,
}

impl DownloadResult {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    pub fn cleanup(&self) -> Result<(), String> {
        fs::remove_dir_all(&self.download_dir)
            .map_err(|e| format!("Error while removing download dir: {e:?}"))?;

        Ok(())
    }

    pub fn move_files_to_memes_dir(&self) -> Result<Vec<PathBuf>, String> {
        let memes_dir = &CONFIGURATION.memes_directory;

        self.files
            .par_iter()
            .map(|file_path| {
                let name = file_path.file_name().unwrap();
                let new_file_path = memes_dir.join(name);

                fs::copy(file_path, &new_file_path)
                    .map_err(|e| format!("Error while copying file: {e:?}"))?;

                fs::remove_file(file_path)
                    .map_err(|e| format!("Error while removing file: {e:?}"))?;

                Ok(new_file_path)
            })
            .collect::<Result<Vec<_>, String>>()
    }
}

pub fn download_tmp_file(url: &str) -> Result<DownloadResult, String> {
    let download_dir =
        create_temp_dir().map_err(|e| format!("Error while getting temp dir: {e:?}"))?;
    trace!("Downloading to temp dir: {:#?}", &download_dir);
    let files = downloader::download_file(url, &download_dir)?;

    Ok(DownloadResult {
        download_dir,
        files,
    })
}

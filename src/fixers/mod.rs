use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub mod crop;
pub mod file_extensions;
pub mod file_name;
pub mod video_formats;

pub fn fix_files(paths: &Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    paths
        .par_iter()
        .map(|path| {
            let path = file_extensions::fix_file_extension(path)?;
            let path = file_name::fix_file_name(&path)?;
            let path = video_formats::convert_file_into_known(&path)?;
            let path = crop::auto_crop_video(&path)?;
            Ok(path)
        })
        .collect()
}

type FixerReturn = Result<PathBuf, String>;

use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub mod crop;
pub mod file_extensions;
pub mod file_name;
pub mod video_formats;

pub fn fix_files(paths: &Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    let fixers: Vec<Fixer> = vec![
        file_extensions::fix_file_extension,
        file_name::fix_file_name,
        video_formats::convert_file_into_known,
        crop::auto_crop_video,
    ];

    paths
        .par_iter()
        .map(|path| {
            let mut p = path.clone();
            for filter in &fixers {
                p = filter(&p)?;
            }
            Ok(p)
        })
        .collect()
}

type FixerReturn = Result<PathBuf, String>;
type Fixer = fn(&PathBuf) -> FixerReturn;

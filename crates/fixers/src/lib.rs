#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::missing_errors_doc)]

use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub mod crop;
pub mod file_extensions;
pub mod file_name;
pub mod image_formats;
pub mod split_scenes;
pub mod video_formats;

pub fn fix_files(paths: &Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    let fixers: Vec<Fixer> = vec![
        file_extensions::fix_file_extension,
        file_name::fix_file_name,
        video_formats::convert_file_into_known,
        image_formats::convert_file_into_preferred,
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

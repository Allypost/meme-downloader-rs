#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::single_match_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::uninlined_format_args)]

#[macro_use(defer)]
extern crate scopeguard;

use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub mod crop;
pub mod file_extensions;
pub mod file_name;
pub mod media_formats;
pub mod split_scenes;

pub fn fix_files(paths: &Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    let fixers: Vec<Fixer> = vec![
        file_extensions::fix_file_extension,
        file_name::fix_file_name,
        media_formats::convert_into_preferred_formats,
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

#[macro_use(defer)]
extern crate scopeguard;

use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use resolve_path::PathResolveExt;

pub mod crop;
pub mod file_extensions;
pub mod file_name;
pub mod media_formats;
pub mod split_scenes;
mod util;

pub fn fix_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let fixers: Vec<Fixer> = vec![
        file_extensions::fix_file_extension,
        file_name::fix_file_name,
        media_formats::convert_into_preferred_formats,
        crop::auto_crop_video,
    ];

    paths
        .par_iter()
        .map(|path| {
            let mut p = path.resolve().canonicalize().map_err(|e| {
                format!("Failed to canonicalize {path:?}: {e:?}", path = path, e = e)
            })?;
            for filter in &fixers {
                p = filter(&p)?;
            }
            Ok(p)
        })
        .collect()
}

type FixerReturn = Result<PathBuf, String>;
type Fixer = fn(&PathBuf) -> FixerReturn;

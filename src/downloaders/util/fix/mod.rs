use super::results::check_results;
use std::path::PathBuf;

mod crop;
mod file_extensions;
mod video_formats;

#[allow(clippy::module_name_repetitions)]
pub fn fix_files(paths: &mut [PathBuf]) -> Result<Vec<PathBuf>, String> {
    let paths = check_results(file_extensions::fix_file_extensions(paths))?;
    let paths = check_results(video_formats::convert_files_into_known(&paths))?;
    let paths = check_results(crop::auto_crop_videos(&paths))?;

    Ok(paths)
}

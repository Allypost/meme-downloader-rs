use std::fmt::Display;
use std::{fmt::Debug, path::PathBuf};

pub mod file_extensions;
pub mod video_formats;

#[allow(clippy::module_name_repetitions)]
pub fn fix_files(paths: &mut [PathBuf]) -> Result<Vec<PathBuf>, String> {
    let paths = check_results(file_extensions::fix_file_extensions(paths))?;
    let paths = check_results(video_formats::convert_files_into_known(&paths))?;

    Ok(paths)
}

#[allow(clippy::clone_double_ref)]
fn check_results<TVal: Debug, TErr: Display>(
    result: Vec<Result<TVal, TErr>>,
) -> Result<Vec<TVal>, String> {
    if result.iter().any(Result::is_err) {
        let mapped = result.iter().filter(|x| x.is_err()).map(|x| {
            return x.as_ref().unwrap_err().clone();
        });

        let mut ret = vec![];
        for r in mapped {
            ret.push(r.to_string());
        }

        return Err(ret.join(", "));
    }

    let mut ret = vec![];
    for r in result.into_iter().flatten() {
        ret.push(r);
    }

    Ok(ret)
}

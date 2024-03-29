use std::path::{Path, PathBuf};

use app_logger::trace;
use filetime::FileTime;

pub fn transfer_file_times(path_from: &PathBuf, path_to: &PathBuf) -> Result<(), String> {
    trace!(
        "Getting file times of {from:?} and setting them on {to:?}",
        from = path_from,
        to = path_to,
    );

    let old_meta = path_from.metadata().map_err(|e| {
        format!(
            "Failed to get metadata of {path:?}: {e:?}",
            path = path_from,
        )
    })?;

    trace!("Setting file times of {path:?}", path = path_to);
    filetime::set_file_times(
        path_to,
        FileTime::from_last_access_time(&old_meta),
        FileTime::from_last_modification_time(&old_meta),
    )
    .map_err(|e| {
        format!(
            "Failed to set file times of {path:?}: {e:?}",
            path = path_from,
        )
    })
}

pub fn transferable_file_times(
    path_from: PathBuf,
) -> Result<impl FnOnce(&Path) -> Result<(), String>, String> {
    trace!("Getting file times of {path:?}", path = path_from);

    let old_meta = path_from
        .metadata()
        .map_err(|e| format!("Failed to get metadata of {old:?}: {e:?}", old = path_from))?;

    Ok(move |path_to: &Path| {
        trace!("Setting file times of {new:?}", new = path_from);
        filetime::set_file_times(
            path_to,
            FileTime::from_last_access_time(&old_meta),
            FileTime::from_last_modification_time(&old_meta),
        )
        .map_err(|e| {
            format!(
                "Failed to set file times of {new:?}: {e:?}",
                new = path_from
            )
        })
    })
}

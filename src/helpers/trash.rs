use log::debug;
use std::{fs, io, path::PathBuf};

pub fn move_to_trash(f: &PathBuf) -> Result<(), io::Error> {
    debug!("Sending {f:?} into trash");

    trash::delete(f)
        .or_else(|e| {
            debug!("Failed to put {f:?} into trash: {e:?}");
            debug!("Deleting old file {f:?}");
            fs::remove_file(f)
        })
        .map_err(|e| {
            debug!("Failed to delete old file {f:?}: {e:?}");
            e
        })
}

use crate::file::audiofile::AudioFile;
use crate::file::mp3::MP3;
use crate::file::ogg::OGG;
use anyhow::Result;
use std::convert::TryFrom;
use std::path::Path;
use log::{info, debug};


pub fn get_audiofiles(
    dir: &Path,
    depth: u64,
) -> Result<Vec<Box<dyn AudioFile>>> {
    let audiofiles = _get_audiofiles(dir, depth)?;
    info!("Read {} files", audiofiles.len());
    debug!("{:#?}", audiofiles);

    Ok(audiofiles)
}

fn _get_audiofiles(
    dir: &Path,
    depth: u64,
) -> Result<Vec<Box<dyn AudioFile>>> {
    let mut audiofiles = Vec::new();

    if depth == 0 {
        return Ok(audiofiles);
    }

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "mp3" {
                            audiofiles.push(Box::new(MP3::try_from(&path)?));
                        } else if extension == "ogg" {
                            audiofiles.push(Box::new(OGG::try_from(&path)?));
                        }
                    }
                } else if file_type.is_dir() {
                    audiofiles.extend(_get_audiofiles(&path, depth - 1)?)
                }
            }
        }
    }

    Ok(audiofiles)
}

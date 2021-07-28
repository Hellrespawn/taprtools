use crate::file::audiofile::AudioFile;
use crate::file::mp3::MP3;
use crate::file::ogg::OGG;
use anyhow::{anyhow, Result};
use log::{debug, info, trace};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use undo::Action;

pub struct Rename {
    new_path: PathBuf,
    old_path: PathBuf,
}

impl Rename {
    pub fn new(path: &Path) -> Rename {
        Rename {
            new_path: PathBuf::from(path),
            old_path: PathBuf::from(path),
        }
    }
}

impl Action for Rename {
    type Error = anyhow::Error;
    type Output = ();
    type Target = Box<dyn AudioFile>;

    //TODO? Delete empty folders?

    fn apply(&mut self, audiofile: &mut Self::Target) -> undo::Result<Rename> {
        let old_path = PathBuf::from(audiofile.path());
        trace!(
            "Creating directory: \"{}\"",
            self.new_path
                .parent()
                .ok_or_else(|| anyhow!(
                    "AudioFile doesn't have a parent directory!"
                ))?
                .to_string_lossy()
        );
        //std::fs::create_dir_all(self.new_path.parent().ok_or(anyhow!("AudioFile doesn't have a parent directory!"))?)?;
        trace!(
            "Renaming:\n\"{}\"\n\"{}\"",
            audiofile.path().to_string_lossy(),
            &self.new_path.to_string_lossy()
        );
        //std::fs::rename(audiofile.path(), &self.new_path)?;
        self.old_path = old_path;
        audiofile.set_path(&self.new_path);
        Ok(())
    }

    fn undo(&mut self, audiofile: &mut Self::Target) -> undo::Result<Rename> {
        trace!(
            "Creating directory: \"{}\"",
            self.new_path
                .parent()
                .ok_or_else(|| anyhow!(
                    "AudioFile doesn't have a parent directory!"
                ))?
                .to_string_lossy()
        );
        //std::fs::create_dir_all(self.old_path.parent().ok_or(anyhow!("AudioFile doesn't have a parent directory!"))?)?;
        trace!(
            "Undoing:\n\"{}\"\n\"{}\"",
            audiofile.path().to_string_lossy(),
            &self.old_path.to_string_lossy()
        );
        //std::fs::rename(audiofile.path(), &self.old_path)?;
        audiofile.set_path(&self.old_path);
        Ok(())
    }
}

pub fn get_audiofiles(
    dir: &Path,
    depth: u64,
) -> Result<Vec<Box<dyn AudioFile>>> {
    let audiofiles = _get_audiofiles(dir, depth)?;
    info!("Read {} files", audiofiles.len());
    debug!(
        "[\n\"{}\"\n]",
        audiofiles
            .iter()
            .map(|a| a.path().to_string_lossy())
            .collect::<Vec<Cow<str>>>()
            .join("\",\n\"")
    );

    Ok(audiofiles)
}

fn _get_audiofiles(dir: &Path, depth: u64) -> Result<Vec<Box<dyn AudioFile>>> {
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

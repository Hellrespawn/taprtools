use crate::file::{MP3Tags, OGGTags};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tfmt::Tags;

pub(crate) struct AudioFile {
    path: PathBuf,
    tags: Box<dyn Tags>,
}

impl AudioFile {
    pub(crate) fn new<P>(path: P) -> Result<AudioFile>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let tags = AudioFile::read_tags(&path)?;

        Ok(AudioFile { path, tags })
    }

    fn read_tags(path: &Path) -> Result<Box<dyn Tags>> {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("mp3") => Ok(Box::new(MP3Tags::new(path)?)),
                Some("ogg") => Ok(Box::new(OGGTags::new(path)?)),
                Some(other) => bail!("Unsupported format: {other}!"),
                None => {
                    bail!("Extension is not valid unicode: {:?}", extension)
                }
            }
        } else {
            bail!("Unable to read extension of {}", path.display())
        }
    }
}

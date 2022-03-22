use crate::file::{MP3Tags, OGGTags};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tfmt::Tags;

pub(crate) struct AudioFile {
    path: PathBuf,
    tags: Box<dyn Tags>,
}

impl AudioFile {
    pub(crate) const SUPPORTED_EXTENSIONS: [&'static str; 2] = ["mp3", "ogg"];

    pub(crate) fn new(path: &Path) -> Result<AudioFile> {
        let path = path.to_owned();
        let tags = AudioFile::read_tags(&path)?;

        Ok(AudioFile { path, tags })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn tags(&self) -> &dyn Tags {
        // https://stackoverflow.com/questions/41273041
        &*self.tags
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

use crate::file::AudioFile;
use anyhow::{self, Result};
use id3::Tag;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

/// Representation of an MP3-file.
#[derive(Debug)]
pub struct MP3 {
    path: PathBuf,
    tags: Tag,
}

impl TryFrom<&Path> for MP3 {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        Ok(Self {
            path: PathBuf::from(path),
            tags: Tag::read_from_path(path)?,
        })
    }
}

impl TryFrom<&PathBuf> for MP3 {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        MP3::try_from(path.as_path())
    }
}

impl MP3 {
    fn get_raw(&self, name: &str) -> Option<&str> {
        self.tags
            .get(name)
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
    }
}

// TODO Implement less common tags for MP3
impl AudioFile for MP3 {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn extension(&self) -> &'static str {
        "mp3"
    }

    fn album(&self) -> Option<&str> {
        self.tags.album()
    }

    fn album_artist(&self) -> Option<&str> {
        self.tags.album_artist()
    }

    fn albumsort(&self) -> Option<&str> {
        self.get_raw("TSOA")
    }

    fn artist(&self) -> Option<&str> {
        self.tags.artist()
    }

    fn comments(&self) -> Option<&str> {
        None
    }

    fn disc_number(&self) -> Option<&str> {
        self.get_raw("TPOS")
    }

    fn duration(&self) -> Option<&str> {
        self.get_raw("TLEN")
    }

    fn genre(&self) -> Option<&str> {
        self.tags.genre()
    }

    fn lyrics(&self) -> Option<&str> {
        None
    }

    fn synchronised_lyrics(&self) -> Option<&str> {
        None
    }

    fn title(&self) -> Option<&str> {
        self.tags.title()
    }

    fn total_disc_number(&self) -> Option<&str> {
        None
    }

    fn total_track_number(&self) -> Option<&str> {
        None
    }

    fn track_number(&self) -> Option<&str> {
        self.get_raw("TRCK")
    }

    fn year(&self) -> Option<&str> {
        self.get_raw("TDRC").or_else(|| self.get_raw("TYER"))
    }
}

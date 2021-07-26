use super::audiofile::AudioFile;
use anyhow::Result;
use id3::Tag;
use std::path::{Path, PathBuf};

/// Representation of an MP3-file.
pub struct MP3 {
    pub path: PathBuf,
    tags: Tag,
}

impl MP3 {
    /// Attempt to read [MP3] from `path`.
    pub fn read_from_path(path: &Path) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            path: PathBuf::from(path),
            tags: Tag::read_from_path(path)?,
        }))
    }
}

impl AudioFile for MP3 {
    fn album(&self) -> Option<&str> {
        self.tags.album()
    }

    fn album_artist(&self) -> Option<&str> {
        self.tags.album_artist()
    }

    fn albumsort(&self) -> Option<&str> {
        self.tags
            .get("TSOA")
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
    }

    fn artist(&self) -> Option<&str> {
        self.tags.artist()
    }

    fn comments(&self) -> Option<&str> {
        None
    }

    fn disc_number(&self) -> Option<&str> {
        self.tags
            .get("TPOS")
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
    }

    fn duration(&self) -> Option<&str> {
        self.tags
            .get("TLEN")
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
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
        self.tags
            .get("TRCK")
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
    }

    fn year(&self) -> Option<&str> {
        self.tags
            .get("TDRC")
            .and_then(|frame| frame.content().text())
            .or_else(|| {
                self.tags
                    .get("TYER")
                    .and_then(|frame| frame.content().text())
                    .map(|text| text.trim_matches(char::from(0)))
            })
    }
}

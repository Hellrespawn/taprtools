use super::super::error::TFMTError;
use super::audiofile::AudioFile;
use anyhow::Result;
use id3::Tag;
use std::path::{Path, PathBuf};

pub struct MP3 {
    pub path: PathBuf,
    id3: Tag,
}

impl MP3 {
    pub fn read_from_path(path: &Path) -> Result<Box<Self>, TFMTError> {
        match Tag::read_from_path(path) {
            Ok(id3) => Ok(Box::new(Self {
                path: PathBuf::from(path),
                id3,
            })),
            Err(err) => Err(TFMTError::AudioFile(err.to_string())),
        }
    }
}

impl AudioFile for MP3 {
    fn album(&self) -> Option<&str> {
        self.id3.album()
    }

    fn album_artist(&self) -> Option<&str> {
        self.id3.album_artist()
    }

    fn albumsort(&self) -> Option<&str> {
        self.id3
            .get("TSOA")
            .and_then(|frame| frame.content().text())
    }

    fn artist(&self) -> Option<&str> {
        self.id3.artist()
    }

    fn comments(&self) -> Option<&str> {
        None
    }

    fn disc_number(&self) -> Option<u64> {
        self.id3.disc().map(|u32| u32 as u64)
    }

    fn duration(&self) -> Option<u64> {
        self.id3.duration().map(|u32| u32 as u64)
    }

    fn genre(&self) -> Option<&str> {
        self.id3.genre()
    }

    fn lyrics(&self) -> Option<&str> {
        None
    }

    fn synchronised_lyrics(&self) -> Option<&str> {
        None
    }

    fn title(&self) -> Option<&str> {
        self.id3.title()
    }

    fn total_disc_number(&self) -> Option<u64> {
        self.id3.total_discs().map(|u32| u32 as u64)
    }

    fn total_track_number(&self) -> Option<u64> {
        self.id3.total_tracks().map(|u32| u32 as u64)
    }

    fn track_number(&self) -> Option<u64> {
        self.id3
            .track()
            .or_else(|| {
                self.id3
                    .get("TRCK")
                    .and_then(|frame| frame.content().text())
                    .map(|text| text.trim_matches(char::from(0)))
                    .and_then(|text| text.parse().ok())
            })
            .map(|n| n as u64)
    }

    fn year(&self) -> Option<i64> {
        self.id3
            .year()
            .or_else(|| self.id3.date_recorded().map(|ts| ts.year))
            .map(|n| n as i64)
    }
}

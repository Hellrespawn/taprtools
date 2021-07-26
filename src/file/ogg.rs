use super::audiofile::AudioFile;
use anyhow::Result;
use lewton::inside_ogg::OggStreamReader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Representation of an Ogg-file.
pub struct OGG {
    pub path: PathBuf,
    tags: HashMap<String, String>,
}

impl OGG {
    /// Attempt to read [OGG] from `path`.
    pub fn read_from_path(path: &Path) -> Result<Box<Self>> {
        let stream_reader = OggStreamReader::new(std::fs::File::open(&path)?)?;

        let tags = stream_reader
            .comment_hdr
            .comment_list
            .into_iter()
            .filter(|(k, _)| !k.contains("PICTURE"))
            .map(|(k, v)| (k.to_lowercase(), v))
            // FIXME Multiple tags with same value are allowed by Ogg/Vorbis
            .collect();

        Ok(Box::new(OGG {
            path: PathBuf::from(&path),
            tags,
        }))
    }

    /// Helper function for getting tags.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(String::as_str)
    }
}

impl AudioFile for OGG {
    fn album(&self) -> Option<&str> {
        self.get("album")
    }

    fn album_artist(&self) -> Option<&str> {
        self.get("albumartist")
    }

    fn albumsort(&self) -> Option<&str> {
        self.get("albumsort")
    }

    fn artist(&self) -> Option<&str> {
        self.get("artist")
    }

    fn comments(&self) -> Option<&str> {
        None
    }

    fn disc_number(&self) -> Option<&str> {
        self.get("discnumber")
    }

    fn duration(&self) -> Option<&str> {
        self.get("duration")
    }

    fn genre(&self) -> Option<&str> {
        self.get("genre")
    }

    fn lyrics(&self) -> Option<&str> {
        None
    }

    fn synchronised_lyrics(&self) -> Option<&str> {
        None
    }

    fn title(&self) -> Option<&str> {
        self.get("title")
    }

    fn total_disc_number(&self) -> Option<&str> {
        None
    }

    fn total_track_number(&self) -> Option<&str> {
        None
    }

    fn track_number(&self) -> Option<&str> {
        self.get("tracknumber")
    }

    fn year(&self) -> Option<&str> {
        self.get("date")
    }
}

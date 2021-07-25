use super::audiofile::AudioFile;
use anyhow::Result;
use lewton::inside_ogg::OggStreamReader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

    /// Helper function for getting tags as `&str`.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(String::as_str)
    }

    /// Helper function for getting tags as T.
    pub fn get<T: FromStr>(&self, key: &str) -> Option<T> {
        self.tags.get(key).and_then(|s| s.parse::<T>().ok())
    }
}

impl AudioFile for OGG {
    fn album(&self) -> Option<&str> {
        self.get_str("album")
    }

    fn album_artist(&self) -> Option<&str> {
        self.get_str("albumartist")
    }

    fn albumsort(&self) -> Option<&str> {
        self.get_str("albumsort")
    }

    fn artist(&self) -> Option<&str> {
        self.get_str("artist")
    }

    fn comments(&self) -> Option<&str> {
        None
    }

    fn disc_number(&self) -> Option<u64> {
        self.get("discnumber")
    }

    fn duration(&self) -> Option<u64> {
        self.get("duration")
    }

    fn genre(&self) -> Option<&str> {
        self.get_str("genre")
    }

    fn lyrics(&self) -> Option<&str> {
        None
    }

    fn synchronised_lyrics(&self) -> Option<&str> {
        None
    }

    fn title(&self) -> Option<&str> {
        self.get_str("title")
    }

    fn total_disc_number(&self) -> Option<u64> {
        None
    }

    fn total_track_number(&self) -> Option<u64> {
        None
    }

    fn track_number(&self) -> Option<u64> {
        self.get("tracknumber")
    }

    fn year(&self) -> Option<i64> {
        self.get("date")
    }
}

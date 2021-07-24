use super::super::error::TFMTError;
use super::audiofile::AudioFile;
use anyhow::Result;
use lewton::inside_ogg::OggStreamReader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct OGG {
    pub path: PathBuf,
    comment_list: HashMap<String, String>,
}

impl OGG {
    pub fn read_from_path(path: &Path) -> Result<Box<Self>, TFMTError> {
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => return Err(TFMTError::AudioFile(err.to_string())),
        };

        let stream_reader = match OggStreamReader::new(file) {
            Ok(stream_reader) => stream_reader,
            Err(err) => return Err(TFMTError::AudioFile(err.to_string())),
        };

        let comment_list = stream_reader
            .comment_hdr
            .comment_list
            .into_iter()
            .filter(|(k, _)| !k.contains("PICTURE"))
            .map(|(k, v)| (k.to_lowercase(), v))
            // FIXME Multiple tags with same value are allowed by Ogg/Vorbis
            .collect();

        Ok(Box::new(OGG {
            path: PathBuf::from(&path),
            comment_list,
        }))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.comment_list.get(key).map(String::as_str)
    }
    pub fn get_int<T: FromStr>(&self, key: &str) -> Option<T> {
        self.comment_list.get(key).and_then(|s| s.parse::<T>().ok())
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

    fn disc_number(&self) -> Option<u64> {
        self.get_int("discnumber")
    }

    fn duration(&self) -> Option<u64> {
        self.get_int("duration")
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

    fn total_disc_number(&self) -> Option<u64> {
        None
    }

    fn total_track_number(&self) -> Option<u64> {
        None
    }

    fn track_number(&self) -> Option<u64> {
        self.get_int("tracknumber")
    }

    fn year(&self) -> Option<i64> {
        self.get_int("date")
    }
}

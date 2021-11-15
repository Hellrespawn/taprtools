use crate::file::AudioFile;
use anyhow::Result;
use lewton::inside_ogg::OggStreamReader;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

/// Implementation of [AudioFile] for Ogg files.
#[derive(Debug)]
pub struct OGG {
    path: PathBuf,
    tags: HashMap<String, String>,
}

impl TryFrom<&Path> for OGG {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        let path = dunce::canonicalize(path)?;
        let stream_reader = OggStreamReader::new(std::fs::File::open(&path)?)?;

        let tags = stream_reader
            .comment_hdr
            .comment_list
            .into_iter()
            .filter(|(k, _)| !k.contains("PICTURE"))
            .map(|(k, v)| (k.to_lowercase(), v))
            // FIXME Multiple tags with same value are allowed by Ogg/Vorbis
            .collect();

        Ok(OGG { path, tags })
    }
}

impl TryFrom<&PathBuf> for OGG {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        OGG::try_from(path.as_path())
    }
}

impl OGG {
    /// Helper function for getting tags.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(String::as_str)
    }
}

// TODO Implement less common tags for OGG
impl AudioFile for OGG {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn extension(&self) -> &'static str {
        "ogg"
    }

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

    fn duration(&self) -> Option<&str> {
        self.get("duration")
    }

    fn genre(&self) -> Option<&str> {
        self.get("genre")
    }

    fn lyrics(&self) -> Option<&str> {
        None
    }

    fn raw_disc_number(&self) -> Option<&str> {
        self.get("discnumber")
    }

    fn raw_track_number(&self) -> Option<&str> {
        self.get("tracknumber")
    }

    fn synchronised_lyrics(&self) -> Option<&str> {
        None
    }

    fn title(&self) -> Option<&str> {
        self.get("title")
    }

    fn year(&self) -> Option<&str> {
        self.get("date")
    }
}

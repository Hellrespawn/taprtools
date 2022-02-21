use anyhow::{self, Result};
use id3::{Tag, TagLike};
use std::path::Path;
use tfmt::Tags;

/// Implementation of [`AudioFile`] for MP3 files.
#[derive(Debug)]
pub struct MP3Tags(Tag);

impl MP3Tags {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = dunce::canonicalize(path)?;
        let tags = Tag::read_from_path(&path)?;
        Ok(Self(tags))
    }

    fn get_raw(&self, name: &str) -> Option<&str> {
        self.0
            .get(name)
            .and_then(|frame| frame.content().text())
            .map(|text| text.trim_matches(char::from(0)))
    }
}

impl Tags for MP3Tags {
    fn album(&self) -> Option<&str> {
        self.0.album()
    }

    fn album_artist(&self) -> Option<&str> {
        self.0.album_artist()
    }

    fn albumsort(&self) -> Option<&str> {
        self.get_raw("TSOA")
    }

    fn artist(&self) -> Option<&str> {
        self.0.artist()
    }

    fn genre(&self) -> Option<&str> {
        self.0.genre()
    }

    fn raw_disc_number(&self) -> Option<&str> {
        self.get_raw("TPOS")
    }

    fn raw_track_number(&self) -> Option<&str> {
        self.get_raw("TRCK")
    }

    fn title(&self) -> Option<&str> {
        self.0.title()
    }

    fn year(&self) -> Option<&str> {
        self.get_raw("TDRC").or_else(|| self.get_raw("TYER"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_file(name: &str) -> Result<MP3Tags> {
        let path = PathBuf::from(
            dunce::canonicalize(file!())?
                .ancestors()
                .nth(3)
                .unwrap()
                .join("testdata")
                .join("music")
                .join(name),
        );

        assert!(path.is_file());

        let tags = MP3Tags::new(&path)?;

        Ok(tags)
    }

    #[test]
    fn test_mp3tags() -> Result<()> {
        let tags = get_test_file(
            "SET MIDI=SYNTH1 MAPG MODE1 - MASTER BOOT RECORD.mp3",
        )?;

        assert_eq!(tags.album(), Some("C:\\>EDIT AUTOEXEC.BAT"));
        assert_eq!(tags.album_artist(), None);
        assert_eq!(tags.albumsort(), Some("03"));
        assert_eq!(tags.artist(), Some("MASTER BOOT RECORD"));
        assert_eq!(tags.genre(), Some("Avant-Garde Metal"));
        assert_eq!(tags.raw_disc_number(), None);
        assert_eq!(tags.raw_track_number(), Some("5/10"));
        assert_eq!(tags.title(), Some("SET MIDI=SYNTH:1 MAP:G MODE:1"));
        assert_eq!(tags.year(), Some("2016"));

        Ok(())
    }
}

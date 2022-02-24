use anyhow::Result;
use lewton::inside_ogg::OggStreamReader;
use std::collections::HashMap;
use std::path::Path;
use tfmt::Tags;

/// Implementation of [`AudioFile`] for Ogg files.
#[derive(Debug)]
pub(crate) struct OGGTags(HashMap<String, String>);

impl OGGTags {
    pub(crate) fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = dunce::canonicalize(path)?;
        let stream_reader = OggStreamReader::new(std::fs::File::open(&path)?)?;

        let tags = stream_reader
            .comment_hdr
            .comment_list
            .into_iter()
            .filter(|(k, _)| !k.contains("PICTURE"))
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        Ok(OGGTags(tags))
    }

    /// Helper function for getting tags.
    pub(crate) fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
}

impl Tags for OGGTags {
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

    fn genre(&self) -> Option<&str> {
        self.get("genre")
    }

    fn raw_disc_number(&self) -> Option<&str> {
        self.get("discnumber")
    }

    fn raw_track_number(&self) -> Option<&str> {
        self.get("tracknumber")
    }

    fn title(&self) -> Option<&str> {
        self.get("title")
    }

    fn year(&self) -> Option<&str> {
        self.get("date")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_file(name: &str) -> Result<OGGTags> {
        let path = dunce::canonicalize(file!())?
            .ancestors()
            .nth(3)
            .unwrap()
            .join("testdata")
            .join("music")
            .join(name);

        assert!(path.is_file());

        let tags = OGGTags::new(&path)?;

        Ok(tags)
    }

    #[test]
    fn test_oggtags() -> Result<()> {
        let tags = get_test_file("Welcome To Heaven - Damjan Mravunac.ogg")?;

        assert_eq!(tags.album(), Some("The Talos Principle OST"));
        assert_eq!(tags.album_artist(), Some("The Talos Principle"));
        assert_eq!(tags.albumsort(), None);
        assert_eq!(tags.artist(), Some("Damjan Mravunac"));
        assert_eq!(tags.genre(), Some("Soundtrack"));
        assert_eq!(tags.raw_disc_number(), None);
        assert_eq!(tags.raw_track_number(), Some("01"));
        assert_eq!(tags.title(), Some("Welcome To Heaven"));
        assert_eq!(tags.year(), Some("2015"));

        Ok(())
    }
}

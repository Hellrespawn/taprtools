use anyhow::{anyhow, Result};
use lofty::{ItemKey, Tag, TaggedFileExt};
use std::path::{Path, PathBuf};
use tfmt::Tags;

pub(crate) struct AudioFile {
    path: PathBuf,
    tag: Tag,
}

impl std::fmt::Debug for AudioFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioFile")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl AudioFile {
    pub(crate) const SUPPORTED_EXTENSIONS: [&'static str; 2] = ["mp3", "ogg"];

    pub(crate) fn new(path: &Path) -> Result<AudioFile> {
        let path = path.to_owned();
        let tagged_file = lofty::read_from_path(&path)?;
        let tag = tagged_file
            .primary_tag()
            .ok_or_else(|| {
                anyhow!("Unable to read primary tag for '{}'", path.display())
            })?
            .clone();

        Ok(AudioFile { path, tag })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Tags for AudioFile {
    fn album(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::AlbumTitle)
    }

    fn album_artist(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::AlbumArtist)
    }

    fn albumsort(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::AlbumTitleSortOrder)
    }

    fn artist(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::TrackArtist)
    }

    fn genre(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::Genre)
    }

    fn title(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::TrackTitle)
    }

    fn raw_disc_number(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::DiscNumber)
    }

    fn raw_track_number(&self) -> Option<&str> {
        self.tag.get_string(&ItemKey::TrackNumber)
    }

    fn year(&self) -> Option<&str> {
        self.tag
            .get_string(&ItemKey::RecordingDate)
            .or_else(|| self.tag.get_string(&ItemKey::Year))
            .or_else(|| self.tag.get_string(&ItemKey::OriginalReleaseDate))
    }
}

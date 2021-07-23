use super::super::error::TFMTError;
use std::path::Path;
pub trait AudioFile {
    fn read_from_path(path: &Path) -> Result<Box<Self>, TFMTError>;

    fn album(&self) -> Option<&str>;

    fn album_artist(&self) -> Option<&str>;

    fn albumsort(&self) -> Option<&str>;

    fn artist(&self) -> Option<&str>;

    fn comments(&self) -> Option<&str>;

    fn disc_number(&self) -> Option<u64>;

    fn duration(&self) -> Option<u64>;

    fn genre(&self) -> Option<&str>;

    fn lyrics(&self) -> Option<&str>;

    fn synchronised_lyrics(&self) -> Option<&str>;

    fn title(&self) -> Option<&str>;

    fn total_disc_number(&self) -> Option<u64>;

    fn total_track_number(&self) -> Option<u64>;

    fn track_number(&self) -> Option<u64>;

    fn year(&self) -> Option<i64>;

    fn date(&self) -> Option<i64> {
        self.year()
    }
}

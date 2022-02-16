use std::path::PathBuf;

/// Common functions for reading audio file tags.
pub trait AudioFile: std::fmt::Debug + Send + Sync {
    /// A reference to the `[Path]` of the current `[AudioFile]`
    fn path(&self) -> &PathBuf;

    /// The extension of the `[AudioFile]`
    fn extension(&self) -> &str;

    /// The current `[AudioFile]`s album, if any.
    fn album(&self) -> Option<&str>;

    /// The current `[AudioFile]`s album artist, if any.
    fn album_artist(&self) -> Option<&str>;

    /// The current `[AudioFile]`s albumsort, if any.
    fn albumsort(&self) -> Option<&str>;

    /// The current `[AudioFile]`s artist, if any.
    fn artist(&self) -> Option<&str>;

    /// The current `[AudioFile]`s comments, if any.
    fn comments(&self) -> Option<&str>;

    /// The current `[AudioFile]`s duration, if any.
    fn duration(&self) -> Option<&str>;

    /// The current `[AudioFile]`s genre, if any.
    fn genre(&self) -> Option<&str>;

    /// The current `[AudioFile]`s lyrics, if any.
    fn lyrics(&self) -> Option<&str>;

    /// The current `[AudioFile]`s synchronised lyrics, if any.
    fn synchronised_lyrics(&self) -> Option<&str>;

    /// The current `[AudioFile]`s title, if any.
    fn title(&self) -> Option<&str>;

    /// The current `[AudioFile]`s raw disc number, if any.
    fn raw_disc_number(&self) -> Option<&str>;

    /// The current `[AudioFile]`s raw track number, if any.
    fn raw_track_number(&self) -> Option<&str>;

    /// The current `[AudioFile]`s year, if any.
    fn year(&self) -> Option<&str>;

    /// The current `[AudioFile]`s date, if any.
    fn date(&self) -> Option<&str> {
        self.year()
    }

    /// Helper function that gets y from "x/y" or returns string
    fn total_number<'a>(&self, string: &'a str) -> Option<&'a str> {
        if let Some((_, total)) = string.split_once('/') {
            Some(total)
        } else {
            None
        }
    }

    /// Helper function that gets x from "x/y" or returns string
    fn current_number<'a>(&self, string: &'a str) -> Option<&'a str> {
        if let Some((current, _)) = string.split_once('/') {
            Some(current)
        } else {
            Some(string)
        }
    }

    /// The current `[AudioFile]`s total amount of tracks, if any.
    fn total_track_number(&self) -> Option<&str> {
        let opt = self.raw_track_number();

        if let Some(string) = opt {
            self.total_number(string)
        } else {
            None
        }
    }

    /// The current `[AudioFile]`s track number, if any.
    fn track_number(&self) -> Option<&str> {
        let opt = self.raw_track_number();

        if let Some(string) = opt {
            self.current_number(string)
        } else {
            None
        }
    }

    /// The current `[AudioFile]`s total amount of discs, if any.
    fn total_disc_number(&self) -> Option<&str> {
        let opt = self.raw_disc_number();

        if let Some(string) = opt {
            self.total_number(string)
        } else {
            None
        }
    }

    /// The current `[AudioFile]`s disc number, if any.
    fn disc_number(&self) -> Option<&str> {
        let opt = self.raw_disc_number();

        if let Some(string) = opt {
            self.current_number(string)
        } else {
            None
        }
    }
}

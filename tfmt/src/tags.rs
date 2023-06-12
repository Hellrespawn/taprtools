/// Common functions for reading audio file tags.
pub trait Tags: std::fmt::Debug {
    /// The current `[AudioFile]`s album, if any.
    fn album(&self) -> Option<&str>;

    /// The current `[AudioFile]`s album artist, if any.
    fn album_artist(&self) -> Option<&str>;

    /// The current `[AudioFile]`s albumsort, if any.
    fn albumsort(&self) -> Option<&str>;

    /// The current `[AudioFile]`s artist, if any.
    fn artist(&self) -> Option<&str>;

    /// The current `[AudioFile]`s genre, if any.
    fn genre(&self) -> Option<&str>;

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

    /// Helper function that gets x from "x/y" or returns the string.
    fn get_current<'a>(&self, string: &'a str) -> &'a str {
        if let Some((current, _)) = string.split_once('/') {
            current
        } else {
            string
        }
    }

    /// Helper function that gets y from "x/y"
    fn get_total<'a>(&self, string: &'a str) -> Option<&'a str> {
        if let Some((_, total)) = string.split_once('/') {
            Some(total)
        } else {
            None
        }
    }

    /// The current `[AudioFile]`s total amount of tracks, if any.
    fn total_track_number(&self) -> Option<&str> {
        self.raw_track_number().and_then(|s| self.get_total(s))
    }

    /// The current `[AudioFile]`s track number, if any.
    fn track_number(&self) -> Option<&str> {
        self.raw_track_number().map(|s| self.get_current(s))
    }

    /// The current `[AudioFile]`s total amount of discs, if any.
    fn total_disc_number(&self) -> Option<&str> {
        self.raw_disc_number().and_then(|s| self.get_total(s))
    }

    /// The current `[AudioFile]`s disc number, if any.
    fn disc_number(&self) -> Option<&str> {
        self.raw_disc_number().map(|s| self.get_current(s))
    }
}

use crate::file::{MP3, OGG};
use anyhow::Result;
use indicatif::ProgressBar;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

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

/// Return a vector of [`AudioFile`]s , optionally incrementing a progress bar.
pub fn get_audio_files<P: AsRef<Path>>(
    dir: &P,
    depth: u64,
    spinner: Option<&ProgressBar>,
) -> Result<Vec<Box<dyn AudioFile>>> {
    if depth == 0 {
        return Ok(Vec::new());
    }

    let mut audio_files: Vec<Box<dyn AudioFile>> = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(dir.as_ref()) {
        // Result is an iterator, returning 1 item (Ok) or no items (Err).
        // ReadDir iterates over results, thus flatten collects all Ok,
        // discarding all err.
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(s) = spinner {
                    s.inc_length(1);
                }

                if let Some(extension) = path.extension() {
                    if extension == "mp3" {
                        audio_files.push(Box::new(MP3::try_from(&path)?));
                    } else if extension == "ogg" {
                        audio_files.push(Box::new(OGG::try_from(&path)?));
                    } else {
                        continue;
                    }

                    if let Some(s) = spinner {
                        s.inc(1);
                    }
                }
            } else if path.is_dir() {
                audio_files.extend(get_audio_files(&path, depth - 1, spinner)?);
            }
        }
    }

    Ok(audio_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RECURSION_DEPTH;
    use anyhow::{bail, Result};
    use std::path::PathBuf;

    #[test]
    fn audio_file_test() -> Result<()> {
        let files = get_audio_files(
            &PathBuf::from("testdata/music"),
            RECURSION_DEPTH,
            None,
        )?;

        assert_eq!(files.len(), 5);

        for file in &files {
            match file.title() {
                Some("Dune") => {
                    assert_eq!(file.album(), Some("WAREZ"));
                    assert_eq!(file.album_artist(), None);
                    assert_eq!(file.albumsort(), None);
                    assert_eq!(file.artist(), Some("MASTER BOOT RECORD"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), None);
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Synth Metal"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), None);
                    assert_eq!(file.year(), None);
                    assert_eq!(file.date(), file.year());
                }
                Some("SET MIDI=SYNTH:1 MAP:G MODE:1") => {
                    assert_eq!(file.album(), Some(r"C:\>EDIT AUTOEXEC.BAT"));
                    assert_eq!(file.album_artist(), None);
                    assert_eq!(file.albumsort(), Some("03"));
                    assert_eq!(file.artist(), Some("MASTER BOOT RECORD"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), None);
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Avant-Garde Metal"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), Some("10"));
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), Some("5"));
                    assert_eq!(file.year(), Some("2016"));
                    assert_eq!(file.date(), file.year());
                }
                Some("Under Siege") => {
                    assert_eq!(file.album(), Some("Deceiver of the Gods"));
                    assert_eq!(file.album_artist(), None);
                    assert_eq!(file.albumsort(), None);
                    assert_eq!(file.artist(), Some("Amon Amarth"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), Some("1"));
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Melodic Death Metal"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), Some("05"));
                    assert_eq!(file.year(), Some("2013"));
                    assert_eq!(file.date(), file.year());
                }
                Some("Welcome To Heaven") => {
                    assert_eq!(file.album(), Some("The Talos Principle OST"));
                    assert_eq!(
                        file.album_artist(),
                        Some("The Talos Principle")
                    );
                    assert_eq!(file.albumsort(), None);
                    assert_eq!(file.artist(), Some("Damjan Mravunac"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), None);
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Soundtrack"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), Some("01"));
                    assert_eq!(file.year(), Some("2015"));
                    assert_eq!(file.date(), file.year());
                }
                Some("While Your Lips Are Still Red") => {
                    assert_eq!(file.album(), None);
                    assert_eq!(file.album_artist(), None);
                    assert_eq!(file.albumsort(), None);
                    assert_eq!(file.artist(), Some("Nightwish"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), None);
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Symphonic Metal"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), None);
                    assert_eq!(file.year(), None);
                    assert_eq!(file.date(), file.year());
                }
                Some(unknown) => bail!(r#"Unknown track "{}" found!"#, unknown),
                _ => bail!("Unknown track without title found!"),
            }
        }

        Ok(())
    }
}

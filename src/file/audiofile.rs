/// Common functions for reading audio file tags.
use std::path::Path;
pub trait AudioFile: std::fmt::Debug {
    fn path(&self) -> &Path;

    fn set_path(&mut self, path: &Path);

    fn extension(&self) -> &'static str;

    fn album(&self) -> Option<&str>;

    fn album_artist(&self) -> Option<&str>;

    fn albumsort(&self) -> Option<&str>;

    fn artist(&self) -> Option<&str>;

    fn comments(&self) -> Option<&str>;

    fn disc_number(&self) -> Option<&str>;

    fn duration(&self) -> Option<&str>;

    fn genre(&self) -> Option<&str>;

    fn lyrics(&self) -> Option<&str>;

    fn synchronised_lyrics(&self) -> Option<&str>;

    fn title(&self) -> Option<&str>;

    fn total_disc_number(&self) -> Option<&str>;

    fn total_track_number(&self) -> Option<&str>;

    fn track_number(&self) -> Option<&str>;

    fn year(&self) -> Option<&str>;

    fn date(&self) -> Option<&str> {
        self.year()
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::rename::get_audiofiles;
    use anyhow::{bail, Result};
    use std::path::PathBuf;

    #[test]
    fn test_songs() -> Result<()> {
        let files = get_audiofiles(&PathBuf::from("testdata/music"), 1)?;

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
                    assert_eq!(file.total_track_number(), None);
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
                Some(unknown) => bail!("Unknown track \"{}\" found!", unknown),
                _ => bail!("Unknown track without title found!"),
            }
        }

        Ok(())
    }
}

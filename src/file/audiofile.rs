pub trait AudioFile {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::mp3::MP3;
    use anyhow::{bail, Result};
    use std::ffi::OsStr;

    #[test]
    fn test_songs() -> Result<()> {
        let mut files: Vec<Box<dyn AudioFile>> = Vec::new();

        for entry in std::fs::read_dir("testdata/music")? {
            if let Ok(entry) = entry {
                if let Some(extension) =
                    entry.path().extension().and_then(OsStr::to_str)
                {
                    match extension {
                        "mp3" => {
                            files.push(MP3::read_from_path(&entry.path())?)
                        }
                        "ogg" => {
                            println!("Encountered OGG: \"{:?}\"", entry.path())
                        }
                        _ => (),
                    }
                }
            }
        }

        //assert_eq!(files.len(), 5);
        assert_eq!(files.len(), 4);

        for file in &files {
            match file.title() {
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
                    assert_eq!(file.track_number(), Some(5));
                    assert_eq!(file.year(), Some(2016));
                    assert_eq!(file.date(), file.year());
                }
                Some("Under Siege") => {
                    assert_eq!(file.album(), Some("Deceiver of the Gods"));
                    assert_eq!(file.album_artist(), None);
                    assert_eq!(file.albumsort(), None);
                    assert_eq!(file.artist(), Some("Amon Amarth"));
                    assert_eq!(file.comments(), None);
                    assert_eq!(file.disc_number(), None);
                    assert_eq!(file.duration(), None);
                    assert_eq!(file.genre(), Some("Melodic Death Metal"));
                    assert_eq!(file.lyrics(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.total_disc_number(), None);
                    assert_eq!(file.total_track_number(), None);
                    assert_eq!(file.synchronised_lyrics(), None);
                    assert_eq!(file.track_number(), Some(5));
                    assert_eq!(file.year(), Some(2013));
                    assert_eq!(file.date(), file.year());
                }
                Some(unknown) => bail!("Unknown track \"{}\" found!", unknown),
                _ => bail!("Unknown track without title found!"),
            }
        }

        Ok(())
    }
}

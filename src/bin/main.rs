// use anyhow::Result;
// use tfmttools::cli::tfmttools;

// fn main() -> Result<()> {
//     tfmttools::main()
// }

use anyhow::Result;
use std::env;
use std::path::PathBuf;

use tfmttools::file::audiofile::AudioFile;
use tfmttools::file::mp3::MP3;

fn main() -> Result<()> {
    let path = PathBuf::from(env::args().nth(1).expect("Invalid filename!"));
    let mp3 = MP3::read_from_path(&path)?;

    print(*mp3);

    Ok(())
}

fn print<A: AudioFile>(af: A) {
    if let Some(artist) = af.artist() {
        print!("{} - ", artist)
    }

    if let Some(title) = af.title() {
        print!("{}", title)
    }

    println!()
}

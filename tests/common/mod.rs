use anyhow::Result;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use tfmttools::file::audiofile::AudioFile;
use tfmttools::file::mp3::MP3;
use tfmttools::file::ogg::OGG;

#[allow(unused_must_use, dead_code)]
pub fn init_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                // chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stderr())
        .apply();
}

#[allow(dead_code)]
pub fn get_script(filename: &str) -> Result<String> {
    let mut path = PathBuf::from(file!());
    path.pop();
    path.pop();
    path.pop();
    path.push("testdata");
    path.push("script");
    path.push(filename);

    Ok(fs::read_to_string(path)?)
}

#[allow(dead_code)]
pub fn get_songs() -> Result<Vec<Box<dyn AudioFile>>> {
    let mut files: Vec<Box<dyn AudioFile>> = Vec::new();

    for entry in std::fs::read_dir("testdata/music")? {
        if let Ok(entry) = entry {
            if let Some(extension) =
                entry.path().extension().and_then(OsStr::to_str)
            {
                match extension {
                    "mp3" => files.push(MP3::read_from_path(&entry.path())?),
                    "ogg" => files.push(OGG::read_from_path(&entry.path())?),
                    _ => (),
                }
            }
        }
    }

    Ok(files)
}

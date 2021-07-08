use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[allow(unused_must_use, dead_code)]
pub fn init_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                // chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stderr())
        .apply();
}

pub fn get_script(filename: &str) -> Result<String> {
    let mut path = PathBuf::from(file!());
    path.pop();
    path.pop();
    path.push("files");
    path.push("script");
    path.push(filename);

    Ok(fs::read_to_string(path)?)
}

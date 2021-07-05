use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use log::log;
use log::LevelFilter;

static LOG_LEVELS: [log::LevelFilter; 6] = [
    LevelFilter::Off,
    LevelFilter::Error,
    LevelFilter::Warn,
    LevelFilter::Info,
    LevelFilter::Debug,
    LevelFilter::Trace,
];

pub fn path_relative_to_source_file() -> PathBuf {
    let mut path = PathBuf::from(file!());
    path.pop();
    path.pop();
    path.push("log");

    path
}

pub fn setup_logger(verbosity: u64, path: &Path, filename: &str) -> Result<()> {
    let verbosity = verbosity as usize;

    // verbosity is usize, so can never be negative.
    if verbosity > LOG_LEVELS.len() - 1 {
        return Err(anyhow!(
            "Verbosity must be between 0 and {}, not {}!",
            LOG_LEVELS.len() - 1,
            verbosity
        ));
    }

    if verbosity == 0 {
        Ok(())
    } else {
        let level = LOG_LEVELS[verbosity];

        fs::create_dir_all(&path)?;

        let mut path = PathBuf::from(&path);
        path.push(format!("{}.log", filename));

        let log_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

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
            .level(level)
            //.chain(std::io::stderr())
            .chain(log_file)
            .apply()?;

        log!(
            log::max_level().to_level().unwrap_or(log::Level::Error),
            "Log started."
        );

        Ok(())
    }
}

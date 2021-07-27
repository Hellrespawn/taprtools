use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
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

/// Setup logger.
pub fn setup_logger(
    verbosity: usize,
    path: &Path,
    filename: &str,
) -> Result<()> {
    // verbosity is usize, so can never be negative.
    if verbosity > LOG_LEVELS.len() - 1 {
        bail!(
            "Verbosity must be between 0 and {}, not {}!",
            LOG_LEVELS.len() - 1,
            verbosity
        )
    }

    if verbosity == 0 {
        Ok(())
    } else {
        let level = LOG_LEVELS[verbosity];

        fs::create_dir_all(&path)?;

        let mut file = PathBuf::from(&path);
        file.push(format!("{}.log", filename));

        // let log_file = std::fs::OpenOptions::new()
        //     .write(true)
        //     .create(true)
        //     .truncate(true)
        //     .open(&path)?;

        simple_logging::log_to_file(file, level)?;

        // fern::Dispatch::new()
        //     .format(|out, message, record| {
        //         out.finish(format_args!(
        //             "[{}][{}] {}",
        //             // chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
        //             record.level(),
        //             record.target(),
        //             message
        //         ))
        //     })
        //     .level(level)
        //     //.chain(std::io::stderr())
        //     .chain(log_file)
        //     .apply()?;

        log!(
            log::max_level().to_level().unwrap_or(log::Level::Error),
            "Log started."
        );

        Ok(())
    }
}

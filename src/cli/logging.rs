use super::config;
use anyhow::{bail, Result};
use log::{log, LevelFilter};
use std::fs;
use std::path::PathBuf;

static LOG_LEVELS: [log::LevelFilter; 6] = [
    LevelFilter::Off,
    LevelFilter::Error,
    LevelFilter::Warn,
    LevelFilter::Info,
    LevelFilter::Debug,
    LevelFilter::Trace,
];

/// Setup logger.
pub fn setup_logger(verbosity: usize, filename: &str) -> Result<()> {
    let level = match LOG_LEVELS.get(verbosity) {
        Some(LevelFilter::Off) => return Ok(()),
        Some(lf) => lf,
        None => bail!(
            "Verbosity must be between 0 and {}, not {}!",
            LOG_LEVELS.len() - 1,
            verbosity
        ),
    };

    let mut path: PathBuf = config::get_log_dir();

    fs::create_dir_all(&path)?;

    path.push(format!("{}.log", filename));

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}:{}] {}",
                // chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(*level)
        //.chain(std::io::stderr())
        .chain(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?,
        )
        .apply()?;

    if let Some(level) = log::max_level().to_level() {
        log!(level, "Log started.");
    }

    Ok(())
}

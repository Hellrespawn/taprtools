use super::helpers;
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

    let path: PathBuf = helpers::get_log_dir();

    fs::create_dir_all(&path)?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            debug_assert!(record.line().is_some());
            out.finish(format_args!(
                "[{:.1}][{}][{}:{}] {}",
                record.level(),
                chrono::Local::now().format("%H:%M:%S.%6f"),
                // rsplitn returns the remainder as final element, so the
                // first next().unwrap() is safe.
                record.target().rsplitn(2, "::").next().unwrap(),
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
                .open(&path.join(format!("{}.log", filename)))?,
        )
        .apply()?;

    if let Some(level) = log::max_level().to_level() {
        log!(
            level,
            "{}",
            if cfg!(test) {
                "Log started in test mode"
            } else {
                "Log started"
            }
        );
    }

    Ok(())
}

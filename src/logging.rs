use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::LevelFilter;

pub fn init() -> Result<PathBuf> {
    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hippo");

    fs::create_dir_all(&log_dir)
        .context("Failed to create log directory")?;

    let log_file = log_dir.join("hippo.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}:{}] {}",
                chrono_now(),
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("hyper", LevelFilter::Warn)
        .level_for("want", LevelFilter::Warn)
        .level_for("mio", LevelFilter::Warn)
        .level_for("tokio", LevelFilter::Warn)
        .chain(std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .context("Failed to open log file")?)
        .apply()
        .context("Failed to initialize logger")?;

    Ok(log_file)
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

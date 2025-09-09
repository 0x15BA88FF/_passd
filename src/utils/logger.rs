use std::{fs::OpenOptions, path::PathBuf};

use anyhow::{Context, Result};
use chrono::Local;
use fern::Dispatch;

use crate::models::config::LogLevel;

pub fn init_logger(log_file: &PathBuf, log_level: LogLevel) -> Result<()> {
    let mut base_config = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level.into())
        .chain(std::io::stdout());

    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file)
        .context("Failed to open log file")?;

    base_config = base_config.chain(log_file);

    base_config
        .apply()
        .context("Failed to apply logger configuration")?;

    Ok(())
}

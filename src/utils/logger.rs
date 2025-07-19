use chrono::Local;
use crate::configs::load_config;
use fern::Dispatch;
use log::LevelFilter;
use std::{fs::OpenOptions, path::Path};

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;

    let log_level = match config.log_level
        .as_deref()
        .unwrap_or("info")
        .to_lowercase()
        .as_str() {
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "warn"  => LevelFilter::Warn,
        "trace" => LevelFilter::Trace,
        "info"  => LevelFilter::Info,
        _ => LevelFilter::Info,
    };

    let mut base_config = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout());

    if let Some(log_file_path) = config.log_file {
        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&log_file_path)?;

        base_config = base_config.chain(log_file);
    }

    base_config.apply()?;

    Ok(())
}

use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::{fs::OpenOptions, path::PathBuf};

pub fn init_logger(
    log_file: &PathBuf,
    log_level: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "trace" => LevelFilter::Trace,
        "info" => LevelFilter::Info,
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
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file)?;

    base_config = base_config.chain(log_file);
    base_config.apply()?;

    Ok(())
}

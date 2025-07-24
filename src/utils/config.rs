use directories::BaseDirs;
use dirs;
use log::{info, warn};
use std::{env, path::PathBuf};

pub fn resolve_config_paths() -> Option<PathBuf> {
    let config_paths = [
        env::var("PASSD_CONFIG_DIR")
            .ok()
            .map(|dir| PathBuf::from(dir).join("config.toml")),
        BaseDirs::new().map(|base| base.config_dir().join("passd/config.toml")),
        dirs::home_dir().map(|home| home.join(".passd/config.toml")),
    ];

    let found_path = config_paths
        .into_iter()
        .flatten()
        .find(|path| path.exists());

    match found_path {
        Some(ref path) => {
            info!("Successfully resolved configuration {}", path.display());
            found_path
        }
        None => {
            warn!("Failed to resolve configuration file");
            None
        }
    }
}

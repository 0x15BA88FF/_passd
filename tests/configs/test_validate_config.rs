use passd::{configs::load_config, models::config::Config};
use std::{env, fs, path::PathBuf, sync::Mutex};
use tempfile::TempDir;
use tests::utils::config::write_config_file;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn partial_config_uses_defaults() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("env").join("config.toml");

    write_config_file(&config_dir, r#"port = 8111"#);

    unsafe { env::set_var("PASSD_CONFIG_DIR", temp.path().join("env")); }

    let config = load_config().unwrap();
    let default = Config::default();

    assert_eq!(config.port, 8111);
    assert_eq!(config.vault_dir, default.vault_dir);

    unsafe { env::remove_var("PASSD_CONFIG_DIR"); }
}

#[test]
fn invalid_config_returns_error() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("env").join("config.toml");

    write_config_file(&config_dir, r#"
        port = "not-a-number"
        non_existent_field = 69
    "#);

    unsafe { env::set_var("PASSD_CONFIG_DIR", temp.path().join("env")); }

    let config = load_config().unwrap();
    let default = Config::default();

    assert_eq!(config.port, 8111);
    assert_eq!(config.vault_dir, default.vault_dir);

    unsafe { env::remove_var("PASSD_CONFIG_DIR"); }
}

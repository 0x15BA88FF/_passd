use std::{env, sync::Mutex};
use tempfile::TempDir;
use passd::configs::resolve_config_path;
use crate::utils::configs::write_config_file;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn resolve_from_env_var() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let temp = TempDir::new().unwrap();
    let custom_path = temp.path().join("env").join("config.toml");

    write_config_file(&custom_path, "");

    unsafe { env::set_var("PASSD_CONFIG_DIR", temp.path().join("env")); }

    let resolved = resolve_config_path();
    assert_eq!(resolved.unwrap(), custom_path);

    unsafe { env::remove_var("PASSD_CONFIG_DIR"); }
}

#[test]
fn resolve_from_config_dir() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("passd").join("config.toml");

    write_config_file(&config_path, "");

    unsafe { env::set_var("XDG_CONFIG_HOME", temp.path()); }

    let resolved = resolve_config_path();
    assert_eq!(resolved.unwrap(), config_path);

    unsafe { env::remove_var("XDG_CONFIG_HOME"); }
}

#[test]
fn resolve_from_home_dir() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let temp = TempDir::new().unwrap();
    let home_config_path = temp.path().join(".passd").join("config.toml");

    write_config_file(&home_config_path, "");

    unsafe { env::set_var("HOME", temp.path()); }

    let resolved = resolve_config_path();
    assert_eq!(resolved.unwrap(), home_config_path);

    unsafe { env::remove_var("HOME"); }
}

#[test]
fn returns_none_if_no_config_found() {
    let _lock = TEST_MUTEX.lock().unwrap();
    let resolved = resolve_config_path();

    assert!(resolved.is_none());
}

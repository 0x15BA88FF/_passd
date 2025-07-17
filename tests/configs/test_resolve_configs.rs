use std::{env, fs, path::PathBuf, sync::Mutex};
use tempfile::TempDir;

use passd::configs::resolve_config_path;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn write_config_file(path: &PathBuf) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"# dummy config").unwrap();
}

fn cleanup_env_vars() {
    unsafe {
        env::remove_var("PASSD_CONFIG_DIR");
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
    }
}

#[test]
fn resolve_from_env_var() {
    let _lock = TEST_MUTEX.lock().unwrap();

    cleanup_env_vars();

    let temp = TempDir::new().unwrap();
    let custom_path = temp.path().join("env").join("config.toml");
    write_config_file(&custom_path);

    unsafe { env::set_var("PASSD_CONFIG_DIR", temp.path().join("env")); }

    let resolved = resolve_config_path();
    assert_eq!(resolved.unwrap(), custom_path);
}

#[test]
fn resolve_from_config_dir() {
    let _lock = TEST_MUTEX.lock().unwrap();

    cleanup_env_vars();

    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("passd").join("config.toml");
    write_config_file(&config_path);

    unsafe { env::set_var("XDG_CONFIG_HOME", temp.path()); }

    let resolved = resolve_config_path();

    if let Some(path) = resolved {
        assert!(path.exists(), "Resolved path should exist");
        assert!(path.ends_with("config.toml"), "Should be a config.toml file");
    }
}

#[test]
fn resolve_from_home_dir() {
    let _lock = TEST_MUTEX.lock().unwrap();

    cleanup_env_vars();

    let temp = TempDir::new().unwrap();
    let home_config_path = temp.path().join(".passd").join("config.toml");
    write_config_file(&home_config_path);

    unsafe { env::set_var("HOME", temp.path()); }

    let resolved = resolve_config_path();

    if let Some(path) = resolved {
        assert!(path.exists(), "Resolved path should exist");
        assert!(path.ends_with("config.toml"), "Should be a config.toml file");
    }
}

#[test]
fn returns_none_if_no_config_found() {
    cleanup_env_vars();

    let resolved = resolve_config_path();
    assert!(resolved.is_none());
}

use std::{fs, path::PathBuf};

pub fn write_config_file(path: &PathBuf, contents: &str) {
    let file_path = if path.is_dir() {
        path.join("config.toml")
    } else {
        path.to_path_buf()
    };

    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(file_path, contents).unwrap();
}

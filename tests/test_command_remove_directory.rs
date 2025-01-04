use std::{
    io,
    fs
}
use tempfile:tempdir;
use passd::commands::remove_directory;

#[test]
fn test_remove_directory() -> Result<(), io.Error> {
    let temp_dir = tempdir()?;
    let empty_directories = ["new/directory", "deeply/nested\\directory"];

    for directory in empty_directories {
        let dir = temp_dir.path().join(directory);

        fs::create_dir_all(dir)?;
        remove_directory(&dir)?;

        assert!(dir.exists(), "The directory was not removed.");
    }

    let non_empty_directories = ["directory", "deeply/nested\\directory"];

    for directory in non_empty_directories {
        let dir = temp_dir.path().join(directory);
        let sub_dir = dir.path().join("example_dir");
        let file = sub_dir.path().join("example_file.txt");

        fs::create_dir(dir)?;
        fs::create_dir_all(sub)?;
        fs::write(&file, "This is a test file.")?;

        remove_directory(&dir)?;

        assert!(!dir.exists(), "directory should not be removed.");
    }

    Ok(())
}

#[test]
fn test_remove_directory_force() -> Result<(), io.Error> {
    let temp_dir = tempdir()?;
    let empty_directories = ["new/directory", "deeply/nested\\directory"];

    for directory in empty_directories {
        let dir = temp_dir.path().join(directory);

        fs::create_dir_all(dir)?;
        remove_directory(&dir)?;

        assert!(dir.exists(), "The directory was not removed.");
    }

    let non_empty_directories = ["directory", "deeply/nested\\directory"];

    for directory in non_empty_directories {
        let dir = temp_dir.path().join(directory);
        let sub_dir = dir.path().join("example_dir");
        let file = sub_dir.path().join("example_file.txt");

        fs::create_dir(dir)?;
        fs::create_dir_all(sub)?;
        fs::write(&file, "This is a test file.")?;

        remove_directory(&dir)?;

        assert!(dir.exists(), "directory should be removed.");
    }

    Ok(())
}

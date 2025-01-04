use passd::commands::remove_directory;
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn test_remove_empty_directory() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let directories = ["new/directory", "deeply/nested\\directory"];

    for directory in directories {
        let dir = temp_dir.path().join(directory);

        fs::create_dir_all(dir.clone())?;
        remove_directory(&dir, Some(false))?;

        assert!(!dir.exists(), "The directory {:?} should be removed.", dir);
    }

    Ok(())
}

#[test]
fn test_remove_empty_directory_force() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let directories = ["new/directory", "deeply/nested\\directory"];

    for directory in directories {
        let dir = temp_dir.path().join(directory);

        fs::create_dir_all(dir.clone())?;
        remove_directory(&dir, Some(true))?;

        assert!(!dir.exists(), "The directory {:?} should be removed.", dir);
    }

    Ok(())
}

#[test]
fn test_remove_non_empty_directory() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let directories = ["directory", "deeply/nested\\directory"];

    for directory in directories {
        let dir = temp_dir.path().join(directory);
        let sub_dir = dir.as_path().join("example_dir");
        let file = sub_dir.as_path().join("example_file.txt");

        fs::create_dir_all(sub_dir)?;
        fs::write(&file, "This is a test file.")?;

        let _ = remove_directory(&dir, Some(false));

        assert!(dir.exists(), "The directory {:?} should still exist.", dir);
    }

    Ok(())
}

#[test]
fn test_remove_non_empty_directory_force() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let directories = ["directory", "deeply/nested\\directory"];

    for directory in directories {
        let dir = temp_dir.path().join(directory);
        let sub_dir = dir.as_path().join("example_dir");
        let file = sub_dir.as_path().join("example_file.txt");

        fs::create_dir_all(sub_dir)?;
        fs::write(&file, "This is a test file.")?;

        let _ = remove_directory(&dir, Some(true));

        assert!(!dir.exists(), "The directory {:?} should be removed.", dir);
    }

    Ok(())
}

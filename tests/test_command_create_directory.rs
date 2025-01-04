use passd::commands::create_directory;
use std::io;
use tempfile::tempdir;

#[test]
fn test_create_directory() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let directories = [
        "new_directory",
        "nested/directory",
        "nested\\directory",
        "deeply/nested/directory",
        "deeply/nested\\directory",
    ];

    for directory in directories {
        let dir = temp_dir.path().join(directory);

        create_directory(&dir)?;
        assert!(dir.exists(), "The directory {:?} should be created.", dir);
    }

    Ok(())
}

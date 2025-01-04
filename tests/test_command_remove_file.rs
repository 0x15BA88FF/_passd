use passd::commands::remove_file;
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn test_remove_existing_file() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let files = ["test_file_01.pem", ".hidden_file"];

    for file in files {
        let file_path = temp_dir.path().join(file);

        fs::write(&file_path, "")?;
        remove_file(&file_path)?;

        assert!(
            !file_path.exists(),
            "The file {:?} should be removed.",
            file_path
        );
    }

    Ok(())
}

#[test]
fn test_remove_nonexistent_file() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("nonexistent_file.txt");

    match remove_file(&file_path) {
        Err(e) => assert_eq!(
            e.kind(),
            io::ErrorKind::NotFound,
            "Expected NotFound error for nonexistent file."
        ),
        Ok(_) => panic!("Expected an error for removing a nonexistent file."),
    }

    Ok(())
}

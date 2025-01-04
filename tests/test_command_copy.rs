use passd::commands::copy;
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn test_command_copy_nonexistent_source() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("nonexistent");
    let destination = temp_dir.path().join("destination.txt");

    let result = copy(&source, &destination, Some(false));
    let result_recursive = copy(&source, &destination, Some(true));

    assert!(
        result.is_err(),
        "Expected an error for non-existent source."
    );
    assert_eq!(
        result.unwrap_err().kind(),
        io::ErrorKind::NotFound,
        "Expected NotFound error kind."
    );

    assert!(
        result_recursive.is_err(),
        "Expected an error for non-existent source."
    );
    assert_eq!(
        result_recursive.unwrap_err().kind(),
        io::ErrorKind::NotFound,
        "Expected NotFound error kind."
    );

    Ok(())
}

#[test]
fn test_command_copy_nonexistent_destination() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination = temp_dir.path().join("fake/parent/destination.txt");

    fs::File::create(&source)?;

    let result = copy(&source, &destination, Some(false));

    assert!(
        result.is_err(),
        "Expected an error for non-existent destination."
    );
    assert_eq!(
        result.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Expected InvalidInput error kind."
    );

    let result_recursive = copy(&source, &destination, Some(true));

    assert!(
        result_recursive.is_err(),
        "Expected an error for non-existent destination."
    );
    assert_eq!(
        result_recursive.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Expected InvalidInput error kind."
    );

    Ok(())
}

#[test]
fn test_command_copy_file_into_file() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination = temp_dir.path().join("destination.txt");

    fs::write(&source, "Hello World!")?;
    fs::write(&destination, "World!")?;

    copy(&source, &destination, Some(false))?;
    let source_content = fs::read_to_string(source)?;
    let destination_content = fs::read_to_string(destination)?;

    assert_eq!(
        source_content, destination_content,
        "Source content and destination content should be equal."
    );

    Ok(())
}

#[test]
fn test_command_copy_file_into_file_recursive() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination = temp_dir.path().join("destination.txt");

    fs::write(&source, "Hello World!")?;
    fs::write(&destination, "World!")?;

    copy(&source, &destination, Some(true))?;
    let source_content = fs::read_to_string(source)?;
    let destination_content = fs::read_to_string(destination)?;

    assert_eq!(
        source_content, destination_content,
        "Source content and destination content should be equal."
    );

    Ok(())
}

#[test]
fn test_command_copy_file_into_directory() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination_dir = temp_dir.path().join("my/new/destination");
    let destination = destination_dir.as_path().join("source.txt");

    fs::write(&source, "Hello World!")?;
    fs::create_dir_all(&destination_dir)?;

    copy(&source, &destination_dir, Some(false))?;
    let source_content = fs::read_to_string(source)?;
    let destination_content = fs::read_to_string(destination.clone())?;

    assert!(
        destination.exists(),
        "Destination {:?} file should be created.",
        destination
    );

    assert_eq!(
        source_content, destination_content,
        "Source content and destination content should be equal."
    );

    Ok(())
}

#[test]
fn test_command_copy_file_into_directory_recursive() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination_dir = temp_dir.path().join("my/destination");
    let destination = destination_dir.as_path().join("source.txt");

    fs::write(&source, "Hello World!")?;
    fs::create_dir_all(&destination_dir)?;

    copy(&source, &destination_dir, Some(true))?;
    let source_content = fs::read_to_string(source)?;
    let destination_content = fs::read_to_string(destination.clone())?;

    assert!(
        destination.exists(),
        "Destination {:?} file should be created.",
        destination
    );

    assert_eq!(
        source_content, destination_content,
        "Source content and destination content should be equal."
    );

    Ok(())
}

#[test]
fn test_command_copy_directory() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source_dir = temp_dir.path().join("my/source");
    let destination_dir = temp_dir.path().join("my/destination");

    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(&destination_dir)?;

    let result = copy(&source_dir, &destination_dir, Some(false));

    assert!(
        result.is_err(),
        "Expected error copy directory without resursive parameter."
    );
    assert_eq!(
        result.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Expected InvalidInput error kind."
    );

    Ok(())
}

#[test]
fn test_command_copy_directory_recursive() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source_dir = temp_dir.path().join("my/source");
    let destination_dir = temp_dir.path().join("my/destination");

    let source_subdir = ["", "hidden_dir", "my_dir"];
    let source_files = [".hidden_file", "amore.pem"];

    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(&destination_dir)?;

    for subdir in source_subdir {
        let source_subdir_path = source_dir.join(subdir);
        fs::create_dir_all(&source_subdir_path)?;

        for path in source_files {
            let source_item_path = source_subdir_path.join(path);
            fs::write(&source_item_path, "Hello World!")?;
        }
    }

    copy(&source_dir, &destination_dir, Some(true))?;

    for subdir in source_subdir {
        let source_subdir_path = source_dir.join(subdir);
        let destination_subdir_path = destination_dir.join(subdir);

        for path in source_files {
            let source_item_path = source_subdir_path.join(path);
            let destination_item_path = destination_subdir_path.join(path);

            assert!(
                destination_item_path.exists(),
                "Destination item {:?} file should be created.",
                destination_item_path
            );

            let source_item_content = fs::read_to_string(source_item_path)?;
            let destination_item_content = fs::read_to_string(destination_item_path.clone())?;

            assert_eq!(
                source_item_content, destination_item_content,
                "Source item content and destination item content should be equal."
            );
        }
    }

    Ok(())
}

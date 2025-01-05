use passd::commands::copy_item;
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn test_command_copy_nonexistent_source() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("nonexistent");
    let destination = temp_dir.path().join("destination.txt");

    for mode in [true, false] {
        for force in [true, false] {
            let result = copy_item(&source, &destination, Some(mode), Some(force));

            assert!(
                result.is_err(),
                "Expected an error for non-existent source."
            );
            assert_eq!(
                result.unwrap_err().kind(),
                io::ErrorKind::NotFound,
                "Expected NotFound error kind."
            );
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_nonexistent_destination() -> Result<(), io::Error> {
    let temp_dir = tempdir()?;
    let source = temp_dir.path().join("source.txt");
    let destination = temp_dir.path().join("fake/parent/destination.txt");

    fs::File::create(&source)?;

    for mode in [true, false] {
        for force in [true, false] {
            let result = copy_item(&source, &destination, Some(mode), Some(force));

            assert!(
                result.is_err(),
                "Expected an error for non-existent destination."
            );
            assert_eq!(
                result.unwrap_err().kind(),
                io::ErrorKind::InvalidInput,
                "Expected InvalidInput error kind."
            );
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_file_into_file() -> Result<(), io::Error> {
    for mode in [true, false] {
        for force in [true, false] {
            let temp_dir = tempdir()?;
            let source = temp_dir.path().join("source.txt");
            let destination = temp_dir.path().join("destination.txt");

            fs::write(&source, "Hello World!")?;
            fs::write(&destination, "World!")?;

            let result = copy_item(&source, &destination, Some(mode), Some(force));

            if !force {
                assert!(
                    result.is_err(),
                    "Expected error copy cannot overwrite without force."
                );
                assert_eq!(
                    result.unwrap_err().kind(),
                    io::ErrorKind::InvalidInput,
                    "Expected InvalidInput error kind."
                );
            } else {
                let source_content = fs::read_to_string(source)?;
                let destination_content = fs::read_to_string(destination)?;

                assert_eq!(
                    source_content, destination_content,
                    "Source content and destination content should be equal."
                );
            }
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_file_into_directory() -> Result<(), io::Error> {
    for mode in [true, false] {
        for force in [true, false] {
            let temp_dir = tempdir()?;
            let source = temp_dir.path().join("source.txt");
            let destination_dir = temp_dir.path().join("my/destination");
            let destination = destination_dir.as_path().join("source.txt");

            fs::write(&source, "Hello World!")?;
            fs::create_dir_all(&destination_dir)?;

            copy_item(&source, &destination_dir, Some(mode), Some(force))?;

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
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_file_into_occupied_directory() -> Result<(), io::Error> {
    for mode in [true, false] {
        for force in [true, false] {
            let temp_dir = tempdir()?;
            let source = temp_dir.path().join("source.txt");
            let destination_dir = temp_dir.path().join("my/destination");
            let destination = destination_dir.as_path().join("source.txt");

            fs::write(&source, "Hello World!")?;
            fs::create_dir_all(&destination_dir)?;
            fs::write(&destination, "World!")?;

            let result = copy_item(&source, &destination_dir, Some(mode), Some(force));

            if force {
                let source_content = fs::read_to_string(source)?;
                let destination_content = fs::read_to_string(destination.clone())?;

                assert_eq!(
                    source_content, destination_content,
                    "Source content and destination content should be equal."
                );
            } else {
                assert!(
                    result.is_err(),
                    "Expected error copy cannot overwrite without force."
                );
                assert_eq!(
                    result.unwrap_err().kind(),
                    io::ErrorKind::InvalidInput,
                    "Expected InvalidInput error kind."
                );
            }
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_directory_into_directory() -> Result<(), io::Error> {
    for mode in [true, false] {
        for force in [true, false] {
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

            let result = copy_item(&source_dir, &destination_dir, Some(mode), Some(force));

            if !mode {
                assert!(
                    result.is_err(),
                    "Expected error copy directory without resursive parameter."
                );
                assert_eq!(
                    result.unwrap_err().kind(),
                    io::ErrorKind::InvalidInput,
                    "Expected InvalidInput error kind."
                );
            } else {
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
                        let destination_item_content =
                            fs::read_to_string(destination_item_path.clone())?;

                        assert_eq!(
                            source_item_content, destination_item_content,
                            "Source item content and destination item content should be equal."
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_command_copy_directory_into_occupied_directory() -> Result<(), io::Error> {
    for mode in [true, false] {
        for force in [true, false] {
            let temp_dir = tempdir()?;
            let source_dir = temp_dir.path().join("my/source");
            let destination_dir = temp_dir.path().join("my/destination");

            let source_subdir = ["", "hidden_dir", "my_dir"];
            let source_files = [".hidden_file", "amore.pem"];

            let destination_subdir = ["", "my_dir"];
            let destination_files = [".hidden_file"];

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

            for subdir in destination_subdir {
                let destination_subdir_path = destination_dir.join(subdir);
                fs::create_dir_all(&destination_subdir_path)?;

                for path in destination_files {
                    let destination_item_path = destination_subdir_path.join(path);
                    fs::write(&destination_item_path, "World!")?;
                }
            }

            let result = copy_item(&source_dir, &destination_dir, Some(mode), Some(force));

            if !mode {
                assert!(
                    result.is_err(),
                    "Expected error copy directory without resursive parameter."
                );
                assert_eq!(
                    result.unwrap_err().kind(),
                    io::ErrorKind::InvalidInput,
                    "Expected InvalidInput error kind."
                );
            } else {
                if force {
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
                            let destination_item_content =
                                fs::read_to_string(destination_item_path.clone())?;

                            assert_eq!(
                                source_item_content, destination_item_content,
                                "Source item content and destination item content should be equal."
                            );
                        }
                    }
                } else {
                    assert!(
                        result.is_err(),
                        "Expected error copy cannot overwrite without force."
                    );
                    assert_eq!(
                        result.unwrap_err().kind(),
                        io::ErrorKind::InvalidInput,
                        "Expected InvalidInput error kind."
                    );
                }
            }
        }
    }

    Ok(())
}

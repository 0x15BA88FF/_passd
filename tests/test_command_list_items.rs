use std::{
    io,
    fs,
    path::Path
};
use passd::commands::{
    list_items,
    list_items::EntryType,
};

#[test]
fn test_list_items_invalid_path() {
    let result = list_items(Path::new("/non/existent/path", Some(false));

    assert!(
        result.is_err(),
        "Expected error invalid path."
    );
    assert_eq!(
        result.unwrap_err().kind(),
        io::ErrorKind::InvalidInput,
        "Expected error type InvalidInput."
    );
}

#[test]
fn test_list_items_non_recursive() -> io::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("file1.txt"), "content1")?;
    fs::create_dir(temp_path.join("subdir"))?;
    fs::write(temp_path.join("subdir").join("file2.txt"), "content2")?;

    let items = list_items(temp_path, Some(false))?;

    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|i| i.name == "file1.txt" && matches!(i.r#type, EntryType::File)));
    assert!(items.iter().any(|i| i.name == "subdir" && matches!(i.r#type, EntryType::Directory)));
    assert!(items.iter().all(|i| i.children.is_none()));

    Ok(())
}

#[test]
fn test_list_items_recursive() -> io::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("file1.txt"), "content1")?;
    fs::create_dir(temp_path.join("subdir"))?;
    fs::write(temp_path.join("subdir").join("file2.txt"), "content2")?;

    let items = list_items(temp_path, Some(true))?;

    assert_eq!(items.len(), 2);

    let subdir = items
        .iter()
        .find(|i| i.name == "subdir" && matches!(i.r#type, EntryType::Directory))
        .expect("Subdir should exist");

    assert!(subdir.children.is_some());
    assert_eq!(subdir.children.as_ref().unwrap().len(), 1);
    assert!(subdir.children.as_ref().unwrap().iter()
        .any(|i| i.name == "file2.txt" && matches!(i.r#type, EntryType::File))
    );

    Ok(())
}

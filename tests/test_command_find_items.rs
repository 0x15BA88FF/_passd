use passd::commands::find_items;
use std::fs;
use std::io;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_find_item_no_results() -> io::Result<()> {
    let temp_dir = tempdir()?;

    let filter = |p: &Path| p.is_file() && p.extension().map(|ext| ext == "json").unwrap_or(false);
    let results = find_items(temp_dir.path(), &filter)?;

    assert!(results.is_empty());

    Ok(())
}

#[test]
fn test_find_item_files_only() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path();

    let file2 = root_path.join("file2.rs");
    let file1 = root_path.join("file1.txt");
    let file3 = root_path.join("file3.md");
    let sub_dir = root_path.join("sub_dir");
    let sub_file = sub_dir.join("sub_file.rs");

    fs::create_dir(&sub_dir)?;
    fs::File::create(&file1)?;
    fs::File::create(&file2)?;
    fs::File::create(&file3)?;
    fs::File::create(&sub_file)?;

    let filter = |p: &Path| p.is_file() && p.extension().map(|ext| ext == "rs").unwrap_or(false);
    let results = find_items(root_path, &filter)?;

    let results: Vec<String> = results
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(results.len(), 2);
    assert!(results.contains(&file2.to_string_lossy().to_string()));
    assert!(results.contains(&sub_file.to_string_lossy().to_string()));

    Ok(())
}

#[test]
fn test_find_item_directories_only() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path();

    let sub_dir1 = root_path.join("sub_dir1");
    let sub_dir2 = root_path.join("sub_dir2");

    fs::create_dir(&sub_dir1)?;
    fs::create_dir(&sub_dir2)?;

    let filter = |p: &Path| p.is_dir();
    let results = find_items(root_path, &filter)?;

    let results: Vec<String> = results
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(results.len(), 2);
    assert!(results.contains(&sub_dir1.to_string_lossy().to_string()));
    assert!(results.contains(&sub_dir2.to_string_lossy().to_string()));

    Ok(())
}

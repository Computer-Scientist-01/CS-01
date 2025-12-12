use cs_01::commands::init::init;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_init_standard() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir_path).unwrap();

    let result = init(false, "main");
    assert!(result.is_ok());

    let cs01_dir = dir_path.join(".CS01");
    assert!(cs01_dir.exists(), ".CS01 dir not found");
    assert!(cs01_dir.join("config").exists(), "config not found");

    let head = fs::read_to_string(cs01_dir.join("HEAD")).unwrap();
    assert!(head.contains("ref: refs/heads/main"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_init_bare() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(&dir_path).unwrap();

    let result = init(true, "master");
    assert!(result.is_ok());

    assert!(dir_path.join("config").exists());
    assert!(!dir_path.join(".CS01").exists());

    let head = fs::read_to_string(dir_path.join("HEAD")).unwrap();
    assert!(head.contains("ref: refs/heads/master"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_init_already_exists() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(&dir_path).unwrap();

    // First init
    init(false, "main").unwrap();
    let config_path = dir_path.join(".CS01/config");
    let meta_before = fs::metadata(&config_path).unwrap();

    // Second init
    // My init function currently returns Ok(()) and prints warning but does NOT error.
    // I should capture stdout if I want to verify warning, but functionally it should just not overwrite (timestamp check).
    std::thread::sleep(std::time::Duration::from_millis(10));
    let result = init(false, "other");
    assert!(result.is_ok());

    let meta_after = fs::metadata(&config_path).unwrap();
    assert_eq!(
        meta_before.modified().unwrap(),
        meta_after.modified().unwrap(),
        "Should not modify existing repo"
    );

    std::env::set_current_dir(original_dir).unwrap();
}

use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_init_command() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // Run the init command in the temp directory
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed:\nStdout: {}\nStderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Check if .CS01 directory exists
    let cs01_dir = root.join(".CS01");
    assert!(cs01_dir.exists());
    assert!(cs01_dir.is_dir());

    // Check if config exists
    let config_file = cs01_dir.join("config");
    assert!(config_file.exists());
    let config_content = std::fs::read_to_string(config_file).unwrap();
    assert!(config_content.contains("bare = false"));

    // Check HEAD
    let head_file = cs01_dir.join("HEAD");
    assert!(head_file.exists());
    let head_content = std::fs::read_to_string(head_file).unwrap();
    assert!(head_content.contains("ref: refs/heads/main"));

    // Check description
    let description_file = cs01_dir.join("description");
    assert!(description_file.exists());
    let desc_content = std::fs::read_to_string(description_file).unwrap();
    assert!(desc_content.contains("Unnamed repository"));

    // Check hooks
    let hooks_dir = cs01_dir.join("hooks");
    assert!(hooks_dir.exists());
    assert!(hooks_dir.join("pre-commit.sample").exists());

    // Check info/exclude
    let info_exclude = cs01_dir.join("info/exclude");
    assert!(info_exclude.exists());
    let exclude_content = std::fs::read_to_string(info_exclude).unwrap();
    assert!(exclude_content.contains("# cs01 ls-files --others --exclude-from=.cs01/info/exclude"));

    // Check objects subdirs
    assert!(cs01_dir.join("objects/info").exists());
    assert!(cs01_dir.join("objects/pack").exists());

    // Check refs/tags
    assert!(cs01_dir.join("refs/tags").exists());
}

#[test]
fn test_init_command_with_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let target_dir = root.join("my-new-repo");

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // Run the init command with a target path
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
            "my-new-repo",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed:\nStdout: {}\nStderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Check if .CS01 directory exists in the target dir
    let cs01_dir = target_dir.join(".CS01");
    assert!(cs01_dir.exists());
    assert!(cs01_dir.is_dir());

    // Check if updated config defaults exist
    let config_file = cs01_dir.join("config");
    let config_content = std::fs::read_to_string(config_file).unwrap();
    assert!(config_content.contains("bare = false"));
    assert!(config_content.contains("filemode = true"));
    assert!(config_content.contains("logallrefupdates = true"));
}

#[test]
fn test_reinit_command() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // 1. First init
    Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
        ])
        .current_dir(root)
        .output()
        .unwrap();

    // 2. Delete the config file (simulate corruption/loss)
    let config_path = root.join(".CS01/config");
    std::fs::remove_file(&config_path).unwrap();
    assert!(!config_path.exists());

    // 3. Re-run init
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
        ])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reinitialized existing standard CS01 repository"));

    // 4. Config should be restored
    assert!(config_path.exists());
}

#[test]
fn test_init_bare_command() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // Run the init command with --bare
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
            "--bare",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed:\nStdout: {}\nStderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // In bare repo, files are in root
    let config_file = root.join("config");
    assert!(config_file.exists());
    let config_content = std::fs::read_to_string(config_file).unwrap();
    assert!(config_content.contains("bare = true"));
}

#[test]
fn test_init_nested_repo_protection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // 1. Init outer repo
    Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
        ])
        .current_dir(root)
        .output()
        .expect("Failed to init outer repo");

    // 2. Try to init inner repo (should fail)
    let inner_dir = root.join("inner");
    std::fs::create_dir(&inner_dir).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
        ])
        .current_dir(&inner_dir)
        .output()
        .expect("Failed to execute nested init");

    // Should NOT be successful (we decided to bail)
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Refusing to create nested repository"));

    // Ensure no .CS01 created in inner
    assert!(!inner_dir.join(".CS01").exists());
}

#[test]
fn test_init_absolute_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let target_abs_path = root.join("abs_repo");

    // Get the path to the current project's Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = std::path::Path::new(manifest_dir).join("Cargo.toml");

    // Init using absolute path
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--",
            "init",
            target_abs_path.to_str().unwrap(),
        ])
        .current_dir(root) // Running from somewhere else
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(target_abs_path.join(".CS01").exists());
}

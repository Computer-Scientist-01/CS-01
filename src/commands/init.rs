use anyhow::{Context, Result};
use colored::*;

use crate::modules::{
    files::{WriteOptions, cs01_path, write_files_from_tree},
    repo_structure::build_repo_tree,
};

pub fn init(bare: bool, initial_branch: &str, path: &str) -> Result<()> {
    let root_path = if path == "." {
        std::env::current_dir()?
    } else {
        std::path::PathBuf::from(path)
    };

    if !root_path.exists() {
        std::fs::create_dir_all(&root_path).context("Failed to create target directory")?;
    }

    let repo_dir = if bare {
        root_path.clone()
    } else {
        root_path.join(".CS01")
    };

    // Tough Topic: Re-initialization
    // We must detect if a repo already exists to avoid overwriting critical data (like objects/HEAD),
    // but we SHOULD allow running 'init' to repair missing files (like config).
    let is_reinit = if bare {
        repo_dir.join("HEAD").exists() || repo_dir.join("objects").exists()
    } else {
        repo_dir.exists()
    };

    // Critical: Nested Repository Protection
    // We explicitly forbid creating a repository *inside* another repository (unless it's a re-init of the same repo).
    // This prevents confusing state where inner commands might accidentally affect the outer repo.
    // Critical: Nested Repository Protection
    // We explicitly forbid creating a repository *inside* another repository (unless it's a re-init of the same repo).
    // This prevents confusing state where inner commands might accidentally affect the outer repo.
    if !is_reinit && let Some(existing_root) = cs01_path(None, Some(&root_path)) {
        let existing_root = existing_root.canonicalize()?;
        let target_root = root_path.canonicalize()?;

        if existing_root != target_root {
            println!(
                "{}",
                format!(
                    "Warning: You are attempting to initialize a repository inside an existing one at {}.",
                    existing_root.display()
                )
                .yellow()
            );
            anyhow::bail!(
                "Refusing to create nested repository inside {}",
                existing_root.display()
            );
        }
    }

    // Build the repository structure (config, HEAD, etc.)
    let tree_to_write = build_repo_tree(bare, initial_branch)?;

    let opts = WriteOptions {
        dir_perms: 0o755,
        overwrite: false,
        dry_run: false,
    };

    // Note: write_files_from_tree helps us implement safe re-init because
    // `overwrite: false` ensures we don't blow away existing HEAD/refs.
    write_files_from_tree(&tree_to_write, &root_path, &opts)?;

    let repo_type = if bare { "bare" } else { "standard" };

    let folder_note = if bare {
        "".to_string()
    } else {
        " (with .CS01 directory)"
            .truecolor(128, 128, 128)
            .to_string() // gray
    };

    let (action, state) = if is_reinit {
        ("Reinitialized", "existing")
    } else {
        ("Initialized", "empty")
    };

    let display_path = root_path.canonicalize().unwrap_or(root_path);

    let message = format!(
        "{} {} {} CS01 repository in {}{}",
        action,
        state,
        repo_type,
        display_path.display(),
        folder_note
    );

    println!("{}", message.green());

    Ok(())
}

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
pub enum TreeNode {
    File(String),
    Directory(HashMap<String, TreeNode>),
}

/// Checks if the current working directory (or target directory) is inside a CS01 repository root.
pub fn in_repo(cwd: Option<&Path>) -> bool {
    cs01_path(None, cwd).is_some()
}

/// Computes the full path relative to the CS01 root.
/// Scans upward for `config` (with [core]) or `.CS01` directory.
pub fn cs01_path(relative_path: Option<&str>, start_dir: Option<&Path>) -> Option<PathBuf> {
    // TODO:- thats little bit tough part for me so i need to do practice
    let start_dir = start_dir
        .map(|p: &Path| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let relative_path = relative_path.unwrap_or("");
    let mut current_dir = start_dir.clone();

    // Loop until we hit root
    loop {
        let potential_config = current_dir.join("config");
        let potential_cs01 = current_dir.join(".CS01");

        // Check for config file
        if potential_config.exists() && potential_config.is_file() {
            if let Ok(content) = fs::read_to_string(&potential_config) {
                if content.trim().starts_with("[core]") {
                    // Found it
                    return Some(current_dir.join(relative_path));
                }
            }
        }

        // Check for .CS01 dir
        if potential_cs01.exists() && potential_cs01.is_dir() {
            return Some(current_dir.join(relative_path));
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

pub struct WriteOptions {
    pub dir_perms: u32,
    pub overwrite: bool,
    pub dry_run: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            dir_perms: 0o755,
            overwrite: true,
            dry_run: false,
        }
    }
}

/// Recursively writes a nested tree object to disk.
pub fn write_files_from_tree(tree: &TreeNode, prefix: &Path, options: &WriteOptions) -> Result<()> {
    if options.dry_run {
        println!("[DRY-RUN] Processing at {:?}", prefix);
    }

    match tree {
        TreeNode::File(content) => {
            if !options.overwrite && prefix.exists() {
                // Skip
                return Ok(());
            }
            if options.dry_run {
                println!(
                    "[DRY-RUN] Write file {:?} ({} bytes)",
                    prefix,
                    content.len()
                );
            } else {
                if let Some(parent) = prefix.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(prefix, content)
                    .with_context(|| format!("Failed to write {:?}", prefix))?;
            }
        }
        TreeNode::Directory(children) => {
            if !prefix.exists() {
                if options.dry_run {
                    println!("[DRY-RUN] Create dir {:?}", prefix);
                } else {
                    fs::create_dir_all(prefix)
                        .with_context(|| format!("Failed to create dir {:?}", prefix))?;
                }
            }

            for (name, node) in children {
                write_files_from_tree(node, &prefix.join(name), options)?;
            }
        }
    }

    Ok(())
}

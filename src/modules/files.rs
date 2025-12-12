use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub enum TreeNode {
    File(String),
    Directory(HashMap<String, TreeNode>),
}

/// Helper to check if `cwd` is within a CS01 repo.
pub fn in_repo(cwd: Option<&Path>) -> bool {
    cs01_path(None, cwd).is_some()
}

/// Locates the root of the CS01 repository.
///
/// Critical: This function traverses UPDWARDS from `start_dir`.
/// It identifies the root by looking for:
/// 1. `.CS01` directory (Standard)
/// 2. `config` file containing `[core]` section (Bare)
pub fn cs01_path(relative_path: Option<&str>, start_dir: Option<&Path>) -> Option<PathBuf> {
    let start_dir = start_dir
        .map(|p: &Path| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let relative_path = relative_path.unwrap_or("");
    let mut current_dir = start_dir.clone();

    loop {
        let potential_config = current_dir.join("config");
        let potential_cs01 = current_dir.join(".CS01");

        if potential_config.is_file()
            && let Ok(content) = fs::read_to_string(&potential_config)
            && content.trim().starts_with("[core]")
        {
            return Some(current_dir.join(relative_path));
        }

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

/// Writes a `TreeNode` structure to disk.
///
/// Note: Recursively handles directory creation.
/// If `options.overwrite` is false, it preserves existing files.
pub fn write_files_from_tree(tree: &TreeNode, prefix: &Path, options: &WriteOptions) -> Result<()> {
    if options.dry_run {
        println!("[DRY-RUN] Processing at {:?}", prefix);
    }

    match tree {
        TreeNode::File(content) => {
            if !options.overwrite && prefix.exists() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_files_from_tree() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let mut children = HashMap::new();
        children.insert("file.txt".to_string(), TreeNode::File("hello".to_string()));
        let tree = TreeNode::Directory(children);

        let opts = WriteOptions {
            overwrite: true,
            ..Default::default()
        };

        write_files_from_tree(&tree, root, &opts).unwrap();

        let file_path = root.join("file.txt");
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), "hello");
    }

    #[test]
    fn test_cs01_path_no_repo() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        assert!(cs01_path(None, Some(root)).is_none());
    }

    #[test]
    fn test_write_files_from_tree_dry_run() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let mut children = HashMap::new();
        children.insert("file.txt".to_string(), TreeNode::File("hello".to_string()));
        let tree = TreeNode::Directory(children);

        let opts = WriteOptions {
            dry_run: true,
            ..Default::default()
        };

        // Execution should succeed
        write_files_from_tree(&tree, root, &opts).unwrap();

        // But no file should be created
        let file_path = root.join("file.txt");
        assert!(!file_path.exists());
    }

    #[test]
    fn test_cs01_path_deep_resolution() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create a fake repo structure: root/.CS01
        let cs01_dir = root.join(".CS01");
        fs::create_dir(&cs01_dir).unwrap();

        // Create deep path: root/a/b/c
        let deep_path = root.join("a/b/c");
        fs::create_dir_all(&deep_path).unwrap();

        // Check if resolving from deep path finds the root
        let found = cs01_path(None, Some(&deep_path));
        assert!(found.is_some());
        assert_eq!(
            found.unwrap().canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }
}

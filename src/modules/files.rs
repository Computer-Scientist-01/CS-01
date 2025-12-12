use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// This enum represents a structure of files and folders in memory.
/// - `File`: Contains the text content of a file.
/// - `Directory`: Contains a list of other TreeNodes (files or subdirectories).
pub enum TreeNode {
    File(String),
    Directory(HashMap<String, TreeNode>),
}

/// Helper function to check if we are currently inside a CS01 repository.
/// It returns true if it finds a `.CS01` folder or a valid `config` file in the current or parent directories.
pub fn in_repo(cwd: Option<&Path>) -> bool {
    cs01_path(None, cwd).is_some()
}

/// This function tries to find the root of the CS01 repository.
/// It starts from the current directory (or `start_dir`) and moves upwards.
/// It looks for:
/// 1. A `config` file that starts with `[core]` (indicating a bare repo).
/// 2. A `.CS01` directory (indicating a normal repo).
pub fn cs01_path(relative_path: Option<&str>, start_dir: Option<&Path>) -> Option<PathBuf> {
    // Use the provided start directory or default to the current working directory.
    let start_dir = start_dir
        .map(|p: &Path| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let relative_path = relative_path.unwrap_or("");
    let mut current_dir = start_dir.clone();

    // Loop upwards through parent directories until we hit the root (/) or find the repo.
    loop {
        let potential_config = current_dir.join("config");
        let potential_cs01 = current_dir.join(".CS01");

        // Check if there is a 'config' file (likely a bare repo).
        if potential_config.exists() && potential_config.is_file() {
            if let Ok(content) = fs::read_to_string(&potential_config) {
                // If it looks like a valid git/cs01 config, we found the root.
                if content.trim().starts_with("[core]") {
                    return Some(current_dir.join(relative_path));
                }
            }
        }

        // Check if there is a '.CS01' directory (standard repo).
        if potential_cs01.exists() && potential_cs01.is_dir() {
            return Some(current_dir.join(relative_path));
        }

        // Move to the parent directory. If we can't move up anymore, we stop.
        if !current_dir.pop() {
            break;
        }
    }

    // We didn't find a repository.
    None
}

/// Options for writing files to the disk.
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

/// This function takes a `TreeNode` (our memory representation of files) and writes it to the actual disk.
/// It recursively creates directories and files.
pub fn write_files_from_tree(tree: &TreeNode, prefix: &Path, options: &WriteOptions) -> Result<()> {
    if options.dry_run {
        println!("[DRY-RUN] Processing at {:?}", prefix);
    }

    match tree {
        TreeNode::File(content) => {
            // If overwrite is disabled and file exists, skip it.
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
                // Ensure the parent directory exists before writing the file.
                if let Some(parent) = prefix.parent() {
                    fs::create_dir_all(parent)?;
                }
                // Write the content to the file.
                fs::write(prefix, content)
                    .with_context(|| format!("Failed to write {:?}", prefix))?;
            }
        }
        TreeNode::Directory(children) => {
            // If the directory doesn't exist, create it.
            if !prefix.exists() {
                if options.dry_run {
                    println!("[DRY-RUN] Create dir {:?}", prefix);
                } else {
                    fs::create_dir_all(prefix)
                        .with_context(|| format!("Failed to create dir {:?}", prefix))?;
                }
            }

            // Recursively process all children (files and subdirectories) inside this directory.
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
        assert_eq!(found.unwrap().canonicalize().unwrap(), root.canonicalize().unwrap());
    }
}

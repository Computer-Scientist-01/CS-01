use std::collections::HashMap;

use anyhow::Result;
use colored::*;
use serde_json::json;

use crate::modules::{
    config::obj_to_str,
    files::{TreeNode, WriteOptions, cs01_path, write_files_from_tree},
};

/// This particular function initializes a new CS01 repository.
/// It creates the hidden .CS01 directory (or bare structure), the config file, and the initial branches.
pub fn init(bare: bool, initial_branch: &str, path: &str) -> Result<()> {
    // Resolve the target directory
    let root_path = if path == "." {
        std::env::current_dir()?
    } else {
        std::path::PathBuf::from(path)
    };

    // Create the directory if it doesn't exist
    if !root_path.exists() {
        std::fs::create_dir_all(&root_path)?;
    }

    // Determine the path where the repo metadata lives (.CS01 or root for bare)
    let repo_dir = if bare {
        root_path.clone()
    } else {
        root_path.join(".CS01")
    };

    // Check if re-initializing
    let is_reinit = if bare {
        // For bare, check if HEAD or objects exists (config might be missing if we are recovering)
        repo_dir.join("HEAD").exists() || repo_dir.join("objects").exists()
    } else {
        // For standard, check if .CS01 dir exists
        repo_dir.exists()
    };

    // Check for nested repository (if not re-initializing)
    if !is_reinit {
        // Check if we are inside a repo (searching upwards from root_path)
        if let Some(existing_root) = cs01_path(None, Some(&root_path)) {
            // We found a repo.
            // If the found repo root is NOT the same as our target root, it means we are inside it.
            // (Note: cs01_path returns the path to the root of the working tree)
            
            // To be precise: cs01_path returns the derived path. 
            // If we are at /a/b/c and /a is a repo, cs01_path returns /a/b/c (if relative_path is None it returns current_dir joined with nothing, wait).
            // Let's check files.rs: "return Some(current_dir.join(relative_path));"
            // If we pass relative_path=None, it returns current_dir (which is the root found).
            
            // Wait, cs01_path implementation:
            // It bubbles UP. When it finds .CS01 in `current_dir`, it returns `current_dir.join(relative_path)`.
            // So if `cs01_path(None, ...)` returns something, it is the ROOT of the repo found.
            
            // Actually, looking at `files.rs`, `cs01_path` returns `current_dir.join(relative_path)`.
            // `current_dir` is the directory CONTAINING .CS01.
            
            // So if I am at `/repo/subdir`, and I run `init .`, `root_path` is `/repo/subdir`.
            // `cs01_path(None, Some(/repo/subdir))` will find `.CS01` at `/repo`.
            // It will return `/repo`.
            
            // So `existing_root` is `/repo`.
            // `root_path` is `/repo/subdir`.
            // Since `/repo` != `/repo/subdir`, we are nesting.
            
            // Case 2: I am at `/repo`, run `init .`. settings `root_path` = `/repo`.
            // `cs01_path` finds `.CS01` at `/repo`. Returns `/repo`.
            // `existing_root` == `root_path`. This is re-init (which we handled above with `is_reinit` check).
            
            // However, `is_reinit` checked for strict existence of `.CS01` in `repo_dir`.
            // `cs01_path` is more robust.
            
            let existing_root = existing_root.canonicalize()?;
            let target_root = root_path.canonicalize()?;
             
            if existing_root != target_root {
                 println!(
                    "{}",
                    format!("Warning: You are attempting to initialize a repository inside an existing one at {}.", existing_root.display()).yellow()
                );
                // For now, we just warn (like git sometimes does), but maybe we should stop?
                // The plan said "prints a warning/error". Let's error to be safe for now, or just warn.
                // Git usually allows it but warns about embedded.
                // Let's return Err to make it "safe" as requested.
                // "verify that running ... does not create a nested ... directory".
                // So we must stop.
                anyhow::bail!("Refusing to create nested repository inside {}", existing_root.display());
            }
        }
    }

    // We'll point the main branch to this reference.
    let branch_ref = format!("ref: refs/heads/{}", initial_branch);

    // 2. Prepare the configuration content.
    // We create a JSON structure for the initial config, including "bare" status.
    let config_json = json!({
        "core": {
            "": {
                "bare": bare,
                "repositoryformatversion": 0,
                "filemode": true,
                "logallrefupdates": true
            }
        }
    });

    // Convert that JSON to the ini-style string format our system uses.
    let config_content = obj_to_str(&config_json)?;

    // 3. Construct existing internal file structure in memory first.
    let mut internal_structure = HashMap::new();

    // HEAD file: points to the current active branch (e.g., "ref: refs/heads/main").
    internal_structure.insert(
        "HEAD".to_string(),
        TreeNode::File(format!("{}\n", branch_ref)),
    );

    // config file: contains the repository settings we generated above.
    internal_structure.insert("config".to_string(), TreeNode::File(config_content));

    // description file
    internal_structure.insert(
        "description".to_string(),
        TreeNode::File(
            "Unnamed repository; edit this file 'description' to name the repository.\n".to_string(),
        ),
    );

    // hooks directory with sample files
    let mut hooks = HashMap::new();
    let sample_hooks = vec![
        "applypatch-msg.sample",
        "commit-msg.sample",
        "fsmonitor-watchman.sample",
        "post-update.sample",
        "pre-applypatch.sample",
        "pre-commit.sample",
        "pre-merge-commit.sample",
        "prepare-commit-msg.sample",
        "pre-push.sample",
        "pre-rebase.sample",
        "pre-receive.sample",
        "push-to-checkout.sample",
        "sendemail-validate.sample",
        "update.sample",
    ];
    for hook in sample_hooks {
        hooks.insert(hook.to_string(), TreeNode::File("".to_string()));
    }
    internal_structure.insert("hooks".to_string(), TreeNode::Directory(hooks));

    // info directory
    let mut info = HashMap::new();
    info.insert(
        "exclude".to_string(),
        TreeNode::File(
            "# cs01 ls-files --others --exclude-from=.cs01/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n".to_string(),
        ),
    );
    internal_structure.insert("info".to_string(), TreeNode::Directory(info));

    // objects directory: this will store our file blobs and trees.
    let mut objects = HashMap::new();
    objects.insert("info".to_string(), TreeNode::Directory(HashMap::new()));
    objects.insert("pack".to_string(), TreeNode::Directory(HashMap::new()));
    internal_structure.insert("objects".to_string(), TreeNode::Directory(objects));

    // refs structure: stores branch pointers.
    // We create refs/heads/[initial_branch] which also points to the same ref (a bit recursive for init).
    let mut heads = HashMap::new();
    heads.insert(initial_branch.to_string(), TreeNode::File(branch_ref));

    let mut refs = HashMap::new();
    refs.insert("heads".to_string(), TreeNode::Directory(heads));
    refs.insert("tags".to_string(), TreeNode::Directory(HashMap::new()));

    internal_structure.insert("refs".to_string(), TreeNode::Directory(refs));

    // 4. Decide where to put this structure.
    // If it's a "bare" repo, these files go directly in the current directory.
    // If it's a standard repo, they go inside a ".CS01" hidden directory.
    let tree_to_write = if bare {
        TreeNode::Directory(internal_structure)
    } else {
        let mut root = HashMap::new();
        root.insert(".CS01".to_string(), TreeNode::Directory(internal_structure));
        TreeNode::Directory(root)
    };

    let opts = WriteOptions {
        dir_perms: 0o755,
        overwrite: false,
        dry_run: false,
    };

    // 5. Actually write all the files and folders to disk.
    // Note: overwrite=false ensures we don't blow away existing HEAD/refs if re-initializing,
    // but missing files (like a deleted config) will be restored.
    write_files_from_tree(&tree_to_write, &root_path, &opts)?;

    let repo_type = if bare { "bare" } else { "standard" };

    // Construct the success message with partial coloring to look nice.
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

    // Make path absolute for display if possible, otherwise use what we have
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

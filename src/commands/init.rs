
use crate::modules::config::obj_to_str;
use crate::modules::files::{TreeNode, write_files_from_tree, WriteOptions, in_repo};
use serde_json::json;
use std::collections::HashMap;
use anyhow::Result;

pub fn init(bare: bool, initial_branch: &str) -> Result<()> {
    if in_repo(None) {
        println!("CS01 repository already exists in this directory.");
        return Ok(());
    }

    let branch_ref = format!("ref: refs/heads/{}", initial_branch);

    // Prepare structure
    
    // config content
    let config_json = json!({
        "core": {
            "": {
                "bare": bare,
                "repositoryformatversion": 0
            }
        }
    });

    let config_content = obj_to_str(&config_json)?;
    
    // Construct internal structure (HEAD, config, objects, refs)
    let mut internal_structure = HashMap::new();
    
    // HEAD: "ref: refs/heads/main\n"
    internal_structure.insert("HEAD".to_string(), TreeNode::File(format!("{}\n", branch_ref)));
    
    // config
    internal_structure.insert("config".to_string(), TreeNode::File(config_content));
    
    // objects (empty dir)
    internal_structure.insert("objects".to_string(), TreeNode::Directory(HashMap::new()));
    
    // refs/heads/main
    let mut heads = HashMap::new();
    // Replicating TS behavior where refs/heads/main content is "ref: refs/heads/main"
    heads.insert(initial_branch.to_string(), TreeNode::File(branch_ref));
    
    let mut refs = HashMap::new();
    refs.insert("heads".to_string(), TreeNode::Directory(heads));
    
    internal_structure.insert("refs".to_string(), TreeNode::Directory(refs));
    
    let tree_to_write = if bare {
        TreeNode::Directory(internal_structure)
    } else {
        let mut root = HashMap::new();
        root.insert(".CS01".to_string(), TreeNode::Directory(internal_structure));
        TreeNode::Directory(root)
    };

    let cwd = std::env::current_dir()?;
    let opts = WriteOptions {
        dir_perms: 0o755,
        overwrite: false,
        dry_run: false,
    };
    
    write_files_from_tree(&tree_to_write, &cwd, &opts)?;
    
    let repo_type = if bare { "bare" } else { "standard" };
    let folder_note = if bare { "" } else { " (with .CS01 directory)" };
    
    println!("Initialized empty {} CS01 repository in {}{}", repo_type, cwd.display(), folder_note);
    
    Ok(())
}

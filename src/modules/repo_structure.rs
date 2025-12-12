use std::collections::HashMap;

use anyhow::Result;
use serde_json::json;

use crate::modules::{config::obj_to_str, files::TreeNode};

/// Generates the directory structure for a new CS01 repository.
///
/// Returns a `TreeNode` representing the entire file hierarchy.
/// If `bare` is true, returns the structure directly (config, HEAD, etc. at top level).
/// If `bare` is false, wraps the structure in a `.CS01` directory.
pub fn build_repo_tree(bare: bool, initial_branch: &str) -> Result<TreeNode> {
    let branch_ref = format!("ref: refs/heads/{}", initial_branch);

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

    let config_content = obj_to_str(&config_json)?;

    let mut internal_structure = HashMap::new();

    internal_structure.insert(
        "HEAD".to_string(),
        TreeNode::File(format!("{}\n", branch_ref)),
    );

    internal_structure.insert("config".to_string(), TreeNode::File(config_content));

    internal_structure.insert(
        "description".to_string(),
        TreeNode::File(
            "Unnamed repository; edit this file 'description' to name the repository.\n"
                .to_string(),
        ),
    );

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

    let mut info = HashMap::new();
    info.insert(
        "exclude".to_string(),
        TreeNode::File(
            "# cs01 ls-files --others --exclude-from=.cs01/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n".to_string(),
        ),
    );
    internal_structure.insert("info".to_string(), TreeNode::Directory(info));

    let mut objects = HashMap::new();
    objects.insert("info".to_string(), TreeNode::Directory(HashMap::new()));
    objects.insert("pack".to_string(), TreeNode::Directory(HashMap::new()));
    internal_structure.insert("objects".to_string(), TreeNode::Directory(objects));

    let mut heads = HashMap::new();
    heads.insert(initial_branch.to_string(), TreeNode::File(branch_ref));

    let mut refs = HashMap::new();
    refs.insert("heads".to_string(), TreeNode::Directory(heads));
    refs.insert("tags".to_string(), TreeNode::Directory(HashMap::new()));

    internal_structure.insert("refs".to_string(), TreeNode::Directory(refs));

    if bare {
        Ok(TreeNode::Directory(internal_structure))
    } else {
        let mut root = HashMap::new();
        root.insert(".CS01".to_string(), TreeNode::Directory(internal_structure));
        Ok(TreeNode::Directory(root))
    }
}

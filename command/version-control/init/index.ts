import fs from "fs";

import config from "../../../modules/Config-module";
import { writeFilesFromTree, inRepo } from "../../../modules/Files-module/files.module";
import type { TreeNode, WriteOptions } from "../../../modules/Files-module/files.module";

export interface InitOptions {
    bare?: boolean;
    initialBranch?: string;
}

interface CS01Structure extends Record<string, TreeNode> {
    readonly HEAD: string;
    readonly config: string;
    readonly objects: Record<string, TreeNode>; 
    readonly refs: {
        readonly heads: Record<string, string>; 
    };
}

/**
 * Initializes the current directory as a new CS01 repository.
 * Aborts if already in a repo. Supports bare mode and custom initial branch.
 * @param options - Initialization options.
 * @returns True if successful, false if already a repo.
 * @throws Error on FS failures or invalid options.
 * @example
 * init({ bare: true }); // Bare repo
 * init({ initialBranch: 'develop' }); // Non-bare with custom branch
 */
export default function init(options: InitOptions = {}): boolean {
    const { bare = false, initialBranch = 'main' } = options;

    if (inRepo()) {
        console.warn('CS01 repository already initialized here.');
        return false;
    }

    if (typeof bare !== 'boolean') {
        throw new Error('bare must be a boolean');
    }

    const branchRef = `ref: refs/heads/${initialBranch}`;
    const cs01Structure: CS01Structure = {
        HEAD: `ref: refs/heads/${initialBranch}\n`,
        config: config.objToStr({
            core: {
                '': {
                    bare, // true/false
                    repositoryformatversion: 0, // Add Git-like version for future-proofing
                },
            },
        }),
        objects: {},
        refs: {
            heads: {
                [initialBranch]: branchRef,
            },
        },
    };

  
    const treeToWrite: TreeNode = bare ? cs01Structure : { '.CS01': cs01Structure };
    const writeOpts: WriteOptions = {
        dirPerms: 0o755,
        overwrite: false, // Prevent accidental overwrites
        onError: (err, path) => {
            throw new Error(`Init failed at ${path}: ${err.message}`);
        },
    };

    try {
        if (!fs.existsSync(process.cwd()) || !fs.statSync(process.cwd()).isDirectory()) {
            throw new Error('Current directory is not a valid writable directory.');
        }

        writeFilesFromTree(treeToWrite, process.cwd(), writeOpts);

        // Success feedback
        const repoType = bare ? 'bare' : 'standard';
        console.log(
            `Initialized empty ${repoType} CS01 repository in ${process.cwd()}${bare ? '' : ' with .CS01 dir'}.`
        );
        return true;
    } catch (error) {
        console.error(`Failed to initialize CS01 repository: ${error}`);
        throw error;
    }
}
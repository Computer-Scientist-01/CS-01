import fs from "fs";
import chalk from "chalk";
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
 * Initializes the current directory as a CS01 repository.
 * Handles bare and standard repos and custom initial branches.
 * @param options - Repository initialization options.
 * @returns True if successful, false if already a repo.
 * @throws Error on file system or invalid options.
 */
export default function init(options: InitOptions = {}): boolean {
    const { bare = false, initialBranch = 'main' } = options;

    // Basic checks
    if (inRepo()) {
        console.warn(chalk.yellow('CS01 repository already exists in this directory.'));
        return false;
    }
    if (typeof bare !== 'boolean') {
        throw new Error('Option "bare" must be boolean.');
    }
    if (!initialBranch || typeof initialBranch !== 'string' || initialBranch.trim().length === 0) {
        throw new Error('Option "initialBranch" must be a non-empty string.');
    }
    const branchRef = `ref: refs/heads/${initialBranch}`;

    // Prepare structure
    const cs01Structure: CS01Structure = {
        HEAD: `${branchRef}\n`,
        config: config.objToStr({
            core: {
                '': {
                    bare,
                    repositoryformatversion: 0,
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
        overwrite: false,
        onError: (err, path) => {
            throw new Error(`Initialization failed at ${path}: ${err.message}`);
        },
    };

    try {
        // Directory validation
        const cwd = process.cwd();
        if (!fs.existsSync(cwd) || !fs.statSync(cwd).isDirectory()) {
            throw new Error('Current directory is not valid or writable.');
        }

        writeFilesFromTree(treeToWrite, cwd, writeOpts);


        const repoType = bare ? 'bare' : 'standard';
        const folderNote = bare ? '' : chalk.gray(' (with .CS01 directory)');
        console.log(
            chalk.green(`Initialized empty ${repoType} CS01 repository in ${cwd}${folderNote}`)
        );
        return true;
    } catch (error) {
        console.error(
            chalk.redBright(`Failed to initialize CS01 repository:`) +
            '\n' + (error instanceof Error ? error.message : String(error))
        );
        throw error;
    }
}

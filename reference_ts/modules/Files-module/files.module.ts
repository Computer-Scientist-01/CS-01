import fs from "fs";
import nodePath from "path";

// Cache for the detected repository root to avoid unnecessary filesystem traversal on repeat checks.
const repoRootCache: { root?: string } = {};

/**
 * Checks if the current working directory (or target directory) is inside a CS01 repository root.
 * Detection is based on the existence of a valid config or .CS01 dir.
 * @param cwd - Optional directory to check (defaults to process.cwd()).
 * @returns True if a valid CS01 repo root is found, false otherwise.
 */
export function inRepo(cwd: string = process.cwd()): boolean {
    return CS01Path(cwd) !== undefined;
}

// TODO: Learn more about UTF-8 and other options of readFileSync for broader file compatibility.
/**
 * Reads the contents of a file as a UTF-8 string.
 * Gracefully returns undefined if the file doesn't exist, can't be read, or input is not a string.
 * @param path - The file path to read.
 * @throws Error only on actual I/O failures (e.g., permissions), not on "file not found."
 */
export function read(path: string): string | undefined {
    if (typeof path !== 'string') { return undefined; }
    try {
        if (fs.existsSync(path)) {
            // Note: fs.readFileSync returns a string if encoding is set.
            return fs.readFileSync(path, 'utf-8');
        }
    } catch (error) {
        console.warn(`Failed to read ${path}: ${error}`);
    }
    return undefined;
}

/**
 * Resolves a given path relative to the CS01 repository root.
 * Scans upward from startDir for a 'config' file containing [core] or a '.CS01' directory.
 * Uses an in-memory cache for efficient repeated lookups under the same root.
 * @param relativePath - Relative path to join to the root (defaults to root itself).
 * @param startDir - Optional starting directory (defaults to process.cwd()).
 * @returns The absolute full path, or undefined if not inside a repo.
 * @throws Error for invalid inputs or critical scan failures.
 */
export function CS01Path(relativePath?: string, startDir: string = process.cwd()) {
    if (typeof relativePath !== 'string') relativePath = '';
    if (typeof startDir !== 'string') {
        throw new Error('startDir must be a string');
    }
    // Use cache if already found and current dir is a subdir or the root itself
    if (repoRootCache.root && startDir.startsWith(repoRootCache.root)) {
        return nodePath.join(repoRootCache.root, relativePath);
    }

    let currentDir = startDir;
    // Traverse up to filesystem root looking for repo indicators.
    while (currentDir !== '/') {
        const potentialConfig = nodePath.join(currentDir, 'config');
        const potentialCS01 = nodePath.join(currentDir, '.CS01');
        // If a config file exists, check that it begins with [core]
        if (fs.existsSync(potentialConfig) && fs.statSync(potentialConfig).isFile()) {
            const configContent = read(potentialConfig);
            // Only accept if config starts with '[core]' (matches Git/INI style sections)
            if (configContent && /^\[core\]/.test(configContent.trim())) {
                repoRootCache.root = currentDir;
                return nodePath.join(currentDir, relativePath);
            }
        }
        // If a .CS01 directory exists, accept as repo root
        if (fs.existsSync(potentialCS01) && fs.statSync(potentialCS01).isDirectory()) {
            repoRootCache.root = currentDir;
            return nodePath.join(currentDir, relativePath);
        }
        // Traverse up one directory level
        currentDir = nodePath.dirname(currentDir);
    }
    return undefined;
}

/**
 * Represents a node in a file tree: either a file (content as string) or a directory (recursive mapping).
 * Allows describing an arbitrary nested structure for batch writing.
 */
export type TreeNode = string | { [name: string]: TreeNode };

/**
 * Write options for batch file tree writing.
 * - dirPerms: Directory permissions (default 0o755)
 * - overwrite: If false, skip existing files
 * - onError: Callback for custom error handling
 * - dryRun: If true, print actions but do not write anything to disk
 */
export interface WriteOptions {
    dirPerms?: number;
    overwrite?: boolean;
    onError?: (error: Error, path: string) => void;
    dryRun?: boolean;
}

/**
 * Recursively writes a nested tree object to disk, creating files and directories as needed.
 * Handles empty directories. Each key is either a filename (with string contents) or a subdirectory.
 * Optionally runs in dry-run mode for previewing changes.
 * @param tree - The nested structure to write (file/directory TreeNode).
 * @param prefix - Root filesystem path at which to write.
 * @param options - Control permissions, overwriting, error handling, and dry-run mode.
 * @throws Error on fatal failures unless onError is provided.
 * @example
 *   writeFilesFromTree({
 *     'hello.txt': 'world',
 *     'dir/': { 'nested.txt': 'deep' }
 *   }, '/tmp/repo');
 */
export function writeFilesFromTree(
    tree: TreeNode,
    prefix: string,
    options: WriteOptions = {}
): void {
    const {
        dirPerms = 0o755,
        overwrite = true,
        onError = (err, path) => { throw err; },
        dryRun = false
    } = options;

    // Validate inputs; empty object creates directory but no files.
    if (typeof prefix !== 'string' || prefix.length === 0) {
        throw new Error('prefix must be a non-empty string');
    }
    if (typeof tree !== 'object' || tree === null) {
        throw new Error('tree must be a non-empty object');
    }
    if (Object.keys(tree).length === 0) {
        // If the tree is empty, just create the directory (if it doesn't exist)
        try {
            if (!fs.existsSync(prefix)) {
                if (dryRun) {
                    console.log(`[DRY-RUN] Would create empty dir: ${prefix}`);
                } else {
                    fs.mkdirSync(prefix, { recursive: true, mode: dirPerms });
                }
            }
            return;
        } catch (error) {
            onError(new Error(`Failed to create empty dir ${prefix}: ${error}`), prefix);
        }
    }

    // Ensure the target directory exists
    try {
        if (!fs.existsSync(prefix)) {
            if (dryRun) {
                console.log(`[DRY-RUN] Would create dir: ${prefix}`);
            } else {
                fs.mkdirSync(prefix, { recursive: true, mode: dirPerms });
            }
        }
    } catch (error) {
        onError(new Error(`Failed to create prefix ${prefix}: ${error}`), prefix);
    }

    // Recursively descend into each node entry
    Object.entries(tree).forEach(([name, node]) => {
        const fullPath = nodePath.join(prefix, name);

        if (typeof node === 'string') {
            // It's a file: create or skip depending on overwrite flag
            try {
                if (!overwrite && fs.existsSync(fullPath)) {
                    console.warn(`Skipping existing file: ${fullPath}`);
                    return;
                }
                if (dryRun) {
                    console.log(`[DRY-RUN] Would write file: ${fullPath} (${node.length} bytes)`);
                } else {
                    fs.writeFileSync(fullPath, node, 'utf8');
                }
            } catch (error) {
                onError(new Error(`Failed to write ${fullPath}: ${error}`), fullPath);
            }
        } else {
            // It's a directory: make and then recursively write contents
            try {
                if (!fs.existsSync(fullPath)) {
                    if (dryRun) {
                        console.log(`[DRY-RUN] Would create dir: ${fullPath}`);
                    } else {
                        fs.mkdirSync(fullPath, { recursive: true, mode: dirPerms });
                    }
                }
                writeFilesFromTree(node, fullPath, options);  // Recursion step
            } catch (error) {
                onError(new Error(`Failed to create dir ${fullPath}: ${error}`), fullPath);
            }
        }
    });
}

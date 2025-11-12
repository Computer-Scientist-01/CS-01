import fs from "fs";
import nodePath from "path";

// Cache for the repo root to avoid repeated scans
const repoRootCache: { root?: string } = {};

/**
 * Checks if the current working directory is inside a CS01 repository.
 * @param cwd - current working directory.
 * @returns True if a valid repo root is found.
 */
export function inRepo(cwd: string = process.cwd()): boolean {
    return CS01Path(cwd) !== undefined;
}


// TODO: Learn more about UTF-8 and other options of readFileSync
/**
 * Reads the contents of a file at the given path as a UTF-8 string.
 * Returns `undefined` if the file doesn't exist or can't be read.
 * @param path - The file path to read.
 * @throws Error if there's an I/O error (e.g., permissions).
 */
export function read(path: string): string | undefined {
    if (typeof path !== 'string') { return undefined; }
    try {
        if (fs.existsSync(path)) {
            return fs.readFileSync(path, 'utf-8');
        }
    } catch (error) {
        console.warn(`Failed to read ${path}: ${error}`);
    }
    return undefined;
}

/**
 * Resolves a path relative to the CS01 repository root.
 * Scans upward from `startDir` for a valid repo marker (config with [core] or .CS01 dir).
 * @param relativePath - Optional path to join to the root (defaults to root itself).
 * @param startDir - Optional starting directory (defaults to process.cwd()).
 * @returns The full absolute path, or `undefined` if no repo found.
 * @throws Error if invalid inputs or scan fails critically.
 */
export function CS01Path(relativePath?: string, startDir: string = process.cwd()) {
    if (typeof relativePath !== 'string') relativePath = '';
    if (typeof startDir !== 'string') {
        throw new Error('startDir must be a string')
    }
    // Check cache first
    if (repoRootCache.root && startDir.startsWith(repoRootCache.root)) {
        return nodePath.join(repoRootCache.root, relativePath);
    }

    let currentDir = startDir;
    while (currentDir !== '/') {
        const potentialConfig = nodePath.join(currentDir, 'config');
        const potentialCS01 = nodePath.join(currentDir, '.CS01');

        // Check for valid config file (more precise regex: starts with [core])
        if (fs.existsSync(potentialConfig) && fs.statSync(potentialConfig).isFile()) {
            const configContent = read(potentialConfig);
            if (configContent && /^\[core\]/.test(configContent.trim())) {
                repoRootCache.root = currentDir;
                return nodePath.join(currentDir, relativePath);
            }
        }

        // Fallback to .CS01 dir
        if (fs.existsSync(potentialCS01) && fs.statSync(potentialCS01).isDirectory()) {
            repoRootCache.root = currentDir;
            return nodePath.join(currentDir, relativePath);
        }


        currentDir = nodePath.dirname(currentDir);
    }
    return undefined;
}

/**
 * Represents a node in the file tree: either a file (string content) or directory (nested TreeNode).
 */
export type TreeNode = string | { [name: string]: TreeNode };

export interface WriteOptions {
    dirPerms?: number;
    overwrite?: boolean;
    onError?: (error: Error, path: string) => void;
    dryRun?: boolean;
}

/**
 * Recursively writes a nested tree object to disk starting at `prefix`.
 * Tree format: { 'dir/': { 'file.txt': 'content' }, 'file.js': 'code' }.
 * Creates directories as needed; files are strings.
 * @param tree - The nested file/directory structure.
 * @param prefix - Root directory path (must exist or be creatable).
 * @param options - Config for perms, overwriting, etc.
 * @throws Error on critical failures (unless onError provided).
 * @example writeFilesFromTree({ 'hello.txt': 'world', 'dir/': { 'nested.txt': 'deep' } }, '/tmp/repo');
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

    // Validate inputs (allow empty objects for empty dirs)
    if (typeof prefix !== 'string' || prefix.length === 0) {
        throw new Error('prefix must be a non-empty string');
    }
    if (typeof tree !== 'object' || tree === null) {  // Removed || Object.keys(tree).length === 0
        throw new Error('tree must be a non-empty object');
    }
    // New: Early return if empty (create dir only if called, but since no keys, nothing to do beyond prefix)
    if (Object.keys(tree).length === 0) {
        // For empty tree: Just ensure prefix dir exists (no files/subdirs to write)
        try {
            if (!fs.existsSync(prefix)) {
                if (dryRun) {
                    console.log(`[DRY-RUN] Would create empty dir: ${prefix}`);
                } else {
                    fs.mkdirSync(prefix, { recursive: true, mode: dirPerms });
                }
            }
            return;  // No recursion needed
        } catch (error) {
            onError(new Error(`Failed to create empty dir ${prefix}: ${error}`), prefix);
        }
    }

    // Ensure prefix exists
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

    // Recurse over tree entries
    Object.entries(tree).forEach(([name, node]) => {
        const fullPath = nodePath.join(prefix, name);

        if (typeof node === 'string') {
            // File: Write content
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
            // Dir: Ensure exists, then recurse
            try {
                if (!fs.existsSync(fullPath)) {
                    if (dryRun) {
                        console.log(`[DRY-RUN] Would create dir: ${fullPath}`);
                    } else {
                        fs.mkdirSync(fullPath, { recursive: true, mode: dirPerms });
                    }
                }
                writeFilesFromTree(node, fullPath, options);  // Recurse
            } catch (error) {
                onError(new Error(`Failed to create dir ${fullPath}: ${error}`), fullPath);
            }
        }
    });
}

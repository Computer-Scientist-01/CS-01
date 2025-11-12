# CS01 Init Command

The `init` command bootstraps a new repository, setting up the essential directory structure and configuration files. This README focuses on the `init` module, detailing its capabilities, mechanics, limitations, and comprehensive test coverage.

## Description

The `init` function initializes the current working directory (or a specified path) as a new CS01 repository. It creates the core directory structure (e.g., `.CS01` for non-bare repos), generates initial configuration, and sets up references for the default branch (`main`).

- **Key Behavior**: 
  - Aborts silently (returns `false`) if the directory is already a CS01 repo (detected via `inRepo()`).
  - Supports bare repositories (no working tree; files at root).

## Features (What It Can Do)

- **Repository Initialization**:
  - Creates a standard repo with `.CS01/` containing:
    - `HEAD`: Points to `refs/heads/main`.
    - `config`: INI-formatted with `[core]` section (e.g., `bare = false`, `repositoryformatversion = 0`).
    - `objects/`: Empty directory for future object storage.
    - `refs/heads/`: Directory with initial `main` ref file.
  - For bare mode (`{ bare: true }`): Places files directly in the root (no `.CS01/`).

- **Idempotency**: Running `init` multiple times in the same dir doesn't alter existing structure.

- **Customization**:
  - `initialBranch?: string` (defaults to `'main'`).
  - Integrates with `writeFilesFromTree` for extensible tree writing (supports dry-run, perms).

## Usage

### Installation
```bash
git clone https://github.com/your-username/CS-01.git
cd CS-01
npm install  # Or bun install
```

### Basic Usage
Import and call in your script or CLI:
```typescript
import init, type { InitOptions } from './command/version-control/init';

// Standard repo
init();  // Returns true; creates .CS01/

// Bare repo
init({ bare: true });  // Returns true; files at root

// Custom branch
init({ initialBranch: 'develop' });

// In CLI (e.g., with yargs)
program.command('init')
  .option('--bare', { type: 'boolean' })
  .option('--initial-branch <name>', { default: 'main' })
  .action((options) => {
    const opts: InitOptions = { bare: options.bare, initialBranch: options.initialBranch };
    if (init(opts)) {
      console.log('CS01 repo initialized!');
    }
  });
```

- **Returns**: `boolean` – `true` on success, `false` if already initialized.
- **Throws**: On FS failures or invalid options (e.g., non-boolean `bare`).

## How It Works

1. **Detection**: Calls `inRepo()` to scan upward for `.CS01/` or valid `config` file with `[core]`. Aborts if found.
2. **Structure Building**: Constructs a `TreeNode` object:
   ```typescript
   {
     HEAD: 'ref: refs/heads/main\n',
     config: objToStr({ core: { '': { bare, repositoryformatversion: 0 } } }),
     objects: {},  // Empty dir
     refs: { heads: { main: 'ref: refs/heads/main' } }
   }
   ```
   - Wraps in `{ '.CS01': structure }` for non-bare.
3. **Writing**: Uses `writeFilesFromTree` to recursively create dirs/files with secure perms (`0o755`).
4. **Validation & Logging**: Ensures CWD is writable; logs success with repo type.

This modular flow (detection → build → write → log) ensures reliability and easy extension (e.g., add hooks).

## Limitations (What It Can't Do For Now)

- **No Interactive Prompts**: Doesn't ask for confirmation on overwrite or bare mode—flags only.
- **Limited Config**: Only basic `[core]` section; no user/email setup or advanced options (e.g., `extensions`).
- **Sync FS Only**: Uses `fs.sync` for simplicity; no async for large inits (future: `writeFilesFromTreeAsync`).
- **Branch Creation**: Sets up ref but doesn't create initial commit/tree—repo is "empty" until first commit.
- **No Global/Shared Repos**: Assumes local init; no support for worktrees or shared configs.
- **Platform-Specific**: Tested on Unix-like (Linux/macOS); Windows paths/permissions untested (use `path.win32` if needed).
- **No Undo**: Init is destructive (creates files); backup advised for experiments.

Future: Add `--template` for pre-populated trees, async mode, and more config options.

## Test Coverage

Tests use Jest (via Bun) with isolated temp directories for purity. All 4 tests pass (100% coverage for core behaviors). Below is a detailed table summarizing each test, its purpose, key assertions, and runtime notes from the latest run (November 12, 2025).

| Test Name                                                                      | Description                                                                                                                 | Key Assertions                                                                                                                                                                                       | Runtime Notes & Output                                                                                       |
| ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| **should create .CS01/ and all required dirs**                                 | Verifies standard init creates the full `.CS01` structure in a fresh dir. Tests file creation, dir setup, and return value. | - `init()` returns `true`.<br>- Dirs exist: `.CS01/objects`, `.CS01/refs`, `.CS01/refs/heads`.<br>- Files: `HEAD` content matches (`ref: refs/heads/main`), `config` INI exact.<br>Total: 7 expects. | Passed in 5ms.<br>Log: "Initialized empty standard CS01 repository in /tmp/cs01-test-4iG9tS with .CS01 dir." |
| **should not change anything if init run twice**                               | Ensures idempotency: Second call detects existing repo, returns `false`, and doesn't alter files.                           | - First `init()`: `true`, creates structure.<br>- Second `init()`: `false`.<br>- Structure unchanged (same asserts as above).<br>Total: 9 expects.                                                   | Passed in 2ms.<br>Log: First init success; second: "CS01 repository already initialized here."               |
| **should not crash when config is a directory**                                | Tests resilience: Pre-creates `config/` as dir (invalid for init), confirms `init()` doesn't throw and still sets up repo.  | - `expect(() => init()).not.toThrow()`.<br>- `.CS01` exists post-init.<br>Total: 2 expects.                                                                                                          | Passed in 1ms.<br>Log: "Initialized empty standard CS01 repository..." (ignores invalid `config/`).          |
| **bare repos > should put all CS01 files and folders in root if specify bare** | Validates bare mode: No `.CS01/`, files/dirs at root; `bare = true` in config.                                              | - `init({ bare: true })` returns `true`.<br>- Dirs exist: `objects`, `refs`, `refs/heads` (at root).<br>- Files: `HEAD` matches, `config` has `bare = true`.<br>- No `.CS01/`.<br>Total: 10 expects. | Passed in 1ms.<br>Log: "Initialized empty bare CS01 repository in /tmp/cs01-test-yPrqqt."                    |

**Test Summary**: 4/4 passed, 28 expects, 77ms total. Run with `bun test`. Coverage focuses on happy path, edges (idempotent, invalid FS), and modes (standard/bare). Full suite in `test/init.test.ts`.

## Contributing

- **Run Tests**: `bun test`.
- **Add Features**: Extend `InitOptions`; update `CS01Structure` interface.
- **Issues**: File bugs for async, templates, or Windows support.

# CS01
    
CS01 is a lightweight, educational version control system (VCS) originally implemented in TypeScript and now rewritten in **Rust**, inspired by Git. It mimics core Git behaviors like repository initialization, file staging, committing, and branching, but with a focus on simplicity, type safety, and modularity. 

This project serves as a learning resource for understanding VCS internals. The original TypeScript codebase is preserved in the `reference_ts/` directory for educational comparison.

## Status

**Current Phase**: Functionality Migration
- [x] Repository Initialization (`init`) - Fully implemented and compatible with TS version.
- [ ] File Staging (`add`) - Upcoming.
- [ ] Committing (`commit`) - Upcoming.

## Installation

### Prerequisites
- **Rust**: Ensure you have a recent version of Rust installed (via `rustup`).

### Building from Source
```bash
git clone https://github.com/Computer-Scientist-01/CS-01
cd CS-01
cargo build --release
```

## Usage

### Initialize a Repository
Initialize a new CS01 repository in the current directory:
```bash
cargo run -- init
```
This creates a template `.CS01` directory with the default configuration.

To initialize a bare repository:
```bash
cargo run -- init --bare
```

To specify a custom initial branch name:
```bash
cargo run -- init --initial-branch=master
```

## Development

### Running Tests
Run the improved integration test suite:
```bash
# Note: Single-threaded execution is required for tests that modify CWD
cargo test -- --test-threads=1
```

### TypeScript Reference
To explore or run the original TypeScript implementation:
```bash
cd reference_ts
bun install
bun test
```

## Contributing
1. Check the `reference_ts` folder to understand the expected behavior.
2. Implement the equivalent logic in Rust.
3. Write matching integration tests.

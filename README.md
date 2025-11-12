# CS01

CS01 is a lightweight, educational version control system (VCS) implemented in TypeScript, inspired by Git. It mimics core Git behaviors like repository initialization, file staging, committing, and branching, but with a focus on simplicity, type safety, and modularity. Built as a learning project, CS01 uses Node.js/Bun for runtime and Jest for testing—ideal for understanding VCS internals without the complexity of full Git.

Current status: **Prototype stage** with `init` fully implemented and tested. Upcoming: `add`, `commit` and more.

## Features

- **Git-Like Structure**: Uses `.CS01` dirs (non-bare), `objects/`, `refs/`, INI configs, and ref files.
- **Secure & Robust**: Sync FS with error handling, secure perms (`0o755`), and validation.

## Installation

1. **Clone the Repo**:
   ```bash
   git clone https://github.com/your-username/CS-01.git
   cd CS-01
   ```

2. **Install Dependencies**:
   - **With npm**: `npm install`
   - **With Bun** (recommended for speed): `bun install`

3. **Build & Test**:
   ```bash
   # Type-check
   npx tsc --noEmit
   
   # Run tests
   bun test  # Or npm test (Jest)
   ```

## Contributing

1. **Setup**: Clone, install, test.
2. **Add Features**: 
   - New command? Add to `command/version-control/`; export from `src/index.ts`.
   - Tests: `test/*.test.ts`; use temp isolation.
3. **PR Guidelines**: Branch from `main`, lint with ESLint, 100% test pass.
4. **Issues**: Report bugs (e.g., Windows paths) or requests (e.g., async FS).


*Updated: November 12, 2025*  

# Copilot Instructions for wld

## Project Overview

**wld** is a command-line tool for controlling WLED smart lights from the terminal. It's a Rust-based CLI application that allows users to manage multiple WLED devices, set brightness, and turn lights on/off.

- **Language**: Rust (edition 2021)
- **Project Type**: CLI application using Cargo
- **Runtime**: Compiled binary (cross-platform: Linux, macOS, Windows)
- **Size**: Small project (~7 Rust source files)
- **Dependencies**: clap (CLI), reqwest (HTTP), wled-json-api-library, serde/toml (config), directories (file paths)

## Build & Test Commands

### Prerequisites
- Rust 1.89.0 or later (CI uses exactly 1.89.0)
- `clippy` and `rustfmt` components must be installed: `rustup component add clippy rustfmt`

### Essential Commands (ALWAYS use in this order)

1. **Build** (from clean state):
   ```bash
   cargo build
   ```
   - Takes ~20-25 seconds for clean build
   - Incremental builds: <1 second
   - **ALWAYS run before testing changes**

2. **Test**:
   ```bash
   cargo test
   ```
   - Takes ~10-11 seconds
   - Runs 14 unit tests in `src/config.rs`
   - Runs 13 integration tests in `tests/cli_integration_tests.rs`
   - **ALWAYS run after making code changes**

3. **Format Check**:
   ```bash
   cargo fmt --check
   ```
   - Instant check
   - **ALWAYS run before committing**
   - Auto-fix with: `cargo fmt`

4. **Lint (Clippy)**:
   ```bash
   cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
   ```
   - Takes ~10-12 seconds
   - **CRITICAL**: Treats warnings as errors (`-D warnings`)
   - **ALWAYS run before committing**

5. **Cargo Check**:
   ```bash
   cargo check --locked --workspace --all-features --all-targets
   ```
   - Fast validation (~1 second after build)
   - Used in pre-commit hooks

6. **Release Build** (if needed):
   ```bash
   cargo build --release
   ```
   - Takes ~45-50 seconds
   - Output: `target/release/wld`

### Pre-commit Hooks

The repository uses pre-commit hooks (`.pre-commit-config.yaml`):
- **Format check**: `cargo fmt --all`
- **Cargo check**: with `--locked --workspace --all-features --all-targets`
- **Clippy**: with same flags as above, treating warnings as errors
- **File checks**: trailing whitespace, YAML/JSON/TOML validation, etc.
- **Codespell**: spell checking (uses `.codespellignore`)

**Note**: Pre-commit requires Python and may fail with network timeouts during initial setup. If this occurs, run the cargo commands directly as listed above.

## Project Structure

### Root Files
- `Cargo.toml` - Project manifest and dependencies
- `Cargo.lock` - Locked dependency versions (**IMPORTANT**: use `--locked` flag)
- `.gitignore` - Excludes `/target` directory
- `.pre-commit-config.yaml` - Pre-commit hook configuration
- `.codespellignore` - Words to ignore in spell checking
- `README.md` - User documentation
- `LICENSE.md` - MIT License

### Source Code (`/src`)
- `main.rs` - CLI entry point, command definitions (clap), device control logic
  - Uses `clap` with derive macros for CLI parsing
  - Commands: `add`, `delete`, `ls`, `set-default`, `on`, `off`, `brightness`
  - Main functions: `set_device_power()`, `set_device_brightness()`
  
- `config.rs` - Configuration management
  - `Config` struct: manages device list and default device
  - Stores config in `~/.wld.toml` (TOML format)
  - Methods: `load()`, `save()`, `add_device()`, `remove_device()`, `set_default()`, `get_device_ip()`
  - Contains comprehensive unit tests

### Tests (`/tests`)
- `cli_integration_tests.rs` - End-to-end CLI tests
  - Uses temporary home directories for isolation
  - Tests all CLI commands with various scenarios
  - Helper: `setup_temp_home()`, `run_command_with_temp_home()`

### GitHub Workflows (`.github/workflows`)

1. **`pre-commit.yml`** - Runs on push/PR
   - Sets up Python and Rust 1.89.0
   - Runs pre-commit checks
   - **CRITICAL**: This is what validates code style and correctness

2. **`build_and_release.yml`** - Runs on all pushes
   - Builds for multiple platforms (Linux, macOS amd64/arm64, Windows)
   - Uses Rust 1.89.0 with specific target triples
   - Runs release builds with `cargo build --release --target=<target>`
   - Creates GitHub releases on tags (v*)
   - Publishes to crates.io on tags
   - **Important**: macOS binaries are signed and notarized

3. **`copilot-setup-steps.yml`** - Workflow dispatch / test workflow
   - Basic setup validation
   - Uses stable Rust toolchain

## Configuration & Storage

- **Config file location**: `~/.wld.toml`
- **Format**: TOML with `devices` (HashMap<String, String>) and `default_device` (Option<String>)
- **First device added** automatically becomes the default
- **Deleting default device** reassigns default to next available device

## Key Implementation Details

### Device Resolution
When a command specifies a device:
1. Check if it's a saved device name → use its IP
2. Otherwise, treat it as a direct IP address
3. If no device specified, use default device
4. Error if no default set and no device specified

### Error Handling
- Functions return `Result<(), Box<dyn std::error::Error>>`
- CLI errors printed to stderr with `eprintln!("Error: {}", e)`
- Exit code 1 on error

### Testing Strategy
- Unit tests in `config.rs` using `#[cfg(test)]` modules
- Integration tests create temporary `$HOME` directories
- Tests use unique temp paths to avoid conflicts

## Common Pitfalls & Workarounds

1. **Cargo.lock changes**: Always use `--locked` flag to ensure reproducible builds
2. **Clippy warnings**: Treated as errors - must fix, cannot ignore
3. **Pre-commit network issues**: Pre-commit may timeout installing dependencies - run cargo commands manually instead
4. **Test isolation**: Integration tests require isolated `$HOME` - use the existing temp home pattern
5. **Build artifacts**: The `target/` directory can grow large (~650MB) - it's gitignored

## Making Changes

### Workflow
1. Make code changes
2. Run `cargo build` to verify compilation
3. Run `cargo test` to ensure tests pass
4. Run `cargo fmt` to auto-format
5. Run `cargo clippy --locked --workspace --all-features --all-targets -- -D warnings` to catch issues
6. Commit changes

### Adding Dependencies
1. Add to `Cargo.toml` under `[dependencies]`
2. Run `cargo build` - this updates `Cargo.lock`
3. Commit both `Cargo.toml` and `Cargo.lock`

### Adding Tests
- Unit tests: Add to `#[cfg(test)] mod tests` in the relevant module
- Integration tests: Add to `tests/cli_integration_tests.rs` following existing patterns

## Trust These Instructions

These instructions have been validated by:
- Running clean builds from scratch
- Testing all commands and documenting timings
- Verifying test execution and isolation
- Examining all workflow files
- Making and validating test code changes

**Only search for additional information if these instructions are incomplete or found to be incorrect.**

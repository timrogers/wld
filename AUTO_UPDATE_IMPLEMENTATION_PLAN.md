# Auto Update Implementation Plan for wld

## Executive Summary

This document outlines a comprehensive plan for implementing auto-update functionality in the `wld` CLI tool using the GitHub Releases API. The implementation will allow users to check for new versions and optionally self-update the binary.

## Current State Analysis

### Repository Information
- **Name**: wld
- **Owner**: timrogers
- **Current Version**: 0.0.2
- **Language**: Rust (edition 2021)
- **Repository**: https://github.com/timrogers/wld

### Current Release Structure

The project uses GitHub Actions to build and release binaries for multiple platforms:

**Supported Platforms & Binary Naming Convention:**
- Linux amd64: `wld_v{VERSION}_linux-amd64`
- Linux aarch64: `wld_v{VERSION}_linux-aarch64`
- macOS amd64: `wld_v{VERSION}_darwin-amd64`
- macOS aarch64: `wld_v{VERSION}_darwin-aarch64`
- macOS Universal: `wld_v{VERSION}_darwin-universal`
- Windows amd64: `wld_v{VERSION}_windows-amd64.exe`

**Release Workflow:**
- Tags following pattern: `v{VERSION}` (e.g., v0.0.2)
- Binaries are signed and notarized for macOS
- All releases published to GitHub Releases
- Also published to crates.io

## Recommended Approach

### Option 1: Using the `self_update` Crate (RECOMMENDED)

#### Why `self_update`?
- **Production-ready**: Well-maintained, widely used in Rust CLI ecosystem
- **Complete solution**: Handles version checking, downloading, and binary replacement
- **Multi-platform**: Supports all platforms wld targets
- **Security**: Optional signature verification support
- **GitHub integration**: Native support for GitHub Releases
- **Minimal code**: Simple API reduces implementation errors

#### Technical Implementation

##### 1. Add Dependency

Add to `Cargo.toml`:
```toml
[dependencies]
self_update = { version = "0.42", features = ["archive-tar", "archive-zip", "compression-flate2", "compression-zip-deflate"] }
```

##### 2. New CLI Command Structure

Add a new `update` subcommand:
```rust
Commands::Update {
    /// Check for updates without installing
    #[arg(short, long)]
    check: bool,
    /// Skip confirmation prompt
    #[arg(short, long)]
    yes: bool,
}
```

##### 3. Core Implementation

**File**: `src/update.rs` (new file)

Key functions:
- `check_for_update() -> Result<Option<String>>`: Query GitHub API for latest version
- `perform_update(force: bool) -> Result<()>`: Download and install update
- `compare_versions(current: &str, latest: &str) -> bool`: Semantic version comparison

**Implementation Details:**

```rust
use self_update::{cargo_crate_version, backends::github::Update};

pub fn check_for_update() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let current_version = cargo_crate_version!();
    
    let latest = self_update::backends::github::ReleaseList::configure()
        .repo_owner("timrogers")
        .repo_name("wld")
        .build()?
        .fetch()?;
    
    if let Some(latest_release) = latest.first() {
        let latest_version = latest_release.version.trim_start_matches('v');
        if latest_version != current_version {
            return Ok(Some(latest_version.to_string()));
        }
    }
    
    Ok(None)
}

pub fn perform_update(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = cargo_crate_version!();
    
    let status = Update::configure()
        .repo_owner("timrogers")
        .repo_name("wld")
        .bin_name("wld")
        .target_version_tag("v")
        .current_version(current_version)
        .no_confirm(yes)
        .build()?
        .update()?;
    
    println!("Update status: `{}`", status.version());
    Ok(())
}
```

##### 4. Binary Name Detection

The `self_update` crate will automatically detect the appropriate binary based on the current platform's target triple. Configure asset name patterns:

```rust
.bin_install_path(std::env::current_exe()?)
.target(&get_target())
```

Platform-specific target detection:
- Linux x86_64: `x86_64-unknown-linux-gnu` → `linux-amd64`
- Linux aarch64: `aarch64-unknown-linux-gnu` → `linux-aarch64`
- macOS x86_64: `x86_64-apple-darwin` → `darwin-amd64` or `darwin-universal`
- macOS aarch64: `aarch64-apple-darwin` → `darwin-aarch64` or `darwin-universal`
- Windows x86_64: `x86_64-pc-windows-msvc` → `windows-amd64.exe`

**Challenge**: The binary naming in releases (`wld_v0.0.2_linux-amd64`) doesn't match the standard target triple format that `self_update` expects. 

**Solution**: Use custom asset name configuration:
```rust
.bin_name(&format!("wld_{{{{version}}}}_{{{{target}}}}{}", 
    if cfg!(windows) { ".exe" } else { "" }))
```

##### 5. User Experience Flow

**Check-only mode** (`wld update --check`):
```
$ wld update --check
Checking for updates...
✓ New version available: v0.0.3 (current: v0.0.2)
Run 'wld update' to install the latest version.
```

**Interactive update** (`wld update`):
```
$ wld update
Checking for updates...
✓ New version available: v0.0.3 (current: v0.0.2)

Release notes:
* Add support for color temperature control
* Fix bug with brightness percentage calculation

Do you want to update? [y/N]: y
Downloading update...
███████████████████████████████████████████ 100% (8.5 MB)
✓ Successfully updated to v0.0.3
Please restart wld to use the new version.
```

**Auto-confirm update** (`wld update --yes`):
```
$ wld update --yes
Checking for updates...
✓ New version available: v0.0.3 (current: v0.0.2)
Downloading update...
███████████████████████████████████████████ 100% (8.5 MB)
✓ Successfully updated to v0.0.3
```

**No update available**:
```
$ wld update
Checking for updates...
✓ You are running the latest version (v0.0.2)
```

##### 6. Integration with Main CLI

In `src/main.rs`, add:
```rust
mod update;

Commands::Update { check, yes } => {
    if check {
        match update::check_for_update()? {
            Some(version) => {
                println!("✓ New version available: v{}", version);
                println!("Run 'wld update' to install the latest version.");
            }
            None => {
                println!("✓ You are running the latest version");
            }
        }
    } else {
        update::perform_update(yes)?;
    }
}
```

#### Advantages of `self_update`
1. ✅ **Complete solution**: Handles all aspects of self-updating
2. ✅ **Tested**: Used by many Rust CLI tools in production
3. ✅ **GitHub-native**: Built-in support for GitHub Releases
4. ✅ **Platform detection**: Automatic target detection
5. ✅ **Progress indication**: Built-in download progress bars
6. ✅ **Error handling**: Comprehensive error types
7. ✅ **Atomic updates**: Safe binary replacement

#### Challenges & Solutions

**Challenge 1: Asset Name Mismatch**
- **Problem**: Release assets use custom naming (`wld_v0.0.2_linux-amd64`)
- **Solution**: Configure custom asset name patterns in `self_update` builder

**Challenge 2: macOS Notarization**
- **Problem**: Downloaded binary won't be notarized
- **Solution**: Keep existing notarization in CI; self-update downloads already-notarized binaries from releases

**Challenge 3: Permission Issues**
- **Problem**: Binary may be in protected location (e.g., `/usr/local/bin`)
- **Solution**: Detect permission errors and suggest `sudo wld update` or moving binary to user-writable location

**Challenge 4: Version Format**
- **Problem**: GitHub releases use `v0.0.2` format
- **Solution**: Strip `v` prefix when comparing versions

### Option 2: Custom Implementation (ALTERNATIVE)

If `self_update` proves problematic, implement a simpler check-only feature:

#### Dependencies
```toml
[dependencies]
octocrab = "0.38"  # GitHub API client
semver = "1.0"     # Version comparison
```

#### Implementation
- Add `wld check-update` command
- Query GitHub Releases API using `octocrab`
- Compare versions using `semver` crate
- Display update notification with download link
- **Do not** attempt binary replacement (user downloads manually)

**Pros:**
- Simpler implementation
- Fewer edge cases
- No permission issues

**Cons:**
- Less convenient for users
- Incomplete solution
- Still requires GitHub API dependency

### Option 3: Passive Update Checks (MINIMAL)

Add a lightweight background check:

#### Implementation
- On every command execution, check a local cache file (`~/.wld_last_update_check`)
- If >24 hours since last check, asynchronously query GitHub API
- Display a one-line notice if update available
- Do not block command execution

**Example:**
```
$ wld on
✓ Turned on device at 192.168.1.100
ℹ️  Update available: v0.0.3 (run 'wld update' to install)
```

**Pros:**
- Non-intrusive
- Keeps users informed
- Minimal implementation

**Cons:**
- Still requires implementing `wld update` command
- May be seen as "nagware"
- Network calls on every invocation (even if cached)

## Recommended Implementation Plan

### Phase 1: Core Update Functionality (Recommended Approach)

**Tasks:**
1. Add `self_update` dependency to `Cargo.toml`
2. Create `src/update.rs` module with:
   - `check_for_update()` function
   - `perform_update()` function
   - Platform-specific target mapping
   - Asset name configuration
3. Add `Update` command to CLI in `src/main.rs`
4. Implement `--check` and `--yes` flags
5. Add user-friendly output with status indicators

**Testing:**
- Test on all supported platforms
- Test with/without internet connection
- Test with GitHub API rate limiting
- Test permission errors
- Test with corrupted downloads

### Phase 2: Enhanced Features (Optional)

**Tasks:**
1. Add release notes display (fetch from GitHub API)
2. Add `--version-info` flag to show changelog
3. Implement rollback functionality (`wld update --rollback`)
4. Add update channel support (stable/beta)
5. Cache downloaded binaries for faster rollback

### Phase 3: Background Checks (Optional)

**Tasks:**
1. Implement update check cache in `~/.wld_update_cache.toml`
2. Add passive update notifications
3. Add `--no-update-check` global flag
4. Add configuration option to disable update checks

## Configuration Options

Add to `~/.wld.toml`:
```toml
[update]
# Enable/disable automatic update checks
auto_check = true

# Update channel (stable, beta, all)
channel = "stable"

# Check frequency in hours
check_frequency = 24

# Auto-install updates without prompting
auto_install = false
```

## Documentation Updates

### README.md Additions

Add new section:
```markdown
#### Updating

- `wld update`: Check for and install the latest version
- `wld update --check`: Check for updates without installing
- `wld update --yes`: Install updates without confirmation
```

### CHANGELOG.md Entry

```markdown
### Added
- Auto-update functionality using GitHub Releases API
- New `wld update` command to check for and install updates
- Support for all platforms (Linux, macOS, Windows)
```

## Security Considerations

1. **Binary Verification**: Use `self_update`'s built-in integrity checks
2. **HTTPS Only**: All downloads over HTTPS from github.com
3. **Signature Verification**: Consider implementing in Phase 2
4. **Atomic Updates**: Replace binary atomically to avoid corruption
5. **Permission Handling**: Fail gracefully if insufficient permissions
6. **Rate Limiting**: Respect GitHub API rate limits (check cache first)

## GitHub API Rate Limits

- **Unauthenticated**: 60 requests/hour per IP
- **Authenticated**: 5000 requests/hour (not needed for public releases)
- **Mitigation**: Cache check results for 24 hours

## Testing Plan

### Unit Tests
- Version comparison logic
- Platform detection
- Asset name generation
- Cache management

### Integration Tests
- Mock GitHub API responses
- Test full update flow
- Test error conditions

### Manual Testing Matrix
- ✅ Linux x86_64 (binary installed in /usr/local/bin)
- ✅ Linux x86_64 (binary installed in ~/bin)
- ✅ Linux aarch64
- ✅ macOS Intel (via Homebrew)
- ✅ macOS Intel (via direct download)
- ✅ macOS Apple Silicon
- ✅ Windows x86_64 (via Scoop/Chocolatey)
- ✅ Windows x86_64 (direct download)

### Edge Cases
- No internet connection
- GitHub API rate limit exceeded
- Corrupted download
- Disk full
- Insufficient permissions
- Binary in use during update (Windows)

## Migration Path

### For Existing Users

Users who installed via:
- **Homebrew**: Updates handled by brew, but `wld update` can override
- **Cargo**: `cargo install -f wld` still works, `wld update` is alternative
- **Direct download**: Full self-update functionality

### Compatibility

- Minimum Rust version: 1.89.0 (already required)
- All existing commands continue to work
- No breaking changes
- Opt-in feature (users can ignore update command)

## Alternative Considerations

### Why Not Use `cargo-update`?
- Requires Rust toolchain installed
- Many users install via binary download
- Longer update time (rebuilds from source)

### Why Not Use System Package Managers?
- Not all users use package managers
- Would require maintaining multiple distribution channels
- Self-update is more universal

### Why Not Notify Only?
- Less user-friendly
- Users may forget to update
- Self-update is standard practice for modern CLI tools

## Implementation Complexity

**Estimated Effort:**
- Phase 1: 4-6 hours (including testing)
- Phase 2: 2-4 hours
- Phase 3: 2-3 hours
- Documentation: 1 hour

**Lines of Code:**
- `src/update.rs`: ~150-200 lines
- `src/main.rs`: ~20 lines (new command)
- Tests: ~100-150 lines
- Total: ~300 lines

## Success Metrics

1. **Functionality**: Users can update from any version to latest
2. **Reliability**: 99%+ success rate on supported platforms
3. **Performance**: Update check completes in <2 seconds
4. **UX**: Clear, informative output at each step
5. **Safety**: Zero reported cases of binary corruption

## References

- `self_update` crate: https://docs.rs/self_update
- GitHub Releases API: https://docs.github.com/en/rest/releases
- Semantic Versioning: https://semver.org/

## Open Questions

1. Should update checks be opt-in or opt-out?
   - **Recommendation**: Opt-in (manual `wld update` command)

2. Should we support pre-release versions?
   - **Recommendation**: Phase 2 feature (add `--pre` flag)

3. Should we notify users on every command execution?
   - **Recommendation**: No, only when explicitly running `wld update`

4. Should we support rollback to previous versions?
   - **Recommendation**: Phase 2 feature

5. Should we verify binary signatures?
   - **Recommendation**: Phase 2 feature (macOS binaries already signed)

## Conclusion

The recommended approach is to implement **Option 1** (using `self_update` crate) in **Phase 1** only, keeping the implementation simple and focused. This provides:

- ✅ Complete, working auto-update functionality
- ✅ Support for all platforms
- ✅ Minimal maintenance burden
- ✅ User-friendly experience
- ✅ Safe, atomic updates

The implementation can be completed in a single PR with approximately 300 lines of code, comprehensive tests, and updated documentation.

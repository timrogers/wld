# Auto Update Implementation - Quick Reference

## TL;DR

Implement auto-update functionality using the `self_update` crate to allow users to update `wld` directly from the command line via GitHub Releases.

## Recommended Solution

**Use the `self_update` crate (v0.42+)**

## Implementation Checklist

### Step 1: Add Dependency
```toml
# Add to Cargo.toml
[dependencies]
self_update = { version = "0.42", features = ["archive-tar", "archive-zip", "compression-flate2", "compression-zip-deflate"] }
```

### Step 2: Create Update Module
Create `src/update.rs` with two main functions:
- `check_for_update()` - Query GitHub API for latest version
- `perform_update(yes: bool)` - Download and install update

### Step 3: Add CLI Command
Add to `Commands` enum in `src/main.rs`:
```rust
Update {
    /// Check for updates without installing
    #[arg(short, long)]
    check: bool,
    /// Skip confirmation prompt
    #[arg(short, long)]
    yes: bool,
}
```

### Step 4: Configure Asset Naming
Map platform targets to release asset names:
- `linux-amd64` for x86_64-unknown-linux-gnu
- `linux-aarch64` for aarch64-unknown-linux-gnu
- `darwin-universal` for both macOS targets (preferred)
- `windows-amd64.exe` for x86_64-pc-windows-msvc

### Step 5: Handle Edge Cases
- Permission errors (suggest user-writable location)
- Network failures (graceful error messages)
- GitHub API rate limits (unlikely with caching)
- Version format (strip 'v' prefix from tags)

## Usage Examples

```bash
# Check for updates (non-destructive)
wld update --check

# Update with confirmation prompt
wld update

# Update without confirmation
wld update --yes
```

## Key Configuration

```rust
Update::configure()
    .repo_owner("timrogers")
    .repo_name("wld")
    .bin_name("wld")
    .target_version_tag("v")
    .current_version(cargo_crate_version!())
    .build()?
    .update()?
```

## Asset Naming Challenge

Release assets use format: `wld_v{VERSION}_{TARGET}{.exe}`

Example: `wld_v0.0.2_linux-amd64`

The `self_update` crate needs custom configuration to match this pattern.

## Testing Priorities

1. ✅ Each platform (Linux, macOS, Windows)
2. ✅ Both installation methods (system-wide vs user-local)
3. ✅ Network failure scenarios
4. ✅ Permission denied scenarios
5. ✅ Version comparison logic

## Estimated Effort

- **Implementation**: 4-6 hours
- **Testing**: 2-3 hours
- **Documentation**: 1 hour
- **Total**: ~8-10 hours

## Why `self_update`?

- ✅ Production-ready and battle-tested
- ✅ Handles all update mechanics automatically
- ✅ Built-in support for GitHub Releases
- ✅ Atomic binary replacement (safe)
- ✅ Cross-platform support
- ✅ Progress indication
- ✅ ~150 lines of code (vs 500+ for custom solution)

## Alternative Approaches Considered

1. **Custom GitHub API implementation** - More complex, more to maintain
2. **Check-only (no auto-install)** - Less user-friendly
3. **Background passive checks** - Potentially annoying "nagware"

## Security Notes

- Downloads over HTTPS only
- GitHub Release assets are already notarized (macOS)
- Atomic binary replacement prevents corruption
- No credentials required (public API)
- Rate limiting handled by caching

## Full Documentation

See `AUTO_UPDATE_IMPLEMENTATION_PLAN.md` for complete details including:
- Multiple implementation options
- Detailed code examples
- Configuration options
- Migration path for existing users
- Testing matrix
- Security considerations
- Open questions and recommendations

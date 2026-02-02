# Auto Update Architecture Diagram

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         wld CLI Tool                            │
│                        (Rust Binary)                            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            │ User runs: wld update
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    src/update.rs Module                         │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │ check_for_update()                                      │  │
│  │ ─────────────────                                       │  │
│  │ • Get current version from CARGO_PKG_VERSION            │  │
│  │ • Query GitHub Releases API                             │  │
│  │ • Compare versions (semver)                             │  │
│  │ • Return Option<String> with latest version             │  │
│  └─────────────────────────────────────────────────────────┘  │
│                            │                                   │
│                            │ If update available               │
│                            ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │ perform_update(yes: bool)                               │  │
│  │ ─────────────────────────                               │  │
│  │ • Confirm with user (unless --yes)                      │  │
│  │ • Detect platform/architecture                          │  │
│  │ • Download correct binary asset                         │  │
│  │ • Verify download integrity                             │  │
│  │ • Replace current binary atomically                     │  │
│  │ • Report success/failure                                │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────┬───────────────────────────────────┬───────────┘
                  │                                   │
                  │ Uses                              │ Uses
                  │                                   │
                  ▼                                   ▼
┌──────────────────────────────────┐  ┌──────────────────────────┐
│    self_update Crate             │  │   Rust Standard Lib      │
│                                  │  │                          │
│ • GitHub backend                 │  │ • std::env::current_exe()│
│ • Release list fetching          │  │ • File I/O operations    │
│ • Binary download                │  │ • Process management     │
│ • Progress indication            │  │                          │
│ • Atomic file replacement        │  │                          │
│ • Platform detection             │  │                          │
└──────────────┬───────────────────┘  └──────────────────────────┘
               │
               │ HTTP/HTTPS
               │
               ▼
┌─────────────────────────────────────────────────────────────────┐
│                   GitHub Releases API                           │
│               https://api.github.com/repos/                     │
│                  timrogers/wld/releases                         │
│                                                                 │
│  Returns:                                                       │
│  • Latest release version (e.g., "v0.0.3")                     │
│  • Release notes/description                                    │
│  • Binary assets for each platform:                            │
│    - wld_v0.0.3_linux-amd64                                    │
│    - wld_v0.0.3_linux-aarch64                                  │
│    - wld_v0.0.3_darwin-amd64                                   │
│    - wld_v0.0.3_darwin-aarch64                                 │
│    - wld_v0.0.3_darwin-universal                               │
│    - wld_v0.0.3_windows-amd64.exe                              │
└─────────────────────────────────────────────────────────────────┘
```

## Update Flow Sequence

```
User                 wld CLI              update.rs           self_update          GitHub API
 │                      │                     │                    │                    │
 │  wld update         │                     │                    │                    │
 ├────────────────────>│                     │                    │                    │
 │                      │                     │                    │                    │
 │                      │ check_for_update() │                    │                    │
 │                      ├───────────────────>│                    │                    │
 │                      │                     │                    │                    │
 │                      │                     │ GET /releases      │                    │
 │                      │                     ├───────────────────────────────────────>│
 │                      │                     │                    │                    │
 │                      │                     │                    │   Release list     │
 │                      │                     │<───────────────────────────────────────┤
 │                      │                     │                    │                    │
 │                      │  New version: v0.0.3                    │                    │
 │                      │<────────────────────┤                    │                    │
 │                      │                     │                    │                    │
 │  New version: v0.0.3│                     │                    │                    │
 │<─────────────────────┤                     │                    │                    │
 │                      │                     │                    │                    │
 │  Update? [y/N]: y   │                     │                    │                    │
 ├────────────────────>│                     │                    │                    │
 │                      │                     │                    │                    │
 │                      │ perform_update(false)                   │                    │
 │                      ├───────────────────>│                    │                    │
 │                      │                     │                    │                    │
 │                      │                     │ Configure builder  │                    │
 │                      │                     ├───────────────────>│                    │
 │                      │                     │                    │                    │
 │                      │                     │                    │ Download binary    │
 │                      │                     │                    ├───────────────────>│
 │                      │                     │                    │                    │
 │  Downloading...     │                     │                    │   Binary data      │
 │  ███████████░░░░ 70%│                     │                    │<───────────────────┤
 │<─────────────────────┤<────────────────────┤<───────────────────┤                    │
 │                      │                     │                    │                    │
 │  Downloaded ✓        │                     │                    │                    │
 │<─────────────────────┤                     │                    │                    │
 │                      │                     │                    │                    │
 │  Replacing binary... │                     │                    │                    │
 │<─────────────────────┤                     │                    │                    │
 │                      │                     │                    │                    │
 │  Success! v0.0.3     │                     │                    │                    │
 │<─────────────────────┤<────────────────────┤<───────────────────┤                    │
 │                      │                     │                    │                    │
```

## Platform-Specific Binary Selection

```
┌─────────────────────────────────────────────────────────────────┐
│                  Runtime Platform Detection                     │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
            ┌───────────────────┴────────────────────┐
            │                                        │
            ▼                                        ▼
    ┌───────────────┐                      ┌────────────────┐
    │   OS Family   │                      │ Architecture   │
    │               │                      │                │
    │ • Linux       │                      │ • x86_64       │
    │ • macOS       │                      │ • aarch64      │
    │ • Windows     │                      │                │
    └───────────────┘                      └────────────────┘
            │                                        │
            └───────────────────┬────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │  Asset Name Mapping   │
                    └───────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐      ┌──────────────────┐    ┌────────────────┐
│   Linux      │      │      macOS       │    │    Windows     │
│              │      │                  │    │                │
│ x86_64  →    │      │ x86_64      →    │    │ x86_64    →    │
│  linux-amd64 │      │  darwin-amd64 *  │    │  windows-amd64 │
│              │      │  or universal    │    │  .exe          │
│ aarch64 →    │      │                  │    │                │
│  linux-      │      │ aarch64     →    │    │                │
│  aarch64     │      │  darwin-aarch64* │    │                │
│              │      │  or universal    │    │                │
└──────────────┘      └──────────────────┘    └────────────────┘

* Prefer universal binary for macOS (works on both architectures)
```

## File System Operations

```
Current Binary Location: /usr/local/bin/wld or ~/bin/wld or C:\wld.exe
                                │
                                │ perform_update()
                                ▼
                    ┌───────────────────────┐
                    │ 1. Create temp file   │
                    │    /tmp/wld.tmp.XXXX  │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ 2. Download to temp   │
                    │    (with progress)    │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ 3. Verify integrity   │
                    │    (checksums)        │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ 4. Set executable     │
                    │    permissions        │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ 5. Atomic rename      │
                    │    temp → current     │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ 6. Cleanup temp files │
                    └───────────────────────┘
```

## Error Handling Flow

```
                      perform_update()
                            │
                            ▼
            ┌───────────────────────────────┐
            │   Try: Fetch release info     │
            └───────┬───────────────────────┘
                    │
        ┌───────────┼───────────┐
        │ Success   │           │ Failure
        ▼           │           ▼
    Continue        │       ┌─────────────────────┐
                    │       │ NetworkError        │
                    │       │ "Cannot connect to  │
                    │       │  GitHub. Check your │
                    │       │  internet."         │
                    │       └─────────────────────┘
                    ▼
        ┌────────────────────────┐
        │ Try: Download binary   │
        └───────┬────────────────┘
                │
    ┌───────────┼───────────┐
    │ Success   │           │ Failure
    ▼           │           ▼
Continue        │       ┌─────────────────────┐
                │       │ DownloadError       │
                │       │ "Failed to download │
                │       │  binary. Retry?"    │
                │       └─────────────────────┘
                ▼
    ┌────────────────────────┐
    │ Try: Replace binary    │
    └───────┬────────────────┘
            │
┌───────────┼───────────┐
│ Success   │           │ Failure
▼           │           ▼
Success!    │       ┌─────────────────────┐
            │       │ PermissionError     │
            │       │ "Insufficient       │
            │       │  permissions. Try:  │
            │       │  sudo wld update"   │
            │       └─────────────────────┘
            │
            ▼
    ┌────────────────────────┐
    │  Any error?            │
    │  Rollback if needed    │
    └────────────────────────┘
```

## Component Dependencies

```
┌─────────────────────────────────────────────────────────────────┐
│                         wld Binary                              │
│                     (Release v0.0.2)                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                │ Uses
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐      ┌──────────────────┐    ┌────────────────┐
│ self_update  │      │   Existing deps  │    │  New module    │
│              │      │                  │    │                │
│ Version: 0.42│      │ • clap 4.5       │    │ src/update.rs  │
│              │      │ • reqwest 0.11   │    │                │
│ Features:    │      │ • serde 1.0      │    │ Functions:     │
│ • github     │      │ • toml 0.8       │    │ • check_for_   │
│ • archive-   │      │ • directories    │    │   update()     │
│   tar        │      │ • wled-json-api  │    │ • perform_     │
│ • archive-   │      │                  │    │   update()     │
│   zip        │      │                  │    │                │
│ • compress-  │      │                  │    │                │
│   flate2     │      │                  │    │                │
└──────────────┘      └──────────────────┘    └────────────────┘
```

## Configuration Options (Future Phase)

```
~/.wld.toml
┌─────────────────────────────────────────────────────────────────┐
│ [update]                                                        │
│ # Enable/disable automatic update checks                       │
│ auto_check = true                                               │
│                                                                 │
│ # Update channel (stable, beta, all)                           │
│ channel = "stable"                                              │
│                                                                 │
│ # Check frequency in hours                                     │
│ check_frequency = 24                                            │
│                                                                 │
│ # Auto-install updates without prompting                       │
│ auto_install = false                                            │
└─────────────────────────────────────────────────────────────────┘
```

## Testing Matrix

```
┌──────────────────────────────────────────────────────────────────┐
│                      Test Scenarios                              │
└──────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐      ┌──────────────────┐    ┌────────────────┐
│   Platform   │      │  Installation    │    │  Edge Cases    │
│   Testing    │      │    Location      │    │                │
│              │      │                  │    │ • No internet  │
│ • Linux x64  │      │ • /usr/local/bin │    │ • Rate limit   │
│ • Linux ARM  │      │ • ~/bin          │    │ • Disk full    │
│ • macOS x64  │      │ • Windows paths  │    │ • Corrupted DL │
│ • macOS ARM  │      │                  │    │ • No perms     │
│ • Windows    │      │                  │    │ • Binary in use│
└──────────────┘      └──────────────────┘    └────────────────┘
```

---

**Legend:**
- → : Data flow
- ├─> : Function call
- <─ : Return value
- ▼ : Sequential step
- │ : Continuation

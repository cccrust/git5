# git4

A minimalist Git implementation written in Rust. This project demonstrates how version control systems work by reimplementing core Git functionality from scratch, using only essential Rust libraries for compression (flate2) and hashing (sha1).

## Overview

git4 is a lightweight Git clone that implements the fundamental concepts of version control:
- Object storage (blobs, trees, commits)
- Reference management (branches, tags)
- Index/staging area
- Three-way file comparison
- Branch merging (fast-forward)
- Remote operations (clone, push, fetch)

## Features

### Core Commands
- `init` - Initialize a new repository
- `add` - Stage files for commit
- `commit` - Create a new commit
- `log` - View commit history
- `status` - Show working tree status
- `diff` - View file changes

### Branching
- `branch` - List/create branches
- `checkout` - Switch branches or restore files
- `merge` - Merge branches (fast-forward)

### Remote Operations
- `clone` - Clone a repository
- `push` - Push to remote
- `fetch` - Fetch from remote
- `remote` - Manage remotes

### Additional Commands
- `tag` - Create/delete tags
- `stash` - Stash changes
- `reset` - Reset HEAD
- `clean` - Clean untracked files
- And many more...

## Installation

```bash
cargo build --release
```

## Quick Start

```bash
# Initialize a new repository
git5 init

# Add files to staging
git5 add .

# Commit changes
git5 commit -m "Initial commit"

# View history
git5 log

# Create a branch
git5 branch feature-branch
git5 checkout -b feature-branch

# Merge changes
git5 merge feature-branch
```

## Architecture

### Object Storage
- **Blobs**: File content
- **Trees**: Directory listings
- **Commits**: Snapshots with metadata

### Storage Location
- `.git4/objects/` - Object database
- `.git4/refs/` - References (branches, tags)
- `.git4/HEAD` - Current HEAD
- `.git4/index` - Staging area

## Dependencies

- `flate2` - Zlib compression
- `sha1` - SHA-1 hashing
- `clap` - CLI argument parsing
- `similar` - Text diffing
- `ureq` - HTTP client for remote operations

## Version History

- v0.1: Project foundation (blobs, trees, commits)
- v0.2: Branch system (branch, checkout)
- v0.3: Status management
- v0.4: Diff and merge
- v0.5: Local remote operations

## License

MIT
# Build and Link Time Optimizations

## Overview

This project is optimized for faster build and link times by using high-performance linkers:

- **Linux (`x86_64-unknown-linux-gnu`)**: `mold`
- **macOS (`x86_64-apple-darwin`)**: `lld`

## Installation

### Linux (Ubuntu/Debian-based)

```sh
sudo apt update
sudo apt install mold clang
```

### macOS

```sh
xcode-select --install  # Install Xcode Command Line Tools
brew install llvm       # Install lld
```

## Verification

Ensure the linkers are installed:

```sh
mold --version  # Linux
ld.lld --version  # macOS
```

## Testing the Setup

Run a build to confirm the optimized linkers are in use:

```sh
cargo build -vv
```

This setup improves build and link times for a more efficient development workflow.


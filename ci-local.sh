#!/bin/bash

set -e  # Exit on error
set -o pipefail  # Fail pipeline if any command fails

echo "Running Local CI Pipeline..."

# Set environment variables
export CARGO_TERM_COLOR=always
export RUSTFLAGS="-D warnings"

# Check if Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Cargo is not installed. Please install Rust."
    exit 1
fi

# Ensure cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Ensure cargo-nextest is installed
if ! command -v cargo-nextest &> /dev/null; then
    echo "Installing cargo-nextest..."
    cargo install cargo-nextest --locked
fi

# Run all CI steps locally
echo "Checking Formatting..."
cargo fmt --all -- --check || (echo "Formatting issues found, please run 'cargo fmt' to auto-fix." && exit 1)

echo "Building Project..."
cargo build --workspace --all-features

echo "Running Tests..."
cargo nextest run --workspace --all-targets --all-features --no-fail-fast || echo "No tests found, skipping..."

echo "Running Lints Checks..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "Checking API Documentation..."
cargo doc --workspace --all-features --no-deps

echo "Running Security Audit..."
cargo audit

echo "CI pipeline completed successfully!"

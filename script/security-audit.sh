#!/bin/bash

# Security audit script for masstemplate
# This script runs cargo-audit to check for security vulnerabilities in dependencies

set -e

echo "Running security audit..."

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "cargo-audit not found. Installing..."
    cargo install cargo-audit
fi

# Run audit on the workspace
cargo audit

echo "Security audit completed successfully."
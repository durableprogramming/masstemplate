#!/bin/bash

# Code coverage script for masstemplate
# This script runs cargo-tarpaulin to generate coverage reports

set -e

echo "Running code coverage analysis..."

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "cargo-tarpaulin not found. Installing..."
    cargo install cargo-tarpaulin
fi

# Run coverage on the workspace
cargo tarpaulin --workspace --out Html --output-dir target/coverage --exclude-files "tests/*" --line

echo "Coverage report generated in target/coverage/tarpaulin-report.html"
echo "Open the HTML file in your browser to view the coverage report."
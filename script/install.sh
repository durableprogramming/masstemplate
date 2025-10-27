#!/bin/bash

# Build the project in release mode
cargo build --release

# Create ~/.local/bin if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary to ~/.local/bin
cp target/release/mtem ~/.local/bin/mtem

echo "Installation complete. Make sure ~/.local/bin is in your PATH."

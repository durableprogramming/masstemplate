# Basic Template Creation Example

This example shows how to create a simple Rust template and apply it.

## 1. Create Template Directory

```bash
mkdir -p ~/.local/masstemplate/rust-basic/src
```

## 2. Add Template Files

Create `~/.local/masstemplate/rust-basic/Cargo.toml`:

```toml
[package]
name = "__PROJECT_NAME__"
version = "0.1.0"
edition = "2021"

[dependencies]
```

Create `~/.local/masstemplate/rust-basic/src/main.rs`:

```rust
fn main() {
    println!("Hello from __PROJECT_NAME__!");
}
```

Create `~/.local/masstemplate/rust-basic/.mtem/post_install.sh`:

```bash
#!/bin/bash
cargo build
echo "Project __PROJECT_NAME__ is ready!"
```

## 3. Apply the Template

```bash
mtem apply rust-basic --dest my-rust-app
```

## 4. Result

You'll get a new directory `my-rust-app` with:
- A Cargo.toml with the project name
- Source code with the project name filled in
- Dependencies already built
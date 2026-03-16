# masstemplate

A command-line tool for creating new projects from local templates.

Masstemplate allows developers to create project templates as simple directories and apply them to quickly bootstrap new projects. It emphasizes local control, simplicity, and extensibility without cloud dependencies.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Configuration](#configuration)
- [Building Templates](#building-templates)
- [Advanced Features](#advanced-features)
- [Contributing](#contributing)
- [License](#license)
- [Support](#support)

## Installation

Masstemplate is written in Rust and can be built from source:

```bash
git clone https://github.com/durableprogramming/masstemplate.git
cd masstemplate
cargo build --release
```

The binary will be available at `target/release/mtem`. You can install it to your system by copying it to a directory in your PATH:

```bash
sudo cp target/release/mtem /usr/local/bin/
```

For detailed installation instructions, see [docs/installation.md](docs/installation.md).

### System Requirements

- Rust 1.70 or later
- Unix-like operating system (Linux, macOS, or Windows with WSL)

## Quick Start

1. Create a template directory:
   ```bash
   mkdir -p ~/.local/masstemplate/rust/src
   echo 'fn main() { println!("Hello, world!"); }' > ~/.local/masstemplate/rust/src/main.rs
   echo '[package]\nname = "my-project"\nversion = "0.1.0"\nedition = "2021"' > ~/.local/masstemplate/rust/Cargo.toml
   ```

2. Apply the template:
   ```bash
   mtem rust
   ```

That's it! Your new Rust project is ready.

## Usage

### Basic Commands

```bash
# List available templates
mtem list

# Apply a template
mtem apply <template-name>

# Get information about a template
mtem info <template-name>
```

### Advanced Usage

```bash
# Apply template to specific directory
mtem apply rust --dest my-project

# Use collision strategy
mtem apply node --collision overwrite

# Skip interactive prompts
mtem apply rust --yes
```

For more usage examples, see [docs/getting-started.md](docs/getting-started.md).

## Configuration

Masstemplate uses a global configuration file at `~/.config/masstemplate/config.toml` for settings like:

- Template search paths
- Default collision strategies
- Logging levels

Example configuration:

```toml
[core]
template_dirs = ["~/.local/masstemplate"]
default_collision = "backup"
verbose = false

[logging]
level = "info"
```

See [docs/masstemplate-config.md](docs/masstemplate-config.md) for complete configuration options.

## Building Templates

Create subdirectories in `~/.local/masstemplate/` for each project type. Fill them with starter files and configurations.

Example Rust template structure:

```
~/.local/masstemplate/rust/
├── Cargo.toml
├── src/
│   └── main.rs
└── .mtem/
    └── post_install.sh
```

### Install Scripts

Add `.mtem/pre_install.sh` and `.mtem/post_install.sh` to your templates for automation:

- `pre_install.sh`: Runs before copying files (e.g., check prerequisites)
- `post_install.sh`: Runs after copying files (e.g., install dependencies)

Scripts execute in the destination directory.

### Ignoring Files

Create `.mtemignore` or `.mtem/ignore` to exclude files from templates:

```
devenv.lock
*.log
.cache/
```

## Advanced Features

### Processing DSL

Use `.mtem/config` files for advanced file processing during template application.

Features:
- Text replacement: `replace __NAME__ MyProject`
- Environment variables: `dotenv set API_KEY=abc123`
- Collision strategies: `collision overwrite`
- File matching: `match *.json { collision merge }`

See [docs/processing-dsl-overview.md](docs/processing-dsl-overview.md) for details.

### Template Composition

Combine multiple templates for complex project structures. Use post-install scripts to apply sub-templates:

```bash
#!/bin/bash
mtem frontend-react
mtem backend-node
# Custom integration steps
```

### Hooks and Extensions

Masstemplate supports various hooks for customization:
- Pre/post install scripts
- File processing hooks
- Custom processors

See [docs/masstemplate-hooks.md](docs/masstemplate-hooks.md) for details.



## Documentation

- [Building Templates](docs/building-templates.md)
- [Processing DSL](docs/processing-dsl-overview.md)
- [Install Scripts](docs/pre-install-scripts.md)
- [Configuration](docs/masstemplate-config.md)
- [Hooks](docs/masstemplate-hooks.md)
- [Sample Templates](sample_templates/)

## Contributing

We welcome contributions from the community! Here's how you can help:

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/durableprogramming/masstemplate.git
   cd masstemplate
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

### Contribution Guidelines

- Follow the existing code style and conventions
- Add tests for new functionality
- Update documentation for API changes
- Use conventional commit messages
- Ensure all tests pass before submitting PRs

### Reporting Issues

- Use GitHub issues for bug reports and feature requests
- Include detailed reproduction steps for bugs
- Specify your operating system and Rust version

### Code of Conduct

This project follows a code of conduct to ensure a welcoming environment for all contributors.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Durable Programming LLC

## Support

- **Issues**: [GitHub Issues](https://github.com/durableprogramming/masstemplate/issues)
- **Discussions**: [GitHub Discussions](https://github.com/durableprogramming/masstemplate/discussions)
- **Documentation**: [docs/](docs/)

For commercial support or enterprise features, contact commercial@durableprogramming.com.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Purpose

Templatize is a CLI tool that converts existing projects into Jinja2 templates by performing selective substitutions. The tool allows users to:

- Replace specific words/phrases with Jinja2 template syntax
- Apply templating selectively to directories only, files only, or both
- Transform existing codebases into reusable templates with parameterized values
- Support word-based replacements that generate appropriate Jinja2 syntax

## Project Structure

This is a Rust workspace project for "Templatize" with a multi-crate architecture:

- **`crates/templatize-bin/`**: Binary crate that produces the `templatize` CLI executable
- **`crates/templatize-core/`**: Core library crate containing templating logic and transformations
- **`xtask/`**: Build automation and development tasks using the xtask pattern

The project uses Cargo workspace with edition 2024 and follows standard Rust project conventions.

## Development Commands

### Building and Running
```bash
# Build the entire workspace
cargo build

# Build and run the main binary
cargo run --bin templatize

# Install the CLI locally
cargo run --bin xtask install
# OR directly:
cargo install --path crates/templatize-bin
```

### Testing
```bash
# Comprehensive test suite (recommended)
cargo xtask test all

# Individual test suites
cargo xtask test core         # Core library tests
cargo xtask test bin          # Binary crate tests
cargo xtask test integration  # Integration and CLI tests

# Traditional cargo testing
cargo test                    # All workspace tests
cargo test -p templatize-core # Core library only
cargo test -p templatize-bin  # Binary crate only
```

### Code Quality
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

### Xtask Automation
The project uses the xtask pattern for build automation. Available commands via `cargo run --bin xtask`:

- `install`: Install the templatize binary locally
- `docker build`: Build Docker image for the application
- `docker rmi`: Remove the application Docker image

## Architecture Notes

- The main binary (`templatize-bin`) handles CLI interface and user interaction
- Core templating logic resides in `templatize-core` for reusability and testing
- Uses `clap` v4 for robust CLI argument parsing and subcommands
- Implements `tracing` for structured logging throughout the application
- Uses `inflections` library for string transformations and case conversions
- Uses `inquire` for interactive prompts and user input validation
- The project follows workspace dependency management with shared versions and common dependencies
- Uses `anyhow` for error handling and `thiserror` for custom error types
- Built with Rust edition 2024 features

## Commands

### `templatize exact`
Replace exact tokens with exact Jinja2 syntax:
```bash
# Replace all instances of "example-name" with "{{ project-name }}"
templatize exact "example-name" "{{ project-name }}" --path --contents /path/to/project

# Interactive mode with diff preview
templatize exact "token" "{{ replacement }}" -p -c --interactive

# Dry run to preview changes
templatize exact "old" "{{ new }}" -p -c --dry-run
```

### `templatize escape`
Escape existing Jinja2 syntax to be embedded in templates:
```bash
# Escape Jinja syntax in templates
templatize escape /path/to/templates

# Interactive mode with diff preview
templatize escape --interactive /path/to/file.txt

# Dry run to preview changes
templatize escape --dry-run
```

## Key Files

- `Cargo.toml`: Workspace configuration and dependency management
- `crates/templatize-bin/src/main.rs`: CLI entry point and command handling
- `crates/templatize-bin/src/cli.rs`: Command-line interface definitions
- `crates/templatize-bin/src/diff.rs`: Diff display and interactive prompts
- `crates/templatize-core/src/lib.rs`: Core processing functions
- `crates/templatize-core/src/templater.rs`: Template transformation logic
- `xtask/src/main.rs`: Build automation and development tasks
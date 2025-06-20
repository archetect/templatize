# Templatize

![Build](https://github.com/archetect/templatize/workflows/Build/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/templatize.svg)](https://crates.io/crates/templatize)
[![Documentation](https://docs.rs/templatize/badge.svg)](https://docs.rs/templatize)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful CLI tool for converting existing projects into reusable Jinja2 templates. Transform your project files and directory structures by replacing tokens with template variables while preserving the original functionality.

## Installation

### Homebrew (macOS/Linux)

```bash
brew install archetect/tap/templatize
```

### Cargo

```bash
cargo install templatize
```

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/archetect/templatize/releases).

### Build from Source

```bash
git clone https://github.com/archetect/templatize
cd templatize
cargo build --release
```

## Quick Start

```bash
# 1. ALWAYS escape existing Jinja syntax first (critical step!)
templatize escape --dry-run

# 2. Apply case shape transformations for compound words
templatize shapes "my-project" "{{ project_name }}" -p -c --dry-run

# 3. Make precise replacements for specific tokens
templatize exact "MyCompany" "{{ company_name }}" -p -c --dry-run

# Remove --dry-run when you're ready to apply changes
```

> ⚠️ **Critical**: Run `escape` before any other commands, or it will break your template variables!

## Commands

### `escape` - Escape Existing Jinja Syntax

Escapes existing Jinja2 template syntax so it can be embedded within templates.

```bash
templatize escape [TARGET] [OPTIONS]
```

**Example:**
```bash
# Escape all Jinja syntax in current directory
templatize escape --interactive

# Before: {{ existing_var }}
# After:  {{'{'}}{ existing_var }
```

**Options:**
- `TARGET` - Target file or directory (defaults to current directory)
- `--dry-run` - Preview changes without applying them
- `--interactive` - Prompt for each change with diff preview

### `shapes` - Case Shape Transformations

Replaces compound words with all their case variants (camelCase, PascalCase, kebab-case, etc.).

```bash
templatize shapes <TOKEN> <REPLACEMENT> [TARGET] [OPTIONS]
```

**Example:**
```bash
templatize shapes "my-project" "{{ project_name }}" -p -c

# Transforms:
# my-project     → {{ project_name }}
# myProject      → {{ projectName }}
# MyProject      → {{ ProjectName }}
# my_project     → {{ project_name }}
# MY_PROJECT     → {{ PROJECT_NAME }}
# My-Project     → {{ Project-Name }}
# MY-PROJECT     → {{ PROJECT-NAME }}
```

**Arguments:**
- `<TOKEN>` - Compound word to replace (must contain hyphens, underscores, or mixed case)
- `<REPLACEMENT>` - Jinja2 template variable (e.g., `{{ project_name }}`)
- `[TARGET]` - Target directory (defaults to current directory)

**Options:**
- `-p, --path` - Transform file and directory names
- `-c, --contents` - Transform file contents
- `--dry-run` - Preview changes without applying them
- `--interactive` - Prompt for each change with diff preview

### `exact` - Precise Token Replacement

Replaces exact token matches with specified Jinja2 syntax.

```bash
templatize exact <TOKEN> <REPLACEMENT> [TARGET] [OPTIONS]
```

**Example:**
```bash
templatize exact "MyCompany Inc." "{{ company_name }}" -p -c

# Replaces only exact matches of "MyCompany Inc." with "{{ company_name }}"
```

**Arguments:**
- `<TOKEN>` - Exact token to replace
- `<REPLACEMENT>` - Jinja2 template variable
- `[TARGET]` - Target directory (defaults to current directory)

**Options:**
- `-p, --path` - Transform file and directory names
- `-c, --contents` - Transform file contents
- `--dry-run` - Preview changes without applying them
- `--interactive` - Prompt for each change with diff preview

## Recommended Workflow

> ⚠️ **Important**: Always run `templatize escape` FIRST, before any other commands. Running escape after creating template variables would escape your newly created `{{ variables }}`, breaking your templates.

### Standard Workflow

Handle existing Jinja syntax first, then apply transformations from broad to specific:

```bash
# Step 1: ALWAYS escape existing Jinja syntax first
templatize escape --interactive

# Step 2: Apply broad case shape transformations  
templatize shapes "my-awesome-project" "{{ project_name }}" -p -c --interactive

# Step 3: Handle specific cases that shapes missed or got wrong
templatize exact "AwesomeLib" "{{ library_name }}" -p -c --interactive

# Step 4: Handle version numbers, dates, or other specific tokens
templatize exact "1.0.0" "{{ version }}" -c --interactive
templatize exact "2024" "{{ current_year }}" -c --interactive
```

### Alternative Workflow (When No Existing Templates)

If your project doesn't contain any existing Jinja syntax, you can skip the escape step:

```bash
# Step 1: Apply broad case shape transformations
templatize shapes "my-project" "{{ project_name }}" -p -c --interactive

# Step 2: Handle specific edge cases
templatize exact "SPECIAL_CONSTANT" "{{ app_constant }}" -c --interactive
```

## Practical Examples

### Example 1: Converting a React Project

```bash
# Navigate to your React project
cd my-react-app

# 1. Escape existing template syntax (if any)
templatize escape --dry-run
templatize escape

# 2. Transform the main project name across all case variants
templatize shapes "my-react-app" "{{ app_name }}" -p -c --dry-run
templatize shapes "my-react-app" "{{ app_name }}" -p -c

# 3. Handle specific branding
templatize exact "My Amazing React App" "{{ app_display_name }}" -c --interactive

# 4. Replace author information
templatize exact "John Doe" "{{ author_name }}" -c --interactive
templatize exact "john@example.com" "{{ author_email }}" -c --interactive

# 5. Handle version and year
templatize exact "1.0.0" "{{ version }}" -c --interactive
templatize exact "2024" "{{ current_year }}" -c --interactive
```

### Example 2: Converting a Python Package

```bash
cd my-python-package

# 1. Escape existing Jinja (common in setup.py templates)
templatize escape --interactive

# 2. Transform package name variants
templatize shapes "my-python-package" "{{ package_name }}" -p -c --interactive

# 3. Handle module-specific names that might differ
templatize exact "MyPythonPackageError" "{{ package_name | title }}Error" -c --interactive

# 4. Replace metadata
templatize exact "A fantastic Python package" "{{ package_description }}" -c --interactive
templatize exact "MIT" "{{ license }}" -c --interactive
```

### Example 3: Converting a Java Project

```bash
cd my-java-project

# 1. Escape existing Jinja syntax (if any)
templatize escape --interactive

# 2. Transform the main project name across all case variants
templatize shapes "my-java-project" "{{ project_name }}" -p -c --interactive

# 3. Handle Java package structure (multi-level directory paths)
templatize exact "com/acme/widgets" "{{ package_root }}" -p --interactive

# 4. Replace specific company/project references
templatize exact "Acme Corporation" "{{ company_name }}" -c --interactive
templatize exact "widgets@acme.com" "{{ contact_email }}" -c --interactive

# 5. Handle version and configuration
templatize exact "1.0.0-SNAPSHOT" "{{ version }}" -c --interactive
```

### Example 4: Converting a Documentation Site

```bash
cd my-docs

# 1. Escape existing Jinja syntax (common in static site generators)
templatize escape --interactive

# 2. Transform project references
templatize shapes "awesome-docs" "{{ project_name }}" -p -c --interactive

# 3. Handle specific documentation strings
templatize exact "Awesome Documentation Site" "{{ site_title }}" -c --interactive
templatize exact "https://awesome-docs.com" "{{ site_url }}" -c --interactive

# 4. Replace navigation and branding
templatize exact "© 2024 Awesome Corp" "{{ copyright_notice }}" -c --interactive
```

## Interactive Mode

Interactive mode (`--interactive` or `-i`) shows you a diff preview of each change and asks for confirmation:

```
📁 File: src/components/MyProject.tsx

📋 Content change:
-  const projectName = 'MyProject';
+  const projectName = '{{ ProjectName }}';

❓ Apply this change? [y/N/q] y
✅ Applied change

📁 Path change: File
-  src/components/MyProject.tsx
+  src/components/{{ ProjectName }}.tsx

❓ Apply this change? [y/N/q] y
✅ Applied change
```

## Best Practices

### 1. Always Use Dry Run First

```bash
# Preview changes before applying
templatize shapes "my-project" "{{ project_name }}" -p -c --dry-run
```

### 2. Use Interactive Mode for Precision

```bash
# Review each change individually
templatize shapes "my-project" "{{ project_name }}" -p -c --interactive
```

### 3. Follow the Correct Order (Critical!)

1. **Escape** existing Jinja syntax FIRST (always!)
2. **Shapes** for broad compound word transformations  
3. **Exact** for precise, specific replacements

> ⚠️ Never run `escape` after `shapes` or `exact` - it will escape your template variables!

### 4. Handle Different Content Types

```bash
# Paths and contents together
templatize shapes "my-app" "{{ app_name }}" -p -c

# Just file contents (safer for first attempts)
templatize exact "MyCompany" "{{ company }}" -c

# Just paths (for renaming files/directories)
templatize shapes "my-project" "{{ project_name }}" -p
```

### 5. Test Your Templates

After templatizing, test your template with a template engine:

```bash
# Using Jinja2 CLI or your preferred templating tool
jinja2 your-template.txt.j2 --format=json < variables.json
```

## Common Patterns

### Package/Project Names

```bash
# Transform all variants of a project name
templatize shapes "my-awesome-app" "{{ app_name }}" -p -c
```

### Company/Author Information

```bash
# Replace exact company references
templatize exact "Acme Corporation" "{{ company_name }}" -c
templatize exact "contact@acme.com" "{{ contact_email }}" -c
```

### Version and Date Information

```bash
# Replace version strings
templatize exact "1.0.0" "{{ version }}" -c
templatize exact "2024-01-01" "{{ release_date }}" -c
templatize exact "2024" "{{ current_year }}" -c
```

### Configuration Values

```bash
# Database and service configurations
templatize exact "localhost:5432" "{{ database_url }}" -c
templatize exact "my-service-key-123" "{{ api_key }}" -c
```

### Package/Namespace Directory Structures

```bash
# Transform multi-level directory paths (Java packages, etc.)
templatize exact "com/acme/widgets" "{{ package_root }}" -p

# Before: src/main/java/com/acme/widgets/entities/User.java
# After:  src/main/java/{{ package_root }}/entities/User.java

# Python package structures
templatize exact "my_company/my_project" "{{ python_package }}" -p

# Before: src/my_company/my_project/models/user.py  
# After:  src/{{ python_package }}/models/user.py
```

## Troubleshooting

### Template Variables Got Escaped

If your template variables like `{{ project_name }}` became `{{'{'}}{ project_name }`, you ran `escape` after creating them:

```bash
# ❌ Wrong order - this breaks your templates
templatize shapes "my-app" "{{ app_name }}" -p -c
templatize escape  # This will escape the {{ app_name }} you just created!

# ✅ Correct order
templatize escape  # Escape existing templates first
templatize shapes "my-app" "{{ app_name }}" -p -c  # Then create new ones
```

### "Not a compound word" Error

The `shapes` command requires compound words with separators:

```bash
# ❌ This will fail
templatize shapes "example" "{{ name }}" -p -c

# ✅ These work
templatize shapes "example-name" "{{ name }}" -p -c
templatize shapes "ExampleName" "{{ name }}" -p -c
templatize shapes "example_name" "{{ name }}" -p -c
```

### Unexpected Replacements

Use `--dry-run` and `--interactive` to preview changes:

```bash
# Preview all changes
templatize shapes "my-app" "{{ app_name }}" -p -c --dry-run

# Review each change individually
templatize shapes "my-app" "{{ app_name }}" -p -c --interactive
```

### Binary Files

Templatize automatically skips binary files when processing contents. Only text files are modified.

## Global Options

- `-v, --verbose` - Show detailed logging information
- `-q, --quiet` - Suppress all output except errors
- `-h, --help` - Show help information

## Project Structure

This project follows a workspace-based architecture:

- **`crates/templatize-bin/`** - CLI application and user interface
- **`crates/templatize-core/`** - Core templating logic and algorithms  
- **`xtask/`** - Build automation and development tasks

### Development Commands

```bash
# Run comprehensive test suite
cargo xtask test all

# Run specific test suites
cargo xtask test core         # Core library tests
cargo xtask test bin          # Binary crate tests  
cargo xtask test integration  # Integration tests

# Build and run with arguments
cargo xtask run -- exact "token" "{{ replacement }}" -p -c

# Install locally
cargo xtask install
```

For detailed technical information:

- [Technical Specification](SPECIFICATION.md)
- [Binary Crate Documentation](crates/templatize-bin/README.md)  
- [Core Library Documentation](crates/templatize-core/README.md)

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to our repository.

## License

MIT License - see LICENSE file for details.
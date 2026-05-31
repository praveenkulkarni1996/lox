# Lox

A Rust implementation of the Lox programming language from [Crafting Interpreters](https://craftinginterpreters.com/).

## Setup

### Pre-commit Hooks

This project uses git pre-commit hooks to ensure code quality. Before committing, the following checks are automatically run:

- `cargo fmt --check` - Code formatting
- `cargo clippy -- -D warnings` - Linting (treats warnings as errors)
- `cargo test` - Unit and integration tests
- `cargo doc --no-deps` - Documentation builds

#### First Time Setup

Run the setup script to configure git to use the project's hooks:

```bash
./scripts/setup-hooks.sh
```

This configures git to automatically run these checks before each commit.

#### What Happens

If any check fails, the commit is blocked. You'll need to fix the issues before committing:

- **Format errors**: Run `cargo fmt` to auto-fix
- **Clippy warnings**: Fix the reported issues or suppress with `#[allow(...)]`
- **Test failures**: Debug and fix the failing tests
- **Doc build failures**: Fix documentation issues

#### Bypassing Hooks (Not Recommended)

If absolutely necessary, you can skip the hooks for a single commit:

```bash
git commit --no-verify -m "your message"
```

However, this bypasses the quality checks and is not recommended.

## Development

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Check code quality:
```bash
cargo fmt --check
cargo clippy -- -D warnings
```

Build documentation:
```bash
cargo doc --no-deps --open
```

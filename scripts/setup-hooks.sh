#!/bin/bash
# Setup script to configure git to use project hooks

set -e

echo "Configuring git to use project hooks..."
git config core.hooksPath scripts/hooks

echo "✓ Git pre-commit hooks configured!"
echo ""
echo "Pre-commit checks will now run automatically before each commit:"
echo "  - cargo fmt --check"
echo "  - cargo clippy -- -D warnings"
echo "  - cargo test"
echo "  - cargo doc --no-deps"

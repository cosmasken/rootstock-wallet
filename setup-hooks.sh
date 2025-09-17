#!/bin/bash
# Setup git hooks for the project

echo "Setting up git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "Git hooks installed successfully!"
echo "Now rustfmt will run automatically before each commit."
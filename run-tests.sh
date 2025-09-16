#!/bin/bash
set -e

echo "🧪 Running Rootstock Wallet Tests"
echo "================================="

echo "📋 Checking code formatting..."
cargo fmt --check

echo "🔍 Running clippy..."
cargo clippy -- -D warnings

echo "🧪 Running tests..."
cargo test --verbose

echo "🏗️  Building release..."
cargo build --release --verbose

echo "✅ All tests passed!"

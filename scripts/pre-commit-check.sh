#!/bin/bash
# Pre-commit check script
# コミット前に実行してCIエラーを防ぐ

set -e  # エラーがあれば即座に終了

echo "🔍 Running pre-commit checks..."
echo ""

# 1. Format check
echo "1️⃣ Format check..."
cargo fmt --check
echo "   ✅ Format check passed"
echo ""

# 2. Clippy (all targets)
echo "2️⃣ Clippy (all targets)..."
cargo clippy --all-targets -- -D warnings
echo "   ✅ Clippy passed"
echo ""

# 3. Tests
echo "3️⃣ Tests..."
cargo test
echo "   ✅ Tests passed"
echo ""

# 4. Release build
echo "4️⃣ Release build..."
cargo build --release
echo "   ✅ Release build passed"
echo ""

echo "✅ All checks passed! Safe to commit."

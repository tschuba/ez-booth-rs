#!/bin/bash

# EZ Booth Test Runner
# Runs all automated tests for the project

set -e  # Exit on error

echo "🧪 EZ Booth Test Suite"
echo "===================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️  wasm-pack not found. Install it with:"
    echo "   cargo install wasm-pack"
    echo ""
    echo "Skipping browser integration tests..."
    SKIP_BROWSER_TESTS=true
else
    SKIP_BROWSER_TESTS=false
fi

echo "${BLUE}📦 Running Unit Tests${NC}"
echo "-------------------"
cargo test --workspace --lib
echo ""

if [ "$SKIP_BROWSER_TESTS" = false ]; then
    echo "${BLUE}🌐 Running Browser Integration Tests${NC}"
    echo "------------------------------------"
    wasm-pack test --headless --chrome crates/storage
    echo ""
fi

echo "${GREEN}✅ All tests passed!${NC}"
echo ""
echo "📊 Test Summary:"
cargo test --workspace --lib -- --list | grep -E "^test " | wc -l | xargs echo "  Unit tests:"

if [ "$SKIP_BROWSER_TESTS" = false ]; then
    echo "  Integration tests: 7"
fi

echo ""
echo "For more testing options, see TESTING.md"

#!/bin/bash
# Script to diagnose what's causing the infinite compilation loop

echo "🔍 Diagnosing Compilation Issues"
echo "================================"

# Check for potential circular dependencies
echo "1. Checking for circular dependencies..."
cargo tree --duplicates 2>/dev/null | head -20 || echo "   No obvious duplicates found"

echo ""
echo "2. Checking for problematic test files..."

# Look for potentially problematic patterns in test files
echo "   Searching for recursive or infinite patterns..."

# Check for infinite loops in tests
find tests/ -name "*.rs" -exec grep -l "loop\|while.*true\|for.*in.*infinite" {} \; 2>/dev/null | head -5

# Check for macro recursion
find tests/ -name "*.rs" -exec grep -l "macro_rules\|recursive" {} \; 2>/dev/null | head -5

# Check for large test files that might cause memory issues
echo "   Large test files (>1000 lines):"
find tests/ -name "*.rs" -exec wc -l {} \; 2>/dev/null | awk '$1 > 1000 {print $2 " (" $1 " lines)"}' | head -5

echo ""
echo "3. Checking compilation cache..."
if [ -d "target" ]; then
    echo "   Target directory size: $(du -sh target 2>/dev/null | cut -f1)"
    echo "   Number of cached files: $(find target -type f 2>/dev/null | wc -l)"
else
    echo "   No target directory found"
fi

echo ""
echo "4. Checking for memory and CPU intensive dependencies..."

# Check dependencies that might cause issues
cargo tree --format="{p}" | grep -E "syn|quote|proc-macro|serde_derive|tokio-macros" | head -10

echo ""
echo "5. System resource check..."
echo "   Available memory: $(free -h | awk 'NR==2{print $7}')"
echo "   CPU cores: $(nproc)"
echo "   Load average: $(uptime | grep -o 'load average: .*')"

echo ""
echo "6. Rust compilation environment..."
echo "   Rust version: $(rustc --version)"
echo "   Cargo version: $(cargo --version)"
echo "   Target architecture: $(rustc -vV | grep host | cut -d' ' -f2)"

echo ""
echo "💡 Recommendations:"
echo "   - Use the prevent_test_loops.sh script for safe testing"
echo "   - Consider running tests individually"
echo "   - Clear target directory if issues persist: rm -rf target"
echo "   - Monitor system resources during compilation"
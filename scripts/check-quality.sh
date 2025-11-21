#!/usr/bin/env bash
# Quality checks script - mimics pre-commit hooks
# Run this manually if you don't have pre-commit installed

set -e  # Exit on first error

echo "🔍 Running code quality checks..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

# Function to run a check
run_check() {
    local name="$1"
    shift
    echo -n "  [$name] "
    if "$@" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo -e "${RED}    Failed: $*${NC}"
        FAILED=1
    fi
}

# Rust checks
echo "📦 Rust checks:"
run_check "cargo fmt" cargo fmt --all -- --check
run_check "cargo clippy" cargo clippy --all-targets --all-features -- -D warnings
run_check "cargo test" cargo test --lib
run_check "cargo doc" cargo doc --no-deps

echo ""
echo "📝 File checks:"

# Check for trailing whitespace (excluding .lock files)
if git ls-files | grep -v '\.lock$' | xargs grep -l '[[:space:]]$' > /dev/null 2>&1; then
    echo -e "  [trailing whitespace] ${RED}✗${NC}"
    echo -e "${RED}    Files with trailing whitespace:${NC}"
    git ls-files | grep -v '\.lock$' | xargs grep -l '[[:space:]]$' | sed 's/^/      /'
    FAILED=1
else
    echo -e "  [trailing whitespace] ${GREEN}✓${NC}"
fi

# Check for files without final newline
check_final_newline() {
    local file="$1"
    if [ -f "$file" ] && [ -s "$file" ]; then
        if [ "$(tail -c 1 "$file" | wc -l)" -eq 0 ]; then
            return 1
        fi
    fi
    return 0
}

FILES_WITHOUT_NEWLINE=()
while IFS= read -r file; do
    if ! check_final_newline "$file"; then
        FILES_WITHOUT_NEWLINE+=("$file")
    fi
done < <(git ls-files | grep -v '\.lock$')

if [ ${#FILES_WITHOUT_NEWLINE[@]} -gt 0 ]; then
    echo -e "  [final newline] ${RED}✗${NC}"
    echo -e "${RED}    Files without final newline:${NC}"
    printf '      %s\n' "${FILES_WITHOUT_NEWLINE[@]}"
    FAILED=1
else
    echo -e "  [final newline] ${GREEN}✓${NC}"
fi

# Summary
echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some checks failed. Please fix the issues above.${NC}"
    echo ""
    echo "To fix formatting issues:"
    echo "  cargo fmt --all"
    echo ""
    echo "To fix clippy warnings:"
    echo "  cargo clippy --all-targets --all-features -- -D warnings"
    echo ""
    exit 1
fi


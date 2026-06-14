#!/usr/bin/env bash
# Runs all 30 CLI integration tests and reports results.
# Usage: bash scripts/run_cli_tests.sh [--binary path/to/verdict-cli]
#
# Prerequisites:
#   python3 scripts/gen_cli_tests.py  # generate fixtures (requires pyarrow)
#   cargo build -p verdict-cli        # build the CLI

set -euo pipefail

BINARY="${1:-./target/debug/verdict-cli}"
FIXTURE_BASE="fixtures/cli_tests"
PASS=0
FAIL=0
ERRORS=()

if [ ! -f "$BINARY" ]; then
    echo "Binary not found: $BINARY"
    echo "Run: cargo build -p verdict-cli"
    exit 1
fi

if [ ! -d "$FIXTURE_BASE" ]; then
    echo "Fixtures not found: $FIXTURE_BASE"
    echo "Run: python scripts/gen_cli_tests.py"
    exit 1
fi

for dir in "$FIXTURE_BASE"/*/; do
    name=$(basename "$dir")

    if [ -f "$dir/data.parquet" ]; then
        data="$dir/data.parquet"
    elif [ -f "$dir/data.csv" ]; then
        data="$dir/data.csv"
    else
        echo "SKIP  $name  (no data file)"
        continue
    fi

    actual=$(mktemp /tmp/verdict_XXXXXX.json)
    # Run CLI; capture output regardless of exit code (validation failures → exit 1)
    "$BINARY" "$data" "$dir/schema.json" --format json > "$actual" 2>/tmp/verdict_err.txt || true

    cmp_out=$(mktemp /tmp/verdict_cmp_XXXXXX.txt)
    if python3 scripts/compare_results.py "$dir" "$actual" > "$cmp_out" 2>&1; then
        echo "OK    $name"
        PASS=$((PASS + 1))
    else
        echo "FAIL  $name"
        cat "$cmp_out" | sed 's/^/      /'
        FAIL=$((FAIL + 1))
        ERRORS+=("$name")
    fi

    rm -f "$actual" "$cmp_out"
done

echo ""
echo "Results: $PASS passed, $FAIL failed"

if [ "${#ERRORS[@]}" -gt 0 ]; then
    echo "Failed cases:"
    for e in "${ERRORS[@]}"; do
        echo "  - $e"
    done
    exit 1
fi

#!/usr/bin/env bash
# run_mutants.sh — run cargo-mutants with a survival rate threshold check
#
# Usage:
#   ./scripts/run_mutants.sh              # uses default 20% max survival rate
#   ./scripts/run_mutants.sh --threshold 15   # custom threshold
#   ./scripts/run_mutants.sh --list        # just list mutants, don't run
#   ./scripts/run_mutants.sh --file src/format.rs  # only mutants in one file
#
# Exits 0 if survival rate is at or below threshold, 1 if above.
# Baseline (Day 9): 1004 total mutants.

set -euo pipefail

THRESHOLD=20   # max allowed survival rate (percentage)
LIST_ONLY=false
FILE_FILTER=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --threshold)
            THRESHOLD="$2"
            shift 2
            ;;
        --list)
            LIST_ONLY=true
            shift
            ;;
        --file)
            FILE_FILTER="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--threshold N] [--list] [--file PATH]"
            echo ""
            echo "Options:"
            echo "  --threshold N   Max allowed survival rate percentage (default: 20)"
            echo "  --list          Just list mutants without running them"
            echo "  --file PATH     Only test mutants in a specific file"
            echo ""
            echo "Baseline (Day 9): 1004 mutants"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check cargo-mutants is installed
if ! cargo mutants --version >/dev/null 2>&1; then
    echo "cargo-mutants not found. Install with: cargo install cargo-mutants"
    exit 1
fi

# Build filter args
FILTER_ARGS=""
if [[ -n "$FILE_FILTER" ]]; then
    FILTER_ARGS="-f $FILE_FILTER"
fi

# List-only mode
if [[ "$LIST_ONLY" == "true" ]]; then
    # shellcheck disable=SC2086
    MUTANT_COUNT=$(cargo mutants --list $FILTER_ARGS 2>/dev/null | wc -l)
    echo "Total mutants: $MUTANT_COUNT"
    exit 0
fi

echo "=== arc mutation testing ==="
echo "Threshold: ${THRESHOLD}% max survival rate"
echo ""

# Run cargo mutants and capture output
# shellcheck disable=SC2086
cargo mutants $FILTER_ARGS 2>&1 | tee /tmp/mutants_output.txt

echo ""
echo "=== Results ==="

# Parse results from mutants.out/
CAUGHT=0
SURVIVED=0
TIMEOUT=0
UNVIABLE=0

if [[ -f mutants.out/caught.txt ]]; then
    CAUGHT=$(wc -l < mutants.out/caught.txt)
fi
if [[ -f mutants.out/survived.txt ]]; then
    SURVIVED=$(wc -l < mutants.out/survived.txt)
fi
if [[ -f mutants.out/timeout.txt ]]; then
    TIMEOUT=$(wc -l < mutants.out/timeout.txt)
fi
if [[ -f mutants.out/unviable.txt ]]; then
    UNVIABLE=$(wc -l < mutants.out/unviable.txt)
fi

TESTED=$((CAUGHT + SURVIVED))

echo "Caught:   $CAUGHT"
echo "Survived: $SURVIVED"
echo "Timeout:  $TIMEOUT"
echo "Unviable: $UNVIABLE"

if [[ "$TESTED" -eq 0 ]]; then
    echo ""
    echo "No mutants were tested. Nothing to check."
    exit 0
fi

# Calculate survival rate (integer math, rounded up to be conservative)
SURVIVAL_RATE=$(( (SURVIVED * 100 + TESTED - 1) / TESTED ))

echo ""
echo "Survival rate: ${SURVIVAL_RATE}% ($SURVIVED / $TESTED)"
echo "Threshold:     ${THRESHOLD}%"

if [[ "$SURVIVAL_RATE" -gt "$THRESHOLD" ]]; then
    echo ""
    echo "FAIL: survival rate ${SURVIVAL_RATE}% exceeds threshold ${THRESHOLD}%"
    echo ""
    echo "Surviving mutants (test gaps):"
    if [[ -f mutants.out/survived.txt ]]; then
        cat mutants.out/survived.txt
    fi
    exit 1
else
    echo ""
    echo "PASS: survival rate ${SURVIVAL_RATE}% is within threshold ${THRESHOLD}%"
    exit 0
fi

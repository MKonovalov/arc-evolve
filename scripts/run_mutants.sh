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
#
# Per-file survival artifact:
# When a real (non --list) run completes, this script ALSO writes a
# machine-readable per-file mutation-survival summary to `.arc/mutants_per_file.json`
# (a gitignored location). This is a *sensor* artifact for arc's risk scorer —
# wiring it in as a prediction signal is a FUTURE step, not done here.
#
# Schema of `.arc/mutants_per_file.json`:
# {
#   "tool": "cargo-mutants",
#   "git_head": "abcd123...",             // current commit (short hash)
#   "generated_at": "2026-08-20T16:40:00Z",
#   "files": [
#     {
#       "path": "src/format/mod.rs",      // file path (cargo-mutants' own key)
#       "mutants": 12,                    // total mutants generated in this file
#       "killed": 10,                     // caught by a failing test
#       "survived": 2                     // not caught by any test (incl. timeout)
#     }
#   ],
#   "summary": {
#     "files": 3,                         // number of files with mutants
#     "mutants": 30,
#     "killed": 25,
#     "survived": 5
#   }
# }
#
# The write is purely additive: it never changes the pass/fail exit code of the
# script. If cargo-mutants isn't installed, or produces no usable mutants.json /
# outcomes.json, the JSON write is skipped silently and the normal threshold
# check still runs.

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

# --- Per-file survival sensor artifact (additive; never affects exit code) ---
# Writes .arc/mutants_per_file.json from cargo-mutants' mutants.json (all
# generated mutants, keyed by file path) joined against outcomes.json (per-id
# results). This is fragile against schema drift across cargo-mutants versions,
# so any failure simply skips the write. See header comment for the schema.
emission_failed=0
if [[ -f mutants.out/mutants.json && -f mutants.out/outcomes.json ]]; then
    if ! python3 - <<'PYEOF' >/dev/null 2>&1
import json, sys, os, datetime

# Locate cargo-mutants' output; the script runs from the repo root, but be
# defensive about cwd variants.
mutants_path = None
for cand in ("mutants.out/mutants.json", os.path.join(os.getcwd(), "mutants.out/mutants.json")):
    if os.path.isfile(cand):
        mutants_path = cand
        break
if mutants_path is None:
    sys.exit(0)  # nothing to emit; not an error
out_dir = os.path.dirname(mutants_path)

def load_json(path):
    try:
        with open(path, "r", encoding="utf-8") as fh:
            return json.load(fh)
    except Exception:
        return None

mutants = load_json(os.path.join(out_dir, "mutants.json"))
outcomes = load_json(os.path.join(out_dir, "outcomes.json"))
if not isinstance(mutants, dict) or outcomes is None:
    sys.exit(0)  # unexpected shape; skip emit

# Build per-id outcome lookup. outcomes.json may be keyed by id, or contain a
# "mutants" list whose items each carry an "id"/"mutant" plus a result word.
result_by_id = {}
if isinstance(outcomes, dict):
    for key, val in outcomes.items():
        if key == "mutants" and isinstance(val, list):
            for item in val:
                if isinstance(item, dict):
                    mid = item.get("id") or (item.get("mutant") or {}).get("id")
                    if mid is not None:
                        st = item.get("test_status") or item.get("result") or ""
                        result_by_id[str(mid)] = str(st)
        elif key != "mutants" and isinstance(val, (str, int)):
            # tolerate a flat {"<id>": "<outcome>"} map
            result_by_id[key] = str(val)

KILLED = {"caught", "caughtmutant", "killed"}
SURVIVED = {"missed", "missedmutant", "survived", "timeout", "timeoutmutant"}

def classify(st):
    s = str(st).strip().lower()
    if s in KILLED:
        return "killed"
    if s in SURVIVED:
        return "survived"
    return "other"

files = []
for path, mlist in mutants.items():
    if not isinstance(mlist, list):
        continue
    kills = surv = 0
    for m in mlist:
        if not isinstance(m, dict):
            continue
        mid = m.get("id")
        if mid is None:
            continue
        c = classify(result_by_id.get(str(mid), ""))
        if c == "killed":
            kills += 1
        elif c == "survived":
            surv += 1
    if kills or surv:
        files.append({
            "path": path,
            "mutants": len(mlist),
            "killed": kills,
            "survived": surv,
        })
files.sort(key=lambda f: (f["survived"], f["path"]), reverse=True)

summary = {
    "files": len(files),
    "mutants": sum(f["mutants"] for f in files),
    "killed": sum(f["killed"] for f in files),
    "survived": sum(f["survived"] for f in files),
}

git_head = ""
try:
    import subprocess
    git_head = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        capture_output=True, text=True, check=False,
    ).stdout.strip()
except Exception:
    git_head = ""

artifact = {
    "tool": "cargo-mutants",
    "git_head": git_head,
    "generated_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "files": files,
    "summary": summary,
}

os.makedirs(".arc", exist_ok=True)
with open(".arc/mutants_per_file.json", "w", encoding="utf-8") as fh:
    json.dump(artifact, fh, indent=2, sort_keys=True)
PYEOF
    then
        echo "Wrote .arc/mutants_per_file.json (per-file mutation survival sensor)"
    else
        emission_failed=1
    fi
fi

if [[ "$emission_failed" == "1" ]]; then
    echo "(note: could not emit .arc/mutants_per_file.json — sensor write skipped)"
fi

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

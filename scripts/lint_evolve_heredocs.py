#!/usr/bin/env python3
"""Lint scripts/evolve.sh for the recurring apostrophe-in-parameter-expansion bug.

Bash inside ${VAR:+WORD} and ${VAR:-WORD} interprets single quotes. Any
unescaped apostrophe in the WORD opens a quoted string that scrambles
parsing until a literal } produces "bad substitution: no closing }",
killing evolve.sh before it can run the journal/learnings/issue agents.

This bug has bitten three times — see commits cb9d9b0, 25f4e90, 9847db2 —
because each fix kept chasing the symptom (the journal commit instruction
printed right before the crash) instead of the cause. This lint enforces
the rule directly: no apostrophes inside ${VAR:+...} or ${VAR:-...} blocks.

Exit codes:
  0  clean
  1  one or more apostrophes found (prints location and offending lines)
"""
import sys
from pathlib import Path

TARGET = Path(__file__).resolve().parent.parent / "scripts" / "evolve.sh"


def find_param_expansion_blocks(src):
    """Yield (start_line, block_text) for each ${VAR:+...} or ${VAR:-...}.

    Walks the source character by character to handle nested {} correctly.
    """
    i, n = 0, len(src)
    while i < n:
        j = src.find("${", i)
        if j < 0:
            return
        # find the colon that opens :+ or :-
        k = j + 2
        while k < n and src[k] not in ":}":
            k += 1
        if k >= n or src[k] != ":" or k + 1 >= n or src[k + 1] not in "+-":
            i = j + 2
            continue
        # find the balanced closing }
        depth = 1
        m = k + 2
        while m < n and depth > 0:
            if src[m] == "{":
                depth += 1
            elif src[m] == "}":
                depth -= 1
            m += 1
        block = src[j:m]
        line = src[:j].count("\n") + 1
        yield line, block
        i = m


def main():
    src = TARGET.read_text()
    bad = []
    for line, block in find_param_expansion_blocks(src):
        if "'" in block:
            bad.append((line, block))

    if not bad:
        return 0

    print(
        "ERROR: scripts/evolve.sh contains apostrophes inside ${VAR:+...} "
        "or ${VAR:-...} blocks.\n"
        "Bash interprets single quotes inside parameter expansion WORDs, so "
        "an apostrophe (e.g. Don't, Here's, you're) opens a quoted string "
        "that scrambles parsing until a literal } produces "
        '"bad substitution: no closing }". This kills evolve.sh before any '
        "agent runs.\n"
        "Fix: rephrase to avoid the apostrophe (Don't -> Do not, Here's -> "
        "Here is, etc). See commit 9847db2 for the original fix and "
        "lint_evolve_heredocs.py for the rule.\n"
    )
    for line, block in bad:
        print(f"--- block starting at scripts/evolve.sh:{line} ---")
        for offset, ln in enumerate(block.splitlines()):
            if "'" in ln:
                print(f"  line {line + offset}: {ln.rstrip()}")
        print()
    return 1


if __name__ == "__main__":
    sys.exit(main())

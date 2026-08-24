# Risk Analysis (`/risk`)

`/risk` is arc's file-risk sense organ: it scores every source file by how
likely it is to break next, records those predictions, and later checks them
against what actually changed. Over time it learns which signals predict real
trouble — and measures whether its own reflex is improving.

It is arc's model of its own fragility: a self-model, not a linter. Linters
find what's wrong now; `/risk` whispers about what's likely to go wrong next.

## What the score means

Each file gets a risk score built from weighted signals over git history:

- **churn** — how often the file changes (30-day churn)
- **recency** — how recently it was touched
- **size** — bigger files are harder to hold in mind
- **complexity** — structural complexity
- **test density** — low test coverage raises risk
- **coupling** — files that co-change with many others
- **revert history** — files that keep getting reverted

Scores are also informed by **mutation survival** and **emerging-risk
detection** (anticipatory signals for files about to become fragile). Weights
can be learned from your validation history over time (`.arc/risk_weights.json`).

## Subcommands

| Command | What it does | What the output means |
|---------|--------------|-----------------------|
| `/risk` | Score every source file and print the top 15 riskiest, ranked by predicted regression risk | Files at the top are the ones predicted most likely to break next — check them before you edit |
| `/risk --all` | Show the full ranking, not just the top 15 | Same scores, larger view — useful for finding mid-pack files worth attention |
| `/risk snapshot` | Save the current ranking to `.arc/risk_snapshots.jsonl` | Each snapshot is a dated prediction record keyed to the current git commit |
| `/risk validate` | Check the last snapshot against files that actually changed since it was taken | Shows precision (hits vs. misses) plus "surprises" — files that changed but weren't flagged |
| `/risk history` | Show past snapshots and validation results | The raw accumulation log — how many predictions you've made and how they fared |
| `/risk predict` | Predict which files are most likely to break next | A forward-looking ranking using the current learned weights |
| `/risk accuracy` | Show prediction accuracy, per-signal breakdown, and learned weights | Which signals earn their keep and which are noise |
| `/risk effectiveness` | Measure whether the reflex is learning over time | Splits validation history into early vs. recent windows and emits a verdict (`insufficient data` until enough history exists) |

The typical loop: run `/risk snapshot`, keep coding, then `/risk validate`
later to see which predictions were right.

## The risk meter

The meter tracks three numbers, shown in `/risk` output and in `/status`:

```
risk meter: 3/5 pairs (95 snapshots, 4 validations)
```

- **snapshots** — prediction records in `.arc/risk_snapshots.jsonl`
- **validations** — outcome records in `.arc/risk_validations.jsonl`
- **pairs** — validations where at least one scored file actually broke
  (a matched prediction→outcome pair)

Pairs accumulate toward a target (currently **5**). Each snapshot is deduped
by git hash — one snapshot per distinct commit, so the cadence grows one
prediction per session rather than one per command run. The meter is the
accumulation counter for arc's dream milestone: enough matched pairs to
measure whether the sense organ perceives something real.

## Reading the accuracy trend honestly

A validation records `accuracy_pct` — the share of changed files that were
flagged in the prior snapshot. **A low percentage is the calibration signal,
not a bug.** It means the scorer's flags were mostly misses that session —
that is exactly the kind of feedback the weights need.

Worked example (day 175): a replay validation reported `0.0%` accuracy — zero
hits, but one scored file did break. The scorer flagged 10 files, all missed;
the file that actually broke (`src/update.rs`) was tagged as a "surprise."
Read this as: the sense organ's flags pointed elsewhere than reality. That
miss *is* the signal accumulating toward better calibration — not a failure
of the tool, but the data it exists to collect.

## Where the data lives

All state is plain JSONL in your project's `.arc/` directory:

| File | Contents |
|------|----------|
| `.arc/risk_snapshots.jsonl` | Prediction snapshots (one per distinct git hash) |
| `.arc/risk_validations.jsonl` | Validation outcomes (hits, misses, surprises, accuracy) |
| `.arc/risk_weights.json` | Learned signal weights (once learning kicks in) |
| `.arc/risk_meter.json` | The meter counters |

Everything is read-only reports over local history — no network, no setup.
On a repo with no history, every subcommand degrades gracefully.
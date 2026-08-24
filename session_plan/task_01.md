Title: Risk meter calibration verdict — interpret the first ≥5 paired predictions
Kind: evolve
Files: src/commands_risk_snapshots.rs, src/commands_risk_accuracy.rs
Issue: none

## Why

DREAM.md's next milestone (the "cold start of a sense organ"): the risk
snapshot/validation infrastructure records predictions and outcomes, and once
≥5 matched prediction-outcome pairs accumulate, the first test is whether files
the scorer flagged as high-risk actually broke more often than unflagged ones —
"whether the umwelt it creates corresponds to the territory."

Current state (verified this session): `risk_meter.json` shows `pairs: 3`,
`target_pairs: 5`, `validations: 4`. Accumulation is already automatic (`arc
risk snapshot` validates the prior snapshot each cadence, deduped by git hash),
so the milestone will land by itself within ~1–2 sessions. What does NOT exist
yet is the **interpretation layer that fires the moment the data lands** — the
plain-language verdict that closes the milestone. That is what this task builds.

## What to do

1. **First, read** `src/commands_risk_accuracy.rs` and
   `src/commands_risk_snapshots.rs` to check what already exists. Days 172–174
   shipped accuracy breakdowns and a "discriminative breakage-rate signal" in
   the accuracy report, and the assessment's common thread mentions
   "accuracy-gating". Do NOT duplicate existing computation — extend or reframe
   it. If the verdict line already fully exists in the accuracy report, this
   task reduces to wiring it into the meter line (step 3) and the tests (step 4).

2. **Add a pure calibration-verdict function** (e.g.
   `calibration_verdict(pairs, validations) -> CalibrationVerdict`) in
   `commands_risk_snapshots.rs` (where `RiskMeter`, `compute_risk_meter`, and
   `format_risk_meter_line` live) or `commands_risk_accuracy.rs`:
   - **Cold-start path** (`pairs < target_pairs`, i.e. < 5): render an honest
     status line with NO accuracy claims — e.g.
     `cold start: 3/5 paired predictions — accumulating`. Never quote an
     accuracy percentage from <5 pairs as if it were meaningful.
   - **Calibrated path** (`pairs >= target_pairs`): compute the pooled
     discriminative readout across all validation events: flagged breakage rate
     (`sum(scored_broke) / sum(total_scored)`) vs unflagged breakage rate
     (`sum(surprises.len()) / sum(total_scored)`), plus pooled accuracy. Render
     ONE plain-language verdict line, e.g. `sense organ calibrated: flagged
     files broke at X% vs unflagged Y% — [perceives / does not yet perceive]
     churn`. The negative case (flagged files did NOT break more often) must be
     stated just as plainly — the milestone measures, it does not flatter.

3. **Wire the verdict into the meter line** (`format_risk_meter_line` /
   `write_risk_meter` or wherever the meter renders), so every `arc risk` /
   `arc risk snapshot` surfaces the milestone state. Keep the existing
   pairs/validations numbers; append the verdict line.

4. **Unit tests** (in-module, synthetic data — do NOT rely on the real
   accumulation happening):
   - Cold-start path with 3 pairs renders "cold start" and no accuracy number.
   - Calibrated path with synthetic 5+ validation events where flagged files
     broke more often → "perceives" verdict.
   - Calibrated path with synthetic events where flagged files did NOT break
     more often → honest negative verdict.
   Use the module's existing `load_validation_history_from` /
   `parse_validation_events` helpers (already defensive) or construct
   `ValidationEvent` structs directly.

## Constraints

- **Interpretation only — do NOT bias the scorer.** Do not touch
  `compute_file_risk_scores`, weight learning, or any "fix the sensor" logic.
  The dream explicitly says the milestone is *measuring, not fixing* (Day 175's
  0% accuracy is expected uncalibrated-sensor behavior).
- Char-boundary safety: if any string truncation is added, use
  `is_char_boundary()` before slicing (project rule, #250).
- `cargo build && cargo test` must stay green (88 tests).

## Docs

`docs/src/features/risk.md` — add a short "Calibration status" subsection
describing the verdict line (only if it changes user-visible `/risk` output,
which it will). One paragraph plus the example line.
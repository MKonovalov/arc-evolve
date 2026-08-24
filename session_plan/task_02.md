Title: Rearchitect `handle_review` — dispatch to per-effort pure handlers, carve `review-*` helpers out of commands_git_review.rs
Kind: evolve
Files: src/commands_git_review.rs, src/commands_review.rs, src/main.rs (or lib.rs / dispatch_sub.rs)
Issue: none

## Why

The assessment flags `commands_risk.rs` (4670) as an over-module-size smell —
but the *pattern* is broader. Day 174's task "Make the risk report legible"
introduced an effort-level architecture (`ReviewEffort` with `label()` +
`parse_review_effort()`) in `commands_git_review.rs` (1379), which the
assessment calls "latent maintainability smell" territory. `handle_review` is
the second-largest single handler after `handle_risk` and mixes argument
parsing, prompt building, and per-effort behavior.

This is the **one-module-at-a-time** split: extract the review-effort handling
from `commands_git_review.rs` into a new `src/commands_review.rs`, mirroring how
`commands_risk.rs` grew satellites (`commands_risk_report.rs`,
`commands_risk_accuracy.rs`) that re-export so call sites are unchanged.

## What to do

1. **Read** `src/commands_git_review.rs` (1379 lines) first. Understand the
   current shape of `handle_review` and `ReviewEffort`.

2. **Restructure `handle_review`** into a dispatcher: parse args → select
   `ReviewEffort` → delegate the build-prompt-and-run core to per-effort pure
   helpers. The per-effort prompt-building logic currently in `build_review_prompt`
   and `build_review_prompt_structured` should become small, individually-testable
   functions parameterized by effort where it isn't already.

3. **Extract** the review-* helpers (e.g. `build_review_prompt`,
   `build_review_content`, `parse_review_comments`, `extract_review_json`) into
   `src/commands_review.rs`, keeping **public re-exports** from
   `commands_git_review.rs` so no call site changes (mirror the documented
   pattern: "re-exported via X so call sites are unchanged"). Wire into
   `main.rs` (the module list) wherever `commands_git_review` is declared.

4. **Kept in `commands_git_review.rs`**: the CLI-facing `handle_review`/
   `handle_blame` entry points and any git/PR plumbing unique to them. The file
   should end up smaller than ~800 lines.

5. **Unit tests**: at least one per-effort prompt-builder test (it is a pure
   function) plus a `parse_review_effort` round-trip test if none exists.
   Copy existing tests over, never delete them.

## Constraints

- **Mechanical split — zero behavior change.** `handle_review` behavior must be
  byte-for-byte identical from the user's perspective. Any test that exists for
  review/blame must still pass unchanged.
- At most 3 files are touched in this task: `commands_git_review.rs`,
  `commands_review.rs` (new), `main.rs`.
- `cargo build && cargo test` green; `cargo clippy --all-targets -- -D warnings` clean.

## Note

If a module list lives in `src/main.rs` AND a `lib.rs` re-export layer exists,
touch only what is needed to register the new module. Check before editing.
# Assessment — Day 173

## Build Status
**PASS.** `cargo build` clean (Finished dev profile, 0.14s). `cargo test` passes: 88 passed, 0 failed, 1 ignored. `./target/debug/arc --version` → `arc v0.1.16 (90157d3 2026-08-20) linux-x86_64`. `--help` renders. No clippy/fmt issues flagged in the last sessions (all-green build/test in CI).

## Recent Changes (last 3 sessions)
- **Day 173 08:44** — "Self-improvement (small, committed)": added 44 lines of test to `src/commands_risk_accuracy.rs` (a new test case in the accuracy-stats module). Small, green.
- **Day 173 01:27** — Two tasks: (1) Gated the risk-accuracy lift on the dream's ≥5-pair cold-start threshold + surfaced milestone progress (`Pairs: N/5` in the report); (2) refreshed `CLAUDE_CODE_GAP.md` against the 2026 competitor landscape.
- **Day 172 16:36** — Added a **discriminative breakage-rate signal** to risk accuracy ("do flagged files fail more?" — `scored_broke`/`total_scored` fields now in validation events); de-noised validation by filtering non-source churn out of accuracy.
- **Day 172 08:42/01:28** — Closed the risk-validation cold-start: `risk snapshot` now also validates the prior snapshot (order: validate prior, then record new); opt-in `arc_RISK_AUTOSNAPSHOT=1` auto-replay on REPL exit.

The theme across the last ~4 sessions is **one coherent goal**: un-freezing the dream's cold-start so validation pairs actually accumulate. That is now succeeding (see below).

## Source Architecture
~106k lines Rust across src/. Key modules (largest first):
- `commands_risk.rs` (4392) — risk scorer, learn_weights_from_history, replay_validate_core
- `cli.rs` (3475) — CLI parsing; `commands_search.rs` (3158); `commands_project.rs` (3146); `commands_git.rs` (3131)
- `commands_spawn.rs` (3264) — /spawn subagent orchestration, worktree isolation, handoff commits, `--parallel` manifest
- `watch.rs` (3336), `tool_wrappers.rs` (2940), `repl.rs` (2793 — REPL loop, bang passthrough)
- `agent_builder.rs` (2446), `help.rs` (2469), `symbols.rs` (3679)
- `commands_risk_accuracy.rs` (800) — accuracy stats, lift gating, cold-start `Pairs: N/5` label
- `commands_risk_snapshots.rs` (1068) — snapshot/validation persistence, dedup by git hash

Entry points: `main.rs` → `cli.rs` (parse) → `repl.rs`/`prompt.rs`. Risk subsystem is split across `commands_risk.rs` + `commands_risk_{accuracy,emerging,report,snapshots}.rs` — well-factored and re-exported through `commands_risk`.

## Self-Test Results
- Build/test green; binary runs; `--help` renders.
- **`arc risk validate` works non-interactively** — ran it, printed a Prediction-vs-Result table, "Precision@10: 0/10 predicted files had issues". This confirms the CLI-callable risk-validation path (the human-approved cadence enabler) is functioning end-to-end.
- `risk snapshots` file at 84 lines (accumulating); `risk_validations.jsonl` at 2 entries.

## Evolution History (last 5 runs)
All last 5 evolve.yml runs: **success** (conclusion=success, 2026-08-20 01:26 → 16:38). No failures, no reverts in window (CLAUDE.md claims "0 of last ~10 sessions had reverts"). Earlier trajectory shows recurring CI noise that is infra-only: `[3×] failed to log in to github.com using token (gh_token)` — a GitHub token/auth issue, not a code failure.

⚠️ **Trajectory flags "Self-improvement (small, committed)" as possibly stuck** — 5× across days 169–173. This is the planner-fallback task (chosen when Phase A2 produces no task files). It is not a real stuck loop — each instance lands a small committed test (e.g., 44 lines to risk_accuracy tests). But its *repetition* is a signal the main backlog/self-directed slots are not producing differentiated work, and that the loop is defaulting to micro-test-padding.

## Capability Gaps (vs Claude Code, refreshed Day 173 morning)
From CLAUDE_CODE_GAP.md + fresh research. Claude Code's newest (2026) surface arc does NOT yet have:
- **Agent Teams / dynamic workflows** — Claude Code now coordinates *tens-to-hundreds* of parallel subagents with self-written orchestration scripts, peer-to-peer messaging, shared task lists, and cross-session messaging. arc has `/spawn` (worktree-based, `--parallel` manifest) + RLM substrate + SharedState, but no self-authored dynamic orchestration at this scale.
- **Cloud / background agents persisting across sessions** (`--cloud`/`--teleport`) — explicitly flagged in the gap doc as the largest gap arc "cannot close in small steps"; local session-bound by design.
- **Skill marketplace curation** — `/skill install`/`search` exist; missing signed bundles, curation/ratings, a formal marketplace.
- **Code intelligence via LSP** — Claude Code connects to a language server for symbol navigation + live type errors. arc has grep/ast-grep/symbols but no live LSP type-error surface.

arc's defensible differentiators (per the Day-173 gap refresh): self-hosted/open-source, runs anywhere a terminal does, and the **risk/self-model sense organ no competitor has**.

Relevant external work: `journals/llm-wiki.md` is the arcpedia codebase's own growth journal (unrelated to arc's loop; ongoing storage-migration work) — nothing to act on here.

## Bugs / Friction Found
- **arcpedia is unreachable from CI** (confirmed this session: `/api/wiki/search` returns a Cloudflare 302). The yopedia skill documents this; recall/ingest silently no-op in the evolve loop. The day-172 change to "surface the arcpedia-unreachable reality" already made this legible rather than silently hidden. Not actionable from within evolve; just re-verified.
- **Validation-pair accounting is still noisy at low N.** First validation event (day 171) showed 10% accuracy dominated by non-source churn in `surprises`; the day-172 de-noise fix (`scored_broke`/`total_scored`, source-churn filtering) helped — the day-172 event reports 66.7% with only 1 surprise. But with only 2 pairs, no firm claim is warranted (the code correctly gates the lift display behind `pairs >= 5`). Cold start is *functional*, just young.

## Open Issues Summary
- No open `agent-self` issues. No open community issues currently (issue list empty).
- The planner-fallback task ("Self-improvement (small, committed)") is the de-facto default because the issue/self-driven backlog is empty — see trajectory note above.

## Research Findings
- **Martin Fowler, "Maintainability sensors for coding agents" + "Harness engineering for coding agent users" (2026)** — the strongest finding. Fowler's "Agent = Model + Harness" framing and his *sensors* taxonomy (static analysis, dependency rules, coupling data, modularity review, mutation testing as a **regression sensor**) directly validate arc's risk-scorer-as-sense-organ dream. He argues sensors "increase the probability of good agent outputs and enable self-correction before issues reach human eyes" — which is exactly arc's homeostatic reflex (risk notes on edits, fix prompt risk context, auto-context risk annotation). Coupling data + mutation-testing-as-sensor are two concrete sensor types arc could add to its risk scorer to strengthen the umwelt beyond its current signals (churn/size/test-density/coupling). arc *does* already run `scripts/run_mutants.sh` (mutation testing exists) — wiring mutation results as a feedback signal would be a natural, dream-aligned next step.
- **Claude Code 2026**: Agent Teams, dynamic workflows (self-written orchestration of many parallel subagents), agent view (background multi-session dispatch/monitor), cross-session messaging, plugins/marketplaces — all GA. These are the current frontier arc lags on.
- **arcpedia recall/ingest**: skipped silently (unreachable from CI, per skill). No prior research notes retrievable in-loop today.

### Recommended planning focus
1. The dream's cold-start milestone is the live, coherent theme: validation pairs now accumulate (2 today via `risk snapshot`/replay). The next natural step is letting the pairs cross the ≥5 threshold and then *measuring* whether flagged files fail more — plus, per Fowler, strengthening the sense organ (e.g., mutation-test outcomes as a feedback signal, or coupling data) so the sensor perceives more of the territory.
2. Off-shape, non-risk work to avoid the fallback-task rut: LSP type-error surface or a slice of the agent-teams/dynamic-workflow direction (spawn's `--parallel` manifest is a step).
3. The `gh_token` CI login noise is infra, not code — optional to ignore.

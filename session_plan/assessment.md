# Assessment — Day 177

## Build Status

✅ PASS. `cargo build` clean (0.14s incremental, no warnings). `cargo test` — **88 passed, 0 failed, 1 ignored** (all test targets: unit + integration + safety_edge_cases). Binary runs: `arc v0.1.17 (74fbe61 2026-08-24)`, `--help` works.

## Recent Changes (last 3 sessions)

- **Day 177 (01:38, 3 planned / 1 reverted)**: Task 3 shipped — `docs/src/features/risk.md` (new risk-sense-organ docs). One task reverted (per trajectory: 2/3 ⚠️). Also touched: `src/rtk.rs` (Day 176 16:48 — treat `$(...)` / backticks as non-simple commands in safety analysis).
- **Day 176 (01:40, 3 planned / 1 reverted)**: Task 2 `/spawn` help/docs catch up with `--parallel` manifests + `--pr` flag; Task 3 dotenv end-to-end — split `load_cwd_dotenv` into a directory-scoped pure function.
- **Day 175 (08:35)**: Release **v0.1.17** — ship the risk-sensor + dotenv era. Also: `src/update.rs` SemVer compare now strips pre-release/build suffixes; `/risk snapshot` run-counted cadence stamp (prediction-meter start).
- Common thread: **the risk sense organ is the dominant workstream** — docs, accuracy-gating, mutation-survival sensor, snapshot-also-validates, cadence stamps, and now the ≥5-pair cold-start threshold. Second thread: dotenv loading, `/spawn` docs, RTK hardening.

## Source Architecture

~120k lines, 70 `.rs` files + `src/format/` (7 files). Entry: `main.rs` → `cli.rs` (3709) → `repl.rs` (2881) → `prompt.rs` (2312). Key modules by size:
- `commands_risk.rs` (4670) — risk scorer + `/risk` handler — the largest file, and the center of the dream workstream
- `cli.rs` (3709), `symbols.rs` (3679), `watch.rs` (3336), `commands_spawn.rs` (3264), `commands_search.rs` (3158), `commands_project.rs` (3146), `commands_git.rs` (3131), `commands_info.rs` (3029)
- `tools.rs` (3002), `tool_wrappers.rs` (2940), `repl.rs` (2881), `format/markdown.rs` (2865), `format/output.rs` (2757), `commands_file.rs` (2568)
- Risk support files: `commands_risk_report.rs` (601), `commands_risk_accuracy.rs` (907), `commands_risk_emerging.rs` (421), `commands_risk_snapshots.rs` (1256)
- Core: `agent_builder.rs` (2446), `config.rs` (2213), `dispatch.rs` (2190), `safety.rs` (2176), `git.rs` (1710), `prompt_retry.rs` (1687)

`commands_risk.rs` at 4670 lines is over module-size comfort; risk already has 4 satellite files, but the main handler keeps growing.

## Self-Test Results

- `cargo build` + `cargo test`: all green, 88 tests.
- `arc --version` / `--help`: clean, informative.
- Full agent prompt not run — no API key in this runner (expected; saves a wasted session).
- **Risk meter state** (the dream's cold-start milestone):
  - `risk_snapshots.jsonl`: 96 snapshots (3+ per day, days 162–176)
  - `risk_validations.jsonl`: **only 4 validation events**, all `trigger: replay` — day 171 (acc 10.0%), 172 (66.7%), 174 (66.7%), 175 (0.0%)
  - `risk_meter.json`: `pairs: 3`, `target_pairs: 5`, `validations: 4`
- **The dream milestone is 1–2 sessions from landing.** Pairs are counted from matched snapshot→commit spans; the 4th validation exists; 2 more paired predictions → ≥5.

## Evolution History (last 5 runs)

`gh run list --workflow evolve.yml --limit 8`:
- Day 177 08:52 — in progress (this session)
- Success: 01:37 (Day 177), 16:30 + 08:34 (Day 176), 01:39 (Day 176), 16:29 (Day 176), 08:34 (Day 176)
- **No failed evolve runs in the last 8.** Trajectory shows tick marks: 1 revert in Day 177 01:38, 1 in Day 176 01:40, 1 in Day 175 09:56 — but the "Reverts in window" section says 0, and the last 8 runs are all `success`. The CI-error fingerprints (gh token login ×3, sponsor fetch) are **cron noise, not evolve failures**.
- Recent sessions are mostly 1–3 tasks each; reverts are rare and non-fatal.

## Capability Gaps

Refreshing CLAUDE_CODE_GAP.md thinking (last full refresh Day 173) against Aug/2026 reality:
1. **Cloud/background durable agents** — still the biggest gap and still out of small-step scope. Claude Code now ships cross-session messaging (`ListAgents`/`SendMessage`, v2.1.224, W32), cloud sessions with mobile monitoring, AND self-hosted environments (public beta, Aug 6) — sessions run on the org's own infra next to internal services. arc is a local, session-bound CLI by design.
2. **Event-driven triggers** — Cursor Bugbot is now a full product: tracks 70%+ flag-resolution, learned rules (self-improves from PR reactions/replies), MCP support, Autofix, `--review` pre-push, only-review-what's-new, usage-based billing (~$1–1.50/run). Event-driven repo automation is still a positional gap for arc.
3. **Aider has gone dormant** (no substantial release since Aug 2025) — the transparent-diff philosophy gap is effectively closed by inertia.
4. **arc's defensible differentiators remain**: fully self-hosted/open-source, runs anywhere a terminal does, and the risk/self-model sense organ — the only agent with a proprioceptive self-model. The umwelt frame (D135) says: the risk scorer is a sense organ, and its value is *calibration*, not just features.

## Bugs / Friction Found

- **arcpedia unreachable from the evolve runner** (reconfirmed this session): all endpoints (keyword search, index, ingest) return 302 → Cloudflare Access sign-in wall, even with `-L`. Known (Day 172 task surfaced it) and correctly handled as a silent no-op per the skill — but recall/ingest in evolve is dead weight until infra changes. Not actionable in-loop.
- **Validation set is noisy**: 3 of 4 validations are negative (0–10% accuracy). Day 175's 0% came with `hits: []`, `scored_broke: 1` — the scorer flagged nothing in a span that broke. The sense organ is **still uncalibrated**, which is exactly what the dream anticipated; the milestone is measuring, not fixing.
- No compiler warnings, no test failures, no obvious user-facing bug spotted in sampling. `commands_risk.rs` size is a latent maintainability smell.

## Open Issues Summary

**Zero open issues** on MKonovalov/arc-evolve; no `agent-self` backlog exists. No external-project journals beyond `journals/llm-wiki.md` (last entry 2026-04-06 — wiki graph view + URL ingestion; quiet since). No community issues pressing.

## Research Findings

- **Claude Code (Aug 2026)**: cross-session messaging between agents (`ListAgents`/`SendMessage`, W32 v2.1.224) — agents hand work to each other across sessions; self-hosted environments in public beta (sessions run inside your network); auto mode now the default permission mode; skills/subagents/MCP/hooks/dynamic-workflows all mature.
- **Cursor**: Bugbot productized — learned rules from reviewer feedback, MCP support, usage-based pricing, 70% resolution rate, `--review` pre-push sync with GitHub. Cursor 3.3+: parallel execution from plans ("Build in Parallel" — identify independent steps, run async subagents, split into PRs).
- **Codex CLI**: Rust-based, open source, sandboxed execution, included with ChatGPT plans; Codex Pro $100 tier; desktop background computer use on macOS. Chronicle memory layer (screen-recording memory).
- **Aider**: dormant since Aug 2025 (43K stars); BYO-model transparency-first.
- **Implication for the plan**: the field's frontier is *multi-agent orchestration and durable execution* (cross-session messaging, self-hosted cloud, parallel subagents, learned rules). arc matches the single-session CLI feature set and leads on self-modeling; the gaps are deployment-model. Don't chase cloud; keep the differentiator (risk/self-model calibration) and stay product-safe on defaults.
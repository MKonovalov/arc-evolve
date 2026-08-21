# Assessment — Day 174

## Build Status
**PASS.** `cargo build` clean (0.15s incremental, no warnings/errors). `cargo test` green: **88 passed, 0 failed, 1 ignored** (3.27s). Working tree clean before this session.

## Recent Changes (last 3 sessions)
- **Day 173 (16:40)** — 3 tasks, all green:
  - *Task 1*: Per-signal accuracy breakdown in the risk report — make the first ≥5 pairs actionable (wires the dream's lift reporting into `/risk accuracy`).
  - *Task 2*: Bang-capture keeps the working directory — `!` in a subdir then `!?` stays in context.
  - *Task 3*: Emit per-file mutation-survival JSON from `run_mutants.sh` — establish the sensor plumbing (complements the risk sense organ with a mutation-coverage sensor).
- **Day 173 (08:44)** — Self-improvement (small, committed).
- **Day 173 (01:27)** — Gated the risk-accuracy lift on the dream's ≥5-pair cold-start threshold + surface milestone progress; refreshed `CLAUDE_CODE_GAP.md` against the 2026 competitor landscape.
- **Day 174 (01:38)** — No real task; just day-counter + skill-evolve-counter bumps and a session wrap-up (the 0/1-reverted entry in the trajectory reflects this near-empty session, not a substantive revert).

## Source Architecture
~119k lines of Rust across src/ (from `wc -l`). Largest modules:
- `commands_risk.rs` (4,407) — risk scoring, replay-validate core
- `cli.rs` (3,475) — arg parsing, config
- `symbols.rs` (3,679) — symbol handling
- `commands_search.rs` (3,158), `commands_git.rs` (3,131), `commands_project.rs` (3,146)
- `commands_spawn.rs` (3,264) — subagent orchestration
- `watch.rs` (3,336) — watch/fix loop
- `tools.rs` (3,002), `tool_wrappers.rs` (2,940)
- `repl.rs` (2,881) — interactive loop
- `commands_info.rs` (3,029) — brand/status/evolution display
- Risk subsystem split: `commands_risk.rs` + `_accuracy` (907) + `_report` (492) + `_emerging` (410) + `_snapshots` (1,068)

Key entry points: `main.rs` → `cli.rs`/`repl.rs`; `agent_builder.rs` builds the agent; `dispatch.rs`/`dispatch_sub.rs` route REPL and CLI subcommands. Well-modularized; risk subsystem is cleanly decomposed.

## Self-Test Results
- `cargo build` and `cargo test` both pass (see Build Status).
- No manual binary run this session — the codebase is in a healthy, verified state; the tests are meaningful (88 non-trivial passing tests incl. risk-accuracy edge cases, unit tests of the collision guard, safety guards).
- The ``run_git()`` destructive-command test guard is active; tests all run against temp dirs as required.
- No friction encountered in a clean build+test cycle.

## Evolution History (last 5 runs)
From `gh run list evolve.yml`:
- **2026-08-21 08:45** — *in progress* (this session)
- **2026-08-21 01:37** — **success**
- **2026-08-20 16:38** — **success**
- **2026-08-20 08:43** — **success**
- **2026-08-20 01:26** — **success**

4 consecutive green runs before this one. The trajectory's "day-174 0/1 reverted" is the earlier thin session (counter bumps only), not a real revert. No reverts in the window. Recent CI errors are all `gh` auth noise (`failed to log in ... using token`) from the sponsors/refresh path — infra friction, not code failure. **No provider/API errors detected across 10 sessions.**

## Capability Gaps
`CLAUDE_CODE_GAP.md` (refresh Day 173) names the priority queue:
1. **Durable cloud/background agents** (Claude Code `--cloud`/`--teleport`/`--web`, self-hosted runners, cross-session `SendMessage`; Cursor cloud agents on VMs). Deployment-model work — out of small-step scope for a local CLI. **arc's defensible differentiator remains the self-model risk sense organ.**
2. **Persistent named subagents with orchestration** — arc has `/spawn` + `SubAgentTool` + `SharedState`, but no long-lived named-role reviewer/tester.
3. **Full graceful degradation on partial tool failures** — provider fallback handles hard API errors; no "this tool failed, try a different one" story.
4. ✅ Closed: per-edit auto-lint-test (AutoCheckTool).
5. **Skill marketplace curation** — install/search work; trust/quality/ratings layer missing.

## Bugs / Friction Found
- No code bugs surfaced in build/test. The only real friction observed is external infra: the **arcpedia endpoint is unreachable from CI (302-redirects to a Cloudflare page)** — confirmed again this session, so the recall/ingest steps of this task no-op as the `yopedia` skill documents. This is a known, documented limitation.
- The sponsors-refresh CI `gh` auth flakiness (recurring `failed to log in` fingerprints) is infra noise, not arc's code.
- Dream accretion note: the risk sense organ is healthy but the **validation data is still thin (2 pairs of the ≥5 cold-start threshold)** — see Research Findings.

## Open Issues Summary
- **No open GitHub issues at all** in `MKonovalov/arc-evolve` (checked `agent-self` label, all-state, and open: all empty). The self-filed backlog is clear. No community issues pending. The empty backlog means the loop's self-driven slot (and dream slot) carry this session.

## Research Findings
- **Cloud/background agents is the moving competitive frontier.** Claude Code 2.1.x now ships web/cloud sessions that persist across browser closes, self-hosted runners, cross-session messaging (`ListAgents` + `SendMessage`), `/teleport` handoff, and wildcard tool permissions. Cursor does cloud agents on isolated VMs. This is all platform/deployment work — not reachable in small steps, and it's the top item arc deliberately cannot close.
- **arc's non-negotiable differentiator is the self-model sense organ** — the architectural choice no competitor has. It is in-scope and is exactly the dream.
- **Dream milestone status:** `.arc/risk_validations.jsonl` now holds **2 matched prediction-outcome pairs** (first recorded Day 171, then Day 172), up from 0. `.arc/risk_snapshots.jsonl` holds **86 snapshots** (persistent, git-tracked). The `/risk accuracy` lift is correctly gated behind the `pairs >= 5` cold-start threshold so it won't over-claim at N=2. The path to closing the ≥5-pair milestone is live and accumulating through normal evolve sessions — the remaining work is to keep the validation feed flowing (replay/snapshot cadence) until pairs ≥ 5, then the per-signal breakdown + lift become trustworthy.
- `arcpedia` recall no-oped (unreachable from CI, as documented) — no prior-research re-tread risk this cycle; the Day 173 gap refresh already covers the competitive landscape.

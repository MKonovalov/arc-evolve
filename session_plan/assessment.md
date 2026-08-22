# Assessment — Day 175

## Build Status
**PASS.** `cargo build` clean (incremental, no warnings). `cargo fmt -- --check` clean. `cargo test` green: **4358 passed + 88 passed, 0 failed, 1 ignored** (~42s). Working tree had no uncommitted source changes at session start.

## Recent Changes (last 3 sessions)
- **Day 175 (01:26)** — 2 tasks, all green (trajectory says 3/3; journal lists 2 commits + wrap-up):
  - *Task 2*: Dotenv-style API key loading — `arc` finds keys in `.env` files (hand-rolled parser, no dependency; precedence process-env > `.env`; landed via eval-fix).
  - *Task 3*: `/risk snapshot` should record when the prediction meter STARTS — a run-counted cadence stamp (eval-fix 4; ensures the cadence/accumulation is measured from the right origin).
- **Day 174 (08:46)** — 2 tasks, green: folded the mutation-survival sensor into risk file scoring (Task 1, eval-fix 2); made the risk report legible with per-file signal tags incl. the mutation sensor (Task 2).
- **Day 174 (16:39)** — no commits (thin session; counter bumps only).
- **Day 173 (16:40)** — 3 tasks green: per-signal accuracy breakdown; bang-capture keeps working directory (`!` in subdir → `!?` stays in context); per-file mutation-survival JSON from `run_mutants.sh`.
- **External journal** `journals/llm-wiki.md` (project: the arcpedia wiki itself, separate TypeScript project): recent entries are May 2026 — StorageProvider migration, MCP write tools, agent self-registration. No entries since early May; falls outside the risk-focused recent arc.

## Source Architecture
~107k lines of Rust across 71 `src/` files. Largest modules:
- `commands_risk.rs` (4,670) — risk scoring, replay-validate core, subcommand dispatch
- `symbols.rs` (3,679), `cli.rs` (3,637), `watch.rs` (3,336), `commands_spawn.rs` (3,264), `commands_search.rs` (3,158), `commands_project.rs` (3,146), `commands_git.rs` (3,131), `tools.rs` (3,002), `tool_wrappers.rs` (2,940)
- Risk subsystem cleanly decomposed: `commands_risk.rs` + `_accuracy` (907) + `_report` (601) + `_emerging` (421) + `_snapshots` (1,256)

Entry points: `main.rs` → `cli.rs` / `repl.rs`; `agent_builder.rs` builds the agent (MCP collision detection, provider fallback); `dispatch.rs` / `dispatch_sub.rs` route REPL + CLI subcommands (incl. `arc risk …`). Newest surface: `cli.rs` dotenv parser (`parse_dotenv`, `load_cwd_dotenv`, `resolve_api_key_env`) landed Day 175 with unit tests.

## Self-Test Results
- **Build + fmt + full test suite green** (see Build Status).
- `./target/debug/arc --version` → `arc v0.1.16 (56d1aca 2026-08-22)`; `--help` renders full options list cleanly.
- `arc risk meter` works: top-15 file risk scores with signal tags; emerging-risks section shows `src/cli.rs` and `src/commands_risk_*.rs` accelerating (4.3×) — cli.rs is a hot file again (recent `.env` work).
- `arc risk accuracy` works and is **learning**: 3 validations, "Learning... (3/5 events)", lift ≈5.1× (provisional), per-signal weight table, per-event hit/surprise detail.
- `arc risk history` works (day/commits/hits/predicted/precision table); `arc risk predict` returns a confidence readout.
- `arc status` and `arc map` work. `arc context`/`arc model show` attempt an API call and fail with **HTTP 401 Auth error** — the harness's `ANTHROPIC_API_KEY` is set but invalid (expected in this sandbox; not a code bug — the diagnostic message itself is good).

## Evolution History (last 5 runs)
From `gh run list evolve.yml`:
- **2026-08-22 08:34** — *in progress* (this session)
- **2026-08-22 01:25** — **success**
- **2026-08-21 16:38** — **success**
- **2026-08-21 08:45** — **success**
- **2026-08-21 01:37** — **success**

4 consecutive green runs before this session (the trajectory's "day-174 tasks 0/1 ⚠️ 1 task reverted" is the thin counter-bump session — a near-empty task, not a substantive revert). Recurring CI error fingerprints in the window are all `gh` auth noise: `failed to log in to github.com using token (gh_token)` [3×] + the sponsors-refresh `fetchfailed` error-body echo — **infra friction, not arc's code**. Zero provider/API errors across the last 10 sessions.

## Capability Gaps
`CLAUDE_CODE_GAP.md` (last refreshed Day 173) priority queue:
1. **Durable cloud/background agents** — Claude Code v2.1.224 (Aug 2026) now ships cross-session messaging (`ListAgents` + `SendMessage`), self-hosted environments, auto-mode default permission, scheduled routines; Cursor has cloud agents on VMs. Deployment-model work — unreachable in small local-CLI steps. arc's defensible differentiator remains the **self-model risk sense organ** (no competitor has a risk-aware self-model).
2. **Persistent named subagents / orchestration** — `/spawn` + `SubAgentTool` + `SharedState` exist; no long-lived named-role reviewer.
3. **Full graceful degradation on partial tool failures** — hard API errors have provider fallback; no per-tool retry-with-alternative story.
4. **Skill marketplace curation** — install/search/trust layer missing (origin: marketplace concept exists).
5. **Aider has gone dormant** (no substantial release since Aug 2025) — the open-source terminal-agent competition is now **opencode** (MIT, 199.6k stars, model-agnostic) and Codex CLI (Apache-2.0).

## Bugs / Friction Found
- **`arc risk snapshot --help` renders the generic top-level `--help`** instead of subcommand help (`RISK_SUBCOMMANDS` doesn't include `--help`; the CLI subcommand parser doesn't route it). Low-severity UX gap, consistent with `arc risk` (no sub) also printing the generic help. Worth a small fix: route `help`/`--help` to the risk subcommand text.
- **`arc context` / `arc model show` require an API key even for local info** — they try to build the agent before printing and die with 401 in a no-key environment. `arc status`, `arc map`, `arc risk *` all work keyless; these two don't. Marginal product friction.
- **No code bugs surfaced in build/test.** The 401 is env-driven (invalid harness key), not a defect.

## Risk Sense Organ — Cold-Start Milestone (the dream's next milestone)
- **Meter: 2 / 5 pairs** (`pairs: 2`, `target_pairs: 5`, `validations: 3`), reflecting the latest snapshot cadence.
- **3 validation events accumulated** (Days 171, 172, 174): accuracy 10% → 67% → 67%; the last two both flagged `src/commands_risk.rs` hit + `src/cli.rs` as a surprise. The sense organ is *learning* — per-signal weights are being trained, and the accuracy/lift reporting is actionable ("make the first ≥5 pairs actionable" shipped Day 173).
- The remaining gap is **accumulation, not infrastructure**: the loop needs more matched prediction→outcome pairs to cross the ≥5 threshold and test "do flagged files fail more?" Infrastructure is done (snapshot also validates; cadence-start stamp landed this morning).

## Open Issues Summary
- **No open GitHub issues at all** in `MKonovalov/arc-evolve` across all labels (checked `agent-self`, open, and all-state: all empty arrays). Self-filed backlog is clear; no community issues pending. The loop's self-driven + dream slots carry the session. (The recurring CI fingerprints are the sponsors-refresh `gh` auth flake — an infra concern, not a code issue, and not filed because it's harness-side.)

## Research Findings
- **Claude Code's frontier is orchestration + infrastructure, not local CLI craft** (Week 30/32 digests, Aug 2026): cross-session `SendMessage`, self-hosted environments (public beta), auto-mode as default permission (Aug 14), `/code-review` background subagent, Opus 5 default w/ 1M context. The web/cloud/teleport story means "your agent outlives your terminal" — arc's `/bg`, session save/resume, and `/spawn` worktree handoff cover the *local* slice, not the durable-orchestration slice.
- **Benchmarks** (Terminal-Bench 2.1, AI Analysis, Aug 21 2026): Codex + GPT-5.6 Sol 89.5% #1; Claude Code + Opus 5 89.1%; opencode is the most-starred open-source agent (199.6k stars) and the model-agnostic default.
- **arcpedia recall + ingest both no-op this session** (302 → Cloudflare block, as documented in the yopedia skill and the Day 174 assessment) — recall returned nothing and the ingest attempt was visibly blocked, so no arcpedia save was made. Competitive-landscape notes above are the fresh input for this cycle; arc's own `CLAUDE_CODE_GAP.md` was refreshed Day 173 and remains the durable record.
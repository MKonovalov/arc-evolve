# Assessment — Day 177

## Build Status
**PASS — all four CI gates green.**
- `cargo build` — ok (clean, cached)
- `cargo test` — 4369 unit + 88 integration passed, 0 failed, 1 ignored
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo fmt -- --check` — clean
- Working tree clean; `DAY_COUNT` = 177.

## Recent Changes (last 3 sessions)
- **Day 177 09:11 (2/2 tasks ✅):** (1) Risk meter calibration verdict — `calibration_verdict()` interprets the first ≥5 paired predictions (docs + 146 lines in `commands_risk_snapshots.rs`; a Day 177 02:54 reverted task was re-attempted here); (2) Rearchitected `handle_review` — carved `review-*` helpers out of `commands_git_review.rs` (666→24 lines) into a new `commands_review.rs` (687 lines), pure per-effort handlers.
- **Day 177 01:38 (1/1 ✅):** Documented the risk sense organ — new `docs/src/features/risk.md`.
- **Day 176:** rtk command-substitution fix (`$(...)`/backticks now non-simple); dotenv loading made testable end-to-end (`load_cwd_dotenv` split into a directory-scoped pure fn); `/spawn` help/docs caught up with `--parallel` manifests + `--pr` flag.

Ongoing thread for days: **the risk sense organ (umwelt)** — snapshot-also-validates, mutation-survival sensor, per-signal accuracy, discriminative breakage-rate signal, calibration verdict.

## Source Architecture
~107.8k lines across `src/` (Rust, single crate, arcagent 0.9 substrate). Key modules:
- `commands_risk.rs` (4670) + `commands_risk_snapshots/report/emerging/accuracy.rs` (1519/601/421/~) — the risk sense organ
- `cli.rs` (3709), `cli_config.rs` (358), `dispatch.rs` (2190), `dispatch_sub.rs` (1395), `commands.rs` (1489)
- `tools.rs` (3002) + `tool_wrappers.rs` (2940) — tool surface; `smart_edit.rs` (1772)
- `repl.rs` (2881), `prompt*.rs` (~6900 across 6 files), `watch.rs` (3336)
- `commands_*` per-feature modules (spawn 3264, search 3158, project 3146, git 3131, info 3029, file 2568, config 1625, session 1632, review 687, git_review 733…)
- `symbols.rs` (3679), `safety.rs` (2176), `agent_builder.rs` (2446), `help*.rs` (~4075), `context.rs` (1065)

## Self-Test Results
- `arc --help` — instant, correct (v0.1.17).
- `arc risk meter` — prints the risk table with per-file T/100 + signal tags; exit 0 (an earlier exit-101 was a SIGPIPE artifact of piping to `head`, not a bug). `commands_risk.rs` (0.67) and `commands_risk_snapshots.rs` (0.61) top the risk table — the sense organ correctly flags its own most-churned files.
- No interactive REPL run (no API key for a live agent call was available for a full prompt cycle in this analysis; non-interactive paths all work).

## Evolution History (last 5 runs)
Evolve workflow — last 5 **all success** (08:52, 01:37 day 177; 16:30, 08:34, 01:39 day 176). The 16:41 run is this session (in progress). No failed runs, no reverts in the window (0 of last 10 sessions). Other loops healthy too: Dream, Skill Evolution, Sponsors Refresh, Social all success. Trajectory data shows 3 session reverts in the window (all re-attempted and shipped same-session). CI error fingerprints are only the known `gh_token` login warnings — no code-level failures.

## Capability Gaps
- **Biggest remaining gap (from `CLAUDE_CODE_GAP.md` priority queue, re-evaluated Day 173):** durable cloud/background agents that persist across sessions (Claude Code `--cloud`/`--teleport`, Cursor isolated-VM agents). Arc is local/session-bound by design; closing it is platform-scale work outside the loop's small-step scope.
- **Other live gaps:** no named-role persistent orchestration (subagent forking); the doc's 🟡 items (subagents, some context/UX polish).
- **Fresh competitor landscape (web search, 2026):** Claude Code now defaults to *auto mode* (per-tool-call classifier, no per-action prompts — arc's permission/confirm system is the safety analogue but interactive); `/design` skill + artboard workflow in the CLI; cross-session messaging (`@` mention + SendMessage, unique session names); subagent forking on by default (inherits full conversation + prompt cache). The ecosystem shifted to terminal-native ("CLI again") — validates arc's surface.
- **arc's differentiator stands:** self-hosting, fully open source, terminal everywhere, and the risk/self-model sense organ no competitor has.

## Bugs / Friction Found
- **The sense organ's cold-start data looks negative (genuine discovery, worth watching):** 2 of 4 recorded validations show low accuracy (10% on day 171, 0% on day 175 — flagged files did NOT break more). Not conclusive yet (n=4, noisy coincidence can dominate), but this is exactly the data the dream's next milestone exists to collect — the call for reaches 5 pairs with a real verdict pending.
- Churn noise persists in the validation feed (`.skill_evolve_counter`, `DAY_COUNT`, `CLAUDE.md` bumps appear as "surprises" — hits/surprises don't filter docs/state noise); day-172's filtering helped but surprises still mix config files into the pairing.
- Self-referential churn trap: the risk sensor flags its own implementation files (`commands_risk.rs` 0.67, top of table) — expected for a self-model, but means the loop's own tooling runs warm.
- `commands_git_review.rs` still exists as a 24-line stub after the carve — could be a re-export; fine to leave, noting it.

## Open Issues Summary
Repo has **0 open issues** (verified via API incl. `agent-self` label — no self-filed backlog items). Community issues: none pending. The external project journal (`journals/llm-wiki.md`) shows arcpedia-side work (StorageProvider migration for the llm-wiki backend, MCP read/write tools, agent self-registration) — healthy external project, not a task source. arcpedia recall/ingest skipped: endpoint 302-redirects to Cloudflare Access login from CI (documented limitation in `skills/yopedia/SKILL.md`), so this assessment used the in-repo competitor record instead.

## Research Findings
- Claude Code: auto-mode default (Aug 14, 2026) — classifier routes every tool call, blocking irreversible/destructive/out-of-environment actions without prompting; cost-premium accounting landed; v2.1.241 current.
- Claude Code cross-session: `@`-mention a session, SendMessage delivery, `subagent_type:"fork"` inherits full conversation + prompt cache — arc's `/spawn --parallel` manifest (issue #341, `.arc/spawn_runs/<id>.json`) is the first step toward matching orchestration.
- Market verdict: "no best one; run two" — harness (context gathering, guardrails, edit application) is what separates experiences, not the model. Arc being a self-evolving open-source CLI with a self-model remains a genuinely distinct position.
- Nothing here rose to arcpedia-ingest bar from CI (unreachable); the durable keeper is the validation-accuracy observation, which belongs to the learnings/reflection step, not a vault.
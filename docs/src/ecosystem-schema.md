# arc-evolve Ecosystem Schema

> This document maps how **arc-evolve** and its sibling repositories interact:
> the agent runtime (`arcagent`), the durable-state layer (`arcagent-state`),
> the GASP event protocol (`gasp`), the portable state repo (`arc-gasp`),
> the GASP sidecar (`tools/gasp-emit`), the bridge (`scripts/gasp_shim.sh`),
> the reusable GitHub Action (`arc-action`), and the sibling harness
> (`arc-harness`).
>
> Last verified: Day 159 (2026-08-06). All facts below were confirmed against
> the actual code, not assumed.

---

## 1. The repositories at a glance

| Repo | Role | Relation to arc-evolve |
|------|------|------------------------|
| **arc-evolve** | The self-evolving coding agent (Rust/arcagent). Source + build + evolve loop. | **This repo.** |
| **arcagent** | Agent runtime crate: provider/model config, tool runtime, streaming, sub-agents, `arcagent_state` integration. | Cargo dependency (`git`, pinned `b11de796`). |
| **arcagent-state** | Durable state & lineage for long-running agents (`GitEventStore`, `arcagentState`). | Cargo dependency of `gasp-emit` (pinned rev). |
| **gasp** | The **GASP protocol/spec** (event vocabulary: `session-start`, `task`, `task-result`, `session-end`). Not a running service. | Referenced by `gasp-emit` (implements the spec). |
| **arc-gasp** | Portable **GASP agent state repo** (`github.com/MKonovalov/arc-gasp`). The destination where GASP events are written. | Written to by `gasp-emit` via `arcagent-state`'s `GitEventStore`. |
| **gasp-emit** | A **sub-tool inside arc-evolve** (`tools/gasp-emit/`). Maps evolve session transitions onto GASP events. | Lives in arc-evolve; depends on `arcagent-state`. |
| **gasp_shim.sh** | Bridge script (in arc-evolve) that builds + invokes `gasp-emit` at evolve transitions. | Part of arc-evolve; called by `evolve.sh`. |
| **arc-action** | Reusable **GitHub Action** (`mkonovalov/arc-action@v1`) to run arc agents on any repo (sponsor-gated). | Separate deploy path; not invoked by the evolve loop. |
| **arc-harness** | Sibling harness/cli that drives arc agents with cross-run state lineage. | Separate; part of the family, outside the evolve loop. |

---

## 2. Component diagram

```
                         ┌──────────────────────────────────────────────┐
                         │            GitHub Actions (CI)                │
                         │  evolve.yml · skill-evolve.yml · dream.yml ·  │
                         │  sponsors.yml · pages.yml                     │
                         └───────────────────┬──────────────────────────┘
                                              │ gh workflow run evolve.yml (cron @:00)
                                              ▼
   ┌──────────────────────────────────────────────────────────────────────────┐
   │  arc-evolve  (THIS repo)                                                    │
   │                                                                            │
   │  scripts/evolve.sh                                                         │
   │     ├─ Step 1: cargo build/test (pre-run; may regenerate gasp-emit lock)   │
   │     ├─ Phase A: assess → plan  (arc agents, --provider/--model)            │
   │     ├─ Phase B: implement  (agent edits src/, runs cargo, commits)         │
   │     └─ Phase C: respond / journal / commit                                │
   │            │                                                               │
   │            └─▶ scripts/gasp_shim.sh  (GASP sidecar bridge)                │
   │                    │  builds + calls                                     │
   │                    ▼                                                       │
   │                 tools/gasp-emit/  (Rust sub-tool)                          │
   │                    │  uses                                                 │
   │                    ▼                                                       │
   │                 arcagent_state  (crate)  ── GitEventStore ──▶ arc-gasp    │
   │                    │  (event vocabulary defined by the GASP SPEC)         │
   │                    ▼                                                       │
   │                 gasp  (the protocol/spec repo)  ← defines event types     │
   │                                                                            │
   │     arc CLI binary (./target/debug/arc)  ──runs──▶  arcagent (runtime)    │
   └───────────────────────────┬──────────────────────────────────────────────┘
                                │ Cargo dependency (git, pinned)
                                ▼
                       ┌──────────────────┐
                       │   arcagent crate  │  provider/model/tool/streaming +
                       │  (b11de796)       │  arcagent_state integration
                       └─────────┬─────────┘
                                 │ re-exports / integrates
                                 ▼
                       ┌──────────────────┐
                       │  arcagent_state   │  durable state + lineage
                       │  (pinned rev)     │  GitEventStore
                       └─────────┬─────────┘
                                 │ GASP event vocabulary
                                 ▼
                       ┌──────────────────┐
                       │  gasp (SPEC)      │  session-start / task /
                       └──────────────────┘  task-result / session-end


   ── SEPARATE DEPLOY PATHS (not part of the evolve loop) ──────────────────────

   arc-action  (GitHub Action: mkonovalov/arc-action@v1)
        │ 1. workflow trigger → 2. check sponsor status → 3. run arc agent
        ▼
   can run arc-evolve / arc-gasp agents on ANY user repo (sponsor-gated)

   arc-harness  (sibling harness/cli)
        │ wraps/invokes arc agents + arcagent_state for cross-run lineage
```

---

## 3. Data flow — one evolve session

1. **CI trigger** → `gh workflow run evolve.yml` (cron `@:00`, plus manual).
2. **`evolve.sh`** reads `PROVIDER`/`MODEL` (secrets, defaulting to
   `opencode-go` / `deepseek-v4-flash` since Day 159) and `OPENCODE_API_KEY`.
3. **Pre-run cargo** builds the `arc` binary and `gasp-emit`, and runs
   `cargo test` (this regenerates `tools/gasp-emit/Cargo.lock` — see §6).
4. **Phase A** (assess + plan): `arc` agents read the repo, write `session_plan/`.
5. **Phase B** (implement): the implement agent edits `src/`, runs
   `cargo fmt/clippy/test`, and commits via `safety_commit` (which refuses to
   sweep protected files).
6. **GASP bridge** (`gasp_shim.sh`) is invoked at transitions:
   `session-start`, `task`, `task-result`, `session-end`.
7. **`gasp-emit`** translates each transition into a GASP event and appends it
   to the **arc-gasp** state repo through **`arcagent_state`'s `GitEventStore`**
   (auth via a GitHub App token minted in the shim).
8. **Phase C**: journal entry + issue responses; `evolve.sh` pushes the
   audit-log and the arc-gasp state.

---

## 4. The `gasp-emit` → `arc-gasp` link (detail)

`tools/gasp-emit/src/main.rs` (verified):

```rust
use arcagent_state::{ ..., Task, TaskId, TaskStatus, arcagentState, GitEventStore };
// commands: session-start | task | task-result | session-end
//   gasp-emit session-start --state-dir D --run-id R --day N --task DESC
//   gasp-emit task          --state-dir D --run-id R --num N --title T
//   gasp-emit task-result   --state-dir D --run-id R --num N --title T --status ...
//   gasp-emit session-end   --state-dir D --run-id R --outcome TEXT
```

- **`gasp-emit`** depends on **`arcagent-state`** (pinned to a specific git rev
  in `tools/gasp-emit/Cargo.toml`).
- **`arcagent-state`** provides `GitEventStore`, which writes events as git
  commits into the **arc-gasp** state repo.
- The **event vocabulary** (`session-start`, `task`, `task-result`,
  `session-end`) is defined by the **GASP SPEC** (`github.com/MKonovalov/gasp`)
  — `gasp` is the spec, not a running service.
- **Auth**: `gasp_shim.sh` mints a GitHub App JWT → installation token → pushes
  to `GASP_STATE_REPO` (default `github.com/MKonovalov/arc-gasp`).

> `gasp-emit` is deliberately **standalone** (its own `[workspace]`, not part of
> the `arc` crate) so the self-evolving agent's own build/tests never depend on
> it. It is built via `cargo build --manifest-path tools/gasp-emit/Cargo.toml
> --target-dir target/gasp-emit`.

---

## 5. Provider / model layer

The agent runtime is **arcagent**. `arc-evolve`'s `src/agent_builder.rs`
(`create_model_config`) maps a `PROVIDER` string to an `arcagent`
`ModelConfig` + `OpenAiCompat`.

| Provider | Base URL | Auth secret | Notes |
|----------|----------|-------------|-------|
| `anthropic` | api.anthropic.com | `ANTHROPIC_API_KEY` | default product path |
| `nousresearch` | inference-api.nousresearch.com/v1 | `NOUS_API_KEY` | `tencent/hy3:free` retired to paid (404) |
| `opencode-zen` | opencode.ai/zen/v1 | `OPENCODE_API_KEY` | `OpenAiCompat::default()` (no `developer` role) |
| `opencode-go` | opencode.ai/zen/go/v1 | `OPENCODE_API_KEY` | **default**; `deepseek-v4-flash` is free + tool-capable |
| `openrouter` | openrouter.ai/api/v1 | `OPENROUTER_API_KEY` | `tencent/hy3:free` retired to paid (404) |
| `custom` | user-supplied | — | local OpenAI-compat |

**Day-159 provider findings (verified empirically):**

- `nousresearch`/`tencent-hy3:free` and `openrouter`/`tencent-hy3:free` both
  **404** — the only free, tool-call-capable model was retired to paid.
- All *other* free models on OpenRouter report `tools: False` (cannot drive
  the agent).
- **`opencode-go` / `deepseek-v4-flash`** is the only **free + tool-capable**
  combination that executes tasks and lands real `src/` commits — but it
  **intermittently stalls** (a never-terminating stream hangs the run until the
  150-min job cap). For reliable, non-flaky commits, add **NousResearch
  credits** (paid `deepseek-v4-flash` on Nous streams reliably + supports
  tools).

---

## 6. The `tools/gasp-emit/Cargo.lock` gotcha (Day 159)

This was the single biggest "loop doesn't commit" trap, and is now fixed.

**What happened:**
- `.gitignore` **force-tracked** `tools/gasp-emit/Cargo.lock` via a `!`
  negation.
- Cargo regenerates this lock on **every** build — both the harness's pre-run
  `cargo build/test` *and* the agent's own `cargo fmt/clippy/test` during a task.
- That made it a **tracked-but-dirty** file. The agent's `git add -A &&
  git commit` swept it into the commit, and the protected-file guard
  (`tools/gasp-emit/` is protected) **reverted the entire commit** — so real
  `src/` changes never reached `main` (`tasks_succeeded` stayed 0).

**The fix (PR #15):**
1. Remove the `!tools/gasp-emit/Cargo.lock` negation → the lock is ignored
   (like the root `Cargo.lock`).
2. `git rm --cached` it (stays on disk; cargo regenerates it harmlessly).
3. **Pin `arcagent-state`** in `tools/gasp-emit/Cargo.toml` to a specific git
   rev — because the lock is now untracked, the pin must live in the manifest
   to keep the build deterministic (otherwise `branch = "main"` would drift to
   whatever `main` is at build time).

After this, `git add -A` no longer sweeps the lock, and the agent's real
`src/` commits survive.

---

## 7. Full fix chain (Day 159)

| # | Symptom | Root cause | Fix | PR |
|---|---------|-----------|-----|----|
| 1 | SSE "Stream ended" on Nous | arcagent premature SSE close on non-`[DONE]` streams | Bump arcagent → `b11de796` (upstream SSE fix) | #12 |
| 2 | OpenCode HTTP 400 `developer` role | `OpenAiCompat::openai()` sets `supports_developer_role: true`; OpenCode rejects it | Use `OpenAiCompat::default()` for `opencode-zen`/`opencode-go` | #13 |
| 3 | Real `src/` commits reverted | Harness's own cargo dirtied protected `tools/gasp-emit/Cargo.lock`; guard aborted whole commit | Restore protected files before `safety_commit` staging | #14 |
| 4 | Same revert (agent's own cargo re-dirties lock) | Lock was force-tracked; `git add -A` swept it | Untrack lock + pin `arcagent-state` rev | #15 |
| 5 | Default pointed at dead model | Default `openrouter/tencent/hy3:free` retired to paid (404) | Default → `opencode-go` / `deepseek-v4-flash` | #16 |

**End-to-end proof:** run on `opencode-go`/`deepseek-v4-flash` produced commit
`5aacac1e` (`src/rtk.rs`, +19/−2, with a regression test),
`tasks_succeeded: 1, reverted: false` — the first genuine `src/` commit from
the loop.

---

## 8. Cross-repo dependency summary

```
arc-evolve
  ├─(Cargo git, pinned b11de796)─▶ arcagent
  │                                └─(integrates)─▶ arcagent_state
  ├─ tools/gasp-emit
  │     └─(Cargo git, pinned rev)─▶ arcagent_state
  │                                └─ GitEventStore ──push──▶ arc-gasp (state repo)
  │                                                        ↑ event vocabulary from
  │                                                          gasp (SPEC repo)
  ├─ scripts/gasp_shim.sh ──builds+calls──▶ tools/gasp-emit
  ├─ scripts/evolve.sh ──invokes──▶ arc CLI (arcagent runtime)
  └─ (deploy path) arc-action ──runs──▶ arc agents on any repo (sponsor-gated)

arc-harness ──drives──▶ arc agents + arcagent_state (cross-run lineage)
```

---

## 9. Open questions / follow-ups

- **OpenCode stall**: `opencode-go`/`deepseek-v4-flash` intermittently stalls
  (never-terminating stream). Worth filing as a tracked issue; durable fix is
  NousResearch credits for a reliable paid model.
- **Provider fallback**: `FALLBACK_PROVIDER` exists in `evolve.sh` but is empty
  by default; wiring a free fallback (e.g., OpenRouter paid-with-credits) would
  improve resilience.
- **GASP lineage**: `arc-gasp` accumulates session/task events; mining it for
  trajectory/skill-evolve signals is the intended downstream use.

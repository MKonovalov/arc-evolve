# Centralized AI Model Management — Design (`arc-models`)

> Status: **proposal (not yet built)**. Companion to the working prototype in
> the `arc-models` repo (resolver + health probe). This document is the
> architecture; the prototype is the first slice.

## 1. Problem (observed this session)

Primary/fallback model config is **smeared across every repo** with no single
source of truth. Three distinct failures this session all trace to that:

| Failure | Root cause | Where config lived |
|---------|-----------|--------------------|
| `tencent/hy3:free` retired to paid → default 404'd | Hardcoded default in `evolve.sh` | `arc-evolve/scripts/evolve.sh:26-29` |
| OpenCode `deepseek-v4-flash` rate-limited (Day 159/160) → agent died, no commit, no GASP state | No health awareness; agent ran blind into a throttled provider | `evolve.sh` + `src/providers.rs` |
| `zai` fallback also dead (Day 160) → double failure, silent | Fallback key invalid, never probed | `FALLBACK_PROVIDER=zai` in `evolve.sh` / `arc-action` |

A centralized, **health-aware** model service would have auto-routed around
all three.

## 2. Design principles

- **Git-backed, not a running API.** Use the same durable/portable/auditable
  pattern as `arc-gasp` (GitEventStore, git-fetch consumption). No extra
  hosting, fully versioned, reviewable, and consistent with the ecosystem's
  "state repo" philosophy.
- **Single source of truth.** One repo (`arc-models`) holds primary + ordered
  fallback list + per-provider health.
- **Health-aware resolver.** A tiny resolver (`arc-resolve-model`) fetches the
  config, drops any `retired` / `rate_limited` / `key_invalid` entry, and emits
  `PROVIDER` / `MODEL` / `FALLBACK_PROVIDER` env. This replaces the hardcoded
  defaults in `evolve.sh`.
- **Probe cron writes health back.** A scheduled probe fires a tiny completion
  call to each provider/model and records status. This is the killer feature:
  it would have flagged OpenCode `rate_limited` *before* Day 159/160, and
  caught the invalid `zai` key.
- **`arc-action` is the chokepoint.** Both `arc-evolve` (`evolve.sh`) and
  `arcpedia` (`pm.yml` / `office-hour.yml`) run agents via `arc-action@v1`.
  Make `arc-action` call `arc-resolve-model` and pass primary/fallback to
  `arc`. One change governs every repo.

## 3. Repository layout (`arc-models`)

```
arc-models/
  config.yml            # single source of truth (primary + fallback + health)
  resolver/
    arc-resolve-model   # shell/py script: emits PROVIDER/MODEL/FALLBACK_PROVIDER
  probe/
    probe.sh            # fires tiny completion to each provider, writes health
  .github/workflows/
    probe.yml           # cron: runs probe.sh, commits health back
  README.md
```

### `config.yml` schema

```yaml
# arc-models/config.yml — single source of truth for AI model routing
primary:
  provider: opencode-go
  model: deepseek-v4-flash
  key_ref: OPENCODE_API_KEY

# Ordered. Resolver tries in order, skipping any entry whose health status is
# retired / rate_limited / key_invalid / unknown.
fallback:
  - provider: nousresearch
    model: deepseek-v4-flash      # paid, after NousResearch credits added
    key_ref: NOUS_API_KEY
  - provider: zai
    model: glm-4-plus
    key_ref: ZAI_API_KEY

# Written by the probe cron. Resolver ignores entries not marked status: ok.
health:
  opencode-go/deepseek-v4-flash:   { status: ok }
  nousresearch/deepseek-v4-flash:  { status: ok }
  zai/glm-4-plus:                 { status: ok }
```

Health status enum: `ok` | `rate_limited` | `key_invalid` | `retired` |
`unknown`. Resolver treats anything other than `ok` as unusable (except it
never drops the *last* remaining entry — it prefers a degraded primary over
nothing, and emits a warning).

## 4. `arc-resolve-model` (resolver)

Emits shell env on stdout, consumed via `eval "$(arc-resolve-model)"`:

```bash
PROVIDER=opencode-go
MODEL=deepseek-v4-flash
FALLBACK_PROVIDER=nousresearch
```

Logic:
1. Fetch `arc-models/config.yml` (git clone/fetch to temp, or `gh api` raw).
2. Build candidate list: `[primary] + fallback`.
3. Drop candidates whose `health[provider/model].status != ok`.
4. First surviving candidate → `PROVIDER`/`MODEL`; next → `FALLBACK`/fallback.
5. If none survive, fall back to the raw `primary` (degraded) + warning.

## 5. Health probe (`probe.sh` + `probe.yml`)

Runs on a cron (e.g., every 30 min). For each `primary` + `fallback` entry:
- Calls the provider's `/v1/models` or a 1-token completion with its `key_ref`.
- Maps result → status: `ok` | `rate_limited` (429/retry-after) |
  `key_invalid` (401) | `retired`/retired (404 "requires credits") | `unknown`.
- Writes the `health:` block back to `config.yml` and commits.

This is what would have caught the Day 160 `zai` key failure and the OpenCode
rate-limit *before* the evolve loop ran blind.

## 6. Wiring (minimal touch)

| Repo | Change | Protected? |
|------|--------|-----------|
| `arc-action` | Call `arc-resolve-model` in `action.yml`, pass `PROVIDER`/`MODEL`/`FALLBACK_PROVIDER` to `arc` | core action file |
| `arc-evolve/scripts/evolve.sh` | Replace `MODEL="${MODEL:-...}"` / `PROVIDER="${PROVIDER:-...}"` with `eval "$(arc-resolve-model)"` (still overridable by env) | **YES — protected** |
| `arcpedia/*.yml` | Already via `arc-action@v1` → inherits automatically | no (already fixed) |

`evolve.sh` keeps env-override semantics: `MODEL`/`PROVIDER` set in the
GitHub secret still win; the resolver only supplies defaults when unset.

## 7. What it would have prevented (this session)

- `tencent/hy3:free` retirement → resolver sees `health: retired` → auto-promotes
  next healthy primary. **No PR #16 needed.**
- OpenCode rate-limit → probe marks `rate_limited` → resolver routes to
  `nousresearch` fallback. **Day 159/160 would have committed.**
- `zai` broken key → probe marks `key_invalid` → excluded; alert raised
  instead of silent double-failure.

## 8. Trade-offs / risks

- **Real subsystem, not a tweak.** Touches `evolve.sh` (protected) + `arc-action`
  + new repo. Coordinated multi-repo change.
- **Tactical vs strategic:** immediate unblock for the rate-limit/no-commit
  problem is **adding NousResearch credits** (one action, no code). `arc-models`
  is the strategic fix so the class never recurs. Do both.
- **Probe cost:** tiny completions every 30 min across 3 providers — negligible
  token cost, but needs valid keys for every candidate (including fallback) to
  probe them.
- **Git race:** probe commits health while a resolver fetches — low risk at
  30-min cadence; resolver tolerates `unknown`.

## 9. Rollout

1. Create `arc-models` repo + `config.yml` (mirrors current working state:
   opencode-go primary, nousresearch + zai fallback).
2. Build `arc-resolve-model` (Python, stdlib only — portable on macOS/Linux CI).
3. Build `probe.sh` + `probe.yml` (writes health).
4. Wire `arc-action` to call the resolver.
5. Wire `evolve.sh` to consume it (replacing hardcoded defaults) — **needs the
   usual protected-file authorization**.
6. arcpedia inherits automatically via `arc-action@v1`.

## 10. Out of scope (v1)

- Live UI to flip primary/fallback (Git PRs are the UI for v1).
- Running API service with auth/hosting (Git-backed is sufficient).
- Cross-model cost optimization (future extension of `health`/`cost` fields).

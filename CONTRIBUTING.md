<!-- generated-by: gsd-doc-writer -->
# Contributing to arc-evolve

Before you open a PR, understand how this project actually works: it isn't a typical
open-source repo where humans write most of the code. **Most commits to `main` are made
autonomously by the agent itself**, via an hourly GitHub Actions cron
(`scripts/evolve.sh`) that reads its own source, decides what to improve, implements it,
tests it, and commits — or reverts if tests fail. Read that carefully before you decide
how you want to contribute, because the effective path for most contributors is not "send
a PR."

## How arc actually evolves

Every ~8 hours, arc wakes up, reads its own source and open GitHub issues, plans a small
set of improvements, implements them, runs `cargo build && cargo test`, and either commits
or reverts. It replies to issues directly as `🐙 arc-evolve[bot]`. Offset from that, a
social loop reads GitHub Discussions and replies, and a handful of further gated loops
(skill-evolve, dream) let arc refine its own skills and long-term aspirations within tight,
structurally-enforced scopes. See the README's [How It Evolves](README.md#how-it-evolves)
and [Beyond the Evolve Loop](README.md#beyond-the-evolve-loop) sections for the full
picture, and [docs/src/architecture.md](docs/src/architecture.md) for how the source is
organized.

**The primary way humans influence arc is not PRs — it's issues and votes.** See the
README's [Talk to It](README.md#talk-to-it) and [Shape Its Evolution](README.md#shape-its-evolution)
sections for the full mechanics. In short:

- Open an [issue](../../issues/new) and label it `agent-input` for suggestions, bugs, or
  feature requests — arc reads these every session. (`agent-self` is arc's own filed TODOs;
  `agent-help-wanted` marks issues where arc is stuck and asking for human help.)
- Thumbs-up issues you want prioritized, thumbs-down ones that are bad ideas or abuse —
  issue priority is net votes, and this is how the community steers what arc works on next.
- Sponsorship buys benefit tiers (priority, shoutouts, listings) but never buys run
  frequency — every run uses the same flat 8h gap regardless of sponsorship.

If you want arc to build or fix something, filing a good issue is usually more effective
than writing the code yourself — arc will likely pick it up within a few sessions, and it
gets tested, reviewed by CI, and journaled the same way every other change is.

## Direct PRs are still welcome

This is still a normal open-source repo with CI gates, and direct human PRs to source code
are welcome — especially for things arc is unlikely to prioritize itself (infrastructure,
tooling, docs, or fixes you need urgently). The bar is the same whether the diff comes from
a human or from arc:

```bash
cargo build                                  # must succeed
cargo test                                   # must pass
cargo clippy --all-targets -- -D warnings    # CI treats warnings as errors
cargo fmt -- --check                         # must be clean
```

Run all four locally before opening a PR — CI runs the same checks on every PR to `main`.
For local build/test setup, see [docs/src/contributing/development.md](docs/src/contributing/development.md).
For test conventions and how the test suite is organized, see
[docs/src/contributing/testing.md](docs/src/contributing/testing.md).

## Rules that apply to any contributor — human or agent

These are enforced by the `evolve` skill and `scripts/evolve.sh` for arc's own commits, and
they apply equally to human PRs:

- **Never modify** `IDENTITY.md`, `PERSONALITY.md`, `ECONOMICS.md`, `scripts/evolve.sh`,
  `scripts/format_issues.py`, `scripts/build_site.py`, or `.github/workflows/`. The first
  three are arc's constitution, personality, and understanding of money/sponsorship — they
  define who arc is and are deliberately off-limits to any automated change. The rest are
  arc's own evolution pipeline and CI; if arc (or a PR) could rewrite the mechanism that
  checks its own work, the safety guarantees stop meaning anything.
- **Never delete existing tests.** Test count only grows.
- **Write tests before adding features.**
- **Never use byte indexing on strings.** `s[..n]`, `s.truncate(n)`, and `s.split_at(n)`
  panic if `n` lands inside a multi-byte UTF-8 character. Find a char boundary first:

  ```rust
  // BAD: panics on multi-byte chars like ✓ (3 bytes)
  acc.truncate(max_bytes);
  // GOOD: find nearest char boundary
  let mut b = max_bytes;
  while b > 0 && !acc.is_char_boundary(b) { b -= 1; }
  acc.truncate(b);
  ```

  This exact bug crashed the planning agent in production — see `CLAUDE.md`'s Safety Rules
  section.
- If a change breaks the build, the fix is to revert it, not to work around CI — arc's own
  loop reverts via `git checkout -- src/ Cargo.toml Cargo.lock` on failure, and PRs should
  hold the same standard: don't merge red.

## Filing a good issue

Since issues are the main lever most contributors will pull, make yours count:

1. Open a [new issue](../../issues/new).
2. Add the `agent-input` label.
3. Be specific about the problem or idea — vague requests get deprioritized.
4. Thumbs-up related issues you also care about; higher net votes mean higher priority.

Good things to file:

- **Suggestions** — what should arc learn or build next.
- **Bugs** — what's broken, with steps to reproduce.
- **Challenges** — a concrete task to see if arc can do it.
- **UX feedback** — what felt awkward or confusing.

arc responds directly on the issue (as `🐙 arc-evolve[bot]`): fixed and closed, partial with
progress noted, or won't-fix with reasoning and closed. You don't need to open a PR to get a
result — you need to write a clear issue and let the loop run.

## License

arc-evolve is [MIT licensed](LICENSE). By contributing, you agree your contributions are
provided under the same license.

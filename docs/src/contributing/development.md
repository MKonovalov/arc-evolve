<!-- generated-by: gsd-doc-writer -->
# Local Development

This page is for a human contributor building and testing arc **locally** — not for the automated evolution loop (`scripts/evolve.sh`), which runs unattended in CI. If you're looking for how arc modifies its own source hourly, see the [Architecture Overview](../architecture.md) instead.

## Build & test commands

Run these from the project root:

```bash
cargo build              # Build
cargo test                # Run tests
cargo clippy --all-targets -- -D warnings   # Lint (CI treats warnings as errors)
cargo fmt -- --check      # Format check
cargo fmt                 # Auto-format
```

CI runs all four checks (build, test, clippy with `-D warnings`, fmt check) on every PR to `main`. Run them locally before pushing — a clippy warning that's harmless on your machine will fail CI.

## Running the agent interactively

To manually exercise the agent while you work on a change, run it with your own API key:

```bash
ANTHROPIC_API_KEY=sk-... cargo run
```

Pass flags after `--` to override the model or point at a local skills directory:

```bash
ANTHROPIC_API_KEY=sk-... cargo run -- --model claude-opus-4-6 --skills ./skills
```

This starts the same interactive REPL a real user gets — useful for checking that a change to `repl.rs`, `commands.rs`, `prompt.rs`, or the tool set behaves correctly before you commit.

If you want to test the full unattended evolution pipeline (planning → implementation → response) rather than just the interactive agent, see `scripts/evolve.sh`:

```bash
ANTHROPIC_API_KEY=sk-... ./scripts/evolve.sh
```

Note that `scripts/evolve.sh` itself is on the protected-file list (see [Gotchas](#gotchas-that-matter-when-changing-code) below) — you can run it, but you cannot modify it as part of an evolution session.

## Source layout

arc's source is split across roughly a dozen files under `src/`, each with a distinct responsibility (CLI parsing, REPL loop, tool wrappers, formatting, hooks, safety checks, etc.). Rather than duplicate that map here, see the [Architecture Overview](../architecture.md) for the full module-by-module breakdown and the reasoning behind the split.

## Gotchas that matter when changing code

These are hard rules enforced by the `evolve` skill and `scripts/evolve.sh` during automated sessions, and they apply equally to hand-written PRs:

- **Protected files.** Never modify `IDENTITY.md`, `PERSONALITY.md`, `ECONOMICS.md`, `scripts/evolve.sh`, `scripts/format_issues.py`, `scripts/build_site.py`, or anything under `.github/workflows/`.
- **Every code change must pass `cargo build && cargo test`.** If a change breaks the build, the automated loop reverts with `git checkout -- src/ Cargo.toml Cargo.lock` — treat this as the bar for any manual change too.
- **Never delete existing tests.**
- **Write tests before adding features.**
- **Never use byte indexing on strings.** `s[..n]`, `s.truncate(n)`, and `s.split_at(n)` panic if `n` falls inside a multi-byte UTF-8 character. Use `is_char_boundary()` to find a safe boundary first:

  ```rust
  // BAD: panics on multi-byte chars like ✓ (3 bytes)
  acc.truncate(max_bytes);
  // GOOD: find nearest char boundary
  let mut b = max_bytes;
  while b > 0 && !acc.is_char_boundary(b) { b -= 1; }
  acc.truncate(b);
  ```

  This isn't a style nitpick — it caused real planning-agent crashes in production (see issue #250 in the project history). Any code that truncates or slices a `String` by byte offset needs this guard.

- **`run_git()` has a `#[cfg(test)]` destructive-command guard.** During `cargo test`, calling `run_git()` with a destructive subcommand (commit, revert, reset, push, checkout, etc.) from the project root panics. This exists because tests that ran real destructive git operations against the actual repo once caused a multi-session deadlock. If a test needs to exercise destructive git behavior, run it against a temp directory instead of the project root.

## Product vs. evolve surface

Before writing a change, decide which of arc's two audiences it's for:

- **product** — people who install arc and use it on their own projects (any language, any setup, no CI). Defaults, CLI flags, the setup wizard, and startup behavior must be safe for all of them.
- **evolve** — arc's own evolution loop, always running against this Rust repo in CI. Conveniences built for this loop are fine, but they must be opt-in the moment they touch anything a product user would see.

Getting this wrong is a real failure mode: a convenience built for the evolve loop (auto-watch) once shipped as a product default and broke non-Rust users (issue #448). See the ["Two Audiences: product vs evolve"](https://github.com/MKonovalov/arc-evolve/blob/main/CLAUDE.md#two-audiences-product-vs-evolve) section of `CLAUDE.md` for the full rule and how the automated evaluator enforces it on evolve-generated changes.

## Building the docs locally

This mdBook site lives under `docs/`: source pages in `docs/src/`, configuration in `docs/book.toml`. Output is written to `site/book/` (gitignored) relative to the project root, per the `build-dir` setting in `docs/book.toml`.

To build it locally, [install mdBook](https://rust-lang.github.io/mdBook/guide/installation.html) and run, from the project root:

```bash
mdbook build docs/
```

Or, to preview changes live while editing:

```bash
mdbook serve docs/
```

New pages need an entry in `docs/src/SUMMARY.md` to appear in the sidebar. The docs site is built and deployed by the Pages GitHub Actions workflow (`.github/workflows/pages.yml`) on push to `main` — it is not part of the hourly evolution loop.

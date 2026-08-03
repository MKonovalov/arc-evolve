<!-- generated-by: gsd-doc-writer -->
# Testing

arc's test suite spans thousands of unit tests colocated with the code plus a
black-box integration suite that exercises the compiled binary as a
subprocess. This page covers how tests are organized, how to run them, and
the one gotcha that trips up almost every new contributor: the `run_git()`
destructive-command guard.

For finding *undertested* code (as opposed to writing and running tests),
see [Mutation Testing](./mutation-testing.md).

## How tests are organized

**Unit tests live next to the code they test.** Every `src/*.rs` module that
has meaningful logic ends with a `#[cfg(test)] mod tests { ... }` block at
the bottom of the file, using `use super::*;` to pull in the module's items.
For example, `src/git.rs` ends with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_git_valid_args() {
        let result = run_git(&["--version"]);
        assert!(result.is_ok(), "git --version should succeed");
        // ...
    }
}
```

This is the convention across the codebase — when you add a function to
`src/commands_git.rs`, add its test to the `mod tests` block already at the
bottom of `src/commands_git.rs`, not to a separate file. There is no
`src/tests/` directory; test code and production code stay in the same file.

**Integration tests live in `tests/`.** `tests/integration.rs` spawns the
compiled `arc` binary as a real subprocess (via `Command::new(env!("CARGO_BIN_EXE_arc"))`)
and asserts on its actual stdout, stderr, and exit code — black-box CLI
testing, not calling internal functions directly. It clears API-key
environment variables and points `HOME` at a nonexistent path so tests never
accidentally pick up real credentials or a contributor's local config:

```rust
fn arc_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arc"));
    cmd.env_remove("ANTHROPIC_API_KEY");
    // ... other provider keys removed too
    cmd.env("HOME", "/nonexistent-arc-test-home");
    cmd
}
```

Use this style for tests that verify CLI behavior end-to-end (flag parsing,
`--help` output, exit codes, error messages) — anything that should keep
working exactly as an end user experiences it, independent of internal
refactors. `tests/safety_edge_cases.rs` is a placeholder file with a comment
explaining why its actual tests run a different way (the `safety` module is
private to `main.rs`); read the file's header comment before assuming it
follows the same pattern as `integration.rs`.

## Running tests

```bash
cargo test
```

This is what CI runs (`.github/workflows/ci.yml`) — no special flags, no
`--test-threads=1`. Tests are written to be safe under the default parallel
execution, which is also why the `run_git()` guard below exists: without it,
parallel destructive git tests would race against each other (and against
your working tree) in the same real repo checkout.

To run a single module's tests:

```bash
cargo test git::tests::
```

To run just the integration suite:

```bash
cargo test --test integration
```

## The `run_git()` destructive-command guard

This is the most common surprise for new test authors, so read this section
before writing a test that touches git.

`src/git.rs` defines `run_git()`, `run_git_in_dir()`, and `run_git_output()`
as the standard way production code shells out to `git`. Under
`#[cfg(test)]`, all three check the invoked subcommand against a list of
destructive commands (`commit`, `reset`, `revert`, `push`, `checkout`,
`clean`, `stash`, `add`, `merge`, `rebase`, `cherry-pick`, `rm`, `mv`) and
**panic** if:

1. the subcommand is in that destructive list, **and**
2. the working directory being targeted is the actual project root (the
   crate's `CARGO_MANIFEST_DIR`) — not a temp directory.

```
SAFETY: run_git() called with destructive command 'commit' from project root
during tests. Use a temp directory or mock instead.
```

The guard exists because a test that calls `run_git(&["commit", ...])` (or
`reset --hard`, `checkout --`, etc.) from the real repo checkout would mutate
or destroy the contributor's actual working tree and git history while
`cargo test` runs — this happened in production and cost multiple days of
lost work before the guard was added (see `CLAUDE.md`'s Safety Rules).

**What this means for you as a test author:**

- Read-only git calls (`status`, `diff`, `log`, `rev-parse`, `--version`) are
  never blocked — write these tests exactly as you would call the function
  in production code.
- If your test genuinely needs to run a destructive git command (e.g. you're
  testing worktree creation, which needs `git init` + `git add` + `git
  commit` to set up a realistic repo), do it inside a `tempfile::tempdir()`,
  not the project root. The guard only fires when the working directory
  matches the project root, so temp-dir operations are unaffected.
- Because the guard compares directories, not command source, you have two
  ways to sidestep it safely inside a temp dir:
  - Call `run_git_in_dir(&tmp_path, &[...])` — the guard checks `tmp_path`
    against the project root and passes.
  - Or shell out with a raw `std::process::Command::new("git").current_dir(&tmp_path)`
    directly, bypassing `run_git()` (and its guard) entirely. This is the
    pattern used by `setup_temp_repo()` in `src/commands_spawn.rs`'s test
    module, which does a real `git init` / `git config` / `git add` / `git
    commit` sequence inside a `tempfile::TempDir` to build a throwaway repo
    for worktree tests.
- Never try to "fix" a guard panic by changing directory with
  `std::env::set_current_dir()` inside a test — it's process-global and
  causes flaky races against other tests running in parallel. Pass an
  explicit temp path to `run_git_in_dir()` (or a raw `Command` with
  `.current_dir(...)`) instead.

## Never delete existing tests

CLAUDE.md's Safety Rules state tests must never be deleted. In practice,
this means:

- If you're refactoring a function, refactor its test alongside it — update
  assertions to match new behavior, but don't remove the test wholesale
  because it's inconvenient.
- If a test is genuinely testing removed functionality (the function itself
  was deleted), that's the one legitimate case for removing the test — and
  it should be removed in the same change that removes the function, not
  left dangling.
- If a test is flaky or slow, fix it or mark it `#[ignore]` with a comment
  explaining why — don't delete it to make CI green.
- Adding new tests when you add new behavior is expected, not optional —
  see the Test Quality expectations below.

## Test quality expectations

The project's [README](https://github.com/MKonovalov/arc-evolve#readme)
documents a "Test Quality" philosophy: a large unit + integration suite
covering CLI flags, command parsing, error quality, exit codes, output
formatting, edge cases, project detection, fuzzy scoring, git operations,
session management, markdown rendering, cost calculation, permission logic,
streaming behavior, and more.

Raw test count is a weak signal on its own, which is why the project also
runs mutation testing to check whether existing tests actually catch
regressions rather than just executing code paths. See
[Mutation Testing](./mutation-testing.md) for how to find code that passes
`cargo test` today but isn't meaningfully covered.

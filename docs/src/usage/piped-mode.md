# Piped Mode

When stdin is not a terminal (i.e., input is piped), arc reads all of stdin as a single prompt, processes it, and exits. This works like single-prompt mode but takes input from a pipe instead of a flag.

## Usage

```bash
echo "explain this code" | arc
cat prompt.txt | arc
git diff | arc
```

## When to use it

Piped mode is useful for:

- **Passing file contents** as part of the prompt
- **Chaining with other commands** in a pipeline
- **Feeding structured input** from scripts

## Examples

**Review a git diff:**
```bash
git diff HEAD~1 | arc --system "Review this diff for bugs."
```

**Analyze a file:**
```bash
cat src/main.rs | arc --system "Find all potential panics in this Rust code."
```

**Process command output:**
```bash
cargo test 2>&1 | arc --system "Explain these test failures and suggest fixes."
```

## Detection

arc detects piped mode automatically by checking if stdin is a terminal. If it is not, piped mode activates. If stdin is a terminal, interactive REPL mode starts instead.

If piped input is empty, arc exits with an error: `No input on stdin.`

## Quiet mode

When both stdin and stdout are piped (fully scripted usage), arc automatically enables quiet mode, suppressing informational `config:` and `context:` loading messages on stderr. You can also enable this explicitly with `--quiet` or `-q`:

```bash
echo "fix the test" | arc -q > result.md  # explicit quiet
echo "fix the test" | arc > result.md     # auto-quiet (both pipes detected)
```

The `arc_QUIET=1` environment variable also enables quiet mode.

## Slash commands aren't dispatched in piped mode

Slash commands (`/doctor`, `/status`, `/help`, etc.) belong to the interactive REPL — they depend on REPL state that piped mode doesn't have. If you pipe a slash command into arc, it won't run it; it would only get sent to the model as a literal string and waste a turn of tokens.

Instead, arc detects this case, prints a one-line warning to stderr, and exits with status code `2`. Use one of these alternatives:

```bash
arc doctor                       # run the subcommand directly
arc --prompt "/doctor"           # send the literal text to the agent
arc                              # interactive REPL
```


# Single-Prompt Mode

Run a single prompt without entering the REPL. arc will process the prompt, print the response, and exit.

## Usage

The simplest way is a bare positional prompt — just put your prompt in quotes:

```bash
arc "explain this codebase"
arc "find all TODO comments"
```

You can also use `--prompt` or `-p` explicitly:

```bash
arc --prompt "explain this codebase"
arc -p "find all TODO comments"
```

Both forms are equivalent. The bare prompt form is shorter and matches how most CLI tools work.

## When to use it

Single-prompt mode is useful for:

- **Quick questions** — get an answer without starting a session
- **Scripting** — run arc as part of a larger workflow
- **CI/CD pipelines** — automate code review or analysis

## Example

```bash
$ arc "count the lines of Rust code in this project"
  ▶ $ find . -name '*.rs' | xargs wc -l ✓ (0.1s)

There are 1,475 lines of Rust code across 1 file (src/main.rs).
```

## Combining with other flags

You can combine prompts with other flags:

```bash
arc "review this diff" --model claude-sonnet-4-20250514
arc "explain the architecture" --thinking high
arc -p "analyze the code" --system "You are a security auditor."
```

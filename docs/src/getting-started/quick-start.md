# Quick Start

Once installed, start arc:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
arc
```

Or pass the API key directly:

```bash
arc --api-key sk-ant-...
```

> **First time?** If you run `arc` without an API key, an interactive setup
> wizard walks you through choosing a provider, entering your API key, picking
> a model, and optionally saving a `.arc.toml` config file. After setup, you
> go straight into the REPL — no restart needed. You can also run the wizard
> anytime with `arc setup`. If you prefer to skip it, set your API key
> environment variable first or press Ctrl+C to cancel. Re-running setup over
> an existing config backs it up to `.arc.toml.bak` and preserves any settings
> the wizard doesn't manage (e.g. `auto_watch`).
>
> **`.env` support:** arc also reads provider API keys from a `.env` file in
> the current directory when the environment variable is not set. This works
> in any directory that keeps secrets in a `.env` file (the standard for a
> decade). Precedence is process environment > `.env` file — if
> `ANTHROPIC_API_KEY` (or the key for your configured provider) is set in the
> shell, it wins; otherwise a `KEY=value` entry in `.env` in your CWD is used.
> Keys are parsed as `KEY=value` lines; `#` comments and an optional `export `
> prefix are supported.

You'll see a banner like this:

```
  arc v0.1.4 — a coding agent growing up in public
  Type /help for commands, /quit to exit

  model: claude-opus-4-6
  git:   main
  cwd:   /home/user/project
```

## Your first prompt

Type a natural language request:

```
main > explain what this project does
```

arc will read files, run commands, and respond. You'll see tool executions as they happen:

```
  ▶ read README.md ✓
  ▶ ls src/ ✓
  ▶ read src/main.rs ✓

This project is a...
```

## Common tasks

**Read and explain code:**
```
> read src/main.rs and explain the main function
```

**Make changes:**
```
> add error handling to the parse_config function in src/config.rs
```

**Run commands:**
```
> run the tests and fix any failures
```

**Search a codebase:**
```
> find all TODO comments in this project
```

## Exiting

Type `/quit`, `/exit`, or press Ctrl+D.

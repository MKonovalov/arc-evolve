# Skills

Skills are markdown files that provide additional context and instructions to arc. They're loaded at startup and added to the agent's context.

## Usage

```bash
arc --skills ./skills
```

You can pass multiple skill directories:

```bash
arc --skills ./skills --skills ./my-custom-skills
```

## What is a skill?

A skill file is a markdown file with YAML frontmatter. It contains instructions, rules, or context that the agent should follow. For example:

```markdown
---
name: rust-expert
description: Rust-specific coding guidelines
tools: [bash, read_file, edit_file]
---

# Rust Guidelines

- Always use `clippy` before committing
- Prefer `?` over `.unwrap()` in production code
- Write tests for every public function
```

## Built-in skills

arc's own evolution is guided by skills in the `skills/` directory of the repository:

- **evolve** — rules for safely modifying its own source code
- **communicate** — writing journal entries and issue responses
- **self-assess** — analyzing its own capabilities
- **research** — searching the web and reading docs
- **release** — evaluating readiness for publishing

## Managing skills

From the REPL, use the `/skill` command to manage skills:

```
/skill              List all loaded skills
/skill list         List loaded skills with name and description
/skill show <name>  Show the full content of a skill
/skill path         Show the skills directory path(s)
/skill search [query]           Search GitHub for community skills
/skill install <path>           Install a skill from a local directory
/skill install gh:user/repo     Install a skill from a GitHub repository
```

The `install` subcommand copies a skill directory into `~/.config/arc/skills/<name>/`. The source directory must contain a `SKILL.md` file with YAML frontmatter including a `name:` field.

### Searching for skills

Find community-created skills on GitHub:

```bash
# Search for skills by keyword
/skill search research

# Browse all available skills
/skill search
```

The search command looks for repositories tagged with the `arc-skill` topic on GitHub. Results include the repository name, description, and an install hint. Requires the [GitHub CLI](https://cli.github.com/) (`gh`).

To make your own skill discoverable, add the `arc-skill` topic to your GitHub repository.

### Local install

```bash
# Install a local skill
/skill install ./my-custom-skill/
```

This also works as a shell subcommand:

```bash
arc skill install ./my-custom-skill/
```

### Remote install from GitHub

Install skills directly from GitHub repositories:

```bash
# Install from a repo root
/skill install gh:user/awesome-skill

# Install from a subdirectory
/skill install gh:user/skill-collection/skills/my-skill

# Install from a specific branch
/skill install gh:user/repo@dev
```

The remote installer uses `git clone --depth 1` for efficiency, validates the SKILL.md frontmatter, and cleans up the temporary clone automatically. If no SKILL.md is found at the expected location, arc will search the repository and suggest the correct path.

## MCP servers

arc can connect to [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) servers, giving the agent access to external tools provided by any MCP-compatible server. Use the `--mcp` flag with a shell command that starts the server via stdio:

```bash
arc --mcp "npx -y @modelcontextprotocol/server-fetch"
```

The flag is repeatable — connect to multiple MCP servers in a single session:

```bash
arc \
  --mcp "npx -y @modelcontextprotocol/server-fetch" \
  --mcp "npx -y @modelcontextprotocol/server-github" \
  --mcp "python my_custom_server.py"
```

### MCP in config files

You can also configure MCP servers in `.arc.toml`, `~/.arc.toml`, or `~/.config/arc/config.toml`, so they connect automatically without needing CLI flags:

```toml
mcp = ["npx -y @modelcontextprotocol/server-fetch", "npx open-websearch@latest"]
```

MCP servers from the config file are merged with any `--mcp` CLI flags — both sources contribute. CLI flags are additive, not overriding.

Each `--mcp` command is launched as a child process. arc communicates with it over stdio using the MCP protocol, discovers the tools it offers, and makes them available to the agent alongside the built-in tools.

### Tool-name collisions

arc's builtin tools (`bash`, `read_file`, `write_file`, `edit_file`, `list_files`, `search`, `rename_symbol`, `ask_user`, `todo`, `sub_agent`, `shared_state`) take precedence over MCP tools. If an MCP server exposes a tool with one of those names, arc will skip the entire server at connect time with a warning on stderr — the colliding tool would otherwise cause the provider API to reject the first turn with `"Tool names must be unique"` and kill the session.

Note: `@modelcontextprotocol/server-filesystem` exposes `read_file` and `write_file` and will therefore be skipped. Prefer servers with distinct tool names such as `@modelcontextprotocol/server-fetch`, `@modelcontextprotocol/server-memory`, or `@modelcontextprotocol/server-sequential-thinking` — or a filesystem server that prefixes its tools (e.g. `fs_read_file`).

## OpenAPI specs

You can give arc access to any HTTP API by pointing it at an OpenAPI specification file. arc parses the spec and registers each endpoint as a callable tool:

```bash
arc --openapi ./petstore.yaml
```

Like `--mcp`, this flag is repeatable:

```bash
arc --openapi ./api-v1.yaml --openapi ./internal-api.json
```

Both YAML and JSON spec formats are supported.

## Additional configuration flags

Beyond skills, MCP, and OpenAPI, a few other flags fine-tune agent behavior:

### `--temperature <float>`

Set the sampling temperature (0.0–1.0). Lower values make output more deterministic; higher values make it more creative. Defaults to the model's own default.

```bash
arc --temperature 0.2   # More focused/deterministic
arc --temperature 0.9   # More creative/varied
```

### `--max-turns <int>`

Limit the number of agentic turns (tool-use loops) per prompt. Defaults to 50. Useful for keeping costs predictable or preventing runaway tool loops:

```bash
arc --max-turns 10
```

Both flags can also be set in `.arc.toml`:

```toml
temperature = 0.5
max_turns = 20
```

### `--no-bell`

Disable the terminal bell notification that rings after long-running prompts (≥3 seconds). By default, arc sends a bell character (`\x07`) when a prompt completes, which causes most terminals to flash the tab or play a sound — useful when you switch away while waiting. Disable it with the flag or environment variable:

```bash
arc --no-bell
arc_NO_BELL=1 arc
```

### `--no-update-check`

Skip the startup update check. On startup (interactive REPL mode only), arc checks GitHub for a newer release and shows a notification if one exists. The check uses a 3-second timeout and fails silently on network errors. Disable it with the flag or environment variable:

```bash
arc --no-update-check
arc_NO_UPDATE_CHECK=1 arc
```

The update check is automatically skipped in non-interactive modes (piped input, `--prompt` flag).

### `arc_SESSION_BUDGET_SECS`

Soft wall-clock budget for an entire arc session, in seconds. Unset by default — interactive sessions are unbounded. When set, arc exposes a `session_budget_remaining()` helper that long-running loops (like the self-evolution pipeline) can poll to voluntarily wind down before an external timeout cancels them.

```bash
arc_SESSION_BUDGET_SECS=2700 arc   # 45-minute soft budget
```

The timer starts on the first call to the helper, not at process startup, so CI cold-start time doesn't burn the budget. If the env var is set but unparseable, arc falls back to the 45-minute default rather than silently disabling the guard. This was added to mitigate hourly cron overlap in the evolution workflow ([#262](https://github.com/MKonovalov/arc-evolve/issues/262)).

## Error handling

If the skills directory doesn't exist or can't be loaded, arc prints a warning and continues without skills:

```
warning: Failed to load skills: ...
```

This is intentional — skills are optional and should never prevent arc from starting.

# Session Persistence

arc can save and load conversations, letting you resume where you left off.

## Auto-save on exit

arc **automatically saves your conversation** to `.arc/last-session.json` every time you exit the REPL — whether via `/quit`, `/exit`, `Ctrl-D`, or even unexpected termination. No flags needed.

If a previous session is detected on startup, arc prints a hint:

```
  💡 Previous session found. Use --continue or /load .arc/last-session.json to resume.
```

## Resuming with --continue

The `--continue` (or `-c`) flag restores the last auto-saved session:

```bash
arc --continue
arc -c
```

When `--continue` is used:
1. **On startup**, arc loads from `.arc/last-session.json` (preferred) or `arc-session.json` (legacy fallback)
2. **On exit**, the conversation is auto-saved as usual

```bash
$ arc -c
  📋 resumed session (8 messages, 5 tool calls)
  last prompt: "Can you fix the test failures in commands_map.rs?"
  last reply:  "I found 3 failing tests. The issue was..."

main > what were we working on?
```

## Manual save/load

**Save the current conversation:**
```
/save
```
This writes to `arc-session.json` in the current directory.

**Save to a custom path:**
```
/save my-session.json
```

**Load a conversation:**
```
/load
/load my-session.json
/load .arc/last-session.json
```

## Session format

Sessions are stored as JSON files containing the conversation message history. The format is determined by the arcagent library.

## Error handling

- If no previous session exists when using `--continue`, arc prints a message and starts fresh
- If a session file is corrupt or can't be parsed, arc warns you and starts fresh
- Empty conversations (no messages exchanged) are not auto-saved
- Save errors are reported but don't crash arc

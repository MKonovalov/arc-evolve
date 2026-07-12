# Git Integration

arc is git-aware. It shows your current branch and provides commands for common git operations.

## Branch display

When you're in a git repository, the REPL prompt shows the current branch:

```
main > _
feature/new-parser > _
```

On startup, the branch is also shown in the status information:

```
  git:   main
```

## Git commands

### /diff

Show a summary of uncommitted changes (equivalent to `git diff --stat`):

```
/diff
```

Output:
```
 src/main.rs | 15 +++++++++------
 README.md   |  3 +++
 2 files changed, 12 insertions(+), 6 deletions(-)
```

If there are no uncommitted changes:
```
  (no uncommitted changes)
```

### /git diff

Show the actual diff content (line-by-line changes), not just a summary:

```
/git diff
```

Shows unstaged changes. To see staged changes instead:

```
/git diff --cached
```

### /git branch

List all branches, with the current branch highlighted in green:

```
/git branch
```

Create and switch to a new branch:

```
/git branch feature/my-new-feature
```

### /blame

Show who last modified each line of a file, with colorized output:

```
/blame src/main.rs
```

Limit to a specific line range:

```
/blame src/main.rs:10-20
```

Output is colorized: commit hashes (dim), author names (cyan), dates (dim), line numbers (yellow).

### /undo

Revert all uncommitted changes. This is equivalent to `git checkout -- .`:

```
/undo
```

Before reverting, `/undo` shows you what will be undone:

```
 src/main.rs | 15 +++++++++------
 1 file changed, 9 insertions(+), 6 deletions(-)
  ✓ reverted all uncommitted changes
```

If there's nothing to undo:
```
  (nothing to undo — no uncommitted changes)
```

## Using git through the agent

arc's bash tool can run any git command. You can ask the agent directly:

```
> commit these changes with message "fix: handle empty input"
> show me the last 5 commits
> create a new branch called feature/parser
```

The agent has full access to git through its shell tool.

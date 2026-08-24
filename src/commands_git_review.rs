//! Code review command handlers: /review, /blame, and non-interactive review.
//!
//! The pure prompt-building / comment-parsing helpers live in
//! `commands_review` (per-effort handlers) — re-exported here so call sites
//! through `commands_git_review` are unchanged.

pub use crate::commands_review::{
    build_review_prompt, build_review_prompt_structured, extract_review_json, has_explicit_effort,
    parse_review_comments, parse_review_effort, review_effort_hint, ReviewComment, ReviewEffort,
};

use crate::commands_session::auto_compact_if_needed;
use crate::format::*;
use crate::git::*;
use crate::prompt::run_prompt;

use arcagent::agent::Agent;
use arcagent::*;

/// Build a review prompt for either staged changes or a specific file.
/// Returns None if there's nothing to review, Some(prompt) otherwise.
pub fn build_review_content(arg: &str) -> Option<(String, String)> {
    let arg = arg.trim();
    if arg.is_empty() {
        // Review staged changes
        match get_staged_diff() {
            None => {
                eprintln!("{RED}  error: not in a git repository{RESET}\n");
                None
            }
            Some(diff) if diff.trim().is_empty() => {
                // Fall back to unstaged diff if nothing staged
                let unstaged = run_git(&["diff"]).unwrap_or_default();
                if unstaged.trim().is_empty() {
                    eprintln!("{DIM}  nothing to review — no staged or unstaged changes{RESET}\n");
                    None
                } else {
                    eprintln!("{DIM}  reviewing unstaged changes...{RESET}");
                    Some(("unstaged changes".to_string(), unstaged))
                }
            }
            Some(diff) => {
                eprintln!("{DIM}  reviewing staged changes...{RESET}");
                Some(("staged changes".to_string(), diff))
            }
        }
    } else if arg.starts_with("--pr") {
        // Review a PR: --pr <number>
        build_review_content_pr(arg)
    } else if arg.contains("..") {
        // Review a commit range: HEAD~3..HEAD, abc123..def456, etc.
        build_review_content_range(arg)
    } else {
        // Review a specific file
        let path = std::path::Path::new(arg);
        if !path.exists() {
            eprintln!("{RED}  error: file not found: {arg}{RESET}\n");
            return None;
        }
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    eprintln!("{DIM}  file is empty — nothing to review{RESET}\n");
                    None
                } else {
                    eprintln!("{DIM}  reviewing {arg}...{RESET}");
                    Some((arg.to_string(), content))
                }
            }
            Err(e) => {
                eprintln!("{RED}  error reading {arg}: {e}{RESET}\n");
                None
            }
        }
    }
}

/// Build review content from a git commit range (e.g. `HEAD~3..HEAD`).
fn build_review_content_range(arg: &str) -> Option<(String, String)> {
    match run_git(&["diff", arg]) {
        Ok(diff) if !diff.trim().is_empty() => {
            eprintln!("{DIM}  reviewing diff {arg}...{RESET}");
            Some((format!("diff {arg}"), diff))
        }
        Ok(_) => {
            eprintln!("{DIM}  no changes in range {arg}{RESET}\n");
            None
        }
        Err(e) => {
            eprintln!("{RED}  error: git diff {arg} failed: {e}{RESET}\n");
            None
        }
    }
}

/// Build review content from a GitHub PR number (e.g. `--pr 123`).
fn build_review_content_pr(arg: &str) -> Option<(String, String)> {
    let pr_num = arg.strip_prefix("--pr").unwrap_or("").trim();
    if pr_num.is_empty() {
        eprintln!("{RED}  error: --pr requires a PR number (e.g. --pr 123){RESET}\n");
        return None;
    }
    // Validate it's a number
    if pr_num.parse::<u64>().is_err() {
        eprintln!("{RED}  error: invalid PR number: {pr_num}{RESET}\n");
        return None;
    }
    // Use gh CLI to get the PR diff
    match std::process::Command::new("gh")
        .args(["pr", "diff", pr_num])
        .output()
    {
        Ok(output) if output.status.success() => {
            let diff = String::from_utf8_lossy(&output.stdout).to_string();
            if diff.trim().is_empty() {
                eprintln!("{DIM}  PR #{pr_num} has no diff{RESET}\n");
                None
            } else {
                eprintln!("{DIM}  reviewing PR #{pr_num}...{RESET}");
                Some((format!("PR #{pr_num}"), diff))
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{RED}  error: gh pr diff {pr_num} failed: {stderr}{RESET}\n");
            None
        }
        Err(e) => {
            eprintln!("{RED}  error: failed to run gh CLI: {e}{RESET}");
            eprintln!("{DIM}  install gh: https://cli.github.com/{RESET}\n");
            None
        }
    }
}

/// Run a non-interactive code review and return the review text.
///
/// This is the entry point for `arc review` CLI subcommand — it builds
/// the review content, creates a one-shot agent, runs the prompt, and
/// returns the agent's response text. No REPL, no interactive session.
///
/// # Arguments
/// * `arg` — the review target: empty (staged/unstaged), commit range, `--pr N`, or file path
/// * `agent_config` — pre-built agent configuration for creating the side agent
///
/// # Returns
/// * `Ok(review_text)` — the review completed successfully
/// * `Err(message)` — nothing to review or an error occurred
pub async fn run_non_interactive_review(
    arg: &str,
    agent_config: &crate::agent_builder::AgentConfig,
) -> Result<String, String> {
    let (effort, remaining) = parse_review_effort(arg);
    let (label, content) =
        build_review_content(&remaining).ok_or_else(|| "nothing to review".to_string())?;

    let prompt = build_review_prompt(&label, &content, effort);

    // Build a side agent — one-shot, no tools, concise
    let mut side_agent = agent_config.build_side_agent();

    let mut rx = side_agent.prompt(&prompt).await;

    let mut collected = String::new();
    let mut md_renderer = MarkdownRenderer::new();

    loop {
        match rx.recv().await {
            Some(AgentEvent::MessageUpdate {
                delta: StreamDelta::Text { delta },
                ..
            }) => {
                collected.push_str(&delta);
                // Stream to stderr so stdout stays clean for piping
                let rendered = md_renderer.render_delta(&delta);
                if !rendered.is_empty() {
                    eprint!("{rendered}");
                }
            }
            Some(AgentEvent::MessageEnd { .. }) => {
                let tail = md_renderer.flush();
                if !tail.is_empty() {
                    eprint!("{tail}");
                }
            }
            Some(AgentEvent::AgentEnd { .. }) => break,
            None => break,
            _ => {}
        }
    }

    side_agent.finish().await;
    eprintln!(); // newline after streamed output

    // Show cost on stderr
    let messages = side_agent.messages();
    let mut usage = Usage::default();
    for msg in messages {
        if let AgentMessage::Llm(arcagent::types::Message::Assistant { usage: u, .. }) = msg {
            usage.input += u.input;
            usage.output += u.output;
            usage.cache_read += u.cache_read;
            usage.cache_write += u.cache_write;
        }
    }
    let total_tokens = usage.input + usage.output;
    if total_tokens > 0 {
        let cost = estimate_cost(&usage, &agent_config.model);
        if let Some(c) = cost {
            eprintln!("{DIM}  review: {} tokens, ${:.4}{RESET}", total_tokens, c);
        } else {
            eprintln!("{DIM}  review: {} tokens{RESET}", total_tokens);
        }
    }

    if collected.trim().is_empty() {
        Err("agent returned empty response".to_string())
    } else {
        Ok(collected)
    }
}

/// Handle the /review command: review staged changes or a specific file.
/// Returns the review prompt if sent to AI, None otherwise.
pub async fn handle_review(
    input: &str,
    agent: &mut Agent,
    session_total: &mut Usage,
    model: &str,
) -> Option<String> {
    let arg = input.strip_prefix("/review").unwrap_or("").trim();
    let (effort, remaining) = parse_review_effort(arg);

    match build_review_content(&remaining) {
        Some((label, content)) => {
            if effort != ReviewEffort::Normal {
                eprintln!("{DIM}  effort: {}{RESET}", effort.label());
            } else if remaining.is_empty() && !has_explicit_effort(arg) {
                // First-impression contract: teach the effort switch when the
                // user didn't pick one. `remaining.is_empty()` keeps it off when
                // a target was given (no-arg is the discoverability moment).
                eprintln!("{}", review_effort_hint());
            }
            let prompt = build_review_prompt(&label, &content, effort);
            run_prompt(agent, &prompt, session_total, model).await;
            auto_compact_if_needed(agent);
            Some(prompt)
        }
        None => None,
    }
}

/// Parsed arguments for `/blame`.
#[derive(Debug, PartialEq)]
pub struct BlameArgs {
    pub file: String,
    pub range: Option<(usize, usize)>,
}

/// Parse `/blame <file>` or `/blame <file>:<start>-<end>`.
pub fn parse_blame_args(input: &str) -> Result<BlameArgs, String> {
    let arg = input.strip_prefix("/blame").unwrap_or(input).trim();

    if arg.is_empty() {
        return Err("Usage: /blame <file> or /blame <file>:<start>-<end>".to_string());
    }

    // Check for <file>:<start>-<end> pattern
    if let Some(colon_pos) = arg.rfind(':') {
        // SAFETY: colon_pos from rfind(':') — ':' is ASCII (1 byte), so colon_pos
        // and colon_pos + 1 are always valid char boundaries.
        let file_part = &arg[..colon_pos];
        let range_part = &arg[colon_pos + 1..];

        if let Some(dash_pos) = range_part.find('-') {
            // SAFETY: dash_pos from find('-') — '-' is ASCII (1 byte), so dash_pos
            // and dash_pos + 1 are always valid char boundaries within range_part.
            let start_str = &range_part[..dash_pos];
            let end_str = &range_part[dash_pos + 1..];

            if let (Ok(start), Ok(end)) = (start_str.parse::<usize>(), end_str.parse::<usize>()) {
                if start == 0 || end == 0 {
                    return Err("Line numbers must be >= 1".to_string());
                }
                if start > end {
                    return Err(format!("Invalid range: start ({start}) > end ({end})"));
                }
                if !file_part.is_empty() {
                    return Ok(BlameArgs {
                        file: file_part.to_string(),
                        range: Some((start, end)),
                    });
                }
            }
        }
    }

    // No valid range found — treat entire input as file path
    Ok(BlameArgs {
        file: arg.to_string(),
        range: None,
    })
}

/// Colorize a single line of `git blame` output.
///
/// Typical git blame line format:
/// `abc1234f (Author Name  2024-01-15 10:30:00 +0000  42) line content`
/// - Commit hash → DIM
/// - Author name → CYAN
/// - Date/time → DIM
/// - Line number → YELLOW
/// - Code content → default
pub fn colorize_blame_line(line: &str) -> String {
    // git blame output: <hash> (<author> <date> <time> <tz> <lineno>) <code>
    // Find the opening paren that starts the author section
    let Some(paren_open) = line.find('(') else {
        return line.to_string();
    };
    let Some(paren_close) = line.find(')') else {
        return line.to_string();
    };
    if paren_close <= paren_open {
        return line.to_string();
    }

    // SAFETY: paren_open from find('('), paren_close from find(')') — both ASCII
    // (1 byte each), so paren_open, paren_open + 1, paren_close, and paren_close + 1
    // are all valid char boundaries.
    let hash = &line[..paren_open];
    let annotation = &line[paren_open + 1..paren_close];
    let code = if paren_close + 1 < line.len() {
        &line[paren_close + 1..]
    } else {
        ""
    };

    // Inside the annotation: "Author Name  2024-01-15 10:30:00 +0000  42"
    // Try to find the date pattern (YYYY-MM-DD) to split author from date+lineno
    let mut author = annotation;
    let mut date_and_lineno = "";

    // Look for a date pattern: 4-digit year followed by -
    for (i, _) in annotation.char_indices() {
        if i + 10 <= annotation.len() {
            // SAFETY: i from char_indices() is always a valid char boundary.
            let slice = &annotation[i..];
            if slice.len() >= 10
                // SAFETY: as_bytes() indexing is safe because slice.len() >= 10
                // and bytes 0-9 are checked individually. The sub-slices [..4],
                // [5..7], [8..10] are safe because all checked bytes are ASCII digits
                // or '-', so every index falls on a char boundary.
                && slice.as_bytes()[4] == b'-'
                && slice.as_bytes()[7] == b'-'
                && slice[..4].chars().all(|c| c.is_ascii_digit())
                && slice[5..7].chars().all(|c| c.is_ascii_digit())
                && slice[8..10].chars().all(|c| c.is_ascii_digit())
            {
                // SAFETY: i from char_indices() — always a valid char boundary.
                author = annotation[..i].trim_end();
                date_and_lineno = &annotation[i..];
                break;
            }
        }
    }

    // Try to split the lineno from date portion
    // The lineno is typically the last whitespace-separated token
    let (date_part, lineno_part) =
        if let Some(last_space) = date_and_lineno.rfind(char::is_whitespace) {
            // SAFETY: rfind(char::is_whitespace) returns the byte index of the
            // start of the matching char — always a valid char boundary.
            let candidate = date_and_lineno[last_space..].trim();
            if candidate.chars().all(|c| c.is_ascii_digit()) && !candidate.is_empty() {
                (&date_and_lineno[..last_space], candidate)
            } else {
                (date_and_lineno, "")
            }
        } else {
            (date_and_lineno, "")
        };

    format!(
        "{DIM}{hash}{RESET}({CYAN}{author}{RESET} {DIM}{date_part}{RESET} {YELLOW}{lineno_part}{RESET}){code}"
    )
}

/// Colorize full `git blame` output (multiple lines).
pub fn colorize_blame(output: &str) -> String {
    output
        .lines()
        .map(colorize_blame_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Handle the `/blame` command.
pub fn handle_blame(input: &str) {
    let args = match parse_blame_args(input) {
        Ok(a) => a,
        Err(e) => {
            println!("  {RED}✗{RESET} {e}");
            return;
        }
    };

    let mut cmd = vec!["blame".to_string()];
    if let Some((start, end)) = args.range {
        cmd.push(format!("-L{start},{end}"));
    }
    cmd.push(args.file.clone());

    let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    match run_git(&cmd_refs) {
        Ok(output) => {
            if output.trim().is_empty() {
                println!("  {DIM}(no blame output){RESET}");
            } else {
                println!();
                println!("{}", colorize_blame(&output));
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no such path") || msg.contains("No such file") {
                println!("  {RED}✗{RESET} File not found: {DIM}{}{RESET}", args.file);
            } else if msg.contains("not a git repository") || msg.contains("fatal: not a git") {
                println!("  {RED}✗{RESET} Not in a git repository");
            } else {
                println!("  {RED}✗{RESET} {msg}");
            }
        }
    }
}

/// Post a PR review with inline comments using `gh api`.
///
/// Takes the PR number and parsed review comments, and posts them as a
/// GitHub pull request review with `COMMENT` event type.
pub fn post_pr_review(pr_number: u32, comments: &[ReviewComment]) -> Result<String, String> {
    if comments.is_empty() {
        return Err("No comments to post".to_string());
    }

    // Build the review body (summary)
    let summary = format!(
        "🐙 **arc review** — {} comment{} on PR #{}",
        comments.len(),
        if comments.len() == 1 { "" } else { "s" },
        pr_number
    );

    // Build the comments array for the API
    // Format: [{"path": "...", "line": N, "body": "..."}]
    let comments_json: Vec<String> = comments
        .iter()
        .map(|c| {
            let escaped_body = c
                .body
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            let escaped_path = c.path.replace('\\', "\\\\").replace('"', "\\\"");
            format!(
                "{{\"path\":\"{}\",\"line\":{},\"body\":\"{}\"}}",
                escaped_path, c.line, escaped_body
            )
        })
        .collect();
    let comments_array = format!("[{}]", comments_json.join(","));

    let escaped_summary = summary.replace('\\', "\\\\").replace('"', "\\\"");

    // Build the full review payload
    let payload = format!(
        "{{\"event\":\"COMMENT\",\"body\":\"{}\",\"comments\":{}}}",
        escaped_summary, comments_array
    );

    // Post via gh api
    let output = std::process::Command::new("gh")
        .args([
            "api",
            &format!("repos/{{owner}}/{{repo}}/pulls/{pr_number}/reviews"),
            "--method",
            "POST",
            "--input",
            "-",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("`gh` CLI not found or failed to start: {e}"))?;

    // Write JSON payload to stdin
    use std::io::Write;
    let mut child = output;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("Failed to write to gh stdin: {e}"))?;
    }

    let result = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for gh: {e}"))?;

    if result.status.success() {
        Ok(format!(
            "✓ Posted review with {} inline comment{} to PR #{}",
            comments.len(),
            if comments.len() == 1 { "" } else { "s" },
            pr_number
        ))
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("GitHub API error: {}", stderr.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{is_unknown_command, KNOWN_COMMANDS};

    #[test]
    fn test_review_command_recognized() {
        assert!(!is_unknown_command("/review"));
        assert!(!is_unknown_command("/review src/main.rs"));
        assert!(
            KNOWN_COMMANDS.contains(&"/review"),
            "/review should be in KNOWN_COMMANDS"
        );
    }

    #[test]
    fn test_review_command_matching() {
        // /review should match exact or with space separator, not /reviewing
        let review_matches = |s: &str| s == "/review" || s.starts_with("/review ");
        assert!(review_matches("/review"));
        assert!(review_matches("/review src/main.rs"));
        assert!(review_matches("/review Cargo.toml"));
        assert!(!review_matches("/reviewing"));
        assert!(!review_matches("/reviewer"));
    }

    #[test]
    fn test_build_review_content_nonexistent_file() {
        let result = build_review_content("nonexistent_file_xyz_12345.rs");
        assert!(result.is_none(), "Nonexistent file should return None");
    }

    #[test]
    fn test_build_review_content_existing_file() {
        // Use CARGO_MANIFEST_DIR for an absolute path to avoid CWD races
        // with other tests that call set_current_dir
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let cargo_toml = format!("{manifest_dir}/Cargo.toml");
        let result = build_review_content(&cargo_toml);
        assert!(result.is_some(), "Existing file should return Some");
        let (label, content) = result.unwrap();
        assert_eq!(label, cargo_toml);
        assert!(!content.is_empty(), "Content should not be empty");
    }

    #[test]
    fn test_build_review_content_empty_arg_in_git_repo() {
        // Empty arg reviews staged/unstaged changes
        // In CI, this may or may not have changes — just verify it doesn't panic
        let result = build_review_content("");
        // Result depends on git state — either Some or None is valid
        if let Some((label, _content)) = result {
            assert!(
                label.contains("changes"),
                "Label should describe what's being reviewed: {label}"
            );
        }
    }

    #[test]
    fn test_review_help_text_present() {
        // Verify /review appears in the help output by checking the handle_help function output
        // We can't easily capture stdout, but we can verify the command is in KNOWN_COMMANDS
        // and that the help text format is correct
        assert!(KNOWN_COMMANDS.contains(&"/review"));
    }

    #[test]
    fn test_parse_blame_args_file_only() {
        let result = parse_blame_args("/blame src/main.rs").unwrap();
        assert_eq!(result.file, "src/main.rs");
        assert_eq!(result.range, None);
    }

    #[test]
    fn test_parse_blame_args_with_range() {
        let result = parse_blame_args("/blame src/main.rs:10-20").unwrap();
        assert_eq!(result.file, "src/main.rs");
        assert_eq!(result.range, Some((10, 20)));
    }

    #[test]
    fn test_parse_blame_args_single_line_range() {
        let result = parse_blame_args("/blame foo.rs:5-5").unwrap();
        assert_eq!(result.file, "foo.rs");
        assert_eq!(result.range, Some((5, 5)));
    }

    #[test]
    fn test_parse_blame_args_no_args() {
        let result = parse_blame_args("/blame");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn test_parse_blame_args_no_args_with_spaces() {
        let result = parse_blame_args("/blame   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_blame_args_invalid_range_reversed() {
        let result = parse_blame_args("/blame foo.rs:20-10");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("start"));
    }

    #[test]
    fn test_parse_blame_args_zero_start() {
        let result = parse_blame_args("/blame foo.rs:0-10");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(">= 1"));
    }

    #[test]
    fn test_parse_blame_args_non_numeric_range_treated_as_file() {
        // If the range part doesn't parse as numbers, treat entire input as filename
        let result = parse_blame_args("/blame some:file:thing").unwrap();
        assert_eq!(result.file, "some:file:thing");
        assert_eq!(result.range, None);
    }

    #[test]
    fn test_parse_blame_args_unicode_file_path() {
        // File paths can contain multi-byte UTF-8 characters (e.g., accented letters).
        // The colon and dash splitting must not panic on such paths.
        let result = parse_blame_args("/blame src/données.rs:10-20").unwrap();
        assert_eq!(result.file, "src/données.rs");
        assert_eq!(result.range, Some((10, 20)));

        // Unicode path without range
        let result2 = parse_blame_args("/blame src/données.rs").unwrap();
        assert_eq!(result2.file, "src/données.rs");
        assert_eq!(result2.range, None);

        // Unicode path with colon but non-numeric range (treated as file)
        let result3 = parse_blame_args("/blame café:résumé").unwrap();
        assert_eq!(result3.file, "café:résumé");
        assert_eq!(result3.range, None);
    }

    #[test]
    fn test_colorize_blame_line_typical() {
        let line = "abc1234f (John Doe  2024-01-15 10:30:00 +0000  42) fn main() {";
        let colored = colorize_blame_line(line);
        // Should contain ANSI codes
        assert!(colored.contains("\x1b["));
        // Should still contain the original content
        assert!(colored.contains("John Doe"));
        assert!(colored.contains("fn main()"));
        assert!(colored.contains("abc1234f"));
    }

    #[test]
    fn test_colorize_blame_line_no_paren() {
        // Lines without parens should be returned unchanged
        let line = "some weird line without parens";
        assert_eq!(colorize_blame_line(line), line);
    }

    #[test]
    fn test_colorize_blame_multiple_lines() {
        let input = "abc123 (Alice 2024-01-15 10:00:00 +0000 1) line1\ndef456 (Bob   2024-01-15 10:00:00 +0000 2) line2";
        let colored = colorize_blame(input);
        let lines: Vec<&str> = colored.lines().collect();
        assert_eq!(lines.len(), 2);
        // Both lines should have ANSI codes
        assert!(lines[0].contains("\x1b["));
        assert!(lines[1].contains("\x1b["));
    }

    #[test]
    fn build_review_content_detects_commit_range() {
        // A range with ".." should be treated as a git diff range
        let arg = "HEAD~3..HEAD";
        // We can't test the actual git command in unit tests, but we can
        // verify the function handles the range format by checking it doesn't
        // try to treat it as a file path.
        let result = build_review_content(arg);
        // In test environment, git may fail — but it should NOT try to
        // read it as a file (which would give "file not found").
        // Either None (git error) or Some (if git works) is fine.
        // The key test: it should not panic.
        let _ = result;
    }

    #[test]
    fn build_review_content_detects_pr_flag() {
        // --pr without a number should print an error
        let result = build_review_content("--pr");
        assert!(result.is_none(), "bare --pr should return None");
    }

    #[test]
    fn build_review_content_pr_invalid_number() {
        let result = build_review_content("--pr abc");
        assert!(result.is_none(), "non-numeric PR should return None");
    }

    #[test]
    fn build_review_prompt_with_diff_label() {
        let prompt = build_review_prompt(
            "diff HEAD~3..HEAD",
            "diff --git a/foo b/foo\n+bar",
            ReviewEffort::Normal,
        );
        assert!(prompt.contains("diff HEAD~3..HEAD"));
        assert!(prompt.contains("+bar"));
    }

    #[test]
    fn build_review_prompt_with_pr_label() {
        let prompt = build_review_prompt("PR #42", "diff --git a/foo b/foo", ReviewEffort::Normal);
        assert!(prompt.contains("PR #42"));
    }
}

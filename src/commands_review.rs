//! Review prompt-building and comment-parsing helpers (extracted from
//! `commands_git_review.rs`). Pure, testable functions parameterized by
//! `ReviewEffort` — the prompt-building core behind `/review` and the
//! structured PR-review pipeline.

use crate::format::*;

/// Review effort level — controls depth and focus of the code review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewEffort {
    /// Focus on bugs and security only. Skip style nits. Be terse.
    Quick,
    /// Default: bugs, security, style, performance, suggestions.
    Normal,
    /// Deep review: also check error handling edge cases, API contract
    /// violations, test coverage gaps, documentation accuracy, concurrency safety.
    Thorough,
}

impl ReviewEffort {
    /// Human-readable label for the effort level.
    pub fn label(&self) -> &'static str {
        match self {
            ReviewEffort::Quick => "quick",
            ReviewEffort::Normal => "normal",
            ReviewEffort::Thorough => "thorough",
        }
    }
}

/// Parse effort flags from review input.
///
/// Strips `--quick` or `--thorough` from the input and returns the
/// effort level plus the remaining argument string (file path, range, etc.).
pub fn parse_review_effort(input: &str) -> (ReviewEffort, String) {
    let mut effort = ReviewEffort::Normal;
    let mut remaining_parts: Vec<&str> = Vec::new();

    for part in input.split_whitespace() {
        match part {
            "--quick" => effort = ReviewEffort::Quick,
            "--thorough" => effort = ReviewEffort::Thorough,
            _ => remaining_parts.push(part),
        }
    }

    (effort, remaining_parts.join(" "))
}

/// Review criteria text for a given effort level.
///
/// Pure per-effort builder — the quick/normal/thorough distinctions live here
/// so each effort's prompt-shape is individually testable.
pub fn review_criteria(effort: ReviewEffort) -> String {
    match effort {
        ReviewEffort::Quick => {
            "Focus on critical issues ONLY:\n\n\
             1. **Bugs** — logic errors, crashes, data corruption\n\
             2. **Security** — injection, unsafe operations, credential exposure\n\n\
             Skip style, performance, and minor suggestions. Be terse — one line per finding."
                .to_string()
        }
        ReviewEffort::Normal => {
            "Look for:\n\n\
             1. **Bugs** — logic errors, off-by-one errors, null/None handling, race conditions\n\
             2. **Security** — injection vulnerabilities, unsafe operations, credential exposure\n\
             3. **Style** — naming, idiomatic patterns, unnecessary complexity, dead code\n\
             4. **Performance** — obvious inefficiencies, unnecessary allocations, N+1 patterns\n\
             5. **Suggestions** — improvements, missing error handling, better approaches\n\n\
             Be specific: reference line numbers or code snippets. Be concise — skip things that look fine.\n\
             If the code looks good overall, say so briefly and note any minor suggestions."
                .to_string()
        }
        ReviewEffort::Thorough => {
            "Perform an exhaustive code review. Check ALL of the following:\n\n\
             1. **Bugs** — logic errors, off-by-one errors, null/None handling, race conditions\n\
             2. **Security** — injection vulnerabilities, unsafe operations, credential exposure\n\
             3. **Style** — naming, idiomatic patterns, unnecessary complexity, dead code\n\
             4. **Performance** — obvious inefficiencies, unnecessary allocations, N+1 patterns\n\
             5. **Error Handling** — missing error paths, swallowed errors, unhelpful error messages\n\
             6. **Edge Cases** — boundary conditions, empty inputs, overflow, unicode handling\n\
             7. **API Contracts** — function signatures match usage, invariants maintained\n\
             8. **Test Coverage** — untested paths, missing assertions, fragile test assumptions\n\
             9. **Documentation** — doc comments match behavior, examples are accurate\n\
             10. **Concurrency** — data races, deadlocks, lock ordering, shared mutable state\n\n\
             Be exhaustive. Reference line numbers. For each finding, explain the risk and suggest a fix.\n\
             Group findings by severity: critical → major → minor → nit."
                .to_string()
        }
    }
}

/// Build the review prompt to send to the AI.
pub fn build_review_prompt(label: &str, content: &str, effort: ReviewEffort) -> String {
    // Truncate if very large
    let max_chars = 30_000;
    let content_preview = if content.len() > max_chars {
        let truncated = safe_truncate(content, max_chars);
        format!(
            "{truncated}\n\n... (truncated, {} more chars)",
            content.len() - max_chars
        )
    } else {
        content.to_string()
    };

    let criteria = review_criteria(effort);

    let effort_label = match effort {
        ReviewEffort::Normal => String::new(),
        other => format!(" [{} review]", other.label()),
    };

    format!(
        "Review the following code ({label}){effort_label}. {criteria}\n\n```\n{content_preview}\n```"
    )
}

/// Build a review prompt that requests structured JSON output for posting.
///
/// The prompt asks the AI to produce both a human-readable review summary
/// and a JSON array of inline comments that can be posted to GitHub.
pub fn build_review_prompt_structured(pr_number: u32, pr_info: &str, diff: &str) -> String {
    let pr_section = if pr_info.trim().is_empty() {
        String::new()
    } else {
        format!("## PR Description\n\n{}\n\n", pr_info.trim())
    };

    format!(
        "Review this pull request (PR #{pr_number}). Analyze the diff for:\n\
         - Potential bugs or logic errors\n\
         - Code quality issues\n\
         - Missing error handling\n\
         - Performance concerns\n\
         - Suggestions for improvement\n\n\
         Be specific — reference file paths and line numbers from the diff.\n\
         Praise good patterns too. Be constructive.\n\n\
         {pr_section}\
         ## Diff\n\n```diff\n{diff}\n```\n\n\
         ## IMPORTANT: Output Format\n\n\
         After your review analysis, output a JSON code block with the inline comments to post.\n\
         Each comment should reference a file path and line number FROM THE DIFF (the '+' side \
         line number for additions, or the original line number for context about existing code).\n\n\
         Output the JSON block like this:\n\n\
         ```json\n\
         [\n\
         \x20 {{\"path\": \"src/example.rs\", \"line\": 42, \"body\": \"Consider adding error handling here\"}},\n\
         \x20 {{\"path\": \"src/lib.rs\", \"line\": 10, \"body\": \"Nice refactor! Much cleaner.\"}}\n\
         ]\n\
         ```\n\n\
         Rules for the JSON:\n\
         - `path` must be the file path as shown in the diff (e.g., `src/main.rs`)\n\
         - `line` must be a line number from the NEW side of the diff (the '+' lines)\n\
         - `body` should be a constructive review comment\n\
         - Include 3-10 comments covering the most important observations\n\
         - Only include comments where you have substantive feedback"
    )
}

/// A single inline review comment for a PR.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewComment {
    pub path: String,
    pub line: u64,
    pub body: String,
}

/// Parse a structured review JSON string into a list of review comments.
///
/// Expects a JSON array of objects with `path`, `line`, and `body` fields:
/// ```json
/// [
///   {"path": "src/main.rs", "line": 42, "body": "Consider error handling here"},
///   {"path": "src/lib.rs", "line": 10, "body": "This could be simplified"}
/// ]
/// ```
pub fn parse_review_comments(json: &str) -> Result<Vec<ReviewComment>, String> {
    // Find the JSON array in the response — it may be wrapped in markdown code fences
    let trimmed = json.trim();
    let json_str = if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            &trimmed[start..=end]
        } else {
            return Err("No closing ']' found in review JSON".to_string());
        }
    } else {
        return Err("No JSON array found in review output".to_string());
    };

    // Parse using a simple JSON parser (no serde dependency needed)
    let mut comments = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut obj_start = None;

    for (i, ch) in json_str.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => {
                if depth == 1 {
                    obj_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 1 {
                    if let Some(start) = obj_start {
                        let obj_str = &json_str[start..=i];
                        if let Some(comment) = parse_single_comment(obj_str) {
                            comments.push(comment);
                        }
                    }
                    obj_start = None;
                }
            }
            '[' if depth == 0 => {
                depth = 1;
            }
            ']' if depth == 1 => {
                break;
            }
            _ => {}
        }
    }

    if comments.is_empty() {
        return Err("No valid review comments found in JSON".to_string());
    }
    Ok(comments)
}

/// Parse a single JSON object string into a ReviewComment.
fn parse_single_comment(obj: &str) -> Option<ReviewComment> {
    let path = extract_json_string_field(obj, "path")?;
    let line = extract_json_number_field(obj, "line")?;
    let body = extract_json_string_field(obj, "body")?;
    Some(ReviewComment { path, line, body })
}

/// Extract a string value for a given key from a JSON object string.
fn extract_json_string_field(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let key_pos = obj.find(&pattern)?;
    let after_key = &obj[key_pos + pattern.len()..];
    // Skip whitespace and colon
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_colon = after_colon.trim_start();
    // Expect a quoted string
    if !after_colon.starts_with('"') {
        return None;
    }
    let content = &after_colon[1..];
    let mut result = String::new();
    let mut escape = false;
    for ch in content.chars() {
        if escape {
            match ch {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                _ => {
                    result.push('\\');
                    result.push(ch);
                }
            }
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if ch == '"' {
            return Some(result);
        } else {
            result.push(ch);
        }
    }
    None // unterminated string
}

/// Extract a numeric value for a given key from a JSON object string.
fn extract_json_number_field(obj: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{}\"", key);
    let key_pos = obj.find(&pattern)?;
    let after_key = &obj[key_pos + pattern.len()..];
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_colon = after_colon.trim_start();
    // Read digits
    let num_str: String = after_colon
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num_str.parse().ok()
}

/// Extract the JSON review block from the AI's response text.
///
/// Looks for a ```json code block and extracts its contents.
pub fn extract_review_json(response: &str) -> Option<String> {
    // Look for ```json block
    let json_start_markers = ["```json\n", "```json\r\n"];
    for marker in &json_start_markers {
        if let Some(start) = response.find(marker) {
            let content_start = start + marker.len();
            if let Some(end) = response[content_start..].find("```") {
                return Some(response[content_start..content_start + end].to_string());
            }
        }
    }
    // Fallback: look for a bare JSON array
    if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            return Some(response[start..=end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_review_prompt_contains_label() {
        let prompt = build_review_prompt("staged changes", "fn main() {}", ReviewEffort::Normal);
        assert!(
            prompt.contains("staged changes"),
            "Prompt should include the label"
        );
    }

    #[test]
    fn build_review_prompt_contains_content() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let prompt = build_review_prompt("test.rs", code, ReviewEffort::Normal);
        assert!(prompt.contains(code), "Prompt should include the code");
    }

    #[test]
    fn build_review_prompt_contains_review_criteria() {
        let prompt = build_review_prompt("file.rs", "let x = 1;", ReviewEffort::Normal);
        assert!(prompt.contains("Bugs"), "Should mention bugs");
        assert!(prompt.contains("Security"), "Should mention security");
        assert!(prompt.contains("Style"), "Should mention style");
        assert!(prompt.contains("Performance"), "Should mention performance");
        assert!(prompt.contains("Suggestions"), "Should mention suggestions");
    }

    #[test]
    fn build_review_prompt_truncates_large_content() {
        let large_content = "x".repeat(50_000);
        let prompt = build_review_prompt("big.rs", &large_content, ReviewEffort::Normal);
        assert!(
            prompt.contains("truncated"),
            "Large content should be truncated"
        );
        assert!(
            prompt.contains("20000 more chars"),
            "Should show remaining char count"
        );
        // The prompt should be shorter than the original content
        assert!(
            prompt.len() < large_content.len(),
            "Prompt should be shorter than 50k"
        );
    }

    #[test]
    fn build_review_prompt_does_not_truncate_small_content() {
        let small_content = "fn hello() { println!(\"hi\"); }";
        let prompt = build_review_prompt("small.rs", small_content, ReviewEffort::Normal);
        assert!(
            !prompt.contains("truncated"),
            "Small content should not be truncated"
        );
        assert!(
            prompt.contains(small_content),
            "Full content should be present"
        );
    }

    #[test]
    fn build_review_prompt_wraps_in_code_block() {
        let prompt = build_review_prompt("test.rs", "let x = 42;", ReviewEffort::Normal);
        assert!(prompt.contains("```"), "Content should be in a code block");
    }

    #[test]
    fn test_build_review_prompt_contains_content() {
        let prompt = build_review_prompt(
            "staged changes",
            "fn main() {\n    println!(\"hello\");\n}",
            ReviewEffort::Normal,
        );
        assert!(
            prompt.contains("staged changes"),
            "Should mention the label"
        );
        assert!(prompt.contains("fn main()"), "Should contain the code");
        assert!(prompt.contains("Bugs"), "Should ask for bug review");
        assert!(
            prompt.contains("Security"),
            "Should ask for security review"
        );
        assert!(prompt.contains("Style"), "Should ask for style review");
        assert!(
            prompt.contains("Performance"),
            "Should ask for performance review"
        );
        assert!(prompt.contains("Suggestions"), "Should ask for suggestions");
    }

    #[test]
    fn test_build_review_prompt_truncates_large_content() {
        let large_content = "x".repeat(40_000);
        let prompt = build_review_prompt("big file", &large_content, ReviewEffort::Normal);
        assert!(
            prompt.contains("truncated"),
            "Large content should be truncated"
        );
        assert!(
            prompt.len() < 40_000,
            "Prompt should be truncated, got {} chars",
            prompt.len()
        );
    }

    #[test]
    fn test_build_review_prompt_quick() {
        let prompt = build_review_prompt("test.rs", "let x = 1;", ReviewEffort::Quick);
        assert!(prompt.contains("Bugs"), "Quick review should mention bugs");
        assert!(
            prompt.contains("Security"),
            "Quick review should mention security"
        );
        assert!(
            !prompt.contains("Style"),
            "Quick review should NOT mention style"
        );
        assert!(
            !prompt.contains("Performance"),
            "Quick review should NOT mention performance"
        );
        assert!(
            prompt.contains("quick review"),
            "Should include effort label"
        );
        assert!(
            prompt.contains("terse"),
            "Quick review should ask for terse output"
        );
    }

    #[test]
    fn test_build_review_prompt_thorough() {
        let prompt = build_review_prompt("test.rs", "let x = 1;", ReviewEffort::Thorough);
        assert!(prompt.contains("Bugs"), "Should mention bugs");
        assert!(prompt.contains("Security"), "Should mention security");
        assert!(prompt.contains("Edge Cases"), "Should mention edge cases");
        assert!(
            prompt.contains("API Contracts"),
            "Should mention API contracts"
        );
        assert!(
            prompt.contains("Test Coverage"),
            "Should mention test coverage"
        );
        assert!(prompt.contains("Concurrency"), "Should mention concurrency");
        assert!(
            prompt.contains("Documentation"),
            "Should mention documentation"
        );
        assert!(
            prompt.contains("thorough review"),
            "Should include effort label"
        );
        assert!(
            prompt.contains("exhaustive"),
            "Thorough review should ask for exhaustive review"
        );
    }

    #[test]
    fn test_build_review_prompt_normal_no_effort_label() {
        let prompt = build_review_prompt("test.rs", "let x = 1;", ReviewEffort::Normal);
        // Normal should not have an effort label suffix
        assert!(
            !prompt.contains("[normal review]"),
            "Normal effort should not show effort label"
        );
    }

    #[test]
    fn test_review_effort_label() {
        assert_eq!(ReviewEffort::Quick.label(), "quick");
        assert_eq!(ReviewEffort::Normal.label(), "normal");
        assert_eq!(ReviewEffort::Thorough.label(), "thorough");
    }

    #[test]
    fn test_parse_review_effort_default() {
        let (effort, remaining) = parse_review_effort("src/main.rs");
        assert_eq!(effort, ReviewEffort::Normal);
        assert_eq!(remaining, "src/main.rs");
    }

    #[test]
    fn test_parse_review_effort_default_empty() {
        let (effort, remaining) = parse_review_effort("");
        assert_eq!(effort, ReviewEffort::Normal);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_review_effort_quick() {
        let (effort, remaining) = parse_review_effort("--quick src/main.rs");
        assert_eq!(effort, ReviewEffort::Quick);
        assert_eq!(remaining, "src/main.rs");
    }

    #[test]
    fn test_parse_review_effort_quick_no_arg() {
        let (effort, remaining) = parse_review_effort("--quick");
        assert_eq!(effort, ReviewEffort::Quick);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_review_effort_thorough() {
        let (effort, remaining) = parse_review_effort("--thorough");
        assert_eq!(effort, ReviewEffort::Thorough);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_review_effort_thorough_with_file() {
        let (effort, remaining) = parse_review_effort("--thorough src/lib.rs");
        assert_eq!(effort, ReviewEffort::Thorough);
        assert_eq!(remaining, "src/lib.rs");
    }

    #[test]
    fn test_parse_review_effort_flag_after_file() {
        // Flag can appear after the file path too
        let (effort, remaining) = parse_review_effort("src/main.rs --quick");
        assert_eq!(effort, ReviewEffort::Quick);
        assert_eq!(remaining, "src/main.rs");
    }

    #[test]
    fn test_parse_review_effort_with_pr_flag() {
        let (effort, remaining) = parse_review_effort("--quick --pr 42");
        assert_eq!(effort, ReviewEffort::Quick);
        assert_eq!(remaining, "--pr 42");
    }

    #[test]
    fn test_review_criteria_differs_per_effort() {
        // Each effort level must produce a distinct criteria body — this is the
        // per-effort pure-handler contract the dispatcher relies on.
        let quick = review_criteria(ReviewEffort::Quick);
        let normal = review_criteria(ReviewEffort::Normal);
        let thorough = review_criteria(ReviewEffort::Thorough);
        assert_ne!(quick, normal);
        assert_ne!(normal, thorough);
        assert_ne!(quick, thorough);
        assert!(quick.contains("critical issues ONLY"));
        assert!(thorough.contains("exhaustive"));
        // Normal is the default and mentions all five default dimensions
        assert!(normal.contains("Suggestions"));
        assert!(normal.contains("Performance"));
    }

    #[test]
    fn parse_review_comments_basic() {
        let json = r#"[
            {"path": "src/main.rs", "line": 42, "body": "Add error handling"},
            {"path": "src/lib.rs", "line": 10, "body": "Nice refactor!"}
        ]"#;
        let comments = parse_review_comments(json).unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].path, "src/main.rs");
        assert_eq!(comments[0].line, 42);
        assert_eq!(comments[0].body, "Add error handling");
        assert_eq!(comments[1].path, "src/lib.rs");
        assert_eq!(comments[1].line, 10);
    }

    #[test]
    fn parse_review_comments_with_markdown_fences() {
        let json = "Here is the review:\n\n```json\n[\n  {\"path\": \"foo.rs\", \"line\": 1, \"body\": \"ok\"}\n]\n```\n\nDone.";
        let comments = parse_review_comments(json).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].path, "foo.rs");
    }

    #[test]
    fn parse_review_comments_empty_array() {
        let json = "[]";
        let result = parse_review_comments(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No valid review comments"));
    }

    #[test]
    fn parse_review_comments_no_json() {
        let result = parse_review_comments("no json here at all");
        assert!(result.is_err());
    }

    #[test]
    fn parse_review_comments_escaped_body() {
        let json = r#"[{"path": "a.rs", "line": 5, "body": "Fix the \"bug\" here\nand here"}]"#;
        let comments = parse_review_comments(json).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].body, "Fix the \"bug\" here\nand here");
    }

    #[test]
    fn extract_review_json_from_markdown() {
        let text = "Great PR! Here's my review:\n\n```json\n[{\"path\": \"x.rs\", \"line\": 1, \"body\": \"ok\"}]\n```\n\nAll looks good.";
        let json = extract_review_json(text).unwrap();
        assert!(json.contains("x.rs"));
    }

    #[test]
    fn extract_review_json_bare_array() {
        let text = "review: [{\"path\": \"y.rs\", \"line\": 2, \"body\": \"fix\"}]";
        let json = extract_review_json(text).unwrap();
        assert!(json.contains("y.rs"));
    }

    #[test]
    fn extract_review_json_none() {
        assert!(extract_review_json("no json here").is_none());
    }

    #[test]
    fn build_review_prompt_structured_has_json_format() {
        let prompt = build_review_prompt_structured(42, "Fix bugs", "+fn main() {}");
        assert!(prompt.contains("PR #42"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("path"));
        assert!(prompt.contains("line"));
        assert!(prompt.contains("body"));
        assert!(prompt.contains("Fix bugs"));
    }

    #[test]
    fn build_review_prompt_structured_empty_pr_info() {
        let prompt = build_review_prompt_structured(10, "", "diff content");
        assert!(prompt.contains("PR #10"));
        // Should not have an empty PR description section
        assert!(!prompt.contains("## PR Description"));
    }

    #[test]
    fn review_comment_equality() {
        let a = ReviewComment {
            path: "a.rs".to_string(),
            line: 1,
            body: "ok".to_string(),
        };
        let b = ReviewComment {
            path: "a.rs".to_string(),
            line: 1,
            body: "ok".to_string(),
        };
        assert_eq!(a, b);
    }
}
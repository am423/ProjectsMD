//! Session Log append operations.
//!
//! Parses and appends entries to the Session Log section of a project.md file.

/// Parse session log entries from section content.
///
/// Returns a vec of entry strings (each is the full line text minus the "- " prefix).
pub fn parse_session_log(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("- ").map(|s| s.to_string())
        })
        .collect()
}

/// Append a new entry to the session log content.
///
/// Adds the entry as a new `- ` bullet at the end of the content.
pub fn append_session_log(content: &str, entry: &str) -> String {
    let mut out = content.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("- {}\n", entry));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_LOG: &str = r#"- **2026-04-25** — Project kickoff. Defined problem, requirements, constraints. Chose Go + OpenWeatherMap. Set up project structure. (1.5 hours)
- **2026-04-26** — Implemented API client (Task 2) with JSON parsing and httptest mocks. API returns 401 for bad keys — added specific handling. (2 hours)
- **2026-04-28** — Built caching layer (Task 3). Hit corruption on Windows with direct writes — switched to atomic rename. All cache tests passing. (2 hours)
- **2026-04-29** — Added --unit flag during Task 3 review. Started Task 4, researched color libraries. (1 hour, ongoing)
"#;

    #[test]
    fn test_parse_session_log() {
        let entries = parse_session_log(EXAMPLE_LOG);
        assert_eq!(entries.len(), 4);
        assert!(entries[0].contains("2026-04-25"));
        assert!(entries[0].contains("Project kickoff"));
        assert!(entries[3].contains("2026-04-29"));
    }

    #[test]
    fn test_append_session_log() {
        let result = append_session_log(EXAMPLE_LOG, "**2026-04-30** — New entry. (1 hour)");
        let entries = parse_session_log(&result);
        assert_eq!(entries.len(), 5);
        assert!(entries[4].contains("2026-04-30"));
    }

    #[test]
    fn test_append_to_empty() {
        let result = append_session_log("", "**2026-04-30** — First entry");
        assert!(result.contains("- **2026-04-30** — First entry"));
    }

    #[test]
    fn test_parse_example_cli_tool() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let (_, body) = crate::frontmatter::parse_frontmatter(&content)
            .unwrap()
            .unwrap();
        let sections = crate::sections::parse_sections(body);
        let log_section = crate::sections::find_section(&sections, "Session Log").unwrap();
        let entries = parse_session_log(&log_section.content);
        assert_eq!(entries.len(), 4);
    }
}

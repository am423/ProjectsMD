//! Current State section parsing.
//!
//! Extracts the current project state from bold-labeled key-value pairs
//! in the Current State section of a project.md file.

/// The current state of a project.
#[derive(Debug, Clone)]
pub struct CurrentState {
    pub phase: String,
    pub last_completed: String,
    pub in_progress: String,
    pub next_action: String,
    pub blockers: String,
    pub notes: String,
}

/// Parse the Current State section content.
///
/// Expects lines in the format: `**Key:** value`
pub fn parse_state(content: &str) -> CurrentState {
    let mut state = CurrentState {
        phase: String::new(),
        last_completed: String::new(),
        in_progress: String::new(),
        next_action: String::new(),
        blockers: String::new(),
        notes: String::new(),
    };

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = extract_bold_value(trimmed, "Phase") {
            state.phase = rest.to_string();
        } else if let Some(rest) = extract_bold_value(trimmed, "Last completed") {
            state.last_completed = rest.to_string();
        } else if let Some(rest) = extract_bold_value(trimmed, "In progress") {
            state.in_progress = rest.to_string();
        } else if let Some(rest) = extract_bold_value(trimmed, "Next action") {
            state.next_action = rest.to_string();
        } else if let Some(rest) = extract_bold_value(trimmed, "Blockers") {
            state.blockers = rest.to_string();
        } else if let Some(rest) = extract_bold_value(trimmed, "Notes") {
            state.notes = rest.to_string();
        }
    }

    state
}

/// Extract the value from a `**Key:** value` formatted line.
fn extract_bold_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("**{}:**", key);
    if let Some(pos) = line.find(&pattern) {
        let rest = &line[pos + pattern.len()..];
        Some(rest.trim())
    } else {
        None
    }
}

/// Serialize a CurrentState back to markdown format.
pub fn write_state(state: &CurrentState) -> String {
    let mut out = String::new();
    out.push_str(&format!("**Phase:** {}\n", state.phase));
    out.push_str(&format!("**Last completed:** {}\n", state.last_completed));
    out.push_str(&format!("**In progress:** {}\n", state.in_progress));
    out.push_str(&format!("**Next action:** {}\n", state.next_action));
    out.push_str(&format!("**Blockers:** {}\n", state.blockers));
    out.push_str(&format!("**Notes:** {}\n", state.notes));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_STATE: &str = r#"**Phase:** build
**Last completed:** Task 3 (caching layer with file-based storage)
**In progress:** Task 4 (color-coded terminal output)
**Next action:** Implement temperature-to-color mapping in display/format.go using fatih/color library. Map: <10°C = blue, 10-25°C = green, >25°C = red. Handle terminal detection (no color in pipes via color.NoColor).
**Blockers:** None
**Notes:** Added --unit flag (metric/imperial) during Task 3 since the API supports it trivially. Cache uses atomic rename pattern to avoid corruption on Windows.
"#;

    #[test]
    fn test_parse_state() {
        let state = parse_state(EXAMPLE_STATE);
        assert_eq!(state.phase, "build");
        assert_eq!(
            state.last_completed,
            "Task 3 (caching layer with file-based storage)"
        );
        assert_eq!(state.in_progress, "Task 4 (color-coded terminal output)");
        assert!(state.next_action.starts_with("Implement temperature"));
        assert_eq!(state.blockers, "None");
        assert!(state.notes.contains("atomic rename"));
    }

    #[test]
    fn test_write_state() {
        let state = parse_state(EXAMPLE_STATE);
        let output = write_state(&state);
        assert!(output.contains("**Phase:** build"));
        assert!(output.contains("**Blockers:** None"));
    }

    #[test]
    fn test_roundtrip() {
        let state = parse_state(EXAMPLE_STATE);
        let output = write_state(&state);
        let state2 = parse_state(&output);
        assert_eq!(state.phase, state2.phase);
        assert_eq!(state.blockers, state2.blockers);
        assert_eq!(state.next_action, state2.next_action);
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
        let state_section = crate::sections::find_section(&sections, "Current State").unwrap();
        let state = parse_state(&state_section.content);
        assert_eq!(state.phase, "build");
        assert_eq!(state.blockers, "None");
    }
}

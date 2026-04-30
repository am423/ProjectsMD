//! Section parsing for project.md files.
//!
//! Splits a project.md body into sections based on `##` headings.
//! Sub-headings (`###`, etc.) are included in their parent section's content.

/// A single section parsed from a project.md file.
#[derive(Debug, Clone)]
pub struct Section {
    /// The heading text (without `##` prefix).
    pub heading: String,
    /// Heading level (2 for `##`, 3 for `###`, etc.).
    pub level: u8,
    /// The section body content (everything until the next heading of equal or lesser level).
    pub content: String,
    /// Byte offset of the heading line in the original body string.
    pub offset: usize,
}

/// Count leading '#' characters and return (level, rest_of_line).
fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let count = trimmed.chars().take_while(|&c| c == '#').count() as u8;
    let rest = &trimmed[count as usize..];
    let heading = rest.trim();
    if heading.is_empty() {
        return None; // Not a real heading
    }
    Some((count, heading))
}

/// Parse markdown body into sections.
///
/// Splits on `##` headings. Sub-headings (`###`, `####`, etc.) are included
/// in their parent section's content.
pub fn parse_sections(body: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::new();
    let mut current_level: u8 = 0;
    let mut current_content = String::new();
    let mut current_offset: usize = 0;
    let mut byte_pos: usize = 0;
    let mut found_any = false;

    for line in body.lines() {
        if let Some((level, heading)) = parse_heading(line) {
            // Only split on level 2 headings (##) — sub-headings stay in parent
            if level == 2 {
                // Save previous section
                if found_any || !current_content.trim().is_empty() {
                    sections.push(Section {
                        heading: current_heading,
                        level: current_level,
                        content: current_content,
                        offset: current_offset,
                    });
                }
                current_heading = heading.to_string();
                current_level = level;
                current_content = String::new();
                current_offset = byte_pos;
                found_any = true;
            } else {
                // Sub-heading (### or deeper) — include in current section
                current_content.push_str(line);
                current_content.push('\n');
            }
        } else {
            if !found_any && line.trim().is_empty() && current_content.is_empty() {
                // Skip leading blank lines before first heading
            } else {
                if !current_content.is_empty() || !line.trim().is_empty() {
                    current_content.push_str(line);
                    current_content.push('\n');
                }
            }
        }
        byte_pos += line.len() + 1; // +1 for newline
    }

    // Push the last section
    if found_any || !current_content.trim().is_empty() {
        sections.push(Section {
            heading: current_heading,
            level: current_level,
            content: current_content,
            offset: current_offset,
        });
    }

    sections
}

/// Find a section by heading name (case-insensitive).
pub fn find_section<'a>(sections: &'a [Section], name: &str) -> Option<&'a Section> {
    let name_lower = name.to_lowercase();
    sections
        .iter()
        .find(|s| s.heading.to_lowercase() == name_lower)
}

/// Update a section's content by heading name.
///
/// Returns `true` if the section was found and updated, `false` otherwise.
pub fn update_section(sections: &mut [Section], name: &str, new_content: &str) -> bool {
    let name_lower = name.to_lowercase();
    if let Some(section) = sections
        .iter_mut()
        .find(|s| s.heading.to_lowercase() == name_lower)
    {
        section.content = new_content.to_string();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_BODY: &str = r#"## What This Is

A CLI tool that fetches current weather data.

## Core Value

Fast, offline-capable weather display.

## Requirements

### Validated

- ✓ Current weather by city name — v0.1

### Active

- [ ] Color-coded terminal output

## Tasks

### Phase: BUILD

- [x] Task 1: Project setup
- [ ] Task 2: API client
"#;

    #[test]
    fn test_parse_sections() {
        let sections = parse_sections(EXAMPLE_BODY);
        let headings: Vec<&str> = sections.iter().map(|s| s.heading.as_str()).collect();
        // Should only have ## level sections
        assert!(headings.contains(&"What This Is"));
        assert!(headings.contains(&"Core Value"));
        assert!(headings.contains(&"Requirements"));
        assert!(headings.contains(&"Tasks"));
        // ### should NOT be separate sections
        assert!(!headings.contains(&"Validated"));
        assert!(!headings.contains(&"Active"));
        assert!(!headings.contains(&"Phase: BUILD"));
    }

    #[test]
    fn test_subsections_in_content() {
        let sections = parse_sections(EXAMPLE_BODY);
        let req = find_section(&sections, "Requirements").unwrap();
        // ### subsections should be in the content
        assert!(req.content.contains("### Validated"));
        assert!(req.content.contains("### Active"));
        assert!(req.content.contains("- ✓ Current weather"));
    }

    #[test]
    fn test_find_section() {
        let sections = parse_sections(EXAMPLE_BODY);
        let req = find_section(&sections, "Requirements").unwrap();
        assert!(req.content.contains("Validated"));
    }

    #[test]
    fn test_find_section_case_insensitive() {
        let sections = parse_sections(EXAMPLE_BODY);
        assert!(find_section(&sections, "core value").is_some());
        assert!(find_section(&sections, "CORE VALUE").is_some());
    }

    #[test]
    fn test_update_section() {
        let mut sections = parse_sections(EXAMPLE_BODY);
        let updated = update_section(&mut sections, "Core Value", "New value\n");
        assert!(updated);
        let sec = find_section(&sections, "Core Value").unwrap();
        assert_eq!(sec.content, "New value\n");
    }

    #[test]
    fn test_update_section_not_found() {
        let mut sections = parse_sections(EXAMPLE_BODY);
        assert!(!update_section(&mut sections, "Nonexistent", "content"));
    }

    #[test]
    fn test_parse_example_cli_tool() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let body = content
            .split_once("---\n")
            .and_then(|(_, rest)| rest.split_once("---\n"))
            .map(|(_, body)| body)
            .unwrap_or(&content);
        let sections = parse_sections(body);
        assert!(find_section(&sections, "What This Is").is_some());
        assert!(find_section(&sections, "Key Decisions").is_some());
        assert!(find_section(&sections, "Session Log").is_some());
        assert!(find_section(&sections, "Architecture").is_some());
        assert!(find_section(&sections, "Requirements").is_some());
        assert!(find_section(&sections, "Tasks").is_some());
        assert!(find_section(&sections, "Current State").is_some());

        // Verify subsections are inside their parent
        let req = find_section(&sections, "Requirements").unwrap();
        assert!(req.content.contains("### Validated"));
        assert!(req.content.contains("### Active"));
        assert!(req.content.contains("### Out of Scope"));

        let tasks = find_section(&sections, "Tasks").unwrap();
        assert!(tasks.content.contains("### Phase:"));
    }
}

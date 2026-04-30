//! YAML frontmatter parsing for project.md files.
//!
//! Extracts and parses the YAML frontmatter block between `---` delimiters.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Project lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectStatus {
    #[serde(rename = "define")]
    Define,
    #[serde(rename = "design")]
    Design,
    #[serde(rename = "build")]
    Build,
    #[serde(rename = "verify")]
    Verify,
    #[serde(rename = "ship")]
    Ship,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "archived")]
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Define => write!(f, "define"),
            ProjectStatus::Design => write!(f, "design"),
            ProjectStatus::Build => write!(f, "build"),
            ProjectStatus::Verify => write!(f, "verify"),
            ProjectStatus::Ship => write!(f, "ship"),
            ProjectStatus::Paused => write!(f, "paused"),
            ProjectStatus::Archived => write!(f, "archived"),
        }
    }
}

/// Parsed YAML frontmatter metadata from a project.md file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub project: String,
    pub status: ProjectStatus,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

/// Parse YAML frontmatter from a project.md string.
///
/// Returns `Some((Frontmatter, body))` if frontmatter delimiters are found,
/// or `None` if the content has no frontmatter block.
pub fn parse_frontmatter(content: &str) -> Result<Option<(Frontmatter, &str)>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Ok(None);
    }

    // Find the end of the opening ---
    let after_open = &trimmed[3..];
    // The opening --- must be on its own line (possibly followed by newline)
    if !after_open.starts_with('\n') && !after_open.starts_with('\r') {
        return Ok(None);
    }

    // Find the closing ---
    let body_start = after_open
        .find("\n---")
        .context("No closing --- found for frontmatter")?;

    let yaml_str = &after_open[..body_start];
    let rest = &after_open[body_start + 4..]; // skip \n---

    // rest may start with \n or \r\n
    let rest = rest.trim_start_matches('\n').trim_start_matches('\r');

    let fm: Frontmatter =
        serde_yaml::from_str(yaml_str).context("Failed to parse YAML frontmatter")?;
    Ok(Some((fm, rest)))
}

/// Serialize a Frontmatter back to a YAML string with `---` delimiters.
pub fn write_frontmatter(fm: &Frontmatter) -> Result<String> {
    let yaml = serde_yaml::to_string(fm).context("Failed to serialize frontmatter")?;
    Ok(format!("---\n{}---\n", yaml))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_FM: &str = r#"---
project: "Weather CLI"
status: build
created: 2026-04-25
updated: 2026-04-29
owner: "Alex Chen"
agent: "Hermes Agent"
tags: [cli, go, weather, devtools]
repository: "https://github.com/alexchen/weather-cli"
priority: medium
---

body here"#;

    #[test]
    fn test_parse_frontmatter() {
        let result = parse_frontmatter(EXAMPLE_FM).unwrap();
        assert!(result.is_some());
        let (fm, body) = result.unwrap();
        assert_eq!(fm.project, "Weather CLI");
        assert_eq!(fm.status, ProjectStatus::Build);
        assert_eq!(fm.created, NaiveDate::from_ymd_opt(2026, 4, 25).unwrap());
        assert_eq!(fm.updated, NaiveDate::from_ymd_opt(2026, 4, 29).unwrap());
        assert_eq!(fm.owner, "Alex Chen");
        assert_eq!(fm.agent.as_deref(), Some("Hermes Agent"));
        assert_eq!(fm.tags, vec!["cli", "go", "weather", "devtools"]);
        assert_eq!(
            fm.repository.as_deref(),
            Some("https://github.com/alexchen/weather-cli")
        );
        assert_eq!(fm.priority.as_deref(), Some("medium"));
        assert!(body.starts_with("body here"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a heading\n\nSome content.\n";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_write_frontmatter() {
        let fm = Frontmatter {
            project: "Test".to_string(),
            status: ProjectStatus::Define,
            created: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            owner: "Owner".to_string(),
            agent: None,
            tags: vec!["tag1".to_string()],
            repository: None,
            priority: None,
        };
        let output = write_frontmatter(&fm).unwrap();
        assert!(output.starts_with("---\n"));
        assert!(output.ends_with("---\n"));
        assert!(output.contains("project: Test"));
    }

    #[test]
    fn test_roundtrip() {
        let (fm, _) = parse_frontmatter(EXAMPLE_FM).unwrap().unwrap();
        let serialized = write_frontmatter(&fm).unwrap();
        let (fm2, _) = parse_frontmatter(&serialized).unwrap().unwrap();
        assert_eq!(fm.project, fm2.project);
        assert_eq!(fm.status, fm2.status);
        assert_eq!(fm.created, fm2.created);
    }

    #[test]
    fn test_parse_example_cli_tool() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let (fm, body) = parse_frontmatter(&content).unwrap().unwrap();
        assert_eq!(fm.project, "Weather CLI");
        assert_eq!(fm.status, ProjectStatus::Build);
        assert!(body.contains("## What This Is"));
    }
}

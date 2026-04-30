//! Project struct — the top-level representation of a project.md file.
//!
//! Holds parsed frontmatter and sections, with load/save operations.

use crate::frontmatter::{parse_frontmatter, write_frontmatter, Frontmatter};
use crate::sections::{find_section, parse_sections, Section};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// A fully parsed project.md file.
#[derive(Debug)]
pub struct Project {
    pub frontmatter: Frontmatter,
    pub sections: Vec<Section>,
    pub raw: String,
    pub path: PathBuf,
}

impl Project {
    /// Load and parse a project.md file from disk.
    pub fn load(path: &Path) -> Result<Project> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read project file: {}", path.display()))?;
        Self::from_str(&raw, path)
    }

    /// Parse a project.md from a string, storing the given path.
    pub fn from_str(raw: &str, path: &Path) -> Result<Project> {
        let (frontmatter, body) = parse_frontmatter(raw)?
            .context("No frontmatter found — not a valid project.md file")?;

        let sections = parse_sections(body);

        Ok(Project {
            frontmatter,
            sections,
            raw: raw.to_string(),
            path: path.to_path_buf(),
        })
    }

    /// Save the project back to its file, reconstructing the full content.
    pub fn save(&self) -> Result<()> {
        let content = self.to_string();
        fs::write(&self.path, &content)
            .with_context(|| format!("Failed to write project file: {}", self.path.display()))?;
        Ok(())
    }

    /// Get a reference to a section by heading name.
    pub fn get_section(&self, name: &str) -> Option<&Section> {
        find_section(&self.sections, name)
    }

    /// Update a section's content by heading name.
    pub fn update_section(&mut self, name: &str, content: &str) {
        crate::sections::update_section(&mut self.sections, name, content);
    }

    /// Update a specific frontmatter field.
    ///
    /// Supported fields: project, status, owner, agent, tags, repository, priority, updated.
    pub fn update_frontmatter_field(&mut self, field: &str, value: &str) {
        use chrono::NaiveDate;

        match field {
            "project" => self.frontmatter.project = value.to_string(),
            "status" => {
                if let Ok(status) = serde_yaml::from_str(value) {
                    self.frontmatter.status = status;
                }
            }
            "owner" => self.frontmatter.owner = value.to_string(),
            "agent" => {
                self.frontmatter.agent = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "tags" => {
                self.frontmatter.tags = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "repository" => {
                self.frontmatter.repository = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "priority" => {
                self.frontmatter.priority = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "updated" => {
                if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    self.frontmatter.updated = date;
                }
            }
            _ => {} // Unknown field, ignore
        }
    }

    /// Reconstruct the full project.md content as a string.
    pub fn to_content(&self) -> String {
        let mut out = String::new();
        out.push_str(&write_frontmatter(&self.frontmatter).unwrap_or_default());
        out.push('\n');
        for section in &self.sections {
            let hashes = "#".repeat(section.level as usize);
            if !section.heading.is_empty() {
                out.push_str(&format!("{} {}\n\n", hashes, section.heading));
            }
            out.push_str(&section.content);
            if !section.content.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_content())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    const EXAMPLE: &str = include_str!(concat!(
        env!("HOME"),
        "/projectsdotmd/example-cli-tool.md"
    ));

    #[test]
    fn test_load_from_str() {
        let path = Path::new("/tmp/test.md");
        let project = Project::from_str(EXAMPLE, path).unwrap();
        assert_eq!(project.frontmatter.project, "Weather CLI");
        assert!(!project.sections.is_empty());
        assert!(project.get_section("What This Is").is_some());
        assert!(project.get_section("Tasks").is_some());
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("md");
        std::fs::write(&path, EXAMPLE).unwrap();

        let mut project = Project::load(&path).unwrap();
        project.update_section("Core Value", "Updated core value\n");
        project.save().unwrap();

        let reloaded = Project::load(&path).unwrap();
        let sec = reloaded.get_section("Core Value").unwrap();
        assert!(sec.content.contains("Updated core value"));
    }

    #[test]
    fn test_update_frontmatter_field() {
        let path = Path::new("/tmp/test.md");
        let mut project = Project::from_str(EXAMPLE, path).unwrap();
        project.update_frontmatter_field("project", "New Name");
        assert_eq!(project.frontmatter.project, "New Name");

        project.update_frontmatter_field("priority", "high");
        assert_eq!(project.frontmatter.priority.as_deref(), Some("high"));

        project.update_frontmatter_field("priority", "");
        assert!(project.frontmatter.priority.is_none());
    }

    #[test]
    fn test_to_content_roundtrip() {
        let path = Path::new("/tmp/test.md");
        let project = Project::from_str(EXAMPLE, path).unwrap();
        let content = project.to_content();
        // Verify we can re-parse the output
        let project2 = Project::from_str(&content, path).unwrap();
        assert_eq!(project.frontmatter.project, project2.frontmatter.project);
        assert_eq!(project.sections.len(), project2.sections.len());
    }
}

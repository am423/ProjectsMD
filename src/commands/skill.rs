//! Skill command — manage agent skills for project.md.
//!
//! Provides install, view, and generate subcommands for the projectsmd agent skill.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::skill;

/// Install the projectsmd skill to a framework skill directory.
pub fn install(
    framework: Option<&str>,
    path: Option<&str>,
    force: bool,
    quiet: bool,
) -> Result<()> {
    let target = skill::install(framework, path, force)?;

    if !quiet {
        println!(
            "{} Skill installed to {}",
            "✓".green().bold(),
            target.display()
        );
    }

    Ok(())
}

/// Print the embedded SKILL.md to stdout.
pub fn view() {
    skill::view();
}

/// Generate a project-specific skill from the current project.md.
pub fn generate(path: &Path, quiet: bool) -> Result<()> {
    let project = crate::project::Project::load(path)
        .context("Failed to load project file")?;

    let target = skill::generate(&project)?;

    if !quiet {
        println!(
            "{} Project skill generated at {}",
            "✓".green().bold(),
            target.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROJECT: &str = r#"---
project: "Test Project"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Test Owner"
tags: [test]
---

## What This Is

A test project.

## Tasks

### Phase: BUILD

- [x] Task 1: Setup
- [ ] Task 2: Implementation

## Session Log

- **2026-01-01** — Project started.
"#;

    #[test]
    fn test_install_to_path() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().to_string_lossy().to_string();

        let result = install(None, Some(&target), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_force() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().to_string_lossy().to_string();

        install(None, Some(&target), false, true).unwrap();
        let result = install(None, Some(&target), false, true);
        assert!(result.is_err());

        let result = install(None, Some(&target), true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let project_path = tmp_dir.path().join("project.md");
        std::fs::write(&project_path, TEST_PROJECT).unwrap();

        let result = generate(&project_path, true);
        assert!(result.is_ok());

        let skill_path = tmp_dir
            .path()
            .join(".skills")
            .join("projectsmd-project")
            .join("SKILL.md");
        assert!(skill_path.exists());

        let content = std::fs::read_to_string(&skill_path).unwrap();
        assert!(content.contains("Test Project"));
    }

    #[test]
    fn test_view_outputs_content() {
        // Just verify view() doesn't panic
        view();
    }
}

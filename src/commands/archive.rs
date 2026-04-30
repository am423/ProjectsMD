//! Archive command — archive a completed project.
//!
//! Sets status to "archived", adds a final Session Log entry,
//! and updates the frontmatter updated date.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::frontmatter::ProjectStatus;
use crate::project::Project;
use crate::session_log::append_session_log;

/// Run the archive command.
///
/// Sets the project status to "archived" and adds a final session log entry
/// with the provided summary.
pub fn run(path: &Path, summary: Option<&str>, quiet: bool) -> Result<()> {
    let mut project = Project::load(path).context("Failed to load project file")?;

    // Check if already archived
    if project.frontmatter.status == ProjectStatus::Archived {
        if !quiet {
            println!("{} Project is already archived.", "⚠".yellow());
        }
        return Ok(());
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Add final session log entry
    let summary_text = summary.unwrap_or("Project archived.");
    let entry = format!("**{}** — ARCHIVED — {}", today, summary_text);

    if let Some(log_section) = project.get_section("Session Log") {
        let new_content = append_session_log(&log_section.content, &entry);
        project.update_section("Session Log", &new_content);
    } else {
        project.update_section("Session Log", &format!("\n- {}\n", entry));
    }

    // Update status and date
    project.update_frontmatter_field("status", "archived");
    project.update_frontmatter_field("updated", &today);

    // Update Current State if it exists
    if let Some(state_section) = project.get_section("Current State") {
        let mut state = crate::state::parse_state(&state_section.content);
        state.phase = "archived".to_string();
        state.in_progress = String::new();
        state.next_action = String::new();
        state.notes = format!(
            "Archived on {}. {}",
            today,
            if summary.is_some() { summary_text } else { "" }
        )
        .trim()
        .to_string();
        let new_state_content = crate::state::write_state(&state);
        project.update_section("Current State", &new_state_content);
    }

    project.save()?;

    if !quiet {
        println!(
            "{} Project archived.",
            "✓".green().bold()
        );
        if let Some(s) = summary {
            println!("  Summary: {}", s);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROJECT: &str = r#"---
project: "Test Project"
status: ship
created: 2026-01-01
updated: 2026-04-29
owner: "Test Owner"
tags: [test]
---

## What This Is

A test project.

## Current State

**Phase:** ship
**Last completed:** Task 3
**In progress:** 
**Next action:** 
**Blockers:** None
**Notes:** 

## Tasks

### Phase: SHIP

- [x] Task 1: Setup
- [x] Task 2: Implementation
- [x] Task 3: Testing

## Session Log

- **2026-01-01** — Project started.
"#;

    fn write_test_project() -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT).unwrap();
        tmp
    }

    #[test]
    fn test_archive_sets_status() {
        let tmp = write_test_project();
        let result = run(tmp.path(), Some("All done!"), true);
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        assert_eq!(project.frontmatter.status, ProjectStatus::Archived);
    }

    #[test]
    fn test_archive_adds_session_log() {
        let tmp = write_test_project();
        run(tmp.path(), Some("All features shipped."), true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("ARCHIVED"));
        assert!(log_section.content.contains("All features shipped."));
    }

    #[test]
    fn test_archive_updates_date() {
        let tmp = write_test_project();
        run(tmp.path(), None, true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(project.frontmatter.updated.to_string(), today);
    }

    #[test]
    fn test_archive_updates_state() {
        let tmp = write_test_project();
        run(tmp.path(), Some("Done."), true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let state_section = project.get_section("Current State").unwrap();
        let state = crate::state::parse_state(&state_section.content);
        assert_eq!(state.phase, "archived");
    }

    #[test]
    fn test_archive_default_summary() {
        let tmp = write_test_project();
        run(tmp.path(), None, true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("Project archived."));
    }

    #[test]
    fn test_already_archived() {
        let tmp = write_test_project();
        // First archive
        run(tmp.path(), Some("First archive"), true).unwrap();
        // Second archive should be a no-op
        let result = run(tmp.path(), Some("Second archive"), true);
        assert!(result.is_ok());

        // Should still have first archive message
        let project = Project::load(tmp.path()).unwrap();
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("First archive"));
    }
}

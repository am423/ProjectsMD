//! Phase command — manage project phases.
//!
//! Handles phase transitions with an evolution checklist:
//! 1. Count completed tasks
//! 2. Check if "What This Is" is still accurate
//! 3. Update status in frontmatter

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::frontmatter::ProjectStatus;
use crate::project::Project;
use crate::tasks::{parse_all_tasks, TaskStatus};

/// Run the phase transition command.
///
/// If `new_status` is provided, transitions to that status.
/// If not provided, runs an interactive evolution checklist.
pub fn run(path: &Path, new_status: Option<&str>, quiet: bool) -> Result<()> {
    let mut project = Project::load(path).context("Failed to load project file")?;

    let target_status = if let Some(status_str) = new_status {
        parse_status(status_str)?
    } else {
        // Interactive mode: suggest next phase
        suggest_next_phase(&project)?
    };

    // Run evolution checklist
    if !quiet {
        println!(
            "{}",
            "── Evolution Checklist ──".bright_blue().bold()
        );
        println!();
    }

    // Step 1: Count completed tasks
    let all_tasks = parse_all_tasks(&project.sections);
    let total = all_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .count();
    let done = all_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    let pending = all_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .filter(|t| t.status == TaskStatus::Pending)
        .count();
    let blocked = all_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .filter(|t| t.status == TaskStatus::Blocked)
        .count();

    if !quiet {
        println!(
            "  {} Tasks: {} done / {} pending / {} blocked / {} total",
            "📊".to_string(),
            done.to_string().green(),
            pending.to_string().yellow(),
            blocked.to_string().red(),
            total
        );
        if pending > 0 {
            println!(
                "  {} There are still {} pending tasks in the current phase.",
                "⚠".yellow(),
                pending
            );
        }
        println!();
    }

    // Step 2: Check "What This Is" section
    if !quiet {
        if let Some(wti_section) = project.get_section("What This Is") {
            let content = wti_section.content.trim();
            if !content.is_empty() {
                println!("  {} Current \"What This Is\":", "📋".to_string());
                for line in content.lines() {
                    println!("    {}", line);
                }
                println!();
                println!(
                    "  {} Is this still accurate?",
                    "❓".to_string()
                );
                println!("    (Press Enter to confirm, or update via `projectsmd view --section \"What This Is\"`)");
                println!();
            }
        }
    }

    // Step 3: Update status
    let old_status = project.frontmatter.status.clone();
    project.update_frontmatter_field("status", &target_status.to_string());
    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );

    // Update Current State phase if it exists
    if let Some(state_section) = project.get_section("Current State") {
        let mut state = crate::state::parse_state(&state_section.content);
        state.phase = target_status.to_string();
        let new_state_content = crate::state::write_state(&state);
        project.update_section("Current State", &new_state_content);
    }

    project.save()?;

    if !quiet {
        println!(
            "{} Phase transitioned: {} → {}",
            "✓".green().bold(),
            old_status.to_string().dimmed(),
            target_status.to_string().bright_cyan()
        );
    }

    Ok(())
}

/// Parse a status string into a ProjectStatus.
fn parse_status(s: &str) -> Result<ProjectStatus> {
    match s.to_lowercase().as_str() {
        "define" => Ok(ProjectStatus::Define),
        "design" => Ok(ProjectStatus::Design),
        "build" => Ok(ProjectStatus::Build),
        "verify" => Ok(ProjectStatus::Verify),
        "ship" => Ok(ProjectStatus::Ship),
        "paused" => Ok(ProjectStatus::Paused),
        "archived" => Ok(ProjectStatus::Archived),
        _ => bail!(
            "Unknown status '{}'. Valid statuses: define, design, build, verify, ship, paused, archived",
            s
        ),
    }
}

/// Suggest the next phase based on the current status.
fn suggest_next_phase(project: &Project) -> Result<ProjectStatus> {
    let current = &project.frontmatter.status;
    let next = match current {
        ProjectStatus::Define => ProjectStatus::Design,
        ProjectStatus::Design => ProjectStatus::Build,
        ProjectStatus::Build => ProjectStatus::Verify,
        ProjectStatus::Verify => ProjectStatus::Ship,
        ProjectStatus::Ship => {
            bail!("Project is already in 'ship' phase. Use 'archive' to archive it.");
        }
        ProjectStatus::Paused => {
            bail!("Project is paused. Specify a target phase: --transition <status>");
        }
        ProjectStatus::Archived => {
            bail!("Project is archived. Unarchive it first.");
        }
    };
    Ok(next)
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

## Current State

**Phase:** build
**Last completed:** Task 1
**In progress:** Task 2
**Next action:** Implement feature B
**Blockers:** None
**Notes:** Good progress

## Tasks

### Phase: BUILD

- [x] Task 1: Setup
- [ ] Task 2: Implementation
- [ ] Task 3: Testing

## Session Log

- **2026-01-01** — Project started.
"#;

    fn write_test_project() -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT).unwrap();
        tmp
    }

    #[test]
    fn test_parse_status_valid() {
        assert_eq!(parse_status("define").unwrap(), ProjectStatus::Define);
        assert_eq!(parse_status("BUILD").unwrap(), ProjectStatus::Build);
        assert_eq!(parse_status("Verify").unwrap(), ProjectStatus::Verify);
        assert_eq!(parse_status("ship").unwrap(), ProjectStatus::Ship);
        assert_eq!(parse_status("paused").unwrap(), ProjectStatus::Paused);
        assert_eq!(parse_status("archived").unwrap(), ProjectStatus::Archived);
    }

    #[test]
    fn test_parse_status_invalid() {
        assert!(parse_status("invalid").is_err());
        assert!(parse_status("testing").is_err());
    }

    #[test]
    fn test_phase_transition_explicit() {
        let tmp = write_test_project();
        let result = run(tmp.path(), Some("verify"), true);
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        assert_eq!(project.frontmatter.status, ProjectStatus::Verify);
    }

    #[test]
    fn test_phase_transition_updates_state() {
        let tmp = write_test_project();
        run(tmp.path(), Some("verify"), true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let state_section = project.get_section("Current State").unwrap();
        let state = crate::state::parse_state(&state_section.content);
        assert_eq!(state.phase, "verify");
    }

    #[test]
    fn test_phase_transition_updates_date() {
        let tmp = write_test_project();
        run(tmp.path(), Some("verify"), true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(project.frontmatter.updated.to_string(), today);
    }

    #[test]
    fn test_suggest_next_phase() {
        let tmp = write_test_project();
        let project = Project::load(tmp.path()).unwrap();
        let next = suggest_next_phase(&project).unwrap();
        assert_eq!(next, ProjectStatus::Verify);
    }

    #[test]
    fn test_suggest_next_from_define() {
        let tmp = write_test_project();
        let mut project = Project::load(tmp.path()).unwrap();
        project.frontmatter.status = ProjectStatus::Define;
        let next = suggest_next_phase(&project).unwrap();
        assert_eq!(next, ProjectStatus::Design);
    }

    #[test]
    fn test_suggest_next_from_ship_fails() {
        let tmp = write_test_project();
        let mut project = Project::load(tmp.path()).unwrap();
        project.frontmatter.status = ProjectStatus::Ship;
        let result = suggest_next_phase(&project);
        assert!(result.is_err());
    }

    #[test]
    fn test_suggest_next_from_paused_fails() {
        let tmp = write_test_project();
        let mut project = Project::load(tmp.path()).unwrap();
        project.frontmatter.status = ProjectStatus::Paused;
        let result = suggest_next_phase(&project);
        assert!(result.is_err());
    }
}

//! Session command — interactive session wrap-up wizard.
//!
//! Guides the user through confirming completed tasks, recording decisions,
//! and writing a session summary. Updates the project.md accordingly.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::decisions::{add_decision, parse_decisions, write_decisions, Outcome};
use crate::project::Project;
use crate::session_log::append_session_log;
use crate::state::{parse_state, write_state};
use crate::tasks::{parse_all_tasks, Task, TaskStatus};

/// Run the session wrap-up command.
///
/// In interactive mode, uses dialoguer to guide the user through:
/// 1. Confirming completed tasks
/// 2. Recording decisions
/// 3. Writing a session summary
///
/// In non-interactive mode, accepts a summary via CLI arg.
pub fn run(path: &Path, non_interactive: bool, summary: Option<&str>, quiet: bool) -> Result<()> {
    let mut project = Project::load(path).context("Failed to load project file")?;

    if non_interactive {
        return run_noninteractive(&mut project, summary, quiet);
    }

    run_interactive(&mut project, quiet)
}

/// Non-interactive session: just record the summary and update the date.
fn run_noninteractive(project: &mut Project, summary: Option<&str>, quiet: bool) -> Result<()> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    if let Some(summary_text) = summary {
        let entry = format!("**{}** — {}", today, summary_text);
        if let Some(log_section) = project.get_section("Session Log") {
            let new_content = append_session_log(&log_section.content, &entry);
            project.update_section("Session Log", &new_content);
        } else {
            project.update_section("Session Log", &format!("\n- {}\n", entry));
        }
    }

    project.update_frontmatter_field("updated", &today);
    project.save()?;

    if !quiet {
        println!("{} Session recorded.", "✓".green());
    }
    Ok(())
}

/// Interactive session wrap-up wizard using dialoguer.
fn run_interactive(project: &mut Project, quiet: bool) -> Result<()> {
    use dialoguer::{Confirm, Input, MultiSelect};

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Step 1: Show completed tasks, ask user to confirm which ones are done
    let all_tasks = parse_all_tasks(&project.sections);
    let pending_tasks: Vec<(String, &Task)> = all_tasks
        .iter()
        .flat_map(|(phase, tasks)| {
            tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .map(move |t| (phase.clone(), t))
        })
        .collect();

    let completed_task_numbers: Vec<u32> = if !pending_tasks.is_empty() {
        println!(
            "{}",
            "── Session Wrap-up ──".bright_blue().bold()
        );
        println!();
        println!(
            "{}",
            "Which tasks did you complete this session?".bright_white().bold()
        );
        println!();

        let items: Vec<String> = pending_tasks
            .iter()
            .map(|(phase, task)| {
                let num = task
                    .number
                    .map(|n| format!("Task {}: ", n))
                    .unwrap_or_default();
                format!("[{}] {}{}", phase.dimmed(), num, task.description)
            })
            .collect();

        let selections = MultiSelect::new()
            .items(&items)
            .defaults(&vec![false; items.len()])
            .interact()
            .context("Failed to read selection")?;

        selections
            .iter()
            .map(|&idx| pending_tasks[idx].1.number.unwrap_or(0))
            .filter(|&n| n > 0)
            .collect()
    } else {
        if !quiet {
            println!("No pending tasks found.");
        }
        vec![]
    };

    // Mark selected tasks as done
    for &task_num in &completed_task_numbers {
        mark_task_done(project, task_num);
    }

    // Step 2: Ask if any decisions were made
    let has_decisions = Confirm::new()
        .with_prompt("Were any decisions made this session?")
        .default(false)
        .interact()
        .context("Failed to read confirmation")?;

    let mut decisions_text = Vec::new();
    if has_decisions {
        loop {
            let decision: String = Input::new()
                .with_prompt("Decision (empty to stop)")
                .allow_empty(true)
                .interact_text()
                .context("Failed to read decision")?;

            if decision.trim().is_empty() {
                break;
            }

            let rationale: String = Input::new()
                .with_prompt("Rationale")
                .allow_empty(true)
                .interact_text()
                .context("Failed to read rationale")?;

            decisions_text.push((decision, rationale));
        }
    }

    // Add decisions to the project
    if !decisions_text.is_empty() {
        if let Some(dec_section) = project.get_section("Key Decisions") {
            let mut decisions = parse_decisions(&dec_section.content);
            for (decision, rationale) in &decisions_text {
                add_decision(&mut decisions, decision, rationale, Outcome::Pending);
            }
            let new_content = write_decisions(&decisions);
            project.update_section("Key Decisions", &new_content);
        } else {
            let mut decisions = Vec::new();
            for (decision, rationale) in &decisions_text {
                add_decision(&mut decisions, decision, rationale, Outcome::Pending);
            }
            let new_content = write_decisions(&decisions);
            project.update_section("Key Decisions", &format!("\n{}", new_content));
        }
    }

    // Step 3: Ask for session summary
    let summary: String = Input::new()
        .with_prompt("Session summary (1-2 sentences)")
        .interact_text()
        .context("Failed to read summary")?;

    // Step 4: Update Current State
    update_state_from_tasks(project, &completed_task_numbers);

    // Step 5: Append Session Log
    let entry = if summary.trim().is_empty() {
        format!("**{}** — Session completed.", today)
    } else {
        format!("**{}** — {}", today, summary)
    };

    if let Some(log_section) = project.get_section("Session Log") {
        let new_content = append_session_log(&log_section.content, &entry);
        project.update_section("Session Log", &new_content);
    } else {
        project.update_section("Session Log", &format!("\n- {}\n", entry));
    }

    // Step 6: Update frontmatter updated date
    project.update_frontmatter_field("updated", &today);

    // Save
    project.save()?;

    if !quiet {
        println!();
        println!(
            "{}",
            "── Session Summary ──".bright_blue().bold()
        );
        if !completed_task_numbers.is_empty() {
            println!(
                "  {} Completed {} task(s): {:?}",
                "✓".green(),
                completed_task_numbers.len(),
                completed_task_numbers
            );
        }
        if !decisions_text.is_empty() {
            println!(
                "  {} Recorded {} decision(s)",
                "✓".green(),
                decisions_text.len()
            );
        }
        if !summary.trim().is_empty() {
            println!("  {} Summary: {}", "✓".green(), summary);
        }
        println!();
        println!("{} Project updated.", "✓".green().bold());
    }

    Ok(())
}

/// Mark a task as done in the project's Tasks section.
fn mark_task_done(project: &mut Project, task_num: u32) {
    if let Some(tasks_section) = project.get_section("Tasks") {
        let content = tasks_section.content.clone();
        let mut new_content = content.clone();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("- [ ] Task {}: ", task_num))
                || trimmed.starts_with(&format!("- [!] Task {}: ", task_num))
            {
                let new_line = trimmed
                    .replacen("- [ ]", "- [x]", 1)
                    .replacen("- [!]", "- [x]", 1);
                new_content = new_content.replacen(trimmed, &new_line, 1);
                break;
            }
        }
        project.update_section("Tasks", &new_content);
    }
}

/// Update Current State based on completed tasks.
fn update_state_from_tasks(project: &mut Project, completed: &[u32]) {
    if completed.is_empty() {
        return;
    }

    if let Some(state_section) = project.get_section("Current State") {
        let mut state = parse_state(&state_section.content);
        let last_completed = completed
            .iter()
            .map(|n| format!("Task {}", n))
            .collect::<Vec<_>>()
            .join(", ");
        state.last_completed = last_completed;
        let new_content = write_state(&state);
        project.update_section("Current State", &new_content);
    }
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
agent: "Test Agent"
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

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Rust | Fast and safe | ✓ Good |

## Session Log

- **2026-01-01** — Project started.
"#;

    fn write_test_project() -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT).unwrap();
        tmp
    }

    #[test]
    fn test_noninteractive_session() {
        let tmp = write_test_project();
        let result = run(tmp.path(), true, Some("Test session summary"), true);
        assert!(result.is_ok());

        // Verify the session log was updated
        let project = Project::load(tmp.path()).unwrap();
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("Test session summary"));
    }

    #[test]
    fn test_noninteractive_no_summary() {
        let tmp = write_test_project();
        let result = run(tmp.path(), true, None, true);
        assert!(result.is_ok());

        // Verify updated date was changed
        let project = Project::load(tmp.path()).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(project.frontmatter.updated.to_string(), today);
    }

    #[test]
    fn test_mark_task_done() {
        let tmp = write_test_project();
        let mut project = Project::load(tmp.path()).unwrap();

        mark_task_done(&mut project, 2);
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("- [x] Task 2: Implementation"));
    }

    #[test]
    fn test_update_state_from_tasks() {
        let tmp = write_test_project();
        let mut project = Project::load(tmp.path()).unwrap();

        update_state_from_tasks(&mut project, &[2, 3]);
        let state_section = project.get_section("Current State").unwrap();
        assert!(state_section.content.contains("Task 2, Task 3"));
    }

    #[test]
    fn test_noninteractive_updates_date() {
        let tmp = write_test_project();
        run(tmp.path(), true, Some("summary"), true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(project.frontmatter.updated.to_string(), today);
    }
}

//! Task command — manage tasks in project.md.
//!
//! Provides task add functionality. Other task operations (list, done, block, unblock)
//! are handled inline in main.rs.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::project::Project;
use crate::tasks::parse_all_tasks;

/// Add a new task to a phase section in the Tasks section.
///
/// If the phase subsection doesn't exist, it is created.
/// Task numbers are auto-incremented based on the highest existing number.
pub fn add(path: &Path, description: &str, phase: &str, quiet: bool) -> Result<()> {
    let mut project = Project::load(path).context("Failed to load project file")?;

    let phase_upper = phase.to_uppercase();

    // Find the next task number
    let all_tasks = parse_all_tasks(&project.sections);
    let max_number = all_tasks
        .iter()
        .flat_map(|(_, tasks)| tasks.iter())
        .filter_map(|t| t.number)
        .max()
        .unwrap_or(0);

    let new_task_number = max_number + 1;
    let new_task_line = format!("- [ ] Task {}: {}", new_task_number, description);

    // Get or create the Tasks section
    if let Some(tasks_section) = project.get_section("Tasks") {
        let content = tasks_section.content.clone();

        // Look for the phase subsection
        let phase_header = format!("### Phase: {}", phase_upper);
        if content.contains(&phase_header) {
            // Find the phase subsection and append the task
            let new_content = append_task_to_phase(&content, &phase_upper, &new_task_line);
            project.update_section("Tasks", &new_content);
        } else {
            // Create a new phase subsection
            let mut new_content = content.clone();
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push('\n');
            new_content.push_str(&format!("### Phase: {}\n\n", phase_upper));
            new_content.push_str(&new_task_line);
            new_content.push('\n');
            project.update_section("Tasks", &new_content);
        }
    } else {
        // Create the Tasks section from scratch
        let new_content = format!(
            "\n### Phase: {}\n\n{}\n",
            phase_upper, new_task_line
        );
        project.update_section("Tasks", &new_content);
    }

    // Update frontmatter
    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );

    project.save()?;

    if !quiet {
        println!(
            "{} Task {}: {} (phase: {})",
            "✓".green(),
            new_task_number,
            description,
            phase_upper
        );
    }

    Ok(())
}

/// Append a task line to the end of a specific phase subsection.
fn append_task_to_phase(content: &str, phase: &str, task_line: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_target_phase = false;
    let mut insert_after: Option<usize> = None;

    let phase_header = format!("### Phase: {}", phase);

    for (i, line) in content.lines().enumerate() {
        lines.push(line.to_string());

        if line.trim() == phase_header {
            in_target_phase = true;
            continue;
        }

        if in_target_phase {
            // If we hit another phase header or end, insert before it
            if line.trim().starts_with("### Phase:") || line.trim().starts_with("## ") {
                insert_after = Some(i - 1);
                in_target_phase = false;
            }
        }
    }

    // If we were still in the target phase at the end, insert at the end
    if in_target_phase {
        insert_after = Some(lines.len() - 1);
    }

    if let Some(pos) = insert_after {
        // Find the last non-empty line before the insert point
        let mut insert_pos = pos;
        while insert_pos > 0 && lines[insert_pos - 1].trim().is_empty() {
            insert_pos -= 1;
        }
        lines.insert(insert_pos + 1, String::new());
        lines.insert(insert_pos + 2, task_line.to_string());
    } else {
        lines.push(String::new());
        lines.push(task_line.to_string());
    }

    lines.join("\n")
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

### Phase: VERIFY

- [ ] Task 3: Testing
"#;

    fn write_test_project() -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT).unwrap();
        tmp
    }

    #[test]
    fn test_add_task_to_existing_phase() {
        let tmp = write_test_project();
        let result = add(tmp.path(), "New feature", "build", true);
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("Task 4: New feature"));
    }

    #[test]
    fn test_add_task_to_new_phase() {
        let tmp = write_test_project();
        let result = add(tmp.path(), "Documentation", "ship", true);
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("### Phase: SHIP"));
        assert!(tasks_section.content.contains("Task 4: Documentation"));
    }

    #[test]
    fn test_add_task_auto_number() {
        let tmp = write_test_project();
        add(tmp.path(), "First new", "build", true).unwrap();
        add(tmp.path(), "Second new", "build", true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("Task 4: First new"));
        assert!(tasks_section.content.contains("Task 5: Second new"));
    }

    #[test]
    fn test_add_task_updates_date() {
        let tmp = write_test_project();
        add(tmp.path(), "Test task", "build", true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(project.frontmatter.updated.to_string(), today);
    }

    #[test]
    fn test_append_task_to_phase() {
        let content = "### Phase: BUILD\n\n- [x] Task 1: Setup\n- [ ] Task 2: Impl\n\n### Phase: VERIFY\n\n- [ ] Task 3: Test\n";
        let result = append_task_to_phase(content, "BUILD", "- [ ] Task 4: New");
        assert!(result.contains("Task 4: New"));
        // Should be inserted in BUILD phase, not VERIFY
        let build_pos = result.find("### Phase: BUILD").unwrap();
        let new_pos = result.find("Task 4: New").unwrap();
        let verify_pos = result.find("### Phase: VERIFY").unwrap();
        assert!(build_pos < new_pos);
        assert!(new_pos < verify_pos);
    }

    #[test]
    fn test_append_task_to_phase_at_end() {
        let content = "### Phase: BUILD\n\n- [x] Task 1: Setup\n";
        let result = append_task_to_phase(content, "BUILD", "- [ ] Task 2: New");
        assert!(result.contains("Task 2: New"));
    }

    #[test]
    fn test_add_preserves_existing_tasks() {
        let tmp = write_test_project();
        add(tmp.path(), "New task", "build", true).unwrap();

        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        // Original tasks should still be there
        assert!(tasks_section.content.contains("Task 1: Setup"));
        assert!(tasks_section.content.contains("Task 2: Implementation"));
        assert!(tasks_section.content.contains("Task 3: Testing"));
    }
}

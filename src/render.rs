//! Terminal rendering for project.md content.
//!
//! Renders project.md content with syntax highlighting and formatting for the terminal.

use crate::decisions::{Decision, Outcome};
use crate::project::Project;
use self::render_helpers::*;
use crate::requirements::Requirements;
use crate::state::CurrentState;
use crate::tasks::Task;
use crate::validate::ValidationResult;
use colored::Colorize;

/// Render the full project document with colors.
pub fn render_project(project: &Project) -> String {
    let mut out = String::new();

    // Frontmatter
    out.push_str(&format!(
        "{}\n",
        "── Frontmatter ──".bright_blue().bold()
    ));
    out.push_str(&format!(
        "  {}: {}\n",
        "Project".bright_white().bold(),
        project.frontmatter.project
    ));
    out.push_str(&format!(
        "  {}: {}\n",
        "Status".bright_white().bold(),
        status_color(&project.frontmatter.status.to_string())
    ));
    out.push_str(&format!(
        "  {}: {}\n",
        "Owner".bright_white().bold(),
        project.frontmatter.owner
    ));
    if let Some(ref agent) = project.frontmatter.agent {
        out.push_str(&format!(
            "  {}: {}\n",
            "Agent".bright_white().bold(),
            agent
        ));
    }
    out.push_str(&format!(
        "  {}: {} → {}\n",
        "Dates".bright_white().bold(),
        project.frontmatter.created,
        project.frontmatter.updated
    ));
    if !project.frontmatter.tags.is_empty() {
        out.push_str(&format!(
            "  {}: {}\n",
            "Tags".bright_white().bold(),
            project.frontmatter.tags.join(", ")
        ));
    }
    out.push('\n');

    // Sections
    for section in &project.sections {
        let hashes = "#".repeat(section.level as usize);
        out.push_str(&format!(
            "{} {} {}\n\n",
            hashes.bright_blue(),
            section.heading.bright_white().bold(),
            "─"
                .repeat(40_usize.saturating_sub(section.heading.len()))
                .dimmed()
        ));

        // Color-code content based on section type
        let content = &section.content;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [x]") {
                out.push_str(&format!("  {}\n", line.green()));
            } else if trimmed.starts_with("- [!]") {
                out.push_str(&format!("  {}\n", line.red()));
            } else if trimmed.starts_with("- [ ]") {
                out.push_str(&format!("  {}\n", line.yellow()));
            } else if trimmed.starts_with("- ✓") {
                out.push_str(&format!("  {}\n", line.green()));
            } else if trimmed.starts_with("**") && trimmed.contains(":**") {
                out.push_str(&format!("  {}\n", line.bright_white()));
            } else if trimmed.starts_with("|") && !trimmed.starts_with("|---") {
                out.push_str(&format!("  {}\n", line.cyan()));
            } else if trimmed.starts_with("|---") {
                out.push_str(&format!("  {}\n", line.dimmed()));
            } else {
                out.push_str(&format!("  {}\n", line));
            }
        }
        out.push('\n');
    }

    out
}

/// Render a status dashboard box with progress.
pub fn render_status(project: &Project) -> String {
    let mut out = String::new();

    let box_width = 60;
    let border = "═".repeat(box_width - 2);

    out.push_str(&format!("╔{}╗\n", border));
    out.push_str(&format!(
        "║ {:<width$} ║\n",
        format!(
            "{} {}",
            "📊".to_string(),
            project.frontmatter.project.bright_white().bold()
        ),
        width = box_width - 4
    ));
    out.push_str(&format!("╠{}╣\n", border));

    // Status line
    out.push_str(&format!(
        "║ {:<width$} ║\n",
        format!(
            "Status: {}",
            status_color(&project.frontmatter.status.to_string())
        ),
        width = box_width - 4
    ));

    // State info
    if let Some(state_section) = project.get_section("Current State") {
        use crate::state::parse_state;
        let state = parse_state(&state_section.content);

        if !state.phase.is_empty() {
            out.push_str(&format!(
                "║ {:<width$} ║\n",
                format!("Phase:  {}", state.phase.bright_cyan()),
                width = box_width - 4
            ));
        }
        if !state.next_action.is_empty() {
            let action_display = if state.next_action.len() > box_width - 18 {
                format!("{}...", &state.next_action[..box_width - 21])
            } else {
                state.next_action.clone()
            };
            out.push_str(&format!(
                "║ {:<width$} ║\n",
                format!("Next:   {}", action_display.bright_yellow()),
                width = box_width - 4
            ));
        }
        if !state.blockers.is_empty() && state.blockers != "None" {
            out.push_str(&format!(
                "║ {:<width$} ║\n",
                format!("Block:  {}", state.blockers.bright_red()),
                width = box_width - 4
            ));
        }
    }

    // Task progress
    let all_tasks = crate::tasks::parse_all_tasks(&project.sections);
    let mut total_done = 0u32;
    let mut total_pending = 0u32;
    let mut total_blocked = 0u32;
    for (_, tasks) in &all_tasks {
        for task in tasks {
            match task.status {
                crate::tasks::TaskStatus::Done => total_done += 1,
                crate::tasks::TaskStatus::Pending => total_pending += 1,
                crate::tasks::TaskStatus::Blocked => total_blocked += 1,
            }
        }
    }
    let total = total_done + total_pending + total_blocked;
    if total > 0 {
        let pct = (total_done as f64 / total as f64 * 100.0) as u32;
        let bar_width = 20;
        let filled = (pct as usize * bar_width) / 100;
        let bar = format!(
            "{}{}",
            "█".repeat(filled).green(),
            "░".repeat(bar_width - filled).dimmed()
        );
        out.push_str(&format!("╠{}╣\n", border));
        out.push_str(&format!(
            "║ {:<width$} ║\n",
            format!(
                "Tasks:  {} {}/{} ({}%)",
                bar, total_done, total, pct
            ),
            width = box_width - 4
        ));
        if total_blocked > 0 {
            out.push_str(&format!(
                "║ {:<width$} ║\n",
                format!(
                    "        {} blocked",
                    total_blocked.to_string().bright_red()
                ),
                width = box_width - 4
            ));
        }
    }

    out.push_str(&format!("╚{}╝\n", border));
    out
}

/// Render tasks with colored checkboxes.
pub fn render_tasks(tasks: &[(String, Vec<Task>)]) -> String {
    let mut out = String::new();

    if tasks.is_empty() {
        out.push_str(&format!("{}\n", "No tasks found.".dimmed()));
        return out;
    }

    for (phase, phase_tasks) in tasks {
        out.push_str(&format!(
            "\n  {} {}\n",
            "▸".bright_blue(),
            format!("Phase: {}", phase).bright_white().bold()
        ));
        out.push_str(&format!("  {}\n", "─".repeat(40).dimmed()));

        if phase_tasks.is_empty() {
            out.push_str(&format!("    {}\n", "(no tasks)".dimmed()));
            continue;
        }

        for task in phase_tasks {
            let (marker, desc_color) = match task.status {
                crate::tasks::TaskStatus::Done => ("✓".green(), true),
                crate::tasks::TaskStatus::Pending => ("○".yellow(), false),
                crate::tasks::TaskStatus::Blocked => ("✗".red(), false),
            };

            let number_str = task
                .number
                .map(|n| format!("{}: ", n))
                .unwrap_or_default();

            if desc_color {
                out.push_str(&format!(
                    "    {} {}{}\n",
                    marker,
                    number_str.dimmed(),
                    task.description.green()
                ));
            } else {
                out.push_str(&format!(
                    "    {} {}{}\n",
                    marker,
                    number_str.dimmed(),
                    task.description
                ));
            }

            for sub in &task.sub_items {
                out.push_str(&format!("      {}\n", sub.dimmed()));
            }
        }
    }

    out.push('\n');
    out
}

/// Render the current state as a dashboard.
pub fn render_state(state: &CurrentState) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{}\n",
        "── Current State ──".bright_blue().bold()
    ));

    let fields = [
        ("Phase", &state.phase),
        ("Last completed", &state.last_completed),
        ("In progress", &state.in_progress),
        ("Next action", &state.next_action),
        ("Blockers", &state.blockers),
        ("Notes", &state.notes),
    ];

    for (label, value) in &fields {
        if !value.is_empty() {
            let colored_value = if *label == "Blockers" && *value != "None" {
                value.bright_red().to_string()
            } else if *label == "Phase" {
                value.bright_cyan().to_string()
            } else if *label == "Next action" {
                value.bright_yellow().to_string()
            } else {
                value.to_string()
            };
            out.push_str(&format!(
                "  {}: {}\n",
                label.bright_white().bold(),
                colored_value
            ));
        }
    }

    out
}

/// Render decisions as a colored table.
pub fn render_decisions(decisions: &[Decision]) -> String {
    let mut out = String::new();

    if decisions.is_empty() {
        out.push_str(&format!("{}\n", "No decisions found.".dimmed()));
        return out;
    }

    out.push_str(&format!(
        "{}\n",
        "── Key Decisions ──".bright_blue().bold()
    ));

    // Calculate widths
    let mut w_decision = "Decision".len();
    let mut w_rationale = "Rationale".len();
    let mut w_outcome = "Outcome".len();

    for d in decisions {
        w_decision = w_decision.max(d.decision.len());
        w_rationale = w_rationale.max(d.rationale.len());
        w_outcome = w_outcome.max(d.outcome.to_cell().len());
    }

    // Header
    out.push_str(&format!(
        "  {:<w1$}   {:<w2$}   {}\n",
        "Decision".bright_white().bold(),
        "Rationale".bright_white().bold(),
        "Outcome".bright_white().bold(),
        w1 = w_decision,
        w2 = w_rationale,
    ));
    out.push_str(&format!(
        "  {}   {}   {}\n",
        "─".repeat(w_decision),
        "─".repeat(w_rationale),
        "─".repeat(w_outcome),
    ));

    // Rows
    for d in decisions {
        let outcome_str = match d.outcome {
            Outcome::Good => d.outcome.to_cell().green().to_string(),
            Outcome::Revisit => d.outcome.to_cell().yellow().to_string(),
            Outcome::Pending => d.outcome.to_cell().dimmed().to_string(),
            Outcome::Unset => d.outcome.to_cell().dimmed().to_string(),
        };
        out.push_str(&format!(
            "  {:<w1$}   {:<w2$}   {}\n",
            d.decision,
            d.rationale,
            outcome_str,
            w1 = w_decision,
            w2 = w_rationale,
        ));
    }

    out.push('\n');
    out
}

/// Render requirements with tier indicators.
pub fn render_requirements(reqs: &Requirements) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{}\n",
        "── Requirements ──".bright_blue().bold()
    ));

    // Validated
    out.push_str(&format!(
        "\n  {} ({}):\n",
        "✓ Validated".green().bold(),
        reqs.validated.len()
    ));
    for v in &reqs.validated {
        let version = if v.version.is_empty() {
            String::new()
        } else {
            format!(" — {}", v.version)
        };
        out.push_str(&format!(
            "    {} {}{}\n",
            "✓".green(),
            v.description,
            version.dimmed()
        ));
    }

    // Active
    out.push_str(&format!(
        "\n  {} ({}):\n",
        "○ Active".yellow().bold(),
        reqs.active.len()
    ));
    for a in &reqs.active {
        out.push_str(&format!("    {} {}\n", "○".yellow(), a.description));
        for sub in &a.sub_items {
            out.push_str(&format!("      {}\n", sub.dimmed()));
        }
    }

    // Out of Scope
    out.push_str(&format!(
        "\n  {} ({}):\n",
        "— Out of Scope".dimmed().bold(),
        reqs.out_of_scope.len()
    ));
    for o in &reqs.out_of_scope {
        let reason = if o.reason.is_empty() {
            String::new()
        } else {
            format!(" — {}", o.reason)
        };
        out.push_str(&format!(
            "    {} {}{}\n",
            "—".dimmed(),
            o.description.dimmed(),
            reason.dimmed()
        ));
    }

    out.push('\n');
    out
}

/// Render validation results with colored pass/fail.
pub fn render_validation(result: &ValidationResult) -> String {
    let mut out = String::new();

    if result.passed() {
        out.push_str(&format!(
            "{} {}\n",
            "✓ PASS".green().bold(),
            "— Project conforms to spec"
        ));
    } else {
        out.push_str(&format!(
            "{} {}\n",
            "✗ FAIL".red().bold(),
            "— Project has validation errors"
        ));
    }

    if !result.errors.is_empty() {
        out.push_str(&format!(
            "\n  {} ({}):\n",
            "Errors".red().bold(),
            result.errors.len()
        ));
        for e in &result.errors {
            out.push_str(&format!("    {} {}\n", "✗".red(), e));
        }
    }

    if !result.warnings.is_empty() {
        out.push_str(&format!(
            "\n  {} ({}):\n",
            "Warnings".yellow().bold(),
            result.warnings.len()
        ));
        for w in &result.warnings {
            out.push_str(&format!("    {} {}\n", "⚠".yellow(), w));
        }
    }

    if !result.info.is_empty() {
        out.push_str(&format!(
            "\n  {} ({}):\n",
            "Info".bright_blue().bold(),
            result.info.len()
        ));
        for i in &result.info {
            out.push_str(&format!("    {} {}\n", "ℹ".bright_blue(), i));
        }
    }

    out
}

/// Helper module for shared rendering utilities.
mod render_helpers {
    use colored::Colorize;

    /// Color a status string based on its value.
    pub fn status_color(status: &str) -> colored::ColoredString {
        match status {
            "define" => status.bright_blue(),
            "design" => status.bright_magenta(),
            "build" => status.bright_yellow(),
            "verify" => status.bright_cyan(),
            "ship" => status.bright_green(),
            "paused" => status.dimmed(),
            "archived" => status.dimmed(),
            _ => status.normal(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_test_project() -> Project {
        let content = r#"---
project: "Test Project"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Test Owner"
agent: "Test Agent"
tags: [test, rust]
---

## What This Is

A test project.

## Core Value

Fast testing.

## Requirements

### Validated

- ✓ Feature A — v0.1

### Active

- [ ] Feature B

### Out of Scope

- Feature C — not needed

## Current State

**Phase:** build
**Last completed:** Task 1
**In progress:** Task 2
**Next action:** Implement feature B
**Blockers:** None
**Notes:** Going well

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Rust | Fast and safe | ✓ Good |
| SQLite | Embedded DB | — Pending |

## Tasks

### Phase: BUILD

- [x] Task 1: Setup
- [ ] Task 2: Implementation
- [!] Task 3: Blocked task
"#;
        Project::from_str(content, Path::new("/tmp/test.md")).unwrap()
    }

    #[test]
    fn test_render_project() {
        let project = make_test_project();
        let output = render_project(&project);
        assert!(output.contains("Test Project"));
        assert!(output.contains("Frontmatter"));
        assert!(output.contains("What This Is"));
    }

    #[test]
    fn test_render_status() {
        let project = make_test_project();
        let output = render_status(&project);
        assert!(output.contains("Test Project"));
        assert!(output.contains("build"));
        assert!(output.contains("Tasks:"));
    }

    #[test]
    fn test_render_tasks() {
        let tasks = vec![
            (
                "BUILD".to_string(),
                vec![
                    Task {
                        status: crate::tasks::TaskStatus::Done,
                        description: "Setup".into(),
                        sub_items: vec![],
                        phase: "BUILD".into(),
                        number: Some(1),
                        line_index: 0,
                    },
                    Task {
                        status: crate::tasks::TaskStatus::Pending,
                        description: "Implementation".into(),
                        sub_items: vec![],
                        phase: "BUILD".into(),
                        number: Some(2),
                        line_index: 1,
                    },
                ],
            ),
        ];
        let output = render_tasks(&tasks);
        assert!(output.contains("Phase: BUILD"));
        assert!(output.contains("Setup"));
        assert!(output.contains("Implementation"));
    }

    #[test]
    fn test_render_tasks_empty() {
        let tasks: Vec<(String, Vec<Task>)> = vec![];
        let output = render_tasks(&tasks);
        assert!(output.contains("No tasks found"));
    }

    #[test]
    fn test_render_state() {
        let state = CurrentState {
            phase: "build".into(),
            last_completed: "Task 1".into(),
            in_progress: "Task 2".into(),
            next_action: "Do the thing".into(),
            blockers: "None".into(),
            notes: "Good progress".into(),
        };
        let output = render_state(&state);
        assert!(output.contains("build"));
        assert!(output.contains("Do the thing"));
        assert!(output.contains("None"));
    }

    #[test]
    fn test_render_decisions() {
        let decisions = vec![
            Decision {
                decision: "Use Rust".into(),
                rationale: "Fast".into(),
                outcome: Outcome::Good,
            },
            Decision {
                decision: "SQLite".into(),
                rationale: "Embedded".into(),
                outcome: Outcome::Pending,
            },
        ];
        let output = render_decisions(&decisions);
        assert!(output.contains("Use Rust"));
        assert!(output.contains("SQLite"));
        assert!(output.contains("Good"));
        assert!(output.contains("Pending"));
    }

    #[test]
    fn test_render_decisions_empty() {
        let decisions: Vec<Decision> = vec![];
        let output = render_decisions(&decisions);
        assert!(output.contains("No decisions found"));
    }

    #[test]
    fn test_render_requirements() {
        use crate::requirements::*;
        let reqs = Requirements {
            validated: vec![ValidatedReq {
                description: "Feature A".into(),
                version: "v0.1".into(),
            }],
            active: vec![ActiveReq {
                description: "Feature B".into(),
                sub_items: vec![],
            }],
            out_of_scope: vec![OutOfScopeItem {
                description: "Feature C".into(),
                reason: "Not needed".into(),
            }],
        };
        let output = render_requirements(&reqs);
        assert!(output.contains("Validated"));
        assert!(output.contains("Active"));
        assert!(output.contains("Out of Scope"));
        assert!(output.contains("Feature A"));
    }

    #[test]
    fn test_render_validation_pass() {
        let result = ValidationResult {
            errors: vec![],
            warnings: vec![],
            info: vec!["All good".into()],
        };
        let output = render_validation(&result);
        assert!(output.contains("PASS"));
    }

    #[test]
    fn test_render_validation_fail() {
        let result = ValidationResult {
            errors: vec!["Missing section".into()],
            warnings: vec!["Missing recommended".into()],
            info: vec![],
        };
        let output = render_validation(&result);
        assert!(output.contains("FAIL"));
        assert!(output.contains("Missing section"));
        assert!(output.contains("Missing recommended"));
    }
}

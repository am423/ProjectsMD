//! Diff command — show changes to project.md.
//!
//! Compares the current project.md to either:
//! - The git HEAD version (if in a git repo)
//! - A .project.md.snapshot file (fallback)
//!
//! Shows: task status changes, Current State changes, new decisions,
//! new session log entries.

use anyhow::{Context, Result};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

/// Run the diff command.
///
/// Compares the current project.md content against a reference version
/// and displays a colored unified diff.
pub fn run(path: &Path, quiet: bool) -> Result<()> {
    let current_content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read project file: {}", path.display()))?;

    let (old_content, source) = get_reference_content(path)?;

    if quiet {
        return Ok(());
    }

    if current_content == old_content {
        println!("{} No changes detected.", "✓".green());
        return Ok(());
    }

    println!(
        "{}",
        format!("── Diff (vs {}) ──", source).bright_blue().bold()
    );
    println!();

    let diff = TextDiff::from_lines(&old_content, &current_content);

    for change in diff.iter_all_changes() {
        let (sign, color_fn): (&str, fn(&str) -> colored::ColoredString) = match change.tag() {
            ChangeTag::Delete => ("-", |s: &str| s.red()),
            ChangeTag::Insert => ("+", |s: &str| s.green()),
            ChangeTag::Equal => (" ", |s: &str| s.dimmed()),
        };

        // Don't show unchanged context lines that are far from changes
        if change.tag() == ChangeTag::Equal {
            // Show first/last 3 context lines around changes
            let line_str = format!("{}{}", sign.dimmed(), color_fn(&change.to_string().trim_end_matches('\n')));
            // Just show all for simplicity
            print!("{}", line_str);
        } else {
            let line_str = format!(
                "{}{}",
                sign.bold(),
                color_fn(&change.to_string().trim_end_matches('\n'))
            );
            print!("{}", line_str);
        }
    }

    // Show a summary of changes
    let mut added = 0;
    let mut removed = 0;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => added += 1,
            ChangeTag::Delete => removed += 1,
            ChangeTag::Equal => {}
        }
    }

    println!();
    println!(
        "{}",
        format!("{} line(s) added, {} line(s) removed", added, removed).dimmed()
    );

    // Show semantic summary
    show_semantic_diff(&old_content, &current_content);

    Ok(())
}

/// Get the reference (old) content from git HEAD or snapshot file.
/// Returns (content, source_description).
fn get_reference_content(path: &Path) -> Result<(String, String)> {
    // Try git first
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    {
        if output.status.success() {
            // Get the relative path from git root
            let rel_path = get_git_relative_path(path).unwrap_or_else(|| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            });

            if let Ok(output) = std::process::Command::new("git")
                .args(["show", &format!("HEAD:{}", rel_path)])
                .output()
            {
                if output.status.success() {
                    let content = String::from_utf8_lossy(&output.stdout).to_string();
                    return Ok((content, "git HEAD".to_string()));
                }
            }
        }
    }

    // Try snapshot file
    let snapshot_path = get_snapshot_path(path);
    if snapshot_path.exists() {
        let content = std::fs::read_to_string(&snapshot_path)
            .context("Failed to read snapshot file")?;
        return Ok((content, "snapshot".to_string()));
    }

    anyhow::bail!(
        "No reference found. Not in a git repo and no snapshot file exists.\n\
         Create a snapshot first: cp {} {}",
        path.display(),
        snapshot_path.display()
    );
}

/// Get the relative path of a file from the git root directory.
fn get_git_relative_path(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };

    abs_path
        .strip_prefix(&git_root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Get the snapshot file path for a project.md file.
fn get_snapshot_path(path: &Path) -> std::path::PathBuf {
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path
        .file_name()
        .map(|f| format!(".{}.snapshot", f.to_string_lossy()))
        .unwrap_or_else(|| ".project.md.snapshot".to_string());
    parent.join(filename)
}

/// Show a semantic summary of changes (what changed, not just lines).
fn show_semantic_diff(old: &str, new: &str) {
    use crate::decisions::parse_decisions;
    use crate::frontmatter::parse_frontmatter;
    use crate::sections::parse_sections;
    use crate::session_log::parse_session_log;
    use crate::tasks::parse_all_tasks;

    let old_parsed = parse_frontmatter(old);
    let new_parsed = parse_frontmatter(new);

    let (old_fm, old_body) = match old_parsed {
        Ok(Some((fm, body))) => (Some(fm), body),
        _ => (None, old),
    };
    let (new_fm, new_body) = match new_parsed {
        Ok(Some((fm, body))) => (Some(fm), body),
        _ => (None, new),
    };

    let old_sections = parse_sections(old_body);
    let new_sections = parse_sections(new_body);

    let mut changes = Vec::new();

    // Task changes
    let old_tasks = parse_all_tasks(&old_sections);
    let new_tasks = parse_all_tasks(&new_sections);

    let old_done: usize = old_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .filter(|t| t.status == crate::tasks::TaskStatus::Done)
        .count();
    let new_done: usize = new_tasks
        .iter()
        .flat_map(|(_, t)| t.iter())
        .filter(|t| t.status == crate::tasks::TaskStatus::Done)
        .count();

    if new_done > old_done {
        changes.push(format!(
            "Tasks completed: {} → {} (+{})",
            old_done, new_done, new_done - old_done
        ));
    }

    // Decision changes
    let old_decisions_count = old_sections
        .iter()
        .find(|s| s.heading == "Key Decisions")
        .map(|s| parse_decisions(&s.content).len())
        .unwrap_or(0);
    let new_decisions_count = new_sections
        .iter()
        .find(|s| s.heading == "Key Decisions")
        .map(|s| parse_decisions(&s.content).len())
        .unwrap_or(0);

    if new_decisions_count > old_decisions_count {
        changes.push(format!(
            "New decisions: {} (+{})",
            new_decisions_count,
            new_decisions_count - old_decisions_count
        ));
    }

    // Session log changes
    let old_log_count = old_sections
        .iter()
        .find(|s| s.heading == "Session Log")
        .map(|s| parse_session_log(&s.content).len())
        .unwrap_or(0);
    let new_log_count = new_sections
        .iter()
        .find(|s| s.heading == "Session Log")
        .map(|s| parse_session_log(&s.content).len())
        .unwrap_or(0);

    if new_log_count > old_log_count {
        changes.push(format!(
            "New session log entries: {} (+{})",
            new_log_count,
            new_log_count - old_log_count
        ));
    }

    // Status change
    if let (Some(old_fm), Some(new_fm)) = (&old_fm, &new_fm) {
        if old_fm.status != new_fm.status {
            changes.push(format!(
                "Status: {} → {}",
                old_fm.status, new_fm.status
            ));
        }
    }

    if !changes.is_empty() {
        println!();
        println!("{}", "── Summary ──".bright_blue().bold());
        for change in &changes {
            println!("  {} {}", "•".bright_cyan(), change);
        }
    }
}

/// Create a snapshot of the current project.md for later diffing.
pub fn create_snapshot(path: &Path) -> Result<()> {
    let snapshot_path = get_snapshot_path(path);
    std::fs::copy(path, &snapshot_path).with_context(|| {
        format!(
            "Failed to create snapshot at {}",
            snapshot_path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROJECT_V1: &str = r#"---
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

- [ ] Task 1: Setup
- [ ] Task 2: Implementation

## Session Log

- **2026-01-01** — Project started.
"#;

    const TEST_PROJECT_V2: &str = r#"---
project: "Test Project"
status: build
created: 2026-01-01
updated: 2026-04-30
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
- **2026-04-30** — Completed setup task.
"#;

    #[test]
    fn test_get_snapshot_path() {
        use std::path::PathBuf;
        let path = Path::new("/home/user/project.md");
        let snapshot = get_snapshot_path(path);
        assert_eq!(snapshot, PathBuf::from("/home/user/.project.md.snapshot"));
    }

    #[test]
    fn test_diff_identical_content() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT_V1).unwrap();

        // Create snapshot
        let snapshot_path = get_snapshot_path(tmp.path());
        std::fs::write(&snapshot_path, TEST_PROJECT_V1).unwrap();

        let result = run(tmp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_with_changes() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT_V2).unwrap();

        // Create snapshot with old version
        let snapshot_path = get_snapshot_path(tmp.path());
        std::fs::write(&snapshot_path, TEST_PROJECT_V1).unwrap();

        let result = run(tmp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_no_reference() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT_V1).unwrap();

        // Make sure no snapshot exists
        let snapshot_path = get_snapshot_path(tmp.path());
        let _ = std::fs::remove_file(&snapshot_path);

        let result = run(tmp.path(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_snapshot() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT_V1).unwrap();

        let result = create_snapshot(tmp.path());
        assert!(result.is_ok());

        let snapshot_path = get_snapshot_path(tmp.path());
        assert!(snapshot_path.exists());

        let content = std::fs::read_to_string(&snapshot_path).unwrap();
        assert_eq!(content, TEST_PROJECT_V1);

        // Cleanup
        let _ = std::fs::remove_file(&snapshot_path);
    }

    #[test]
    fn test_show_semantic_diff() {
        // Just verify it doesn't panic
        show_semantic_diff(TEST_PROJECT_V1, TEST_PROJECT_V2);
    }
}

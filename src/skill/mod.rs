//! Skill module for generating agent skills from project.md files.
//!
//! Embeds SKILL.md and provides install/view/generate functionality.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Embedded SKILL.md content for the projectsmd agent skill.
pub const SKILL_CONTENT: &str = include_str!("SKILL.md");

/// Framework skill directory mappings.
const FRAMEWORK_DIRS: &[(&str, &str)] = &[
    ("claude", ".claude/skills"),
    ("hermes", ".hermes/skills"),
    ("cursor", ".cursor/skills"),
    ("codex", ".codex/skills"),
];

/// Detect which AI framework skill directories exist in the home directory.
fn detect_frameworks() -> Vec<String> {
    let home = match directories::BaseDirs::new() {
        Some(dirs) => dirs.home_dir().to_path_buf(),
        None => return vec![],
    };

    FRAMEWORK_DIRS
        .iter()
        .filter(|(_, dir)| home.join(dir).exists())
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Get the skill directory path for a given framework name.
fn framework_skill_dir(framework: &str) -> Option<PathBuf> {
    let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
    FRAMEWORK_DIRS
        .iter()
        .find(|(name, _)| *name == framework)
        .map(|(_, dir)| home.join(dir))
}

/// Install the SKILL.md to the appropriate framework skill directory.
pub fn install(
    framework: Option<&str>,
    custom_path: Option<&str>,
    force: bool,
) -> Result<PathBuf> {
    let target_dir = if let Some(path) = custom_path {
        PathBuf::from(path)
    } else if let Some(fw) = framework {
        framework_skill_dir(fw).with_context(|| {
            format!(
                "Unknown framework '{}'. Supported: claude, hermes, cursor, codex",
                fw
            )
        })?
    } else {
        let detected = detect_frameworks();
        if detected.is_empty() {
            bail!(
                "No AI framework skill directories found. \
                 Use --framework or --path to specify where to install."
            );
        }
        if detected.len() > 1 {
            println!(
                "{}",
                format!(
                    "Multiple frameworks detected: {}. Using first: {}",
                    detected.join(", "),
                    detected[0]
                )
                .yellow()
            );
        }
        framework_skill_dir(&detected[0]).unwrap()
    };

    // Create the projectsmd subdirectory
    let skill_dir = target_dir.join("projectsmd");
    fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Failed to create skill directory: {}", skill_dir.display()))?;

    let target_file = skill_dir.join("SKILL.md");

    if target_file.exists() && !force {
        bail!(
            "SKILL.md already exists at {}. Use --force to overwrite.",
            target_file.display()
        );
    }

    fs::write(&target_file, SKILL_CONTENT)
        .with_context(|| format!("Failed to write SKILL.md to {}", target_file.display()))?;

    Ok(target_file)
}

/// Print the embedded SKILL.md to stdout.
pub fn view() {
    println!("{}", SKILL_CONTENT);
}

/// Generate a project-specific SKILL.md from the current project.md.
///
/// Reads the project, extracts key info, and writes a tailored skill
/// to `.skills/projectsmd-project/SKILL.md`.
pub fn generate(project: &crate::project::Project) -> Result<PathBuf> {
    let project_dir = project
        .path
        .parent()
        .context("Cannot determine project directory")?;

    let skill_dir = project_dir.join(".skills").join("projectsmd-project");
    fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Failed to create skill directory: {}", skill_dir.display()))?;

    let content = generate_skill_content(project);
    let target_file = skill_dir.join("SKILL.md");
    fs::write(&target_file, &content)
        .with_context(|| format!("Failed to write SKILL.md to {}", target_file.display()))?;

    Ok(target_file)
}

/// Generate project-specific skill content.
fn generate_skill_content(project: &crate::project::Project) -> String {
    use crate::tasks::parse_all_tasks;

    let fm = &project.frontmatter;
    let all_tasks = parse_all_tasks(&project.sections);

    let mut out = String::new();

    out.push_str("---\n");
    out.push_str(&format!("name: {}-skill\n", fm.project.to_lowercase().replace(' ', "-")));
    out.push_str(&format!(
        "description: \"Project-specific skill for {}\"\n",
        fm.project
    ));
    out.push_str("---\n\n");
    out.push_str(&format!("# {} — Project Skill\n\n", fm.project));
    out.push_str(&format!("**Status:** {}\n", fm.status));
    out.push_str(&format!("**Owner:** {}\n", fm.owner));
    if let Some(ref agent) = fm.agent {
        out.push_str(&format!("**Agent:** {}\n", agent));
    }
    out.push('\n');

    // Current tasks by phase
    out.push_str("## Current Tasks\n\n");
    for (phase, tasks) in &all_tasks {
        let pending: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == crate::tasks::TaskStatus::Pending)
            .collect();
        let done: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == crate::tasks::TaskStatus::Done)
            .collect();
        let blocked: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == crate::tasks::TaskStatus::Blocked)
            .collect();

        out.push_str(&format!("### Phase: {} ({} done, {} pending, {} blocked)\n\n", phase, done.len(), pending.len(), blocked.len()));

        for task in &pending {
            let num = task
                .number
                .map(|n| format!("Task {}: ", n))
                .unwrap_or_default();
            out.push_str(&format!("- [ ] {}{}\n", num, task.description));
        }
        for task in &blocked {
            let num = task
                .number
                .map(|n| format!("Task {}: ", n))
                .unwrap_or_default();
            out.push_str(&format!("- [!] {}{}\n", num, task.description));
        }
        for task in &done {
            let num = task
                .number
                .map(|n| format!("Task {}: ", n))
                .unwrap_or_default();
            out.push_str(&format!("- [x] {}{}\n", num, task.description));
        }
        out.push('\n');
    }

    // Quick commands
    out.push_str("## Quick Commands\n\n");
    out.push_str("```bash\n");
    out.push_str("projectsmd status\n");
    out.push_str("projectsmd next\n");
    out.push_str("projectsmd task list --pending\n");
    out.push_str("projectsmd session\n");
    out.push_str("```\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_content_not_empty() {
        assert!(!SKILL_CONTENT.is_empty());
        assert!(SKILL_CONTENT.contains("name: projectsmd"));
        assert!(SKILL_CONTENT.contains("Session Start"));
        assert!(SKILL_CONTENT.contains("Session End"));
        assert!(SKILL_CONTENT.contains("Anti-Patterns"));
    }

    #[test]
    fn test_skill_content_has_frontmatter() {
        assert!(SKILL_CONTENT.starts_with("---"));
        assert!(SKILL_CONTENT.contains("description:"));
    }

    #[test]
    fn test_detect_frameworks() {
        // This test just ensures the function doesn't panic
        let _frameworks = detect_frameworks();
    }

    #[test]
    fn test_framework_skill_dir_known() {
        let result = framework_skill_dir("claude");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_framework_skill_dir_unknown() {
        let result = framework_skill_dir("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_skill_content() {
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
- [!] Task 3: Blocked task

## Session Log

- **2026-01-01** — Project started.
"#;

        let path = std::path::Path::new("/tmp/test_skill.md");
        let project = crate::project::Project::from_str(TEST_PROJECT, path).unwrap();
        let content = generate_skill_content(&project);
        assert!(content.contains("Test Project"));
        assert!(content.contains("BUILD"));
        assert!(content.contains("Task 2: Implementation"));
    }

    #[test]
    fn test_install_to_custom_path() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().to_string_lossy().to_string();

        let result = install(None, Some(&target), false).unwrap();
        assert!(result.exists());

        let content = fs::read_to_string(&result).unwrap();
        assert!(content.contains("projectsmd"));
    }

    #[test]
    fn test_install_force_overwrite() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().to_string_lossy().to_string();

        // First install
        install(None, Some(&target), false).unwrap();

        // Second install without force should fail
        let result = install(None, Some(&target), false);
        assert!(result.is_err());

        // With force should succeed
        let result = install(None, Some(&target), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_project_skill() {
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

        let tmp_dir = tempfile::tempdir().unwrap();
        let project_path = tmp_dir.path().join("project.md");
        std::fs::write(&project_path, TEST_PROJECT).unwrap();
        let project = crate::project::Project::load(&project_path).unwrap();

        let result = generate(&project).unwrap();
        assert!(result.exists());

        let content = fs::read_to_string(&result).unwrap();
        assert!(content.contains("Test Project"));
        assert!(content.contains("Task 2: Implementation"));
    }
}

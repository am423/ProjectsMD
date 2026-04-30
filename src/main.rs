//! projectsmd CLI entry point.
//!
//! Parses command-line arguments and dispatches to the appropriate command handler.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use projectsmd::decisions::{add_decision, parse_decisions, write_decisions, Outcome};
use projectsmd::project::Project;
use projectsmd::render;
use projectsmd::tasks::{parse_all_tasks, TaskStatus};
use projectsmd::validate;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "projectsmd")]
#[command(about = "A project.md lifecycle management tool")]
#[command(version)]
struct Cli {
    /// Path to project.md file
    #[arg(short, long, default_value = "project.md")]
    file: PathBuf,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Suppress output (only exit code)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project.md file
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Project owner
        #[arg(long)]
        owner: Option<String>,
        /// Use brownfield template
        #[arg(long)]
        brownfield: bool,
        /// Import from an existing file
        #[arg(long)]
        from: Option<PathBuf>,
        /// Use a custom template file
        #[arg(long)]
        template: Option<PathBuf>,
    },

    /// Validate a project.md file for conformance
    Validate,

    /// Show current project status
    Status,

    /// Show the next recommended action
    Next,

    /// Manage tasks
    #[command(subcommand)]
    Task(TaskCmd),

    /// Record a key decision
    Decide {
        /// The decision text
        decision: String,
        /// Rationale for the decision
        #[arg(long)]
        rationale: Option<String>,
    },

    /// Record a discovery
    Discover {
        /// The discovery text
        text: String,
    },

    /// Manage project phases
    Phase {
        /// New status to transition to (define, design, build, verify, ship, paused)
        #[arg(long)]
        transition: Option<String>,
    },

    /// Session wrap-up wizard
    Session {
        /// Non-interactive mode
        #[arg(long)]
        non_interactive: bool,
        /// Session summary (for non-interactive mode)
        #[arg(long)]
        summary: Option<String>,
    },

    /// Show diff of project.md changes
    Diff,

    /// Archive the project
    Archive {
        /// Summary for the final session log entry
        #[arg(long)]
        summary: Option<String>,
    },

    /// Render project.md to terminal
    View {
        /// Filter by section name
        #[arg(long)]
        section: Option<String>,
    },

    /// Manage agent skills
    #[command(subcommand)]
    Skill(SkillCmd),
}

#[derive(Subcommand)]
enum TaskCmd {
    /// List all tasks
    List {
        /// Filter by phase
        #[arg(long)]
        phase: Option<String>,
        /// Show only pending tasks
        #[arg(long)]
        pending: bool,
    },
    /// Mark a task as done
    Done {
        /// Task number
        n: u32,
    },
    /// Mark a task as blocked
    Block {
        /// Task number
        n: u32,
        /// Reason for blocking
        #[arg(long)]
        reason: String,
    },
    /// Unblock a task
    Unblock {
        /// Task number
        n: u32,
    },
    /// Add a new task
    Add {
        /// Task description
        description: String,
        /// Phase to add the task to
        #[arg(long)]
        phase: Option<String>,
    },
}

#[derive(Subcommand)]
enum SkillCmd {
    /// Install the projectsmd skill to a framework directory
    Install {
        /// Target framework (claude, hermes, cursor, codex)
        #[arg(long)]
        framework: Option<String>,
        /// Custom installation path
        #[arg(long)]
        path: Option<String>,
        /// Overwrite existing skill
        #[arg(long)]
        force: bool,
    },
    /// Print the embedded SKILL.md to stdout
    View,
    /// Generate a project-specific skill
    Generate,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(cmd) => match cmd {
            Commands::Init {
                name,
                owner,
                brownfield,
                from,
                template,
            } => cmd_init(&cli.file, name, owner, brownfield, from, template),
            Commands::Validate => cmd_validate(&cli.file, cli.quiet),
            Commands::Status => cmd_status(&cli.file, cli.quiet),
            Commands::Next => cmd_next(&cli.file, cli.quiet),
            Commands::Task(task_cmd) => cmd_task(&cli.file, task_cmd, cli.quiet),
            Commands::Decide {
                decision,
                rationale,
            } => cmd_decide(&cli.file, &decision, rationale.as_deref()),
            Commands::Discover { text } => cmd_discover(&cli.file, &text),
            Commands::Phase { transition } => {
                projectsmd::commands::phase::run(&cli.file, transition.as_deref(), cli.quiet)
            }
            Commands::Session {
                non_interactive,
                summary,
            } => projectsmd::commands::session::run(
                &cli.file,
                non_interactive,
                summary.as_deref(),
                cli.quiet,
            ),
            Commands::Diff => projectsmd::commands::diff::run(&cli.file, cli.quiet),
            Commands::Archive { summary } => {
                projectsmd::commands::archive::run(&cli.file, summary.as_deref(), cli.quiet)
            }
            Commands::View { section } => cmd_view(&cli.file, section.as_deref(), cli.quiet),
            Commands::Skill(skill_cmd) => cmd_skill(&cli.file, skill_cmd, cli.quiet),
        },
        None => {
            println!(
                "{} {}",
                "projectsmd".bright_white().bold(),
                "— A project.md lifecycle management tool"
            );
            println!("Run with --help for usage information.");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn load_project(path: &Path) -> Result<Project> {
    Project::load(path).with_context(|| {
        format!(
            "Failed to load project file: {}\nRun 'projectsmd init' to create one.",
            path.display()
        )
    })
}

fn cmd_init(
    path: &Path,
    name: Option<String>,
    owner: Option<String>,
    brownfield: bool,
    from: Option<PathBuf>,
    template: Option<PathBuf>,
) -> Result<()> {
    projectsmd::commands::init::run(
        path,
        name.as_deref(),
        owner.as_deref(),
        brownfield,
        from.as_deref(),
        template.as_deref(),
        false,
    )
}

fn cmd_validate(path: &Path, quiet: bool) -> Result<()> {
    let project = load_project(path)?;
    let result = validate::validate(&project);

    if !quiet {
        print!("{}", render::render_validation(&result));
    }

    if !result.passed() {
        bail!("Validation failed with {} error(s)", result.errors.len());
    }
    Ok(())
}

fn cmd_status(path: &Path, quiet: bool) -> Result<()> {
    let project = load_project(path)?;
    if !quiet {
        print!("{}", render::render_status(&project));
    }
    Ok(())
}

fn cmd_next(path: &Path, quiet: bool) -> Result<()> {
    let project = load_project(path)?;

    if let Some(state_section) = project.get_section("Current State") {
        let state = projectsmd::state::parse_state(&state_section.content);

        if !quiet {
            println!("{}", "── Next Action ──".bright_blue().bold());
            println!();

            if !state.phase.is_empty() {
                println!(
                    "  {}: {}",
                    "Phase".bright_white().bold(),
                    state.phase.bright_cyan()
                );
            }
            if !state.in_progress.is_empty() {
                println!(
                    "  {}: {}",
                    "In progress".bright_white().bold(),
                    state.in_progress
                );
            }
            if !state.next_action.is_empty() {
                println!(
                    "  {}: {}",
                    "Next action".bright_white().bold(),
                    state.next_action.bright_yellow()
                );
            }
            if !state.blockers.is_empty() && state.blockers != "None" {
                println!(
                    "  {}: {}",
                    "Blockers".bright_white().bold(),
                    state.blockers.bright_red()
                );
            }
        }
    } else if !quiet {
        println!("No Current State section found.");
    }

    Ok(())
}

fn cmd_task(path: &Path, task_cmd: TaskCmd, quiet: bool) -> Result<()> {
    match task_cmd {
        TaskCmd::List { phase, pending } => cmd_task_list(path, phase.as_deref(), pending, quiet),
        TaskCmd::Done { n } => cmd_task_done(path, n, quiet),
        TaskCmd::Block { n, reason } => cmd_task_block(path, n, &reason, quiet),
        TaskCmd::Unblock { n } => cmd_task_unblock(path, n, quiet),
        TaskCmd::Add { description, phase } => {
            projectsmd::commands::task::add(
                path,
                &description,
                phase.as_deref().unwrap_or("BUILD"),
                quiet,
            )
        }
    }
}

fn cmd_task_list(path: &Path, phase: Option<&str>, pending: bool, quiet: bool) -> Result<()> {
    let project = load_project(path)?;
    let mut all_tasks = parse_all_tasks(&project.sections);

    // Filter by phase
    if let Some(phase_filter) = phase {
        all_tasks.retain(|(name, _)| name.eq_ignore_ascii_case(phase_filter));
    }

    // Filter pending only
    if pending {
        for (_, tasks) in &mut all_tasks {
            tasks.retain(|t| t.status == TaskStatus::Pending);
        }
        all_tasks.retain(|(_, tasks)| !tasks.is_empty());
    }

    if !quiet {
        print!("{}", render::render_tasks(&all_tasks));
    }
    Ok(())
}

fn cmd_task_done(path: &Path, n: u32, quiet: bool) -> Result<()> {
    let mut project = load_project(path)?;

    let tasks_section = project
        .get_section("Tasks")
        .context("No Tasks section found")?
        .clone();

    let content = tasks_section.content.clone();
    let all_tasks = parse_all_tasks(&project.sections);

    // Check task exists
    let found = all_tasks
        .iter()
        .any(|(_, tasks)| tasks.iter().any(|t| t.number == Some(n)));

    if !found {
        bail!("Task {} not found", n);
    }

    // Rewrite the tasks section with the updated task
    let mut new_content = content.clone();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("- [ ] Task {}: ", n))
            || trimmed.starts_with(&format!("- [!] Task {}: ", n))
        {
            let new_line = trimmed
                .replacen("- [ ]", "- [x]", 1)
                .replacen("- [!]", "- [x]", 1);
            new_content = new_content.replacen(trimmed, &new_line, 1);
            break;
        }
    }

    project.update_section("Tasks", &new_content);
    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );
    project.save()?;

    if !quiet {
        println!("{} Task {} marked as done.", "✓".green(), n);
    }
    Ok(())
}

fn cmd_task_block(path: &Path, n: u32, reason: &str, quiet: bool) -> Result<()> {
    let mut project = load_project(path)?;

    let tasks_section = project
        .get_section("Tasks")
        .context("No Tasks section found")?
        .clone();
    let content = tasks_section.content.clone();

    // Do a single pass: find the task line and replace it with blocked marker + reason
    let mut replaced = false;
    let mut new_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("- [ ] Task {}: ", n))
            || trimmed.starts_with(&format!("- [x] Task {}: ", n))
        {
            let marker_line = trimmed
                .replacen("- [ ]", "- [!]", 1)
                .replacen("- [x]", "- [!]", 1);
            if reason.is_empty() {
                new_lines.push(marker_line);
            } else {
                new_lines.push(marker_line);
                new_lines.push(format!("  - Blocked: {}", reason));
            }
            replaced = true;
        } else {
            new_lines.push(line.to_string());
        }
    }
    let new_content = new_lines.join("\n");

    if !replaced {
        bail!("Task {} not found", n);
    }

    project.update_section("Tasks", &new_content);
    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );
    project.save()?;

    if !quiet {
        println!("{} Task {} blocked. Reason: {}", "✗".red(), n, reason);
    }
    Ok(())
}

fn cmd_task_unblock(path: &Path, n: u32, quiet: bool) -> Result<()> {
    let mut project = load_project(path)?;

    let tasks_section = project
        .get_section("Tasks")
        .context("No Tasks section found")?
        .clone();
    let content = tasks_section.content.clone();

    // Check task exists and is blocked
    let mut found = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("- [!] Task {}: ", n)) {
            found = true;
            break;
        }
    }

    if !found {
        bail!("Blocked task {} not found", n);
    }

    let mut new_content = content.clone();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("- [!] Task {}: ", n)) {
            let new_line = trimmed.replacen("- [!]", "- [ ]", 1);
            new_content = new_content.replacen(trimmed, &new_line, 1);
            break;
        }
    }

    // Remove blocker sub-items
    let mut lines_to_remove = Vec::new();
    let lines: Vec<&str> = new_content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("- [ ] Task {}: ", n)) {
            // Look for subsequent indented blocker lines
            for j in (i + 1)..lines.len() {
                let sub = lines[j].trim();
                if sub.starts_with("- Blocked:") || sub.starts_with("- blocked:") {
                    lines_to_remove.push(j);
                } else if sub.is_empty() || sub.starts_with("- [") {
                    break;
                }
            }
            break;
        }
    }

    if !lines_to_remove.is_empty() {
        let filtered: Vec<&str> = new_content
            .lines()
            .enumerate()
            .filter(|(i, _)| !lines_to_remove.contains(i))
            .map(|(_, l)| l)
            .collect();
        new_content = filtered.join("\n");
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
    }

    project.update_section("Tasks", &new_content);
    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );
    project.save()?;

    if !quiet {
        println!("{} Task {} unblocked.", "✓".green(), n);
    }
    Ok(())
}

fn cmd_decide(path: &Path, decision: &str, rationale: Option<&str>) -> Result<()> {
    let mut project = load_project(path)?;

    let rationale_text = rationale.unwrap_or("");

    if let Some(dec_section) = project.get_section("Key Decisions") {
        let mut decisions = parse_decisions(&dec_section.content);
        add_decision(&mut decisions, decision, rationale_text, Outcome::Pending);
        let new_content = write_decisions(&decisions);
        project.update_section("Key Decisions", &new_content);
    } else {
        // Create the section if it doesn't exist
        let mut decisions = Vec::new();
        add_decision(&mut decisions, decision, rationale_text, Outcome::Pending);
        let new_content = write_decisions(&decisions);
        project.update_section("Key Decisions", &format!("\n{}", new_content));
    }

    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );
    project.save()?;

    println!("{} Decision recorded: {}", "✓".green(), decision);
    if let Some(r) = rationale {
        println!("  Rationale: {}", r);
    }

    Ok(())
}

fn cmd_discover(path: &Path, text: &str) -> Result<()> {
    let mut project = load_project(path)?;

    if let Some(section) = project.get_section("Discoveries") {
        let new_content = format!("{}- {}\n", section.content, text);
        project.update_section("Discoveries", &new_content);
    } else {
        project.update_section("Discoveries", &format!("\n- {}\n", text));
    }

    project.update_frontmatter_field(
        "updated",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
    );
    project.save()?;

    println!("{} Discovery recorded: {}", "✓".green(), text);

    Ok(())
}

fn cmd_view(path: &Path, section: Option<&str>, quiet: bool) -> Result<()> {
    let project = load_project(path)?;

    if quiet {
        return Ok(());
    }

    match section {
        Some(section_name) => {
            if let Some(sec) = project.get_section(section_name) {
                // Render specific section content
                println!(
                    "{} {} {}\n",
                    "##".bright_blue(),
                    sec.heading.bright_white().bold(),
                    "─"
                        .repeat(40_usize.saturating_sub(sec.heading.len()))
                        .dimmed()
                );
                for line in sec.content.lines() {
                    println!("  {}", line);
                }
            } else {
                eprintln!(
                    "{} Section '{}' not found.",
                    "error:".red().bold(),
                    section_name
                );
                eprintln!("Available sections:");
                for sec in &project.sections {
                    eprintln!("  - {}", sec.heading);
                }
                bail!("Section '{}' not found", section_name);
            }
        }
        None => {
            print!("{}", render::render_project(&project));
        }
    }

    Ok(())
}

fn cmd_skill(path: &Path, skill_cmd: SkillCmd, quiet: bool) -> Result<()> {
    match skill_cmd {
        SkillCmd::Install {
            framework,
            path: skill_path,
            force,
        } => projectsmd::commands::skill::install(
            framework.as_deref(),
            skill_path.as_deref(),
            force,
            quiet,
        ),
        SkillCmd::View => {
            projectsmd::commands::skill::view();
            Ok(())
        }
        SkillCmd::Generate => projectsmd::commands::skill::generate(path, quiet),
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

## Core Value

Fast testing.

## Requirements

### Validated

- ✓ Feature A — v0.1

### Active

- [ ] Feature B

### Out of Scope

- Feature C — not needed

## Context

Some context.

## Constraints

Some constraints.

## Current State

**Phase:** build
**Last completed:** Task 1
**In progress:** Task 2
**Next action:** Implement feature B
**Blockers:** None
**Notes:** Good progress

## Architecture

Some architecture.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Rust | Fast and safe | ✓ Good |

## Tasks

### Phase: BUILD

- [x] Task 1: Setup
- [ ] Task 2: Implementation
- [!] Task 3: Blocked task

## Discoveries

Some discovery.

## References

Some reference.

## Session Log

- **2026-01-01** — Project started.
"#;

    fn write_test_file() -> tempfile::NamedTempFile {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), TEST_PROJECT).unwrap();
        tmp
    }

    #[test]
    fn test_load_project() {
        let tmp = write_test_file();
        let project = load_project(tmp.path()).unwrap();
        assert_eq!(project.frontmatter.project, "Test Project");
    }

    #[test]
    fn test_cmd_validate_passes() {
        let tmp = write_test_file();
        let result = cmd_validate(tmp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_status() {
        let tmp = write_test_file();
        let result = cmd_status(tmp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_next() {
        let tmp = write_test_file();
        let result = cmd_next(tmp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_task_list() {
        let tmp = write_test_file();
        let result = cmd_task_list(tmp.path(), None, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_task_list_pending() {
        let tmp = write_test_file();
        let result = cmd_task_list(tmp.path(), None, true, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_task_list_by_phase() {
        let tmp = write_test_file();
        let result = cmd_task_list(tmp.path(), Some("BUILD"), false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_task_done() {
        let tmp = write_test_file();
        let result = cmd_task_done(tmp.path(), 2, true);
        assert!(result.is_ok());

        // Verify the task was marked done
        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("- [x] Task 2:"));
    }

    #[test]
    fn test_cmd_task_done_not_found() {
        let tmp = write_test_file();
        let result = cmd_task_done(tmp.path(), 99, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Task 99 not found"));
    }

    #[test]
    fn test_cmd_task_block() {
        let tmp = write_test_file();
        let result = cmd_task_block(tmp.path(), 2, "waiting on API", true);
        assert!(result.is_ok());

        // Verify the task was blocked
        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("- [!] Task 2:"));
    }

    #[test]
    fn test_cmd_task_block_not_found() {
        let tmp = write_test_file();
        let result = cmd_task_block(tmp.path(), 99, "reason", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_task_unblock() {
        let tmp = write_test_file();
        let result = cmd_task_unblock(tmp.path(), 3, true);
        assert!(result.is_ok());

        // Verify the task was unblocked
        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("- [ ] Task 3:"));
    }

    #[test]
    fn test_cmd_task_unblock_not_found() {
        let tmp = write_test_file();
        let result = cmd_task_unblock(tmp.path(), 1, true); // Task 1 is done, not blocked
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_decide() {
        let tmp = write_test_file();
        let result = cmd_decide(tmp.path(), "Use PostgreSQL", Some("Better for production"));
        assert!(result.is_ok());

        // Verify the decision was added
        let project = Project::load(tmp.path()).unwrap();
        let dec_section = project.get_section("Key Decisions").unwrap();
        assert!(dec_section.content.contains("Use PostgreSQL"));
    }

    #[test]
    fn test_cmd_decide_no_rationale() {
        let tmp = write_test_file();
        let result = cmd_decide(tmp.path(), "New decision", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_discover() {
        let tmp = write_test_file();
        let result = cmd_discover(tmp.path(), "Found a new pattern");
        assert!(result.is_ok());

        // Verify the discovery was added
        let project = Project::load(tmp.path()).unwrap();
        let disc_section = project.get_section("Discoveries").unwrap();
        assert!(disc_section.content.contains("Found a new pattern"));
    }

    #[test]
    fn test_cmd_view() {
        let tmp = write_test_file();
        let result = cmd_view(tmp.path(), None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_view_section() {
        let tmp = write_test_file();
        let result = cmd_view(tmp.path(), Some("Core Value"), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_view_section_not_found() {
        let tmp = write_test_file();
        let result = cmd_view(tmp.path(), Some("Nonexistent"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_task_done_updates_file() {
        let tmp = write_test_file();

        // Mark task 2 as done
        cmd_task_done(tmp.path(), 2, true).unwrap();

        // Verify file was updated
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("- [x] Task 2: Implementation"));
        assert!(content.contains("- [!] Task 3: Blocked task"));
    }

    #[test]
    fn test_cmd_task_block_adds_reason() {
        let tmp = write_test_file();

        // Block task 2
        cmd_task_block(tmp.path(), 2, "waiting on API", true).unwrap();

        // Verify blocker reason added
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("- [!] Task 2:"));
        assert!(content.contains("- Blocked: waiting on API"));
    }

    #[test]
    fn test_cmd_task_add() {
        let tmp = write_test_file();
        let result = projectsmd::commands::task::add(
            tmp.path(),
            "New feature",
            "build",
            true,
        );
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        let tasks_section = project.get_section("Tasks").unwrap();
        assert!(tasks_section.content.contains("Task 4: New feature"));
    }

    #[test]
    fn test_cmd_session_noninteractive() {
        let tmp = write_test_file();
        let result = projectsmd::commands::session::run(
            tmp.path(),
            true,
            Some("Completed session work"),
            true,
        );
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("Completed session work"));
    }

    #[test]
    fn test_cmd_phase_transition() {
        let tmp = write_test_file();
        let result = projectsmd::commands::phase::run(
            tmp.path(),
            Some("verify"),
            true,
        );
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        assert_eq!(project.frontmatter.status, projectsmd::frontmatter::ProjectStatus::Verify);
    }

    #[test]
    fn test_cmd_archive() {
        let tmp = write_test_file();
        let result = projectsmd::commands::archive::run(
            tmp.path(),
            Some("All done!"),
            true,
        );
        assert!(result.is_ok());

        let project = Project::load(tmp.path()).unwrap();
        assert_eq!(project.frontmatter.status, projectsmd::frontmatter::ProjectStatus::Archived);
        let log_section = project.get_section("Session Log").unwrap();
        assert!(log_section.content.contains("ARCHIVED"));
    }

    #[test]
    fn test_cmd_skill_install() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().to_string_lossy().to_string();
        let result = projectsmd::commands::skill::install(
            None,
            Some(&target),
            false,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_skill_generate() {
        let tmp = write_test_file();
        let result = projectsmd::commands::skill::generate(tmp.path(), true);
        assert!(result.is_ok());
    }
}

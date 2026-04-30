//! Init command — initialize a new project.md file.
//!
//! Interactive project creation using dialoguer.
//! Supports greenfield and brownfield templates, custom templates,
//! and importing from existing files.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::template::{populate_with_template, BROWNFIELD_TEMPLATE, DEFAULT_TEMPLATE};

/// Run the init command.
///
/// Creates a new project.md file with interactive prompts or CLI flags.
///
/// # Arguments
/// * `path` - Target path for the new project.md file
/// * `name` - Project name (optional, prompted if not provided)
/// * `owner` - Project owner (optional, prompted if not provided)
/// * `brownfield` - Use brownfield template for existing projects
/// * `from` - Import from an existing file instead of creating new
/// * `template` - Use a custom template file
pub fn run(
    path: &Path,
    name: Option<&str>,
    owner: Option<&str>,
    brownfield: bool,
    from: Option<&Path>,
    template: Option<&Path>,
    quiet: bool,
) -> Result<()> {
    // Handle --from: copy existing file
    if let Some(from_path) = from {
        if !from_path.exists() {
            anyhow::bail!("Source file not found: {}", from_path.display());
        }
        std::fs::copy(from_path, path).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                from_path.display(),
                path.display()
            )
        })?;
        if !quiet {
            println!(
                "{} Copied {} to {}",
                "✓".green(),
                from_path.display(),
                path.display()
            );
        }
        return Ok(());
    }

    // Check if target already exists
    if path.exists() {
        anyhow::bail!(
            "File already exists: {}. Use --from to import, or remove it first.",
            path.display()
        );
    }

    // Gather project metadata interactively
    let metadata = gather_metadata(name, owner)?;

    // Create the project
    create_project(
        path,
        brownfield,
        template,
        &metadata.name,
        &metadata.owner,
        &metadata.agent,
        &metadata.tags,
        &metadata.description,
        &metadata.core_value,
        quiet,
    )
}

/// Run init with all metadata provided (non-interactive).
///
/// This is the non-interactive entry point used by tests and programmatic callers.
pub fn run_with_metadata(
    path: &Path,
    name: &str,
    owner: &str,
    agent: &str,
    tags: &[String],
    description: &str,
    core_value: &str,
    brownfield: bool,
    template: Option<&Path>,
    quiet: bool,
) -> Result<()> {
    // Check if target already exists
    if path.exists() {
        anyhow::bail!(
            "File already exists: {}. Use --from to import, or remove it first.",
            path.display()
        );
    }

    create_project(
        path,
        brownfield,
        template,
        name,
        owner,
        agent,
        tags,
        description,
        core_value,
        quiet,
    )
}

/// Internal function to create a project from metadata.
fn create_project(
    path: &Path,
    brownfield: bool,
    template: Option<&Path>,
    name: &str,
    owner: &str,
    agent: &str,
    tags: &[String],
    description: &str,
    core_value: &str,
    quiet: bool,
) -> Result<()> {
    // Load template
    let template_content = if let Some(template_path) = template {
        std::fs::read_to_string(template_path)
            .with_context(|| format!("Failed to read template file: {}", template_path.display()))?
    } else if brownfield {
        BROWNFIELD_TEMPLATE.to_string()
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    // Populate template
    let content = populate_with_template(
        &template_content,
        name,
        owner,
        agent,
        tags,
        description,
        core_value,
    );

    // Write the file
    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write project file: {}", path.display()))?;

    if !quiet {
        println!("{} Created {}", "✓".green().bold(), path.display());
        println!("  Project: {}", name.bright_white().bold());
        println!("  Owner:   {}", owner);
        if !agent.is_empty() {
            println!("  Agent:   {}", agent);
        }
        if !tags.is_empty() {
            println!("  Tags:    {}", tags.join(", "));
        }
    }

    Ok(())
}

/// Project metadata gathered during init.
struct ProjectMetadata {
    name: String,
    owner: String,
    agent: String,
    tags: Vec<String>,
    description: String,
    core_value: String,
}

/// Gather project metadata interactively or from provided values.
fn gather_metadata(name: Option<&str>, owner: Option<&str>) -> Result<ProjectMetadata> {
    use dialoguer::Input;

    let project_name = if let Some(n) = name {
        n.to_string()
    } else {
        Input::new()
            .with_prompt("Project name")
            .interact_text()
            .context("Failed to read project name")?
    };

    let project_owner = if let Some(o) = owner {
        o.to_string()
    } else {
        Input::new()
            .with_prompt("Owner")
            .interact_text()
            .context("Failed to read owner")?
    };

    let agent: String = Input::new()
        .with_prompt("Agent (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to read agent")?;

    let tags_input: String = Input::new()
        .with_prompt("Tags (comma-separated, press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .context("Failed to read tags")?;

    let tags: Vec<String> = if tags_input.trim().is_empty() {
        Vec::new()
    } else {
        tags_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let description: String = Input::new()
        .with_prompt("What is this project? (2-3 sentences)")
        .interact_text()
        .context("Failed to read description")?;

    let core_value: String = Input::new()
        .with_prompt("Core value (one sentence)")
        .interact_text()
        .context("Failed to read core value")?;

    Ok(ProjectMetadata {
        name: project_name,
        owner: project_owner,
        agent,
        tags,
        description,
        core_value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_creates_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("project.md");

        let result = run_with_metadata(
            &path,
            "Test Project",
            "Test Owner",
            "",
            &[],
            "A test project.",
            "Fast testing.",
            false,
            None,
            true,
        );
        assert!(result.is_ok());
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Test Project"));
        assert!(content.contains("Test Owner"));
        assert!(content.contains("A test project."));
        assert!(content.contains("Fast testing."));
    }

    #[test]
    fn test_init_brownfield() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("project.md");

        let result = run_with_metadata(
            &path,
            "Legacy App",
            "Bob",
            "",
            &[],
            "An existing app.",
            "Maintain stability.",
            true,
            None,
            true,
        );
        assert!(result.is_ok());
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Legacy App"));
        assert!(content.contains("ASSESS"));
    }

    #[test]
    fn test_init_existing_file_fails() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("project.md");
        std::fs::write(&path, "existing content").unwrap();

        let result = run_with_metadata(
            &path,
            "Test",
            "Owner",
            "",
            &[],
            "Desc.",
            "Value.",
            false,
            None,
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_init_from_existing() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let source = tmp_dir.path().join("source.md");
        let target = tmp_dir.path().join("target.md");

        std::fs::write(&source, "---\nproject: imported\n---\n").unwrap();

        let result = run(&target, None, None, false, Some(&source), None, true);
        assert!(result.is_ok());
        assert!(target.exists());

        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("imported"));
    }

    #[test]
    fn test_init_from_nonexistent_fails() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let target = tmp_dir.path().join("project.md");
        let source = tmp_dir.path().join("nonexistent.md");

        let result = run(&target, None, None, false, Some(&source), None, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_custom_template() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("project.md");
        let template = tmp_dir.path().join("custom.md");

        std::fs::write(
            &template,
            "---\nproject: \"{{PROJECT_NAME}}\"\n---\n\nCustom template for {{OWNER}}.\n",
        )
        .unwrap();

        let result = run_with_metadata(
            &path,
            "My Project",
            "Alice",
            "",
            &[],
            "A project.",
            "Fast.",
            false,
            Some(&template),
            true,
        );
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("My Project"));
        assert!(content.contains("Alice"));
        assert!(content.contains("Custom template"));
    }

    #[test]
    fn test_init_with_tags_and_agent() {
        let content = populate_with_template(
            DEFAULT_TEMPLATE,
            "Tagged Project",
            "Owner",
            "Hermes",
            &["web".to_string(), "rust".to_string()],
            "A project with tags.",
            "Fast and reliable.",
        );

        assert!(content.contains("Tagged Project"));
        assert!(content.contains("Owner"));
        assert!(content.contains("Hermes"));
        assert!(content.contains("web, rust"));
        assert!(content.contains("A project with tags."));
        assert!(content.contains("Fast and reliable."));
    }
}

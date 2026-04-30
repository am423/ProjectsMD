//! Template module for generating project.md templates.
//!
//! Embeds default and brownfield templates and provides population functions.

use chrono::Local;

/// The default (greenfield) project.md template.
pub const DEFAULT_TEMPLATE: &str = include_str!("default.md");

/// The brownfield (existing project) template.
pub const BROWNFIELD_TEMPLATE: &str = include_str!("brownfield.md");

/// Populate a template with project metadata.
///
/// Replaces `{{PROJECT_NAME}}`, `{{OWNER}}`, `{{AGENT}}`, `{{TAGS}}`,
/// `{{DESCRIPTION}}`, `{{CORE_VALUE}}`, and `{{DATE}}` placeholders.
pub fn populate_template(
    name: &str,
    owner: &str,
    agent: &str,
    tags: &[String],
    description: &str,
    core_value: &str,
) -> String {
    populate_with_template(
        DEFAULT_TEMPLATE,
        name,
        owner,
        agent,
        tags,
        description,
        core_value,
    )
}

/// Populate a given template string with project metadata.
pub fn populate_with_template(
    template: &str,
    name: &str,
    owner: &str,
    agent: &str,
    tags: &[String],
    description: &str,
    core_value: &str,
) -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let tags_str = tags.join(", ");

    template
        .replace("{{PROJECT_NAME}}", name)
        .replace("{{OWNER}}", owner)
        .replace("{{AGENT}}", agent)
        .replace("{{TAGS}}", &tags_str)
        .replace("{{DESCRIPTION}}", description)
        .replace("{{CORE_VALUE}}", core_value)
        .replace("{{DATE}}", &today)
}

/// Generate a brownfield project template with project metadata.
pub fn populate_brownfield(
    name: &str,
    owner: &str,
    agent: &str,
    tags: &[String],
    description: &str,
    core_value: &str,
) -> String {
    populate_with_template(
        BROWNFIELD_TEMPLATE,
        name,
        owner,
        agent,
        tags,
        description,
        core_value,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_template_not_empty() {
        assert!(!DEFAULT_TEMPLATE.is_empty());
        assert!(DEFAULT_TEMPLATE.contains("{{PROJECT_NAME}}"));
        assert!(DEFAULT_TEMPLATE.contains("## What This Is"));
        assert!(DEFAULT_TEMPLATE.contains("## Requirements"));
        assert!(DEFAULT_TEMPLATE.contains("## Tasks"));
    }

    #[test]
    fn test_brownfield_template_not_empty() {
        assert!(!BROWNFIELD_TEMPLATE.is_empty());
        assert!(BROWNFIELD_TEMPLATE.contains("{{PROJECT_NAME}}"));
        assert!(BROWNFIELD_TEMPLATE.contains("## What This Is"));
        assert!(BROWNFIELD_TEMPLATE.contains("ASSESS"));
    }

    #[test]
    fn test_populate_template() {
        let result = populate_template(
            "My Project",
            "Alice",
            "Hermes",
            &["web".to_string(), "rust".to_string()],
            "A cool project",
            "Fast and reliable.",
        );

        assert!(result.contains("My Project"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Hermes"));
        assert!(result.contains("web, rust"));
        assert!(result.contains("A cool project"));
        assert!(result.contains("Fast and reliable."));
        assert!(!result.contains("{{"));
    }

    #[test]
    fn test_populate_brownfield() {
        let result = populate_brownfield(
            "Legacy App",
            "Bob",
            "Agent",
            &[],
            "Existing app",
            "Maintain stability.",
        );

        assert!(result.contains("Legacy App"));
        assert!(result.contains("Bob"));
        assert!(result.contains("ASSESS"));
        assert!(!result.contains("{{"));
    }

    #[test]
    fn test_populate_empty_tags() {
        let result = populate_template("P", "O", "A", &[], "D", "V");
        assert!(!result.contains("{{TAGS}}"));
    }
}

//! Conformance validation for project.md files.
//!
//! Checks that a project.md file conforms to the expected structure and schema.

use crate::decisions::parse_decisions;
use crate::project::Project;
use crate::requirements::parse_requirements;
use crate::sections::find_section;
use crate::tasks::parse_all_tasks;
use chrono::NaiveDate;

/// Validation result with categorized messages.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationResult {
    /// Returns true if there are no errors.
    pub fn passed(&self) -> bool {
        self.errors.is_empty()
    }

    /// Format a human-readable validation report.
    pub fn report(&self) -> String {
        let mut out = String::new();

        if self.passed() {
            out.push_str("PASS: Project conforms to spec\n");
        } else {
            out.push_str("FAIL: Project has validation errors\n");
        }

        if !self.errors.is_empty() {
            out.push_str(&format!("\nErrors ({}):\n", self.errors.len()));
            for e in &self.errors {
                out.push_str(&format!("  ✗ {}\n", e));
            }
        }

        if !self.warnings.is_empty() {
            out.push_str(&format!("\nWarnings ({}):\n", self.warnings.len()));
            for w in &self.warnings {
                out.push_str(&format!("  ⚠ {}\n", w));
            }
        }

        if !self.info.is_empty() {
            out.push_str(&format!("\nInfo ({}):\n", self.info.len()));
            for i in &self.info {
                out.push_str(&format!("  ℹ {}\n", i));
            }
        }

        out
    }
}

/// Validate a project.md file for conformance with the spec.
pub fn validate(project: &Project) -> ValidationResult {
    let mut result = ValidationResult {
        errors: Vec::new(),
        warnings: Vec::new(),
        info: Vec::new(),
    };

    validate_frontmatter(project, &mut result);
    validate_required_sections(project, &mut result);
    validate_recommended_sections(project, &mut result);
    validate_requirements(project, &mut result);
    validate_tasks(project, &mut result);
    validate_decisions(project, &mut result);
    validate_core_value(project, &mut result);
    validate_current_state(project, &mut result);

    result
}

/// Check frontmatter required fields, status enum, and ISO 8601 dates.
fn validate_frontmatter(project: &Project, result: &mut ValidationResult) {
    let fm = &project.frontmatter;

    // Required fields
    if fm.project.trim().is_empty() {
        result.errors.push("Frontmatter: 'project' field is empty".into());
    }
    if fm.owner.trim().is_empty() {
        result.errors.push("Frontmatter: 'owner' field is empty".into());
    }

    // Check dates are reasonable (not default/epoch)
    let min_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    if fm.created < min_date {
        result.errors.push(format!(
            "Frontmatter: 'created' date {} seems invalid (before 2020)",
            fm.created
        ));
    }
    if fm.updated < min_date {
        result.errors.push(format!(
            "Frontmatter: 'updated' date {} seems invalid (before 2020)",
            fm.updated
        ));
    }
    if fm.updated < fm.created {
        result.errors.push(format!(
            "Frontmatter: 'updated' ({}) is before 'created' ({})",
            fm.updated, fm.created
        ));
    }

    // Status is already validated by serde deserialization, but check it's a known value
    result.info.push(format!("Status: {}", fm.status));

    // Optional field info
    if fm.agent.is_none() {
        result.info.push("No agent specified in frontmatter".into());
    }
    if fm.tags.is_empty() {
        result.info.push("No tags specified".into());
    }
}

/// Check that all required sections are present.
fn validate_required_sections(project: &Project, result: &mut ValidationResult) {
    let required = [
        "What This Is",
        "Core Value",
        "Requirements",
        "Current State",
        "Key Decisions",
        "Tasks",
    ];

    for name in &required {
        if find_section(&project.sections, name).is_none() {
            result.errors.push(format!("Missing required section: '{}'", name));
        }
    }
}

/// Check that recommended sections are present (warnings only).
fn validate_recommended_sections(project: &Project, result: &mut ValidationResult) {
    let recommended = [
        "Context",
        "Constraints",
        "Architecture",
        "Discoveries",
        "References",
        "Session Log",
    ];

    for name in &recommended {
        if find_section(&project.sections, name).is_none() {
            result
                .warnings
                .push(format!("Missing recommended section: '{}'", name));
        }
    }
}

/// Check requirement tiers: all three present.
fn validate_requirements(project: &Project, result: &mut ValidationResult) {
    if let Some(req_section) = find_section(&project.sections, "Requirements") {
        let reqs = parse_requirements(&req_section.content);

        if reqs.validated.is_empty() {
            result.warnings.push(
                "Requirements: No validated requirements found".into(),
            );
        }
        if reqs.active.is_empty() {
            result.warnings.push(
                "Requirements: No active requirements found".into(),
            );
        }

        // Check that tier subsection headers exist in the content
        let content = &req_section.content;
        if !content.contains("### Validated") && !content.contains("### validated") {
            result
                .errors
                .push("Requirements: Missing '### Validated' tier".into());
        }
        if !content.contains("### Active") && !content.contains("### active") {
            result
                .errors
                .push("Requirements: Missing '### Active' tier".into());
        }
        if !content.contains("### Out of Scope") && !content.contains("### out of scope") {
            result
                .errors
                .push("Requirements: Missing '### Out of Scope' tier".into());
        }

        result.info.push(format!(
            "Requirements: {} validated, {} active, {} out of scope",
            reqs.validated.len(),
            reqs.active.len(),
            reqs.out_of_scope.len()
        ));
    }
}

/// Check task syntax: count - [ ], - [x], - [!].
fn validate_tasks(project: &Project, result: &mut ValidationResult) {
    if let Some(tasks_section) = find_section(&project.sections, "Tasks") {
        let all_tasks = parse_all_tasks(&project.sections);

        let mut total_pending = 0;
        let mut total_done = 0;
        let mut total_blocked = 0;

        for (_phase, tasks) in &all_tasks {
            for task in tasks {
                match task.status {
                    crate::tasks::TaskStatus::Pending => total_pending += 1,
                    crate::tasks::TaskStatus::Done => total_done += 1,
                    crate::tasks::TaskStatus::Blocked => total_blocked += 1,
                }
            }
        }

        if all_tasks.is_empty() {
            result
                .warnings
                .push("Tasks: No task phases found".into());
        }

        result.info.push(format!(
            "Tasks: {} done, {} pending, {} blocked (across {} phases)",
            total_done,
            total_pending,
            total_blocked,
            all_tasks.len()
        ));

        // Check for phase headers
        if !tasks_section.content.contains("### Phase:") {
            result
                .warnings
                .push("Tasks: No '### Phase:' subsections found".into());
        }
    }
}

/// Check decisions table has Decision, Rationale, Outcome columns.
fn validate_decisions(project: &Project, result: &mut ValidationResult) {
    if let Some(dec_section) = find_section(&project.sections, "Key Decisions") {
        let decisions = parse_decisions(&dec_section.content);

        if decisions.is_empty() {
            result
                .warnings
                .push("Key Decisions: No decisions found in table".into());
        }

        // Check for table header
        let content = &dec_section.content;
        if !content.contains("Decision") || !content.contains("Rationale") || !content.contains("Outcome") {
            result.errors.push(
                "Key Decisions: Table missing required columns (Decision, Rationale, Outcome)"
                    .into(),
            );
        }

        let good = decisions.iter().filter(|d| d.outcome == crate::decisions::Outcome::Good).count();
        let pending = decisions.iter().filter(|d| d.outcome == crate::decisions::Outcome::Pending).count();
        let revisit = decisions.iter().filter(|d| d.outcome == crate::decisions::Outcome::Revisit).count();

        result.info.push(format!(
            "Decisions: {} total ({} good, {} pending, {} revisit)",
            decisions.len(),
            good,
            pending,
            revisit
        ));
    }
}

/// Check Core Value conciseness (roughly 1-3 sentences).
fn validate_core_value(project: &Project, result: &mut ValidationResult) {
    if let Some(section) = find_section(&project.sections, "Core Value") {
        let content = section.content.trim();
        if content.is_empty() {
            result.errors.push("Core Value: Section is empty".into());
            return;
        }

        // Count sentences (rough heuristic: count sentence-ending punctuation)
        let sentence_count = content
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count();

        if sentence_count == 0 {
            result
                .warnings
                .push("Core Value: No sentence-ending punctuation found — should be 1-3 sentences".into());
        } else if sentence_count > 3 {
            result.warnings.push(format!(
                "Core Value: Found ~{} sentences — should be concise (1-3 sentences)",
                sentence_count
            ));
        }

        if content.len() > 500 {
            result.warnings.push(format!(
                "Core Value: {} characters — consider being more concise",
                content.len()
            ));
        }
    }
}

/// Check Current State completeness.
fn validate_current_state(project: &Project, result: &mut ValidationResult) {
    if let Some(section) = find_section(&project.sections, "Current State") {
        let content = &section.content;

        let required_fields = ["Phase", "Last completed", "Next action", "Blockers"];
        for field in &required_fields {
            let pattern = format!("**{}:**", field);
            if !content.contains(&pattern) {
                result.errors.push(format!(
                    "Current State: Missing required field '{}'",
                    field
                ));
            }
        }

        // Check if Phase value is set
        if let Some(pos) = content.find("**Phase:**") {
            let after = &content[pos + "**Phase:**".len()..];
            let value = after.lines().next().unwrap_or("").trim();
            if value.is_empty() {
                result.errors.push("Current State: Phase value is empty".into());
            }
        }

        // Check if Blockers is explicitly stated
        if let Some(pos) = content.find("**Blockers:**") {
            let after = &content[pos + "**Blockers:**".len()..];
            let value = after.lines().next().unwrap_or("").trim();
            if value.is_empty() {
                result
                    .warnings
                    .push("Current State: Blockers field is empty — specify 'None' if no blockers".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const VALID_PROJECT: &str = r#"---
project: "Test Project"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Test Owner"
agent: "Test Agent"
tags: [test]
---

## What This Is

A test project for validation.

## Core Value

Fast, reliable testing.

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
**Next action:** Do the thing
**Blockers:** None
**Notes:** Some notes

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

## Discoveries

Some discoveries.

## References

Some references.

## Session Log

- **2026-01-01** — Project started.
"#;

    #[test]
    fn test_valid_project_passes() {
        let project = Project::from_str(VALID_PROJECT, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        assert!(result.passed(), "Expected pass, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_missing_required_sections() {
        let content = r#"---
project: "Test"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Owner"
---

## What This Is

Something.
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        assert!(!result.passed());
        assert!(result.errors.iter().any(|e| e.contains("Core Value")));
        assert!(result.errors.iter().any(|e| e.contains("Requirements")));
        assert!(result.errors.iter().any(|e| e.contains("Current State")));
        assert!(result.errors.iter().any(|e| e.contains("Key Decisions")));
        assert!(result.errors.iter().any(|e| e.contains("Tasks")));
    }

    #[test]
    fn test_missing_recommended_sections() {
        let content = r#"---
project: "Test"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Owner"
---

## What This Is

Something.

## Core Value

Short value.

## Requirements

### Validated

- ✓ A — v0.1

### Active

- [ ] B

### Out of Scope

- C — nope

## Current State

**Phase:** build
**Last completed:** Task 1
**Next action:** Do thing
**Blockers:** None

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| A | B | ✓ Good |

## Tasks

### Phase: BUILD

- [ ] Task 1: Something
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        // Should have warnings for missing recommended sections
        assert!(result.warnings.iter().any(|w| w.contains("Context")));
        assert!(result.warnings.iter().any(|w| w.contains("Constraints")));
        assert!(result.warnings.iter().any(|w| w.contains("Architecture")));
        assert!(result.warnings.iter().any(|w| w.contains("Discoveries")));
        assert!(result.warnings.iter().any(|w| w.contains("References")));
        assert!(result.warnings.iter().any(|w| w.contains("Session Log")));
    }

    #[test]
    fn test_empty_project_name() {
        let content = r#"---
project: ""
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Owner"
---

## What This Is

Something.

## Core Value

Fast.

## Requirements

### Validated
- ✓ A — v0.1
### Active
- [ ] B
### Out of Scope
- C — nope

## Current State

**Phase:** build
**Last completed:** T1
**Next action:** T2
**Blockers:** None

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| A | B | ✓ Good |

## Tasks

### Phase: BUILD
- [ ] T1
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        assert!(result.errors.iter().any(|e| e.contains("'project' field is empty")));
    }

    #[test]
    fn test_updated_before_created() {
        let content = r#"---
project: "Test"
status: build
created: 2026-04-29
updated: 2026-01-01
owner: "Owner"
---

## What This Is

Something.

## Core Value

Fast.

## Requirements

### Validated
- ✓ A — v0.1
### Active
- [ ] B
### Out of Scope
- C — nope

## Current State

**Phase:** build
**Last completed:** T1
**Next action:** T2
**Blockers:** None

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| A | B | ✓ Good |

## Tasks

### Phase: BUILD
- [ ] T1
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        assert!(result.errors.iter().any(|e| e.contains("'updated'") && e.contains("before 'created'")));
    }

    #[test]
    fn test_validation_report_format() {
        let project = Project::from_str(VALID_PROJECT, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        let report = result.report();
        assert!(report.contains("PASS"));
    }

    #[test]
    fn test_fail_report_format() {
        let content = r#"---
project: "Test"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Owner"
---
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        let report = result.report();
        assert!(report.contains("FAIL"));
        assert!(report.contains("Errors"));
    }

    #[test]
    fn test_current_state_missing_fields() {
        let content = r#"---
project: "Test"
status: build
created: 2026-01-01
updated: 2026-04-29
owner: "Owner"
---

## What This Is

Something.

## Core Value

Fast.

## Requirements

### Validated
- ✓ A — v0.1
### Active
- [ ] B
### Out of Scope
- C — nope

## Current State

**Phase:** build

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| A | B | ✓ Good |

## Tasks

### Phase: BUILD
- [ ] T1
"#;
        let project = Project::from_str(content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        assert!(result.errors.iter().any(|e| e.contains("Last completed")));
        assert!(result.errors.iter().any(|e| e.contains("Next action")));
        assert!(result.errors.iter().any(|e| e.contains("Blockers")));
    }

    #[test]
    fn test_validate_example_cli_tool() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let project = Project::from_str(&content, Path::new("/tmp/test.md")).unwrap();
        let result = validate(&project);
        // The example should pass validation
        assert!(
            result.passed(),
            "Example should pass, got errors: {:?}",
            result.errors
        );
    }
}

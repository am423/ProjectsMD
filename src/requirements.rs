//! Three-tier requirements management.
//!
//! Parses and manages Validated / Active / Out of Scope requirements
//! from the Requirements section of a project.md file.

/// A validated (confirmed) requirement.
#[derive(Debug, Clone)]
pub struct ValidatedReq {
    pub description: String,
    pub version: String,
}

/// An active (in-progress) requirement.
#[derive(Debug, Clone)]
pub struct ActiveReq {
    pub description: String,
    pub sub_items: Vec<String>,
}

/// A requirement that has been moved out of scope.
#[derive(Debug, Clone)]
pub struct OutOfScopeItem {
    pub description: String,
    pub reason: String,
}

/// All requirements from a project.md file.
#[derive(Debug, Clone)]
pub struct Requirements {
    pub validated: Vec<ValidatedReq>,
    pub active: Vec<ActiveReq>,
    pub out_of_scope: Vec<OutOfScopeItem>,
}

/// Parse the Requirements section content into a structured form.
///
/// Expects content that may contain `### Validated`, `### Active`, and
/// `### Out of Scope` subsections.
pub fn parse_requirements(content: &str) -> Requirements {
    let mut reqs = Requirements {
        validated: Vec::new(),
        active: Vec::new(),
        out_of_scope: Vec::new(),
    };

    let mut current_tier = "";

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("### Validated")
            || trimmed.eq_ignore_ascii_case("### validated")
        {
            current_tier = "validated";
            continue;
        } else if trimmed.eq_ignore_ascii_case("### Active")
            || trimmed.eq_ignore_ascii_case("### active")
        {
            current_tier = "active";
            continue;
        } else if trimmed.eq_ignore_ascii_case("### Out of Scope")
            || trimmed.eq_ignore_ascii_case("### out of scope")
        {
            current_tier = "out_of_scope";
            continue;
        } else if trimmed.starts_with("### ") || trimmed.starts_with("## ") {
            // Different subsection, stop parsing requirements
            current_tier = "";
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        match current_tier {
            "validated" => {
                // Format: "- ✓ Description — v0.1"
                if let Some(rest) = trimmed.strip_prefix("- ✓ ") {
                    let parts: Vec<&str> = rest.split(" — ").collect();
                    if parts.len() == 2 {
                        reqs.validated.push(ValidatedReq {
                            description: parts[0].trim().to_string(),
                            version: parts[1].trim().to_string(),
                        });
                    } else {
                        reqs.validated.push(ValidatedReq {
                            description: rest.to_string(),
                            version: String::new(),
                        });
                    }
                } else if let Some(rest) = trimmed.strip_prefix("- ✅ ") {
                    let parts: Vec<&str> = rest.split(" — ").collect();
                    if parts.len() == 2 {
                        reqs.validated.push(ValidatedReq {
                            description: parts[0].trim().to_string(),
                            version: parts[1].trim().to_string(),
                        });
                    } else {
                        reqs.validated.push(ValidatedReq {
                            description: rest.to_string(),
                            version: String::new(),
                        });
                    }
                }
            }
            "active" => {
                // Format: "- [ ] Description"
                if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                    reqs.active.push(ActiveReq {
                        description: rest.trim().to_string(),
                        sub_items: Vec::new(),
                    });
                } else if trimmed.starts_with("  ") || trimmed.starts_with("\t") {
                    // Sub-item of the last active requirement
                    if let Some(last) = reqs.active.last_mut() {
                        let sub = trimmed.trim();
                        if !sub.is_empty() {
                            last.sub_items.push(sub.to_string());
                        }
                    }
                }
            }
            "out_of_scope" => {
                // Format: "- Description — reason"
                if trimmed.starts_with("- ") {
                    let rest = &trimmed[2..];
                    let parts: Vec<&str> = rest.split(" — ").collect();
                    if parts.len() >= 2 {
                        reqs.out_of_scope.push(OutOfScopeItem {
                            description: parts[0].trim().to_string(),
                            reason: parts[1..].join(" — ").trim().to_string(),
                        });
                    } else {
                        reqs.out_of_scope.push(OutOfScopeItem {
                            description: rest.to_string(),
                            reason: String::new(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    reqs
}

/// Promote an active requirement to validated status.
///
/// `index` is the index into `reqs.active`. Returns `true` on success.
pub fn promote_to_validated(reqs: &mut Requirements, index: usize, version: &str) -> bool {
    if index >= reqs.active.len() {
        return false;
    }
    let active = reqs.active.remove(index);
    reqs.validated.push(ValidatedReq {
        description: active.description,
        version: version.to_string(),
    });
    true
}

/// Move an active requirement to out of scope.
///
/// `index` is the index into `reqs.active`. Returns `true` on success.
pub fn move_to_out_of_scope(reqs: &mut Requirements, index: usize, reason: &str) -> bool {
    if index >= reqs.active.len() {
        return false;
    }
    let active = reqs.active.remove(index);
    reqs.out_of_scope.push(OutOfScopeItem {
        description: active.description,
        reason: reason.to_string(),
    });
    true
}

/// Serialize requirements back to markdown format.
pub fn write_requirements(reqs: &Requirements) -> String {
    let mut out = String::new();

    if !reqs.validated.is_empty() {
        out.push_str("### Validated\n\n");
        for v in &reqs.validated {
            if v.version.is_empty() {
                out.push_str(&format!("- ✓ {}\n", v.description));
            } else {
                out.push_str(&format!("- ✓ {} — {}\n", v.description, v.version));
            }
        }
        out.push('\n');
    }

    if !reqs.active.is_empty() {
        out.push_str("### Active\n\n");
        for a in &reqs.active {
            out.push_str(&format!("- [ ] {}\n", a.description));
            for sub in &a.sub_items {
                out.push_str(&format!("  {}\n", sub));
            }
        }
        out.push('\n');
    }

    if !reqs.out_of_scope.is_empty() {
        out.push_str("### Out of Scope\n\n");
        for o in &reqs.out_of_scope {
            out.push_str(&format!("- {} — {}\n", o.description, o.reason));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_REQS: &str = r#"### Validated

- ✓ Current weather by city name — v0.1
- ✓ File-based caching with 15-min TTL — v0.1

### Active

- [ ] Color-coded terminal output (blue <10°C, green 10-25°C, red >25°C)
- [ ] CLI argument parsing (city, --unit, --no-cache, --help)
- [ ] Graceful error handling for network/API failures
- [ ] Cross-platform binary builds (Linux, macOS, Windows)
- [ ] README with install instructions and usage examples

### Out of Scope

- Forecast data — adds API complexity, different use case entirely
- GUI / web interface — against core value (terminal-first)
- Historical weather data — separate project, different API endpoints
- User accounts — no persistence needed for a CLI tool
- Multiple city display — one city per invocation, keep it simple
"#;

    #[test]
    fn test_parse_requirements() {
        let reqs = parse_requirements(EXAMPLE_REQS);
        assert_eq!(reqs.validated.len(), 2);
        assert_eq!(reqs.validated[0].description, "Current weather by city name");
        assert_eq!(reqs.validated[0].version, "v0.1");
        assert_eq!(reqs.active.len(), 5);
        assert_eq!(
            reqs.active[0].description,
            "Color-coded terminal output (blue <10°C, green 10-25°C, red >25°C)"
        );
        assert_eq!(reqs.out_of_scope.len(), 5);
        assert_eq!(reqs.out_of_scope[0].description, "Forecast data");
        assert_eq!(
            reqs.out_of_scope[0].reason,
            "adds API complexity, different use case entirely"
        );
    }

    #[test]
    fn test_promote_to_validated() {
        let mut reqs = parse_requirements(EXAMPLE_REQS);
        assert!(promote_to_validated(&mut reqs, 0, "v0.2"));
        assert_eq!(reqs.validated.len(), 3);
        assert_eq!(reqs.validated[2].version, "v0.2");
        assert_eq!(reqs.active.len(), 4);
    }

    #[test]
    fn test_move_to_out_of_scope() {
        let mut reqs = parse_requirements(EXAMPLE_REQS);
        assert!(move_to_out_of_scope(
            &mut reqs,
            0,
            "too complex for v1"
        ));
        assert_eq!(reqs.active.len(), 4);
        assert_eq!(reqs.out_of_scope.len(), 6);
    }

    #[test]
    fn test_promote_invalid_index() {
        let mut reqs = parse_requirements(EXAMPLE_REQS);
        assert!(!promote_to_validated(&mut reqs, 99, "v1"));
    }

    #[test]
    fn test_write_requirements() {
        let reqs = parse_requirements(EXAMPLE_REQS);
        let output = write_requirements(&reqs);
        assert!(output.contains("### Validated"));
        assert!(output.contains("### Active"));
        assert!(output.contains("### Out of Scope"));
        assert!(output.contains("- ✓ Current weather by city name — v0.1"));
        assert!(output.contains("- [ ] Color-coded terminal output"));
    }

    #[test]
    fn test_roundtrip() {
        let reqs = parse_requirements(EXAMPLE_REQS);
        let output = write_requirements(&reqs);
        let reqs2 = parse_requirements(&output);
        assert_eq!(reqs.validated.len(), reqs2.validated.len());
        assert_eq!(reqs.active.len(), reqs2.active.len());
        assert_eq!(reqs.out_of_scope.len(), reqs2.out_of_scope.len());
    }

    #[test]
    fn test_parse_example_cli_tool() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let (_, body) = crate::frontmatter::parse_frontmatter(&content)
            .unwrap()
            .unwrap();
        let sections = crate::sections::parse_sections(body);
        let req_section = crate::sections::find_section(&sections, "Requirements").unwrap();
        let reqs = parse_requirements(&req_section.content);
        assert_eq!(reqs.validated.len(), 2);
        assert_eq!(reqs.active.len(), 5);
        assert_eq!(reqs.out_of_scope.len(), 5);
    }
}

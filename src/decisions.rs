//! Key Decisions table management.
//!
//! Parses and manages the Key Decisions section of a project.md file.
//! Decisions are stored as a markdown table.

/// Decision outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Good,
    Revisit,
    Pending,
    Unset,
}

impl Outcome {
    /// Parse an outcome from a markdown table cell.
    pub fn from_cell(s: &str) -> Self {
        let s = s.trim();
        if s.contains("✓") || s.eq_ignore_ascii_case("good") || s.contains("✓ Good") {
            Outcome::Good
        } else if s.contains("⚠") || s.eq_ignore_ascii_case("revisit") || s.contains("⚠ Revisit")
        {
            Outcome::Revisit
        } else if s.contains("—") || s.eq_ignore_ascii_case("pending") || s.contains("— Pending")
        {
            Outcome::Pending
        } else {
            Outcome::Unset
        }
    }

    /// Format the outcome for a markdown table cell.
    pub fn to_cell(&self) -> &str {
        match self {
            Outcome::Good => "✓ Good",
            Outcome::Revisit => "⚠ Revisit",
            Outcome::Pending => "— Pending",
            Outcome::Unset => "",
        }
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_cell())
    }
}

/// A single key decision.
#[derive(Debug, Clone)]
pub struct Decision {
    pub decision: String,
    pub rationale: String,
    pub outcome: Outcome,
}

/// Parse the Key Decisions section content as a markdown table.
///
/// Expects a markdown table with columns: Decision | Rationale | Outcome.
pub fn parse_decisions(content: &str) -> Vec<Decision> {
    let mut decisions = Vec::new();
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip the separator line (|---|---|---|)
        if trimmed.starts_with("|") && trimmed.contains("---") && !trimmed.contains("Decision") {
            // Check if this is the separator (all dashes/cells)
            let is_separator = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .all(|s| s.trim().chars().all(|c| c == '-' || c == ' '));
            if is_separator {
                continue;
            }
        }

        // Skip header row
        if trimmed.starts_with("|") && trimmed.contains("Decision") {
            header_seen = true;
            continue;
        }

        // Parse table rows
        if trimmed.starts_with("|") && header_seen {
            let cells: Vec<&str> = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim())
                .collect();

            if cells.len() >= 3 {
                decisions.push(Decision {
                    decision: cells[0].to_string(),
                    rationale: cells[1].to_string(),
                    outcome: Outcome::from_cell(cells[2]),
                });
            } else if cells.len() == 2 {
                decisions.push(Decision {
                    decision: cells[0].to_string(),
                    rationale: cells[1].to_string(),
                    outcome: Outcome::Unset,
                });
            }
        }
    }

    decisions
}

/// Add a new decision to the list.
pub fn add_decision(
    decisions: &mut Vec<Decision>,
    decision: &str,
    rationale: &str,
    outcome: Outcome,
) {
    decisions.push(Decision {
        decision: decision.to_string(),
        rationale: rationale.to_string(),
        outcome,
    });
}

/// Update the outcome of a decision by index.
/// Returns `true` if the index is valid.
pub fn update_outcome(decisions: &mut [Decision], index: usize, outcome: Outcome) -> bool {
    if let Some(d) = decisions.get_mut(index) {
        d.outcome = outcome;
        true
    } else {
        false
    }
}

/// Serialize decisions back to a markdown table.
pub fn write_decisions(decisions: &[Decision]) -> String {
    if decisions.is_empty() {
        return String::new();
    }

    // Calculate column widths
    let mut w_decision = "Decision".len();
    let mut w_rationale = "Rationale".len();
    let mut w_outcome = "Outcome".len();

    for d in decisions {
        w_decision = w_decision.max(d.decision.len());
        w_rationale = w_rationale.max(d.rationale.len());
        w_outcome = w_outcome.max(d.outcome.to_cell().len());
    }

    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "| {:<w1$} | {:<w2$} | {:<w3$} |\n",
        "Decision",
        "Rationale",
        "Outcome",
        w1 = w_decision,
        w2 = w_rationale,
        w3 = w_outcome,
    ));

    // Separator
    out.push_str(&format!(
        "|{:-<w1$}|{:-<w2$}|{:-<w3$}|\n",
        "",
        "",
        "",
        w1 = w_decision + 2,
        w2 = w_rationale + 2,
        w3 = w_outcome + 2,
    ));

    // Rows
    for d in decisions {
        out.push_str(&format!(
            "| {:<w1$} | {:<w2$} | {:<w3$} |\n",
            d.decision,
            d.rationale,
            d.outcome.to_cell(),
            w1 = w_decision,
            w2 = w_rationale,
            w3 = w_outcome,
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_DECISIONS: &str = r#"| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Go over Python | Single binary, no runtime deps, fast startup (<50ms) | ✓ Good |
| OpenWeatherMap API | Free tier sufficient, well-documented, stable | ✓ Good |
| File-based cache over in-memory | Persists across CLI invocations, no daemon needed | ✓ Good |
| Atomic rename for cache writes | Avoids corrupted reads on Windows | ✓ Good |
| fatih/color for terminal colors | Handles Windows PowerShell ANSI, well-tested | — Pending |
| Add --unit flag | Both Celsius and Fahrenheit users exist | — Pending |
"#;

    #[test]
    fn test_parse_decisions() {
        let decisions = parse_decisions(EXAMPLE_DECISIONS);
        assert_eq!(decisions.len(), 6);
        assert_eq!(decisions[0].decision, "Go over Python");
        assert_eq!(
            decisions[0].rationale,
            "Single binary, no runtime deps, fast startup (<50ms)"
        );
        assert_eq!(decisions[0].outcome, Outcome::Good);
        assert_eq!(decisions[4].outcome, Outcome::Pending);
    }

    #[test]
    fn test_add_decision() {
        let mut decisions = parse_decisions(EXAMPLE_DECISIONS);
        add_decision(&mut decisions, "New choice", "Because reasons", Outcome::Pending);
        assert_eq!(decisions.len(), 7);
        assert_eq!(decisions[6].decision, "New choice");
    }

    #[test]
    fn test_update_outcome() {
        let mut decisions = parse_decisions(EXAMPLE_DECISIONS);
        assert!(update_outcome(&mut decisions, 4, Outcome::Good));
        assert_eq!(decisions[4].outcome, Outcome::Good);
        assert!(!update_outcome(&mut decisions, 99, Outcome::Good));
    }

    #[test]
    fn test_write_decisions() {
        let decisions = parse_decisions(EXAMPLE_DECISIONS);
        let output = write_decisions(&decisions);
        assert!(output.contains("Decision"));
        assert!(output.contains("Go over Python"));
        assert!(output.contains("✓ Good"));
        assert!(output.contains("— Pending"));
    }

    #[test]
    fn test_roundtrip() {
        let decisions = parse_decisions(EXAMPLE_DECISIONS);
        let output = write_decisions(&decisions);
        let decisions2 = parse_decisions(&output);
        assert_eq!(decisions.len(), decisions2.len());
        for (d1, d2) in decisions.iter().zip(decisions2.iter()) {
            assert_eq!(d1.decision, d2.decision);
            assert_eq!(d1.outcome, d2.outcome);
        }
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
        let dec_section = crate::sections::find_section(&sections, "Key Decisions").unwrap();
        let decisions = parse_decisions(&dec_section.content);
        assert_eq!(decisions.len(), 6);
    }
}

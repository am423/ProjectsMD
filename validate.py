#!/usr/bin/env python3
"""
project.md validator — checks conformance with the project.md specification.

Usage:
    python3 validate.py [path/to/project.md]

If no path given, looks for project.md in the current directory.
"""

import re
import sys
import os

try:
    import yaml
    HAS_YAML = True
except ImportError:
    HAS_YAML = False


REQUIRED_FRONTMATTER = ["project", "status", "created", "updated", "owner"]
VALID_STATUSES = {"define", "design", "build", "verify", "ship", "paused", "archived"}
REQUIRED_SECTIONS = [
    "What This Is",
    "Core Value",
    "Requirements",
    "Current State",
    "Key Decisions",
    "Tasks",
]
RECOMMENDED_SECTIONS = [
    "Context",
    "Constraints",
    "Architecture",
    "Discoveries",
    "References",
    "Session Log",
]
REQUIREMENT_TIERS = ["Validated", "Active", "Out of Scope"]


class ValidationResult:
    def __init__(self):
        self.errors = []
        self.warnings = []
        self.info = []

    def error(self, msg):
        self.errors.append(msg)

    def warn(self, msg):
        self.warnings.append(msg)

    def ok(self, msg):
        self.info.append(msg)

    @property
    def passed(self):
        return len(self.errors) == 0

    def report(self):
        lines = []
        lines.append("=" * 60)
        lines.append("  project.md Validation Report")
        lines.append("=" * 60)
        lines.append("")

        if self.info:
            lines.append("PASS:")
            for msg in self.info:
                lines.append(f"  + {msg}")
            lines.append("")

        if self.warnings:
            lines.append("WARNINGS:")
            for msg in self.warnings:
                lines.append(f"  ! {msg}")
            lines.append("")

        if self.errors:
            lines.append("ERRORS:")
            for msg in self.errors:
                lines.append(f"  x {msg}")
            lines.append("")

        lines.append("-" * 60)
        if self.passed:
            lines.append(f"  RESULT: PASS ({len(self.warnings)} warnings)")
        else:
            lines.append(f"  RESULT: FAIL ({len(self.errors)} errors, {len(self.warnings)} warnings)")
        lines.append("-" * 60)

        return "\n".join(lines)


def parse_frontmatter(content):
    """Extract YAML frontmatter from markdown content."""
    match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if not match:
        return None, content

    raw = match.group(1)
    body = content[match.end():]

    if HAS_YAML:
        try:
            return yaml.safe_load(raw), body
        except yaml.YAMLError:
            return None, body
    else:
        # Basic parsing without PyYAML
        fm = {}
        for line in raw.split("\n"):
            line = line.strip()
            if ":" in line and not line.startswith("#"):
                key, _, value = line.partition(":")
                key = key.strip()
                value = value.strip().strip('"').strip("'")
                if value.startswith("[") and value.endswith("]"):
                    # Basic array parsing
                    value = [v.strip().strip('"').strip("'")
                             for v in value[1:-1].split(",") if v.strip()]
                fm[key] = value
        return fm, body


def find_sections(body):
    """Find all ## headings in the document."""
    sections = {}
    for match in re.finditer(r"^##\s+(.+)$", body, re.MULTILINE):
        name = match.group(1).strip()
        sections[name] = match.start()
    return sections


def validate(filepath):
    result = ValidationResult()

    if not os.path.exists(filepath):
        result.error(f"File not found: {filepath}")
        return result

    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    # --- Frontmatter ---
    fm, body = parse_frontmatter(content)

    if fm is None:
        result.error("Missing or invalid YAML frontmatter (must start with ---)")
        return result

    result.ok("YAML frontmatter present")

    for field in REQUIRED_FRONTMATTER:
        if field not in fm:
            result.error(f"Missing required frontmatter field: {field}")
        elif not fm[field]:
            result.error(f"Empty required frontmatter field: {field}")
        else:
            result.ok(f"Frontmatter field '{field}' = {fm[field]}")

    # Validate status
    if "status" in fm and fm["status"]:
        if fm["status"] not in VALID_STATUSES:
            result.error(f"Invalid status '{fm['status']}'. Must be one of: {', '.join(sorted(VALID_STATUSES))}")
        else:
            result.ok(f"Status '{fm['status']}' is valid")

    # Validate dates
    for date_field in ["created", "updated"]:
        if date_field in fm and fm[date_field]:
            date_str = str(fm[date_field])
            if not re.match(r"^\d{4}-\d{2}-\d{2}$", date_str):
                result.warn(f"Field '{date_field}' is not ISO 8601 format (YYYY-MM-DD): {date_str}")
            else:
                result.ok(f"Field '{date_field}' is valid ISO 8601: {date_str}")

    # --- Sections ---
    sections = find_sections(body)

    for section in REQUIRED_SECTIONS:
        if section in sections:
            result.ok(f"Required section found: {section}")
        else:
            result.error(f"Missing required section: ## {section}")

    for section in RECOMMENDED_SECTIONS:
        if section in sections:
            result.ok(f"Recommended section found: {section}")
        else:
            result.warn(f"Missing recommended section: ## {section}")

    # --- Requirements lifecycle ---
    if "Requirements" in sections:
        has_validated = "Validated" in body
        has_active = "Active" in body
        has_out_of_scope = "Out of Scope" in body

        if has_validated:
            result.ok("Requirements tier found: Validated")
        else:
            result.warn("Requirements tier missing: Validated (expected for conformance)")

        if has_active:
            result.ok("Requirements tier found: Active")
        else:
            result.error("Requirements tier missing: Active")

        if has_out_of_scope:
            result.ok("Requirements tier found: Out of Scope")
        else:
            result.error("Requirements tier missing: Out of Scope")

    # --- Tasks ---
    tasks_pending = len(re.findall(r"^- \[ \]", body, re.MULTILINE))
    tasks_done = len(re.findall(r"^- \[x\]", body, re.MULTILINE))
    tasks_blocked = len(re.findall(r"^- \[!\]", body, re.MULTILINE))
    total_tasks = tasks_pending + tasks_done + tasks_blocked

    if total_tasks == 0:
        result.warn("No tasks found (expected checkbox syntax: - [ ], - [x], - [!])")
    else:
        result.ok(f"Tasks found: {total_tasks} total ({tasks_done} done, {tasks_pending} pending, {tasks_blocked} blocked)")

    # --- Key Decisions ---
    if "Key Decisions" in sections:
        # Check for table format
        table_match = re.search(r"\| Decision.*\|.*Rationale.*\|.*Outcome.*\|", body)
        if table_match:
            result.ok("Key Decisions table has correct columns (Decision, Rationale, Outcome)")
        else:
            result.warn("Key Decisions section exists but table may be missing columns (expected: Decision | Rationale | Outcome)")

        # Check for outcome tracking
        has_good = "Good" in body or "✓" in body
        has_revisit = "Revisit" in body or "⚠️" in body
        has_pending = "Pending" in body or "—" in body
        if has_good or has_revisit or has_pending:
            result.ok("Decision outcome tracking detected")

    # --- Core Value ---
    if "Core Value" in sections:
        # Check it's roughly one sentence (not too long)
        cv_start = sections["Core Value"]
        next_section = None
        for name, pos in sections.items():
            if pos > cv_start and name != "Core Value":
                if next_section is None or pos < next_section:
                    next_section = pos

        if next_section:
            cv_text = body[cv_start:next_section].strip()
            # Remove the heading
            cv_text = re.sub(r"^## Core Value\s*", "", cv_text).strip()
            cv_lines = [l for l in cv_text.split("\n") if l.strip() and not l.strip().startswith("<!--")]
            if cv_lines:
                cv_content = " ".join(cv_lines)
                sentence_count = len(re.findall(r"[.!?]+", cv_content))
                if sentence_count > 3:
                    result.warn(f"Core Value has {sentence_count} sentences — should be ~1 sentence")
                else:
                    result.ok("Core Value is concise")

    # --- What This Is ---
    if "What This Is" in sections:
        wti_start = sections["What This Is"]
        next_section = None
        for name, pos in sections.items():
            if pos > wti_start and name != "What This Is":
                if next_section is None or pos < next_section:
                    next_section = pos

        if next_section:
            wti_text = body[wti_start:next_section].strip()
            wti_text = re.sub(r"^## What This Is\s*", "", wti_text).strip()
            wti_lines = [l for l in wti_text.split("\n") if l.strip() and not l.strip().startswith("<!--")]
            if wti_lines:
                wti_content = " ".join(wti_lines)
                if len(wti_content) > 500:
                    result.warn(f"'What This Is' is {len(wti_content)} chars — should be 2-3 sentences")
                else:
                    result.ok("'What This Is' length is appropriate")

    # --- Current State ---
    if "Current State" in sections:
        cs_markers = ["Phase:", "Last completed:", "Next action:", "Blockers:"]
        found = sum(1 for m in cs_markers if m in body)
        if found >= 3:
            result.ok(f"Current State has {found}/{len(cs_markers)} expected fields")
        else:
            result.warn(f"Current State only has {found}/{len(cs_markers)} expected fields (Phase, Last completed, Next action, Blockers)")

    return result


def main():
    if len(sys.argv) > 1:
        filepath = sys.argv[1]
    else:
        filepath = "project.md"

    result = validate(filepath)
    print(result.report())
    sys.exit(0 if result.passed else 1)


if __name__ == "__main__":
    main()

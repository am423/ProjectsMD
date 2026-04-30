# project.md Standard — Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Define and publish the project.md standard — a single-file project management format for agent-human collaboration that captures everything from scoping through execution, enabling seamless session resumption.

**Architecture:** A markdown specification document defining the project.md format, plus a reference implementation (template + agent skill) that any agent framework can adopt.

**Tech Stack:** Markdown specification, YAML frontmatter, agent skill definitions

---

## Research Summary

### From obra/superpowers

Superpowers uses a multi-file pipeline: brainstorming produces a design spec, writing-plans produces an implementation plan, executing-plans/subagent-driven-development handles execution. Key patterns worth adopting:

- **Brainstorming gate:** No implementation until design is approved (HARD-GATE)
- **Bite-sized tasks:** Each step is 2-5 minutes, one action
- **No placeholders:** Every step has actual content — no "TBD" or "fill in later"
- **Verification before claims:** Evidence before assertions, always
- **Spec compliance + code quality review:** Two-stage review after each task
- **Self-review loops:** Check for placeholders, contradictions, ambiguity, scope

What superpowers does NOT solve (and project.md should):
- No single-file project state — context is spread across specs/, plans/, and session memory
- No cross-session resume — you need to re-read multiple files to understand where you are
- No human-readable progress dashboard — progress lives in todo lists and git history
- No explicit "what we learned" capture — decisions and discoveries are ephemeral

### From First Principles (Project Management)

Essential elements every project needs:

1. **Why** — Problem statement, motivation, success criteria
2. **What** — Scope, features, acceptance criteria, constraints
3. **How** — Architecture, approach, technology decisions
4. **Who** — Roles (human = decision-maker, agent = executor)
5. **When** — Milestones, task breakdown, dependencies, timeline
6. **Where are we** — Progress tracking, current state, blockers
7. **What did we learn** — Decisions made, discoveries, course corrections

The best project documents are:
- **Living** — updated as the project evolves, not written once and forgotten
- **Self-contained** — anyone (human or agent) can pick up from the file alone
- **Layered** — high-level summary for quick orientation, deep detail for execution
- **Machine-readable** — structured enough for agents to parse programmatically

---

## The project.md Standard

### Core Innovation

Superpowers uses 3+ files (spec, plan, execution state). project.md collapses these into ONE living document that serves as:
- The human's project brief
- The agent's execution context
- The team's shared memory
- The progress dashboard

### Design Decisions

**Decision 1: Single file, not a directory**
- Rationale: A directory fragments context. A single file forces discipline and makes "pick up where we left off" trivial — just read one file.
- Tradeoff: Large projects may produce long files. Mitigated by clear section structure and collapsible task details.

**Decision 2: YAML frontmatter for metadata**
- Rationale: Machine-parseable status, dates, tags. Agents can read frontmatter without parsing the full document.
- Structure: project name, status, created/updated dates, owner, tags.

**Decision 3: Checkbox task tracking in the document itself**
- Rationale: Progress is visible at a glance. Agents can update checkboxes directly. No external tool dependency.
- Format: `- [ ]` for pending, `- [x]` for complete, `- [!]` for blocked.

**Decision 4: Decision log as a first-class section**
- Rationale: The most common "where were we?" gap is "why did we choose X?" Decisions are the hardest context to recover.
- Format: Date + decision + rationale + alternatives considered.

**Decision 5: "Current State" section updated every session**
- Rationale: This is the resume point. An agent reads this section first to know exactly where work stands.

**Decision 6: Phased structure matching the natural project lifecycle**
- Phases: DEFINE → DESIGN → BUILD → VERIFY → SHIP
- Each phase has its own section with relevant content
- Tasks are grouped by phase, not flat

---

## Task Breakdown

### Task 1: Write the project.md Specification Document

**Objective:** Create the formal specification defining the project.md format, all sections, conventions, and rules.

**Files:**
- Create: `~/projectsdotmd/specification.md`

**Content structure for the spec:**

```
# project.md Specification v1.0

## Overview
## File Format
## Frontmatter Schema
## Required Sections
## Optional Sections
## Task Format
## Status Conventions
## Agent Instructions
## Human Instructions
## Examples
```

**Sections to define in the spec:**

1. **Frontmatter** (YAML)
   - `project`: name
   - `status`: define | design | build | verify | ship | paused | archived
   - `created`: ISO date
   - `updated`: ISO date
   - `owner`: human name
   - `agent`: agent name/framework
   - `tags`: array of tags
   - `repository`: optional repo URL

2. **Summary** (one paragraph)
   - What is this project? Why does it exist? What's the desired outcome?

3. **Problem Statement**
   - What problem are we solving?
   - Who has this problem?
   - What happens if we don't solve it?

4. **Success Criteria**
   - Measurable, specific outcomes
   - Each criterion is testable (can verify "done" or "not done")

5. **Constraints**
   - Technical constraints (languages, platforms, APIs)
   - Resource constraints (time, budget, hardware)
   - Scope constraints (explicit "not doing" list)

6. **Current State** (updated every session)
   - What phase we're in
   - What was last completed
   - What's being worked on now
   - Any active blockers
   - What the agent should do next

7. **Architecture / Design**
   - High-level approach
   - Key technology decisions
   - Component breakdown
   - Data flow (if applicable)
   - File structure (if code project)

8. **Decision Log**
   - Table: Date | Decision | Rationale | Alternatives Considered
   - Every significant choice gets logged here

9. **Tasks** (grouped by phase)
   - Phase: DEFINE
   - Phase: DESIGN
   - Phase: BUILD
   - Phase: VERIFY
   - Phase: SHIP
   - Each task: `- [ ] Description` with optional sub-bullets for details
   - Blocked tasks: `- [!] Description — blocked on: X`

10. **Discoveries / Learnings**
    - Things we learned during the project
    - Gotchas, workarounds, environment quirks
    - Anything the next agent session needs to know

11. **References**
    - Links to related docs, repos, APIs
    - External resources consulted

12. **Session Log** (append-only)
    - Date + brief summary of what was done
    - Not a full transcript — just enough to reconstruct timeline

---

### Task 2: Create the project.md Template

**Objective:** Create a ready-to-use template that agents can copy for new projects.

**Files:**
- Create: `~/projectsdotmd/template.md`

**Content:** A fill-in-the-blank project.md with all sections, placeholder text explaining what goes in each section, and example entries.

---

### Task 3: Write the project.md Agent Skill (SKILL.md)

**Objective:** Create an agent skill that teaches any agent framework how to create, maintain, and resume from project.md files.

**Files:**
- Create: `~/projectsdotmd/agent-skill.md`

**Content structure:**

The skill should define:
1. **When to create a project.md** — any new project with 3+ tasks
2. **How to initialize** — copy template, fill frontmatter, brainstorm with human
3. **How to maintain** — update Current State every session, check off tasks, log decisions
4. **How to resume** — read Current State first, verify task states, proceed
5. **Rules:**
   - Never mark a task complete without verification evidence
   - Always update Current State before ending a session
   - Log every significant decision in the Decision Log
   - Keep Summary to one paragraph (detailed info goes in other sections)

---

### Task 4: Create a Reference Example

**Objective:** Show project.md in action with a real-ish project example.

**Files:**
- Create: `~/projectsdotmd/example-cli-tool.md`

**Content:** A complete project.md for a hypothetical "CLI weather tool" project, showing all sections filled in, tasks partially completed, decision log entries, discoveries, and session log. Demonstrates what a mid-project project.md looks like.

---

### Task 5: Write the README

**Objective:** Explain the project.md standard to humans and agents.

**Files:**
- Create: `~/projectsdotmd/README.md`

**Content:**
- What is project.md and why it exists
- The problem with scattered project context
- How project.md solves it (single file, living document, resumable)
- Quick start: how to create your first project.md
- For agent framework authors: how to integrate
- Comparison with alternatives (superpowers multi-file, Linear, Jira, etc.)

---

### Task 6: Create a Validator Script

**Objective:** A simple script that validates a project.md file against the spec.

**Files:**
- Create: `~/projectsdotmd/validate.py`

**Content:** Python script that:
- Parses YAML frontmatter
- Checks required sections exist
- Validates task format (checkboxes)
- Reports missing or malformed sections
- Outputs a pass/fail summary

---

## Execution Order

1. Task 1 (spec) — foundation everything else builds on
2. Task 2 (template) — derived from spec
3. Task 3 (agent skill) — derived from spec
4. Task 4 (example) — demonstrates spec in practice
5. Task 5 (README) — ties everything together
6. Task 6 (validator) — ensures compliance

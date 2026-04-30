# projectsmd

**A single-file project management CLI for agent-human collaboration.**

`projectsmd` is a single binary that manages `project.md` files — one markdown file that captures everything about a project from start to finish. It lets AI agents pick up exactly where they left off on any project, at any time, and helps humans and agents scope, track, and execute work together.

No databases. No web apps. No external services. Just one file and one binary.

---

## Table of Contents

- [Why This Exists](#why-this-exists)
- [Installation](#installation)
- [Tutorial: Your First Project](#tutorial-your-first-project)
- [Commands Reference](#commands-reference)
- [The project.md Format](#the-projectmd-format)
- [Agent Integration](#agent-integration)
- [Workflows](#workflows)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Design Decisions](#design-decisions)
- [Inspired By](#inspired-by)
- [License](#license)

---

## Why This Exists

AI agents are getting better at building software every day. But there's a fundamental problem: **agents lose context between sessions.** Every time a conversation resets, the agent starts from scratch — it doesn't know what was decided, what's been built, what's blocked, or what matters most.

Meanwhile, project information scatters across tools:

- Requirements live in a Google Doc
- Decisions get buried in Slack threads
- Progress is tracked in Jira or Linear
- Architecture is sketched on a whiteboard
- Learnings evaporate when the session ends

The result: agents waste time re-discovering context, humans waste time re-explaining decisions, and projects drift because nobody has a clear picture of where things stand.

**`projectsmd` fixes this by putting everything in one file.**

---

## Installation

### From source (recommended)

```bash
git clone https://github.com/am423/ProjectsMD.git
cd ProjectsMD
cargo install --path .
```

### From crates.io

```bash
cargo install projectsmd
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/am423/ProjectsMD/releases):

```bash
# Linux (x86_64)
curl -sL https://github.com/am423/ProjectsMD/releases/latest/download/projectsmd-linux-amd64.tar.gz | tar xz
sudo mv projectsmd /usr/local/bin/

# macOS (Apple Silicon)
curl -sL https://github.com/am423/ProjectsMD/releases/latest/download/projectsmd-darwin-arm64.tar.gz | tar xz
sudo mv projectsmd /usr/local/bin/
```

### Verify installation

```bash
projectsmd --version
```

---

## Tutorial: Your First Project

This walkthrough takes you from zero to a managed project in 5 minutes.

### Step 1: Create a project

```bash
mkdir my-project && cd my-project
projectsmd init
```

The interactive wizard asks you:

```
ProjectsMD — New Project
─────────────────────────

Project name: My Web App
Owner: Alice
Agent (optional): Claude
Tags (comma-separated): web, typescript, react

What is this project? (2-3 sentences)
> A web application for managing personal book collections.
> Users can add, rate, and review books. Built with React and TypeScript.

Core value (the ONE thing that matters):
> Fast, intuitive book management — add a book in under 10 seconds.

Constraints (type: value — why, empty line to finish):
> Tech Stack: TypeScript + React — team expertise
> Performance: Page load under 2 seconds — user retention
>
```

This creates `project.md` with all sections populated.

### Step 2: See where you are

```bash
projectsmd status
```

```
── Project Status ──

  Project:    My Web App
  Phase:      define
  Created:    2026-04-29
  Updated:    2026-04-29

  Requirements:  0 validated, 0 active, 2 out of scope
  Tasks:         0 done, 3 pending, 0 blocked
  Decisions:     0 total
  Sessions:      0 logged
```

### Step 3: Add tasks

```bash
# Add tasks to the BUILD phase
projectsmd task add "Set up React project with Vite" --phase BUILD
projectsmd task add "Implement book list component" --phase BUILD
projectsmd task add "Add book form with validation" --phase BUILD
projectsmd task add "Rating and review system" --phase BUILD
projectsmd task add "Local storage persistence" --phase BUILD

# Add verification tasks
projectsmd task add "Unit tests for components" --phase VERIFY
projectsmd task add "E2E test: add and view book" --phase VERIFY

# See all tasks
projectsmd task list
```

### Step 4: Do some work

Work on your project. When you complete a task:

```bash
projectsmd task done 1
projectsmd task done 2
```

Log decisions as you make them:

```bash
projectsmd decide "Use Vite over CRA" --rationale "Faster dev server, better ESM support"
```

Log things you discover:

```bash
projectsmd discover "React 19 use() hook simplifies data loading significantly"
```

### Step 5: End your session

```bash
projectsmd session
```

The wizard walks you through:

```
Session Wrap-up
───────────────

Tasks completed this session:
  1. Set up React project with Vite
  2. Implement book list component

Enter completed task numbers (comma-separated): 1, 2

Any decisions made this session? (y/n): y
Decision: Use Zustand over Redux for state management
Rationale: Simpler API, less boilerplate, sufficient for this scale

Session summary (1-2 sentences):
> Set up React project with Vite. Implemented book list component with
> basic CRUD. Chose Zustand for state management.

Updating project.md...
  ✓ Task 1 marked complete
  ✓ Task 2 marked complete
  ✓ Decision logged
  ✓ Current State updated
  ✓ Session Log appended
  ✓ Frontmatter updated
```

### Step 6: Resume next session

Next time you come back:

```bash
projectsmd next
```

```
Next action: Task 3 — Add book form with validation

Implement a form component with title, author, rating fields.
Use React Hook Form with Zod validation.

Phase: BUILD | Blockers: None
```

The agent reads this and knows exactly what to do. No re-explaining needed.

### Step 7: Transition phases

When all BUILD tasks are done:

```bash
projectsmd phase transition verify
```

This runs the Evolution checklist:
- Promotes shipped requirements to Validated
- Checks if "What This Is" is still accurate
- Updates the project status

### Step 8: Archive when done

```bash
projectsmd archive --summary "v1.0 released. Book management working with local storage."
```

---

## Commands Reference

### `init` — Create a new project.md

```bash
projectsmd init                              # Interactive wizard
projectsmd init --name "my-app" --owner "Alice"
projectsmd init --brownfield                 # For existing codebases
projectsmd init --from old-project.md        # Import context from file
projectsmd init --template custom.md         # Use custom template
```

The wizard walks you through: project name, owner, description, core value, and constraints. It generates a complete `project.md` with all sections properly structured.

**Flags:**
- `--name NAME` — Skip the name prompt
- `--owner OWNER` — Skip the owner prompt
- `--brownfield` — Use the brownfield template (infers Validated requirements from existing code)
- `--from FILE` — Import context from an existing document
- `--template FILE` — Use a custom template instead of the built-in default

### `validate` — Check conformance to the project.md spec

```bash
projectsmd validate

# Example output:
# ✓ PASS — Project conforms to spec
#   Info (4):
#     ℹ Status: build
#     ℹ Requirements: 2 validated, 5 active, 5 out of scope
#     ℹ Tasks: 9 done, 13 pending, 0 blocked (across 5 phases)
#     ℹ Decisions: 6 total (4 good, 2 pending, 0 revisit)
```

Checks:
- YAML frontmatter completeness (required fields, valid status, ISO dates)
- All 6 required sections present
- Three-tier requirements lifecycle (Validated / Active / Out of Scope)
- Task checkbox syntax
- Key Decisions table format
- Core Value conciseness
- Current State completeness

**Flags:**
- `--strict` — Treat warnings as errors
- `--json` — Output as JSON
- `--quiet` — Exit code only (0 = pass, 1 = fail)

### `status` — Show current project status

```bash
projectsmd status

# Example output:
# ── Project Status ──
#
#   Project:    Weather CLI
#   Phase:      build
#   Created:    2026-04-25
#   Updated:    2026-04-29
#
#   Requirements:  2 validated, 5 active, 4 out of scope
#   Tasks:         9 done, 18 pending, 0 blocked
#   Decisions:     6 total (4 good, 2 pending, 0 revisit)
#   Sessions:      4 logged
```

**Flags:**
- `--json` — Output as JSON
- `--compact` — One-line summary

### `next` — Show what to do next

```bash
projectsmd next

# Example output:
# Next action: Task 4 — Display formatting with color coding
#
# Implement temperature-to-color mapping in display/format.go
# using fatih/color library.
#
# Phase: BUILD | Blockers: None
```

Reads Current State and the task list to determine the most logical next action.

**Flags:**
- `--all` — Show all pending tasks, not just the next one
- `--phase PHASE` — Show tasks for a specific phase

### `task` — Manage tasks

```bash
# List tasks
projectsmd task list                         # All tasks
projectsmd task list --phase BUILD           # Tasks in a phase
projectsmd task list --pending               # Only unfinished tasks

# Add tasks
projectsmd task add "Implement auth"         # Default: BUILD phase
projectsmd task add "Write spec" --phase DEFINE

# Complete tasks
projectsmd task done 4                       # Mark task 4 as done

# Block/unblock
projectsmd task block 5 --reason "Waiting on API key approval"
projectsmd task unblock 5
```

Tasks are numbered automatically. BUILD phase tasks get sequential numbers (Task 1, Task 2, ...) for easy reference.

### `decide` — Record a key decision

```bash
projectsmd decide "Use PostgreSQL over MongoDB" \
  --rationale "Need relational data and ACID transactions"

projectsmd decide "Drop IE11 support" --rationale "Less than 0.1% of traffic"

# Set initial outcome
projectsmd decide "Use Vite" --rationale "Faster builds" --outcome good
```

Decisions are logged in a markdown table with three columns: Decision, Rationale, Outcome. Outcomes track whether the decision proved correct over time:
- `good` — ✓ Good (proved correct)
- `revisit` — ⚠️ Revisit (needs reconsideration)
- `pending` — — Pending (too early to evaluate)

### `discover` — Record a discovery

```bash
projectsmd discover "OpenWeatherMap returns 401 for invalid keys, not 403"
projectsmd discover "Go's net/http already handles connection pooling"
```

Discoveries capture gotchas, workarounds, and surprises. They're distinct from decisions — discoveries are observations, not choices.

### `phase` — Manage project phases

```bash
projectsmd phase                             # Show current phase
projectsmd phase --transition design         # Transition to design phase
projectsmd phase --transition build          # Transition to build phase
projectsmd phase --transition verify         # Transition to verify phase
projectsmd phase --transition ship           # Transition to ship phase
projectsmd phase --transition paused         # Pause the project
```

Phase transitions run the Evolution checklist:
1. Requirements that shipped → promote to Validated
2. Requirements that failed → move to Out of Scope
3. New requirements → add to Active
4. "What This Is" still accurate?
5. Key Decisions outcomes updated

Valid phases: `define`, `design`, `build`, `verify`, `ship`, `paused`

### `session` — End-of-session wrap-up

```bash
projectsmd session                           # Interactive wizard
projectsmd session --non-interactive \
  --summary "Implemented auth and wrote tests"
```

The interactive wizard:
1. Shows completed tasks, asks you to confirm
2. Asks if any decisions were made
3. Prompts for a session summary
4. Updates: Current State, Session Log, task checkboxes, frontmatter dates

**Flags:**
- `--non-interactive` — Skip prompts, use provided values
- `--summary TEXT` — Session summary (required in non-interactive mode)

### `diff` — Show changes since last session

```bash
projectsmd diff
```

Shows what changed in project.md since the last session:
- Task status changes (completed, blocked, new)
- Current State changes
- New decisions
- New session log entries

Uses the `similar` crate for text diffing. If not in a git repo, compares to a `.project.md.snapshot` file.

### `archive` — Mark project complete

```bash
projectsmd archive                           # Interactive
projectsmd archive --summary "v1.0 released, all features complete"
```

Sets status to `archived`, adds a final Session Log entry, and updates the frontmatter date.

### `view` — Render project.md to terminal

```bash
projectsmd view                              # Full document, syntax-highlighted
projectsmd view --section "Tasks"            # Just the Tasks section
projectsmd view --section "Key Decisions"    # Just Key Decisions
projectsmd view --section "Current State"    # Just Current State
```

Renders with colors: green for done tasks, red for blocked, dim for pending, colored outcomes for decisions.

### `skill` — Manage agent skills

```bash
# Install the agent skill
projectsmd skill install                     # Auto-detect framework
projectsmd skill install --framework claude  # For Claude Code
projectsmd skill install --framework cursor  # For Cursor
projectsmd skill install --framework codex   # For Codex
projectsmd skill install --framework hermes  # For Hermes
projectsmd skill install --path /custom/path # Custom location
projectsmd skill install --force             # Overwrite existing

# View the embedded skill
projectsmd skill view

# Generate project-specific skill
projectsmd skill generate
```

See [Agent Integration](#agent-integration) for details.

### Global Options

```
-f, --file <FILE>  Path to project.md file [default: project.md]
    --json         Output in JSON format (where supported)
-q, --quiet        Suppress output (only exit code)
-h, --help         Print help
-V, --version      Print version
```

---

## The project.md Format

`project.md` is a structured markdown file with YAML frontmatter. Here's the complete structure:

```markdown
---
project: "My Web App"
status: build
created: 2026-04-25
updated: 2026-04-29
owner: "Alice"
agent: "Claude"
tags: [web, typescript, react]
repository: "https://github.com/alice/my-web-app"
priority: medium
---

## What This Is

A web application for managing personal book collections. Users can add,
rate, and review books. Built with React and TypeScript for a team of
three developers.

## Core Value

Fast, intuitive book management — add a book in under 10 seconds.

## Requirements

### Validated

- ✓ User authentication with email/password — v0.1
- ✓ Book list with search and filter — v0.1

### Active

- [ ] Book form with title, author, rating fields
- [ ] Rating and review system (1-5 stars + text)
- [ ] Local storage persistence
- [ ] Responsive design for mobile

### Out of Scope

- Social features (sharing, following) — adds complexity, different use case
- Cloud sync — requires backend, defer to v2
- Import from Goodreads — API rate limits, maintenance burden

## Context

- Team of 3 developers, all comfortable with React
- Previous version used Angular — migrating to React
- Users complained about slow page loads in v1
- Mobile usage is 40% of traffic

## Constraints

- **Tech Stack**: TypeScript + React — team expertise, code reuse
- **Performance**: Page load under 2 seconds — user retention data
- **Compatibility**: Chrome, Firefox, Safari, Edge (last 2 versions)
- **Storage**: Local storage only — no backend budget yet

## Current State

**Phase:** build
**Last completed:** Task 2 (Book list component)
**In progress:** Task 3 (Book form with validation)
**Next action:** Implement form with React Hook Form + Zod validation
**Blockers:** None
**Notes:** Chose Zustand for state management during Task 2.

## Architecture

Frontend-only React app with local storage:

- `src/components/` — React components
- `src/stores/` — Zustand state stores
- `src/types/` — TypeScript type definitions
- `src/utils/` — Helper functions

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Vite over CRA | Faster dev server, better ESM | ✓ Good |
| Zustand over Redux | Simpler API, less boilerplate | — Pending |
| React Hook Form + Zod | Type-safe validation, minimal re-renders | — Pending |

## Tasks

### Phase: DEFINE

- [x] Identify target users and use cases
- [x] Define success criteria with owner

### Phase: DESIGN

- [x] Choose technology stack
- [x] Design component architecture
- [x] Define data models (Book, Review)

### Phase: BUILD

- [x] Task 1: Set up React project with Vite
- [x] Task 2: Implement book list component
- [ ] Task 3: Add book form with validation
- [ ] Task 4: Rating and review system
- [ ] Task 5: Local storage persistence

### Phase: VERIFY

- [ ] Unit tests for components
- [ ] E2E test: add and view book

### Phase: SHIP

- [ ] README with setup instructions
- [ ] Deploy to Vercel

## Discoveries

- React 19 use() hook simplifies data loading significantly
- Zustand's persist middleware handles localStorage automatically

## References

- [React Hook Form docs](https://react-hook-form.com/)
- [Zustand docs](https://zustand-demo.pmnd.rs/)

## Session Log

- **2026-04-25** — Project kickoff. Defined requirements, chose stack. (1 hour)
- **2026-04-27** — Set up Vite project. Implemented book list. (2 hours)
- **2026-04-29** — Started book form. Chose Zustand. (1 hour, ongoing)

---
*Last updated: 2026-04-29 after completing Task 2*
```

### Section Reference

| Section | Required | Purpose |
|---------|----------|---------|
| **YAML Frontmatter** | Yes | Project metadata (name, status, dates, owner) |
| **What This Is** | Yes | Living description — updated when reality drifts |
| **Core Value** | Yes | The ONE thing that matters most |
| **Requirements** | Yes | Three-tier lifecycle: Validated / Active / Out of Scope |
| **Context** | Recommended | Background, environment, prior work |
| **Constraints** | Recommended | Hard limits with rationale |
| **Current State** | Yes | THE resume point — phase, next action, blockers |
| **Architecture** | Recommended | Technical approach, file structure |
| **Key Decisions** | Yes | Decision log with outcome tracking |
| **Tasks** | Yes | Phase-grouped checklist |
| **Discoveries** | Recommended | Gotchas, workarounds, surprises |
| **References** | Optional | External links |
| **Session Log** | Recommended | Append-only timeline |

### Requirements Lifecycle

Requirements move through three tiers:

```
[Active] ──shipped──> [Validated]     (promoted after proving valuable)
   │
   └──invalidated──> [Out of Scope]   (moved with reasoning)
```

- **Validated** — Shipped and proven. Locked. Requires discussion to change.
- **Active** — Current scope. Hypotheses until validated.
- **Out of Scope** — Explicit boundaries. Always includes reasoning.

### Decision Outcomes

Track whether decisions proved correct:

- ✓ **Good** — Decision proved correct
- ⚠️ **Revisit** — May need reconsideration
- — **Pending** — Too early to evaluate

---

## Agent Integration

### agentskills.io Compliant

`projectsmd` includes a built-in agent skill following the [Agent Skills](https://agentskills.io) open standard. This works with any skills-compatible agent.

### Install the Skill

```bash
# Auto-detect your agent framework
projectsmd skill install

# Or specify explicitly
projectsmd skill install --framework claude   # ~/.claude/skills/projectsmd/
projectsmd skill install --framework cursor   # ~/.cursor/skills/projectsmd/
projectsmd skill install --framework codex    # ~/.codex/skills/projectsmd/
projectsmd skill install --framework hermes   # ~/.hermes/skills/projectsmd/

# Custom path
projectsmd skill install --path /your/custom/skills/projectsmd/
```

### How Agents Use It

The skill teaches agents the full project.md lifecycle:

1. **Session start:** `projectsmd status` → `projectsmd next` → read Current State
2. **During work:** `projectsmd task done N`, `projectsmd decide`, `projectsmd discover`
3. **Session end:** `projectsmd session` (interactive wrap-up)
4. **Phase transitions:** `projectsmd phase transition verify`

### Generate Project-Specific Skills

```bash
projectsmd skill generate
```

Creates a skill tailored to YOUR project — includes project name, current phase, task list, and constraints. Any agent picking up this skill gets full context.

### Progressive Disclosure

Skills use three-tier loading:

1. **Discovery** — Agent reads name + description (knows when to use it)
2. **Activation** — Agent reads full SKILL.md (gets the instructions)
3. **Execution** — Agent runs `projectsmd` commands (does the work)

### Supported Frameworks

| Framework | Install Command | Skill Location |
|-----------|----------------|----------------|
| Claude Code | `--framework claude` | `~/.claude/skills/projectsmd/` |
| Cursor | `--framework cursor` | `~/.cursor/skills/projectsmd/` |
| Codex | `--framework codex` | `~/.codex/skills/projectsmd/` |
| Hermes | `--framework hermes` | `~/.hermes/skills/projectsmd/` |
| Custom | `--path /dir/` | `/dir/projectsmd/` |

---

## Workflows

### Daily Development

```bash
# Start of day
projectsmd status        # Where am I?
projectsmd next          # What's next?

# ... do work ...

# End of day
projectsmd task done 3   # Mark completed tasks
projectsmd session       # Wrap up
```

### Multi-Agent Collaboration

When multiple agents work on the same project:

```bash
# Agent A starts a session
projectsmd task block 3 --reason "Agent B working on API client"
# ... works on other tasks ...
projectsmd session

# Agent B picks up
projectsmd status        # Sees blocked task
projectsmd task unblock 3
# ... works on API client ...
projectsmd task done 3
projectsmd session
```

### Phase Transitions

```bash
# All BUILD tasks done?
projectsmd task list --phase BUILD --pending  # Verify none left
projectsmd phase transition verify            # Move to VERIFY

# All VERIFY tasks done?
projectsmd phase transition ship              # Move to SHIP

# Project complete?
projectsmd archive --summary "v1.0 released"
```

### Brownfield Projects

For existing codebases:

```bash
projectsmd init --brownfield
# The wizard infers Validated requirements from existing code
# Then asks what you want to build next (Active requirements)
```

---

## Configuration

`projectsmd` stores no configuration files. All state lives in `project.md`.

### Custom File Location

```bash
projectsmd --file path/to/my-project.md status
projectsmd -f ~/projects/weather/project.md validate
```

### JSON Output

```bash
projectsmd status --json
projectsmd validate --json
projectsmd task list --json
```

Useful for piping to `jq` or integrating with other tools:

```bash
projectsmd status --json | jq '.phase'
projectsmd task list --json | jq '.tasks[] | select(.status == "pending")'
```

---

## Troubleshooting

### "No project.md found"

```bash
# Check if you're in the right directory
ls project.md

# Or specify the path
projectsmd --file /path/to/project.md status
```

### "Invalid frontmatter"

Your YAML frontmatter has a syntax error. Common issues:

```yaml
# Bad — missing quotes around values with special characters
project: My App: The Sequel

# Good
project: "My App: The Sequel"
```

### "Section not found"

The parser expects exact heading text. Make sure your sections match:

```markdown
## What This Is        # ✓ Correct
## What this is        # ✗ Wrong (case matters)
## Summary             # ✗ Wrong (old format)
```

### Validation warnings vs errors

- **Errors** — Must fix for conformance (missing required sections, invalid status)
- **Warnings** — Should fix but not blocking (missing recommended sections, date format)

Run `projectsmd validate --strict` to treat warnings as errors.

---

## Design Decisions

### Single File, Not a Directory

Context fragmentation is the problem. One file forces discipline and makes resumption trivial — just read one file.

### Three-Tier Requirements Lifecycle

Not just "done/not done." Requirements are hypotheses until shipped and validated. The three tiers (Validated / Active / Out of Scope) prevent silent scope creep and create a clear record of what's been proven.

### Core Value as a Section

The single most important thing. If everything else fails, this must work. One sentence that drives prioritization when tradeoffs arise. If you can't write this in one sentence, the project isn't well-defined yet.

### Key Decisions with Outcome Tracking

Not just "what was decided" but "did it work?" The #1 context lost between sessions is WHY a decision was made and WHETHER it proved correct.

### Current State as Resume Point

An agent reading ONLY the Current State section knows exactly what to do next. Updated every session — non-negotiable.

### Evolution Rules

The document is living, not static. Sections update at phase transitions and milestones. "What This Is" gets updated when reality drifts. Requirements get promoted or retired.

---

## Inspired By

- **[obra/superpowers](https://github.com/obra/superpowers)** — Agent skill pipeline (brainstorm → plan → execute), verification-before-completion philosophy, subagent-driven development
- **[Get Shit Done (GSD)](https://github.com/gsd-build/get-shit-done)** — PROJECT.md template, requirements lifecycle (Validated/Active/Out of Scope), decision outcome tracking, evolution rules, brownfield support
- **[Agent Skills](https://agentskills.io)** — Open standard for agent skill packaging and progressive disclosure
- **Agile/Scrum** — Sprint structure, retrospectives, incremental delivery
- **Shape Up** — Appetite-based scoping, circuit breakers, shaping before building

---

## License

MIT — see [LICENSE](LICENSE).

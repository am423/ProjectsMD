# projectsmd

**A single-file project management CLI for agent-human collaboration.**

A single binary that manages `project.md` files — the complete project lifecycle from scoping through execution. Parse, validate, update, and evolve your project.md without ever leaving the terminal.

```
$ projectsmd status

── Project Status ──

  Project:    my-cool-app
  Phase:      build
  Created:    2026-01-15
  Updated:    2026-04-29

  Requirements:  3 validated, 5 active, 2 out of scope
  Tasks:         7 done, 4 pending, 1 blocked
  Decisions:     5 total (3 good, 1 revisit, 1 pending)
  Sessions:      8 logged
```

---

## Installation

### From crates.io (once published)

```bash
cargo install projectsmd
```

### From source

```bash
git clone https://github.com/projectsdotmd/projectsmd.git
cd projectsmd
cargo install --path .
```

### Pre-built binaries

Download pre-built binaries from [GitHub Releases](https://github.com/projectsdotmd/projectsmd/releases).

---

## Quick Start

```bash
# Create a new project
projectsmd init --name "my-app" --owner "Alice"

# See where you are
projectsmd status

# Find out what to do next
projectsmd next

# Complete a task
projectsmd task done 4

# End-of-session wrap-up (updates Current State + Session Log)
projectsmd session
```

---

## All Commands

### `init` — Create a new project.md

```bash
projectsmd init                              # Interactive (prompts for name/owner)
projectsmd init --name "my-app" --owner "Alice"
projectsmd init --brownfield                 # Use brownfield (existing project) template
projectsmd init --from old-project.md        # Import from an existing file
projectsmd init --template custom.md         # Use a custom template
```

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

### `status` — Show current project status

```bash
projectsmd status
```

### `next` — Show the next recommended action

```bash
projectsmd next
```

### `task` — Manage tasks

```bash
projectsmd task list                         # List all tasks
projectsmd task list --phase BUILD           # Filter by phase
projectsmd task list --pending               # Show only pending tasks
projectsmd task add "Implement auth"         # Add a new task (default: BUILD phase)
projectsmd task add "Write spec" --phase DEFINE
projectsmd task done 4                       # Mark task 4 as done
projectsmd task block 5 --reason "waiting on API"  # Block with reason
projectsmd task unblock 5                    # Unblock a task
```

### `decide` — Record a key decision

```bash
projectsmd decide "Use PostgreSQL" --rationale "Better for production workloads"
projectsmd decide "Drop IE11 support"       # No rationale needed
```

### `discover` — Record a discovery

```bash
projectsmd discover "SQLite doesn't support ALTER COLUMN — need to recreate table"
```

### `phase` — Manage project phases

```bash
projectsmd phase                             # Show current phase and suggested next
projectsmd phase --transition design         # Transition to a new phase
# Valid phases: define, design, build, verify, ship, paused
```

### `session` — End-of-session wrap-up wizard

```bash
projectsmd session                           # Interactive wizard
projectsmd session --non-interactive --summary "Implemented auth and wrote tests"
```

### `diff` — Show diff of project.md changes

```bash
projectsmd diff                              # Show changes since last snapshot
```

### `archive` — Archive the project

```bash
projectsmd archive                           # With interactive summary prompt
projectsmd archive --summary "Migrated to new system, project complete"
```

### `view` — Render project.md to terminal

```bash
projectsmd view                              # Full file, syntax-highlighted
projectsmd view --section "Tasks"            # View a specific section
projectsmd view --section "Key Decisions"
```

### `skill` — Manage agent skills

```bash
projectsmd skill install                     # Auto-detect framework and install
projectsmd skill install --framework claude  # Install for Claude Code
projectsmd skill install --framework cursor  # Install for Cursor
projectsmd skill install --framework codex   # Install for Codex
projectsmd skill install --path ~/.config/custom-agent/skills/
projectsmd skill install --force             # Overwrite existing skill
projectsmd skill view                        # Print the embedded SKILL.md to stdout
projectsmd skill generate                    # Generate project-specific skill from current project.md
```

### Global Options

```
-f, --file <FILE>  Path to project.md file [default: project.md]
    --json         Output in JSON format (where supported)
-q, --quiet        Suppress output (only exit code)
-h, --help         Print help
-V, --version      Print version
```

---

## What is project.md?

`project.md` is a structured markdown file that serves as the **single source of truth** for a project. One file captures everything: what the project is, what's been decided, what's built, what's blocked, and what to do next.

Any agent, in any framework, reads this one file and immediately understands the full picture.

For the full specification, see the [project.md Specification v1.1](https://github.com/projectsdotmd/projectsdotmd/blob/main/specification.md).

The file contains these sections:

| Section | Purpose |
|---------|---------|
| **YAML Frontmatter** | Project name, status, dates, owner |
| **What This Is** | Living description (2-3 sentences) |
| **Core Value** | The ONE thing that matters most |
| **Requirements** | Three-tier lifecycle: Validated / Active / Out of Scope |
| **Context** | Background, environment, prior work |
| **Constraints** | Hard limits with rationale |
| **Current State** | THE resume point — phase, next action, blockers |
| **Architecture** | Technical approach, file structure |
| **Key Decisions** | Decision log with outcome tracking |
| **Tasks** | Phase-grouped checklist |
| **Discoveries** | Gotchas, workarounds, surprises |
| **Session Log** | Append-only timeline |

---

## Agent Integration

### agentskills.io Compliant

`projectsmd` includes a built-in agent skill that follows the [agentskills.io](https://agentskills.io) convention. Install it with one command:

```bash
projectsmd skill install
```

This auto-detects your agent framework and installs the skill to the correct directory.

### Supported Frameworks

| Framework | Install Command |
|-----------|----------------|
| **Claude Code** | `projectsmd skill install --framework claude` |
| **Cursor** | `projectsmd skill install --framework cursor` |
| **Codex** | `projectsmd skill install --framework codex` |
| **Hermes** | `projectsmd skill install --framework hermes` |
| **Custom** | `projectsmd skill install --path /your/skill/dir/` |

### Progressive Disclosure

The skill uses a three-tier progressive disclosure pattern:

1. **Discovery** — On first load, the agent reads the skill to understand project.md conventions
2. **Activation** — When working on a project, the agent reads the project.md file
3. **Execution** — During work, the agent uses `projectsmd` CLI commands to update the file

This means agents get exactly the context they need, when they need it — no more, no less.

### Generate Project-Specific Skills

```bash
projectsmd skill generate
```

Creates a project-specific skill file that includes the current project's context, so any agent can pick up the project with full understanding.

---

## Key Design Decisions

### Single File, Not a Directory

Context fragmentation is the problem. One file forces discipline and makes resumption trivial — just read one file.

### Three-Tier Requirements Lifecycle

Not just "done/not done." Requirements are hypotheses until shipped and validated:

- **Validated** — Shipped and proven valuable (locked, requires discussion to change)
- **Active** — Current scope being built toward (hypotheses until validated)
- **Out of Scope** — Explicit boundaries with reasoning (prevents re-adding later)

### Core Value Section

The single most important thing. If everything else fails, this must work. One sentence that drives prioritization when tradeoffs arise.

### Key Decisions with Outcome Tracking

Not just "what was decided" but "did it work?" Every decision gets an outcome:

- ✓ Good — proved correct
- ⚠️ Revisit — may need reconsideration
- — Pending — too early to evaluate

### Current State as Resume Point

An agent reading ONLY the Current State section knows exactly what to do next. Updated every session — non-negotiable.

### Evolution Rules

The document is living, not static. It has explicit rules for how it updates at phase transitions and milestones. Requirements get promoted or retired. Decisions get outcome tracking. "What This Is" gets updated when reality drifts.

---

## Inspired By

- **[obra/superpowers](https://github.com/obra/superpowers)** — Agent skill pipeline (brainstorm → plan → execute), verification-before-completion philosophy
- **[Get Shit Done (GSD)](https://github.com/gsd-build/get-shit-done)** — PROJECT.md template, requirements lifecycle (Validated/Active/Out of Scope), decision outcome tracking, evolution rules
- **Agile/Scrum** — Sprint structure, retrospectives, incremental delivery
- **Shape Up** — Appetite-based scoping, circuit breakers, shaping before building

---

## Help Output

```
$ projectsmd --help

A project.md lifecycle management tool

Usage: projectsmd [OPTIONS] [COMMAND]

Commands:
  init      Initialize a new project.md file
  validate  Validate a project.md file for conformance
  status    Show current project status
  next      Show the next recommended action
  task      Manage tasks
  decide    Record a key decision
  discover  Record a discovery
  phase     Manage project phases
  session   Session wrap-up wizard
  diff      Show diff of project.md changes
  archive   Archive the project
  view      Render project.md to terminal
  skill     Manage agent skills
  help      Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>  Path to project.md file [default: project.md]
      --json         Output in JSON format
  -q, --quiet        Suppress output (only exit code)
  -h, --help         Print help
  -V, --version      Print version
```

---

## License

MIT

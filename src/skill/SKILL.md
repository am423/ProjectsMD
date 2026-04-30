---
name: projectsmd
description: "Project lifecycle management using project.md files — structured project tracking for AI agents"
---

# projectsmd Skill

## When to Use

- Starting work on any tracked project (has a project.md file)
- Beginning a coding session — check status and get next action
- Ending a coding session — record what was accomplished
- Recording a key decision or discovery during work
- Adding, completing, or blocking tasks
- Transitioning between project phases (define → design → build → verify → ship)
- Creating a new tracked project from scratch
- Comparing project state before/after a session

## When NOT to Use

- For quick, one-off tasks with no project tracking
- When no project.md file exists and you don't need tracking
- For issues/PRs that are better tracked in a dedicated issue tracker
- When the user explicitly asks you to skip project management

## Session Workflow

### Session Start

Always begin a work session by understanding the project state:

```bash
projectsmd status          # Overview: status, phase, tasks, progress
projectsmd next            # What to work on right now
projectsmd task list --pending  # All pending tasks
```

Read the output carefully. The `next` command shows the current phase,
what's in progress, and the recommended next action. Start there.

### During Session

As you work, keep the project.md updated:

```bash
# Task management
projectsmd task add "Implement auth middleware" --phase build
projectsmd task done 4
projectsmd task block 5 --reason "waiting on API credentials"
projectsmd task unblock 5

# Record decisions as they happen
projectsmd decide "Use Redis for session storage" --rationale "Built-in TTL, fast reads"

# Record discoveries
projectsmd discover "The auth API returns 403 not 401 for expired tokens"

# Check progress anytime
projectsmd task list --phase build
```

### Session End

Always wrap up with the session command:

```bash
projectsmd session
```

This launches an interactive wizard that:
1. Confirms which tasks were completed
2. Asks about any decisions made
3. Asks for a brief session summary
4. Updates the project.md with all changes

For non-interactive usage (scripts, CI):
```bash
projectsmd session --non-interactive --summary "Implemented auth middleware, fixed token refresh"
```

## Phase Transitions

When all tasks in a phase are done, transition to the next phase:

```bash
projectsmd phase transition build    # Move to build phase
projectsmd phase transition verify   # Move to verify phase
```

The phase command runs an evolution checklist:
- Counts completed tasks
- Checks if "What This Is" is still accurate
- Updates the status in frontmatter

Valid phases: define, design, build, verify, ship, paused

## Diff and History

Compare current state to previous:

```bash
projectsmd diff    # Show changes since last snapshot/commit
```

## Archiving

When a project is complete:

```bash
projectsmd archive --summary "Project complete. All features shipped, tests passing."
```

This sets status to "archived" and adds a final session log entry.

## Project Initialization

For new projects:

```bash
projectsmd init                          # Interactive wizard
projectsmd init --name "My App" --owner "Alice"  # With flags
projectsmd init --brownfield             # For existing codebases
```

## Integration with Agent Workflows

This skill is framework-agnostic. It works with any AI coding agent.

### Pattern: Context-Aware Session

1. Load context: `projectsmd status` → `projectsmd next`
2. Work on the recommended task
3. Record decisions inline: `projectsmd decide "..." --rationale "..."`
4. Wrap up: `projectsmd session`

### Pattern: Multi-Agent Handoff

The project.md serves as a shared context file between agents:
- Agent A completes a session, runs `projectsmd session`
- Agent B starts, reads `projectsmd status` and `projectsmd next`
- Both agents see the same project state

### Pattern: Automated Check-ins

```bash
projectsmd validate    # Check project.md conforms to spec
projectsmd status      # Get current state for monitoring
```

## Anti-Patterns

- **Editing project.md directly** — Use the CLI to ensure consistency
- **Skipping session wrap-up** — The Session Log is the project's memory; gaps lose context
- **Decisions without rationale** — Always record why, not just what
- **Tasks without phases** — Every task should belong to a phase
- **Ignoring blockers** — Block them explicitly so they're visible
- **Archiving too early** — Run `projectsmd validate` before archiving
- **Multiple active phases** — One phase at a time; complete before transitioning

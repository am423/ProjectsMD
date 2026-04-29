# Agent Skill: Working with project.md

## When to Create a project.md

Create a project.md when:
- The project involves 3+ tasks
- Work will span multiple sessions
- The human wants a clear scope and progress tracking
- The project has non-trivial decisions to document

Skip project.md for:
- One-off questions or single commands
- Quick fixes (< 15 minutes)
- Exploratory research with no defined deliverable

## How to Initialize a New Project.md

### Greenfield (New Project)

1. **Copy the template:** Copy `template.md` into the project directory as `project.md`
2. **Fill frontmatter:** Set project name, status (`define`), created date, owner
3. **Brainstorm with human:** Ask questions one at a time to fill in:
   - What This Is (2-3 sentences, their words)
   - Core Value (the ONE thing that matters)
   - Requirements — Active (current scope as hypotheses)
   - Requirements — Out of Scope (with reasoning)
   - Context (background, prior work, environment)
   - Constraints (with "why" for each)
4. **Don't touch Tasks yet.** Tasks come after Design is approved.
5. **Log the initial session** in Session Log.
6. **Add footer:** `*Last updated: [date] after initialization*`

### Brownfield (Existing Codebase)

1. **Map the codebase first** — read existing code, understand patterns
2. **Infer Validated requirements** from what the code already does:
   - What does the codebase actually do?
   - What patterns are established?
   - What's clearly working and relied upon?
3. **Gather Active requirements** from the human:
   - Present inferred current state
   - Ask what they want to build next
4. **Initialize:**
   - Validated = inferred from existing code (with "— existing" tag)
   - Active = human's goals for this work
   - Out of Scope = boundaries human specifies
   - Context = includes current codebase state

### Brainstorming Rules

- One question per message
- Prefer multiple choice over open-ended
- Propose 2-3 approaches with trade-offs and your recommendation
- Get explicit approval before moving from DEFINE to DESIGN
- Challenge vagueness — make abstract concrete
- Surface assumptions — find edges — reveal motivation

## How to Maintain project.md

### During a Session

- **Update task checkboxes** as you complete them: `- [ ]` → `- [x]`
- **Log decisions immediately** — don't wait until session end
- **Track decision outcomes** when known: ✓ Good / ⚠️ Revisit / — Pending
- **Flag blocked tasks** with `- [!]` and the reason
- **Add discoveries** as you find them
- **Never mark a task complete without verification evidence**

### Ending a Session

Before closing, update:

1. **Current State** — this is mandatory, never skip:
   - Phase: what phase you're in
   - Last completed: which task(s) you finished
   - In progress: what you were working on
   - Next action: specific enough for the next session to start immediately
   - Blockers: anything stopping progress
   - Notes: any context the next session needs
2. **Session Log** — append a one-line summary
3. **Frontmatter** — update `updated` date and `status` if phase changed
4. **Footer** — `*Last updated: [date] after [what happened]*`

### Phase Transitions (Evolution)

When completing a phase, run this checklist:

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated (shipped + proven)? → Move to Validated with version/phase
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted
6. Update `status` in frontmatter

### Milestones

When hitting a milestone, do a full review:

1. Review all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state
5. Review Key Decisions outcomes

### Decision Logging

Log a decision when:
- You choose between alternative approaches
- The human makes a call on scope, design, or priorities
- You discover a constraint that wasn't in the original list
- You reverse a previous decision (update old decision's Outcome to ⚠️ Revisit)

Format:
```
| Decision | Rationale | Outcome |
```

## How to Resume from a project.md

### Step 1: Read the Full File

Read the entire project.md on session start. Don't just read Current State — you need the full context.

### Step 2: Start with Current State

Current State tells you:
- What phase you're in
- What was last completed
- What to do next
- Any blockers

### Step 3: Verify Task States

Before starting work:
- Check that the "last completed" tasks are actually `[x]` in the task list
- If there's a mismatch, ask the human before proceeding
- Verify the "next action" still makes sense given the task list

### Step 4: Check Key Decisions

Look for any decisions with ⚠️ Revisit outcome — these may need discussion before proceeding.

### Step 5: Proceed

Start working on the "next action" from Current State. If it's no longer valid, update Current State and ask.

## Rules

### Non-Negotiable

1. **Never mark a task complete without verification.** Run the test, check the output, confirm it works.
2. **Always update Current State before ending a session.** The next agent session depends on this.
3. **Log every significant decision.** If you'll forget why you chose X over Y in 2 days, log it now.
4. **Keep "What This Is" to 2-3 sentences.** Update it when reality drifts.

### Strong Preferences

5. **One question at a time** during brainstorming
6. **Tasks should be bite-sized** — one action, 2-5 minutes of focused work
7. **Use sub-bullets for task detail** when a task needs more explanation
8. **Don't duplicate information** — reference other sections instead
9. **Append, don't rewrite** — Session Log is append-only
10. **Out of Scope always has reasoning** — prevents re-adding later

### Anti-Patterns

- Skipping Current State updates ("I'll do it next time")
- Vague next actions ("continue working on it")
- Logging decisions without rationale ("chose Go" — why?)
- Expanding scope without updating Requirements/Constraints
- Marking tasks complete without running verification
- Leaving stale blockers in Current State
- Writing "What This Is" as a technical spec instead of a human description

## Integration with Other Skills

### With Brainstorming

Brainstorming fills in the DEFINE phase of project.md:
- What This Is + Core Value
- Requirements (Active + Out of Scope)
- Context + Constraints

### With Writing Plans

Writing Plans populates the BUILD phase tasks. Each task in the plan becomes a `- [ ]` entry under "Phase: BUILD" in project.md.

### With Executing Plans

Executing Plans uses project.md as the source of truth. Tasks are checked off as completed. Current State is updated throughout.

### With Verification Before Completion

Verification ensures task checkboxes are only checked with evidence. project.md's task list is the checklist that verification enforces.

## File Naming

- Always `project.md` — not `PROJECT.md`, not `project-doc.md`
- One per project — in the project root or a projects directory
- For multiple projects, use separate directories: `projects/weather-cli/project.md`

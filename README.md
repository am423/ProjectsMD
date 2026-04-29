# ProjectsMD

**A single-file project management standard for agent-human collaboration.**

ProjectsMD defines `project.md` — one markdown file that captures everything about a project from start to finish. It lets AI agents pick up exactly where they left off on any project, at any time, and helps humans and agents scope, track, and execute work together.

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

**ProjectsMD fixes this by putting everything in one file.**

---

## What Is project.md?

project.md is a structured markdown file that serves as the single source of truth for a project. It contains:

| Section | Purpose |
|---------|---------|
| **What This Is** | Living description of the project (updated when reality drifts) |
| **Core Value** | The ONE thing that matters most — drives prioritization |
| **Requirements** | Three-tier lifecycle: Validated → Active → Out of Scope |
| **Context** | Background info: environment, prior work, user feedback |
| **Constraints** | Hard limits with rationale (tech stack, budget, compatibility) |
| **Current State** | THE resume point — what's done, what's next, any blockers |
| **Architecture** | Technical approach, file structure, data flow |
| **Key Decisions** | Decision log with outcome tracking (✓ Good / ⚠️ Revisit / — Pending) |
| **Tasks** | Phase-grouped checklist: DEFINE → DESIGN → BUILD → VERIFY → SHIP |
| **Discoveries** | Gotchas, workarounds, and surprises learned along the way |
| **Session Log** | Append-only timeline of work sessions |

Any agent, in any framework, reads this one file and immediately understands:
- What the project is and why it exists
- What's been done and what's left
- What was decided and whether those decisions worked
- Exactly what to do next

---

## How It Works

### 1. Start a Project

Copy the template, fill in the frontmatter, and work with your agent to define the project through conversation:

```
What This Is → Core Value → Requirements → Context → Constraints
```

The agent asks questions one at a time, proposes approaches, and gets your approval before moving forward.

### 2. Build Together

The agent creates tasks grouped by phase, works through them, and updates the file as it goes:
- Tasks get checked off: `- [ ]` → `- [x]`
- Decisions get logged with rationale and outcome
- Discoveries get captured in real-time
- Current State gets updated at the end of every session

### 3. Resume Anytime

When you start a new session, the agent reads project.md and picks up exactly where it left off. No re-explaining. No re-discovering. It reads Current State, checks the task list, and starts working.

### 4. Evolve the Document

The document has a lifecycle:
- **After each phase:** Requirements move from hypothesis → validated → out of scope
- **After each milestone:** Full review of all sections
- **"What This Is"** gets updated when the project drifts from its description
- **Key Decisions** get outcome tracking — did the choice prove correct?

---

## Key Design Decisions

### Single File, Not a Directory

Context fragmentation is the problem. One file forces discipline and makes resumption trivial — just read one file.

### Three-Tier Requirements Lifecycle

Not just "done/not done." Requirements are hypotheses until shipped and validated:

- **Validated** — Shipped and proven valuable (locked, requires discussion to change)
- **Active** — Current scope being built toward (hypotheses until validated)
- **Out of Scope** — Explicit boundaries with reasoning (prevents re-adding later)

This comes from the [Get Shit Done](https://github.com/gsd-build/get-shit-done) framework and prevents the common problem of requirements silently expanding.

### Core Value as a Section

The single most important thing. If everything else fails, this must work. One sentence that drives prioritization when tradeoffs arise. If you can't write this in one sentence, the project isn't well-defined yet.

### Key Decisions with Outcome Tracking

Not just "what was decided" but "did it work?" Every decision gets an outcome:
- ✓ Good — proved correct
- ⚠️ Revisit — may need reconsideration
- — Pending — too early to evaluate

This is the #1 context that gets lost between sessions.

### Current State as the Resume Point

An agent reading ONLY the Current State section knows exactly what to do next. Updated every session — non-negotiable.

### Evolution Rules

The document is living, not static. It has explicit rules for how it updates at phase transitions and milestones. Sections get reviewed. Requirements get promoted or retired. Decisions get outcome tracking.

---

## Quick Start

```bash
# 1. Copy the template
cp template.md project.md

# 2. Fill in frontmatter (project name, owner, date)

# 3. Work with your agent to fill sections through conversation

# 4. Agent creates tasks after design is approved

# 5. Agent updates Current State at the end of every session

# 6. Requirements move through: hypothesis → active → validated
```

### Validate Your project.md

```bash
python3 validate.py project.md
```

Checks for:
- Required frontmatter fields
- All required sections present
- Three-tier requirements lifecycle
- Task checkbox syntax
- Key Decisions table format
- Core Value conciseness
- Current State completeness

---

## File Structure

```
project.md
├── YAML Frontmatter     # project, status, created, updated, owner
├── What This Is          # Living description (2-3 sentences)
├── Core Value            # The ONE thing that matters
├── Requirements          # Validated / Active / Out of Scope
├── Context               # Background, environment, prior work
├── Constraints           # Hard limits with rationale
├── Current State         # Phase, last completed, next action, blockers
├── Architecture          # Technical approach, file structure
├── Key Decisions         # Decision | Rationale | Outcome
├── Tasks                 # Phase-grouped (DEFINE → DESIGN → BUILD → VERIFY → SHIP)
├── Discoveries           # Gotchas and surprises
├── References            # External links
└── Session Log           # Append-only timeline
```

---

## For Agent Framework Authors

### Integration Points

1. **On project start:** Create project.md from template, brainstorm with human
2. **On session start:** Read project.md, start with Current State
3. **During work:** Update tasks, log decisions, add discoveries
4. **On session end:** Update Current State, append Session Log
5. **On phase transition:** Run Evolution checklist

### Parsing

Agents can extract structured data:
- **Frontmatter:** YAML parse for metadata
- **Current State:** Key-value pairs for resume context
- **Tasks:** Checkbox regex (`- [ ]`, `- [x]`, `- [!]`) for progress
- **Key Decisions:** Table parse for decision history
- **Requirements:** Section-based for scope management

### Conformance

A valid project.md must:
1. Begin with YAML frontmatter containing all required fields
2. Contain all required sections with exact heading text
3. Use three-tier requirements (Validated / Active / Out of Scope)
4. Use checkbox syntax for tasks
5. Use table with Decision/Rationale/Outcome for Key Decisions
6. Use ISO 8601 dates in frontmatter

---

## Comparison with Alternatives

| Feature              | ProjectsMD | Jira/Linear | GitHub Issues | Superpowers |
|---------------------|------------|-------------|---------------|-------------|
| Single file          | ✅          | ❌           | ❌              | ❌            |
| Agent-native         | ✅          | ❌           | ❌              | ✅            |
| Session resumable    | ✅          | ❌           | ❌              | Partial     |
| Decision tracking    | Built-in   | ❌           | ❌              | Separate    |
| Requirements lifecycle | Built-in | ❌           | ❌              | Separate    |
| No external tools    | ✅          | ❌           | ❌              | ✅            |
| Human-readable       | ✅          | Partial     | ✅              | ✅            |
| Version controlled   | ✅          | ❌           | Indirect      | ✅            |

---

## Inspired By

- **[obra/superpowers](https://github.com/obra/superpowers)** — Agent skill pipeline (brainstorm → plan → execute), verification-before-completion philosophy, subagent-driven development
- **[Get Shit Done (GSD)](https://github.com/gsd-build/get-shit-done)** — PROJECT.md template, requirements lifecycle (Validated/Active/Out of Scope), decision outcome tracking, evolution rules, brownfield support
- **Agile/Scrum** — Sprint structure, retrospectives, incremental delivery
- **Shape Up** — Appetite-based scoping, circuit breakers, shaping before building
- **Amazon PRDs** — Working backwards from customer needs
- **RACI matrices** — Clear ownership (human = decision-maker, agent = executor)

---

## License

MIT — see [LICENSE](LICENSE).

---

## Contributing

This is a new standard and feedback is welcome. Open an issue to discuss changes to the specification, or submit a PR with improvements to the template, validator, or documentation.

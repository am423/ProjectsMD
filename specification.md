# project.md Specification v1.1

The definitive reference for the project.md file format. This document defines every section, convention, and rule.

## What is project.md?

project.md is a single-file project management format designed for agent-human collaboration. It serves as the single source of truth for a project — from initial idea through shipped product. Any agent, in any framework, can read a project.md file and immediately understand what the project is, where it stands, and what to do next.

## Motivation

Agents lose context between sessions. Humans lose track of decisions across weeks. Traditional project management scatters information across issue trackers, design docs, chat logs, and commit history. project.md collapses everything into one file that both humans and agents can read, update, and resume from.

**Design principles (in priority order):**
1. **Read once, understand the project.** An agent or human picking up this file for the first time should grasp the full picture in one read.
2. **Living, not static.** The document evolves with the project. Sections update as reality changes.
3. **Scannable.** It's read at the start of every session. If it's too long to scan, it's too long.
4. **Self-contained.** No external dependencies needed to understand the project state.
5. **Machine-parseable.** Agents can extract structured data (frontmatter, checkboxes, tables) without NLP.

## File Format

- **Encoding:** UTF-8
- **Format:** Markdown (CommonMark) with YAML frontmatter
- **Filename:** `project.md` (always this exact name)
- **Location:** Root of the project directory, or a dedicated projects directory
- **Line endings:** LF (Unix-style)

## YAML Frontmatter

Every project.md MUST begin with a YAML frontmatter block delimited by `---`.

### Required Fields

```yaml
---
project: "Project Name"
status: define
created: 2026-04-29
updated: 2026-04-29
owner: "Human Name"
---
```

### Optional Fields

```yaml
---
agent: "Agent Name / Framework"
tags: [tag1, tag2, tag3]
repository: "https://github.com/user/repo"
priority: high
---
```

### Field Definitions

| Field        | Type   | Required | Description                                                  |
|-------------|--------|----------|--------------------------------------------------------------|
| project     | string | yes      | Human-readable project name                                  |
| status      | enum   | yes      | Current project phase (see Status Values below)              |
| created     | date   | yes      | ISO 8601 date when project was created                       |
| updated     | date   | yes      | ISO 8601 date of last update                                 |
| owner       | string | yes      | Human who owns the project                                   |
| agent       | string | no       | Agent framework or name used                                  |
| tags        | array  | no       | Categorization tags                                          |
| repository  | string | no       | URL to associated repository                                 |
| priority    | enum   | no       | `low`, `medium`, `high`, `critical`                          |

### Status Values

| Status   | Meaning                                      |
|----------|----------------------------------------------|
| define   | Scoping the problem and requirements         |
| design   | Designing the solution and architecture      |
| build    | Actively implementing                        |
| verify   | Testing, reviewing, validating               |
| ship     | Deploying, documenting, wrapping up          |
| paused   | Work intentionally stopped                   |
| archived | Project complete or abandoned                |

---

## Required Sections

A valid project.md MUST contain these sections in order. Section headings use ATX-style (`##`).

### 1. What This Is

The current, accurate description of the project — what it does and who it's for. This is a living description: update it whenever reality drifts from what's written here.

```
## What This Is

A CLI tool that fetches weather data from OpenWeatherMap and displays
it in the terminal with color-coded conditions. For developers who
want weather info without leaving their terminal workflow.
```

Rules:
- 2-3 sentences capturing what the product does and who it's for
- Use the human's language and framing, not formal/technical prose
- **Update when the product evolves beyond this description** — if what you're building no longer matches this section, fix it
- No implementation details — that goes in Architecture
- No task lists — that goes in Tasks

### 2. Core Value

The single most important thing. If everything else fails, this must work. Drives prioritization when tradeoffs arise.

```
## Core Value

Fast, offline-capable weather display in the terminal — no browser,
no GUI, no waiting.
```

Rules:
- One sentence
- This is the "north star" — when you have to choose between features, this breaks the tie
- Rarely changes; if it does, it's a significant pivot
- If you can't write this in one sentence, the project isn't well-defined yet

### 3. Requirements

Three-tier lifecycle: Validated (shipped and proven), Active (current scope), Out of Scope (explicit boundaries).

```
## Requirements

### Validated

- ✓ Current weather by city name — v0.1
- ✓ File-based caching with 15-min TTL — v0.1

### Active

- [ ] Color-coded terminal output
- [ ] CLI argument parsing (city, --unit, --no-cache)
- [ ] Graceful error handling for network/API failures
- [ ] README with install instructions

### Out of Scope

- Forecast data — adds API complexity, different use case
- GUI / web interface — against core value (terminal-first)
- Historical weather data — separate project
- User accounts — no persistence needed for a CLI tool
```

**Validated** rules:
- Format: `- ✓ [Requirement] — [version/phase]`
- These are locked — changing them requires explicit discussion
- Only move here after a feature ships AND proves valuable
- Creates clear record of what's been validated

**Active** rules:
- Current scope being built toward
- These are **hypotheses until shipped and validated**
- Move to Validated when shipped and proven
- Move to Out of Scope if invalidated during development
- Keep focused on current milestone work

**Out of Scope** rules:
- Always include reasoning (prevents re-adding later)
- Includes: considered and rejected, deferred to future, explicitly excluded
- Helps manage expectations and prevent scope creep

### 4. Context

Background information that informs implementation decisions. Separate from Architecture — this is the "why we know what we know" section.

```
## Context

- Team of 15 developers, all terminal-native workflows
- Current solution: browser bookmarks to weather.com (2-3 min context switch)
- OpenWeatherMap has a generous free tier (1000 calls/10s)
- Windows PowerShell in older versions doesn't support ANSI by default
- Prior attempt at this (2024) failed because the developer chose Python
  and the startup time was too slow for a "check weather quick" use case
```

Rules:
- Technical environment or ecosystem
- Relevant prior work or experience
- User research or feedback themes
- Known issues or technical debt to address
- Update as new context emerges through development

### 5. Constraints

Hard limits on implementation choices. Include the "why" — constraints without rationale get questioned.

```
## Constraints

- **Tech Stack**: Go — single binary output, no runtime dependencies
- **API**: OpenWeatherMap free tier — no budget for paid services
- **Performance**: Response under 2 seconds — core value is "fast"
- **Compatibility**: Linux, macOS, Windows — team uses all three
- **Dependencies**: No CGo — required for cross-compilation
```

Rules:
- Format: `- **[Type]**: [What] — [Why]`
- Common types: Tech Stack, Timeline, Budget, Dependencies, Compatibility, Performance, Security
- Constraints can be relaxed later — log the relaxation in Key Decisions
- Be explicit about hard limits vs. preferences

### 6. Current State

**This is the most important section for session resumption.** Updated by the agent at the end of every session. An agent reading ONLY this section should know exactly what to do next.

```
## Current State

**Phase:** build
**Last completed:** Task 3 (API client with caching)
**In progress:** Task 4 (color-coded terminal output)
**Next action:** Implement temperature-to-color mapping in display.go
**Blockers:** None
**Notes:** OpenWeatherMap returns Celsius by default; added
  unit flag (metric/imperial) as part of Task 3 since it was trivial.
```

Rules:
- Updated EVERY session — this is non-negotiable
- "Next action" is specific enough to start immediately
- Include any context the next session needs (decisions made, discoveries)
- Remove blockers when resolved; don't leave stale entries

### 7. Architecture / Design

The technical approach. Enough detail for an agent to implement without guessing.

```
## Architecture

Single Go binary with three internal packages:

- `api/` — HTTP client for OpenWeatherMap, response parsing, caching
- `display/` — Terminal output formatting, color mapping
- `config/` — CLI flags, environment variables, defaults

**Data flow:**
1. CLI parses city name and unit flag
2. Check cache (file in ~/.cache/weather-cli/)
3. If cache miss or stale (> 15 min), call API
4. Format response with color coding
5. Display to stdout

**File structure:**
├── main.go
├── go.mod
├── api/
│   ├── client.go
│   ├── client_test.go
│   └── cache.go
├── display/
│   ├── format.go
│   └── format_test.go
├── config/
│   └── config.go
└── README.md
```

Rules:
- Include file structure for code projects
- Describe data flow for anything with I/O
- Note key technology decisions (and log them in Key Decisions)
- Keep it high-level — implementation details go in Tasks

### 8. Key Decisions

A running record of significant choices with outcome tracking. This is the #1 context that gets lost between sessions.

```
## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Go over Python | Single binary, no runtime deps, fast startup | ✓ Good |
| OpenWeatherMap API | Free tier sufficient, well-documented | ✓ Good |
| File-based cache | Persists across CLI invocations | ✓ Good |
| fatih/color for terminal colors | Handles Windows PowerShell ANSI | — Pending |
| Add --unit flag | Both Celsius and Fahrenheit users exist | — Pending |
```

Rules:
- Every significant decision gets logged — not just code decisions
- Include rationale (WHY, not just WHAT)
- Track outcome when known:
  - ✓ Good — decision proved correct
  - ⚠️ Revisit — decision may need reconsideration
  - — Pending — too early to evaluate
- Decisions can be reversed — update the Outcome to ⚠️ Revisit and add a new entry

### 9. Tasks

Checklist of work items grouped by project phase. Each task is actionable and verifiable.

```
## Tasks

### Phase: DEFINE

- [x] Identify target users and use cases
- [x] Research available weather APIs
- [x] Define success criteria

### Phase: DESIGN

- [x] Choose language and tooling
- [x] Design package structure
- [x] Define data models (API response, cache format)

### Phase: BUILD

- [x] Task 1: Project setup (go mod, main.go skeleton)
- [x] Task 2: API client — fetch current weather by city
- [x] Task 3: Caching layer — file-based, 15-min TTL
- [ ] Task 4: Display formatting with color coding
- [ ] Task 5: CLI argument parsing (city, --unit, --no-cache)
- [ ] Task 6: Error handling (network, API errors, invalid city)
- [ ] Task 7: Cross-platform build script

### Phase: VERIFY

- [ ] Unit tests pass for api/ and display/ packages
- [ ] Integration test: fetch → cache → display pipeline
- [ ] Manual test on Linux, macOS, Windows
- [ ] Performance check: response < 2 seconds

### Phase: SHIP

- [ ] README with install instructions and usage examples
- [ ] Release binary builds for all platforms
- [ ] Tag v1.0.0
```

Rules:
- Tasks are grouped by phase: DEFINE, DESIGN, BUILD, VERIFY, SHIP
- Each task is one concrete action (not "build the whole API")
- Tasks in BUILD phase should be numbered for reference
- Blocked tasks use `- [!]` with reason: `- [!] Task 8: ... — blocked on: external API approval`
- Tasks can have sub-bullets for additional detail:

```
- [ ] Task 4: Display formatting with color coding
  - Map temperature ranges to ANSI color codes
  - Handle terminal detection (no color in pipes)
  - Format: `City: 22°C ☀️  Clear | Humidity: 45% | Wind: 12 km/h`
```

### 10. Discoveries

Things learned during the project that aren't decisions but are important context.

```
## Discoveries

- OpenWeatherMap returns 401 (not 403) for invalid API keys
- Windows terminals don't support ANSI by default in older PowerShell
- The free tier actually allows 1000 calls/min, not 60
- Go's `net/http` already handles connection pooling
```

Rules:
- Capture gotchas, workarounds, and surprises
- Include enough context for the discovery to be useful
- Environment-specific quirks belong here
- Don't duplicate Key Decisions entries

### 11. References

Links to external resources.

```
## References

- [OpenWeatherMap API Docs](https://openweathermap.org/current)
- [Go color library](https://github.com/fatih/color)
- [ANSI escape codes reference](https://en.wikipedia.org/wiki/ANSI_escape_code)
```

### 12. Session Log

Append-only timeline of work sessions. Brief summaries, not transcripts.

```
## Session Log

- **2026-04-29** — Initial setup. Defined problem, criteria, constraints. Chose Go + OpenWeatherMap. (2 hours)
- **2026-04-30** — Added caching layer. Hit Windows file-locking issue, switched to atomic rename. (1.5 hours)
```

Rules:
- One entry per session
- Include date, what was accomplished, any notable blockers/issues
- Keep it brief — 1-2 sentences per session
- This is the project's narrative history

---

## Evolution

project.md is a living document. It evolves at phase transitions and milestone boundaries.

### After Each Phase Transition

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated (shipped + proven)? → Move to Validated with version/phase
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted
6. Update `status` in frontmatter

### After Each Milestone

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state (users, feedback, metrics)
5. Review Key Decisions outcomes — mark ✓ Good or ⚠️ Revisit

### Size Constraint

Keep project.md focused and scannable. It's read at the start of every session.

If it grows too large:
- Move detailed historical context to separate docs
- Keep only recent/relevant Key Decisions (full history in git)
- Summarize long Context sections
- The goal: "Read once, understand the project."

---

## Optional Sections

These sections are recommended when applicable but not required.

### Dependencies

```
## Dependencies

| Package                  | Version | Purpose                    |
|--------------------------|---------|----------------------------|
| github.com/fatih/color   | v1.16   | Cross-platform terminal color |
| github.com/spf13/cobra   | v1.8    | CLI argument parsing       |
```

### Risks

```
## Risks

| Risk                                    | Likelihood | Impact | Mitigation                        |
|-----------------------------------------|-----------|--------|-----------------------------------|
| OpenWeatherMap API changes format       | Low       | High   | Pin API version, add validation   |
| Go version incompatibility              | Low       | Medium | Test on Go 1.21+                  |
```

### Brownfield Projects

For existing codebases, initialize differently:

1. **Map the codebase first** — understand what exists
2. **Infer Validated requirements** from existing code:
   - What does the codebase actually do?
   - What patterns are established?
   - What's clearly working and relied upon?
3. **Gather Active requirements** from the human:
   - Present inferred current state
   - Ask what they want to build next
4. **Initialize:**
   - Validated = inferred from existing code
   - Active = human's goals for this work
   - Out of Scope = boundaries human specifies
   - Context = includes current codebase state

---

## Agent Instructions

When working with a project.md file:

1. **On session start:** Read the full file. Start with Current State. Verify the stated "next action" is still valid by checking the task list.
2. **During work:** Update task checkboxes as you complete them. Log decisions immediately — don't wait until session end. Track decision outcomes when known.
3. **On session end:** Update Current State (phase, last completed, next action, blockers). Append to Session Log. Update `updated` in frontmatter.
4. **Verification:** Never mark a task `[x]` without running verification. Use `- [!]` for blocked tasks.
5. **Scope changes:** If the human requests something outside Constraints/Out of Scope, flag it and log the decision either way.
6. **Evolution:** After completing a phase, run the Evolution checklist (see above).

## Human Instructions

When working with a project.md file:

1. **Review Current State** at the start of each session to understand where things stand.
2. **Review Requirements** — are Active requirements still the right ones?
3. **Review Key Decisions** periodically — flag any with ⚠️ Revisit that need discussion.
4. **Archive when done:** Change status to `archived` and add a final Session Log entry.

---

## Conformance

A file is a valid project.md if and only if:

1. It begins with valid YAML frontmatter containing all required fields
2. It contains all required sections with the exact heading text
3. Requirements use the three-tier lifecycle (Validated / Active / Out of Scope)
4. Tasks use checkbox syntax (`- [ ]`, `- [x]`, or `- [!]`)
5. Key Decisions use a table with Decision, Rationale, and Outcome columns
6. Dates in frontmatter are ISO 8601 format (YYYY-MM-DD)
7. Status is one of the defined enum values
8. "What This Is" is 2-3 sentences (not a multi-paragraph essay)

## Versioning

This specification follows semantic versioning:
- **Major:** Breaking changes to required sections or frontmatter fields
- **Minor:** New optional sections or fields, clarifications
- **Patch:** Corrections and wording improvements

### Changelog

- **v1.1** — Integrated GSD patterns: "What This Is" (living description), Core Value, three-tier requirements lifecycle (Validated/Active/Out of Scope), Key Decisions with Outcome tracking, Context section, Evolution rules, Constraints with rationale, brownfield support
- **v1.0** — Initial specification

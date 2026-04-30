---
project: "Weather CLI"
status: build
created: 2026-04-25
updated: 2026-04-29
owner: "Alex Chen"
agent: "Hermes Agent"
tags: [cli, go, weather, devtools]
repository: "https://github.com/alexchen/weather-cli"
priority: medium
---

## What This Is

A CLI tool that fetches current weather data from OpenWeatherMap and displays it in the terminal with color-coded conditions. For developers who want weather info without leaving their terminal workflow. Single binary, no runtime dependencies.

## Core Value

Fast, offline-capable weather display in the terminal — no browser, no GUI, no waiting.

## Requirements

### Validated

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

## Context

- Team of 15 developers, all terminal-native workflows
- Current solution: browser bookmarks to weather.com (2-3 min context switch per check)
- OpenWeatherMap free tier: 1000 calls per 10 seconds (documentation says 60/min — wrong)
- Windows PowerShell < 7 doesn't support ANSI by default — need library that handles this
- Prior attempt (2024) failed: Python choice caused slow startup (~800ms), killed the "quick check" UX
- `os.UserCacheDir()` returns different paths per OS: Linux (~/.cache), macOS (~/Library/Caches), Windows (%LocalAppData%)

## Constraints

- **Tech Stack**: Go — single binary output, fast startup, no runtime dependencies
- **API**: OpenWeatherMap free tier — no budget for paid services
- **Performance**: Response under 2 seconds — core value is "fast"
- **Compatibility**: Linux, macOS, Windows — team uses all three
- **Dependencies**: No CGo — required for cross-compilation to all platforms
- **Cache**: File-based in user cache dir — must work offline after first fetch

## Current State

**Phase:** build
**Last completed:** Task 3 (caching layer with file-based storage)
**In progress:** Task 4 (color-coded terminal output)
**Next action:** Implement temperature-to-color mapping in display/format.go using fatih/color library. Map: <10°C = blue, 10-25°C = green, >25°C = red. Handle terminal detection (no color in pipes via color.NoColor).
**Blockers:** None
**Notes:** Added --unit flag (metric/imperial) during Task 3 since the API supports it trivially. Cache uses atomic rename pattern to avoid corruption on Windows.

## Architecture

Single Go binary with three internal packages:

- `api/` — HTTP client for OpenWeatherMap, response parsing, caching
- `display/` — Terminal output formatting, color mapping
- `config/` — CLI flags, environment variables, defaults

**Data flow:**
1. CLI parses city name and unit flag
2. Check cache (file in ~/.cache/weather-cli/{city}.json)
3. If cache miss or stale (> 15 min), call API
4. Format response with color coding
5. Display to stdout

**File structure:**
```
weather-cli/
├── main.go
├── go.mod
├── go.sum
├── api/
│   ├── client.go          # HTTP client, API response parsing
│   ├── client_test.go     # Tests with httptest server
│   ├── cache.go           # File-based cache with TTL
│   └── cache_test.go      # Cache read/write/expiry tests
├── display/
│   ├── format.go          # Output formatting, color mapping
│   └── format_test.go     # Format output tests
├── config/
│   └── config.go          # CLI flags, env vars, defaults
└── README.md
```

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Go over Python | Single binary, no runtime deps, fast startup (<50ms) | ✓ Good |
| OpenWeatherMap API | Free tier sufficient, well-documented, stable | ✓ Good |
| File-based cache over in-memory | Persists across CLI invocations, no daemon needed | ✓ Good |
| Atomic rename for cache writes | Avoids corrupted reads on Windows | ✓ Good |
| fatih/color for terminal colors | Handles Windows PowerShell ANSI, well-tested | — Pending |
| Add --unit flag | Both Celsius and Fahrenheit users exist | — Pending |

## Tasks

### Phase: DEFINE

- [x] Identify target users and use cases
- [x] Research available weather APIs (OpenWeatherMap, WeatherAPI, Open-Meteo)
- [x] Define requirements with Alex

### Phase: DESIGN

- [x] Choose Go as implementation language
- [x] Design package structure (api, display, config)
- [x] Define data models: WeatherResponse, CacheEntry

### Phase: BUILD

- [x] Task 1: Project setup — go mod init, main.go skeleton, CLI arg parsing
- [x] Task 2: API client — fetch current weather by city name, parse JSON response
- [x] Task 3: Caching layer — file-based cache in ~/.cache/weather-cli/, 15-min TTL
- [ ] Task 4: Display formatting with color coding
- [ ] Task 5: CLI argument parsing (city, --unit, --no-cache, --help)
- [ ] Task 6: Error handling (network timeouts, API errors, invalid city, missing API key)
- [ ] Task 7: Cross-platform build script (GOOS/GOARCH matrix)

### Phase: VERIFY

- [ ] Unit tests pass for api/ and display/ packages
- [ ] Integration test: fetch → cache → display pipeline
- [ ] Manual test on Linux (Ubuntu 24.04)
- [ ] Manual test on macOS (latest)
- [ ] Manual test on Windows (PowerShell)
- [ ] Performance check: response < 2 seconds on cache miss

### Phase: SHIP

- [ ] README with install instructions, usage examples, screenshots
- [ ] goreleaser config for cross-platform binary builds
- [ ] Tag v1.0.0

## Discoveries

- OpenWeatherMap returns HTTP 401 (not 403) for invalid API keys — handle with specific message
- The free tier actually allows 1000 calls per 10 seconds, not 60/min — documentation is misleading
- Go's `net/http` already handles connection pooling — no custom transport needed
- `os.UserCacheDir()` returns different paths per OS: Linux (~/.cache), macOS (~/Library/Caches), Windows (%LocalAppData%)

## References

- [OpenWeatherMap Current Weather API](https://openweathermap.org/current)
- [fatih/color library](https://github.com/fatih/color)
- [Go cross-compilation guide](https://go.dev/doc/install/source#environment)
- [goreleaser](https://goreleaser.com/)

## Session Log

- **2026-04-25** — Project kickoff. Defined problem, requirements, constraints. Chose Go + OpenWeatherMap. Set up project structure. (1.5 hours)
- **2026-04-26** — Implemented API client (Task 2) with JSON parsing and httptest mocks. API returns 401 for bad keys — added specific handling. (2 hours)
- **2026-04-28** — Built caching layer (Task 3). Hit corruption on Windows with direct writes — switched to atomic rename. All cache tests passing. (2 hours)
- **2026-04-29** — Added --unit flag during Task 3 review. Started Task 4, researched color libraries. (1 hour, ongoing)

---
*Last updated: 2026-04-29 after completing Task 3*

# Project: core

## MCP tool usage (mandatory)

This repo ships four MCP servers in `.mcp.json`. All `✓ Connected`. Tools are **deferred** — load schemas via `ToolSearch` before first call, then invoke. Do **not** fall back to plain `grep` / `find` / guesswork when an MCP fits the task.

Decision order for code questions:
1. **Symbol-level intent** ("find references", "rename", "what calls X") → `precision-search` (Serena LSP).
2. **Structural / tree shape** ("show all classes in file", "method signatures", AST queries) → `structural-explorer`.
3. **Free-text / fuzzy lookup** across repo → `codebase-index`.
4. **External library API / docs** → `context7`.

### 1. `precision-search` (Serena, LSP-backed)

Best tool in this repo for semantic code work. Uses LSP — knows definitions, references, types.

Load: `ToolSearch query="precision-search serena symbol"` (or `select:` once names known).

Use for:
- find symbol definition / all references / call sites
- rename symbol, safe edits via symbol body replace
- list symbols in file/module
- onboarding a new area: `get_symbols_overview` first, then drill in
- writing/reading project memories Serena maintains in `.serena/memories/`

Skip for: pure text search on non-code files (use `codebase-index`).

### 2. `structural-explorer` (tree-sitter-analyzer)

AST-level inspection. Faster than Serena for shape questions; no LSP startup.

Load: `ToolSearch query="structural-explorer tree-sitter analyze"`.

Use for:
- "list all functions/classes in file X"
- method signatures, line ranges, decorators
- partial reads of huge files by AST node (avoid loading 5k-line files via `Read`)
- generating code-graph snapshots into `.codegraph/`

Skip for: cross-file reference search (use Serena).

### 3. `codebase-index` (`mcp__codebase-index__*`)

Indexed text/code search. Fastest for "where does string X appear".

Load: `ToolSearch query="select:mcp__codebase-index__search_code_advanced,mcp__codebase-index__find_files,mcp__codebase-index__get_file_summary,mcp__codebase-index__get_symbol_body"`.

Use for:
- `search_code_advanced` — keyword / regex across repo
- `find_files` — glob (replaces bash `find`)
- `get_file_summary` — quick orientation on a large module
- `refresh_index` after large refactors / pulls
- First clone: `set_project_path` → `build_deep_index`

### 4. `context7` (`mcp__context7__*`)

Up-to-date library docs. Training data is stale.

Load: `ToolSearch query="select:mcp__context7__resolve-library-id,mcp__context7__query-docs"`.

Trigger on **any** mention of a library / framework / SDK / CLI: Django, SQLAlchemy, pydantic, FastAPI, google-cloud-pubsub, pytest, uv, etc. Flow: `resolve-library-id` → `query-docs`.

Skip for: business-logic debugging, internal refactors, general programming concepts.

**Note:** `plugin:context7:context7` (npx) is duplicate of project `context7` (HTTP). Prefer the HTTP one.

---

## Commands

```bash
cargo build                          # debug build
cargo build --release                # release (opt-level=z, LTO, single codegen unit)
cargo run -- -p 25 -s 5 -l 15       # run with custom durations (minutes)
cargo check                          # fast type check, no binary
cargo clippy                         # lint
cargo fmt                            # format

# Linux cross-compile (requires `cross`)
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
```

No tests exist in this codebase yet.

## Architecture

Single-binary TUI app. Four source files:

- **`src/main.rs`** — entry point, 250ms event loop, all `draw_*` render functions, per-view key handlers, audio (`rodio` sine waves), desktop notifications (`notify-rust`). Timer countdown and task time accumulation happen on each tick.
- **`src/app.rs`** — `App` struct (entire runtime state), `Task`, `Mode`, `TimerState`, `View`, `InputMode` enums, all state-mutation methods. `App` serializes to JSON at `~/.local/share/pomodorust/state.json`.
- **`src/settings.rs`** — `Settings` (durations, theme, notifications), `ColorTheme` enum, persisted as TOML at `~/.config/pomodorust/config.toml`. Also contains `draw_settings()` — the only render function not in `main.rs`.
- **`src/theme.rs`** — `Theme` struct with all `ratatui::style::Color` fields. Four built-ins: Default, Dracula, Solarized, Nord.

### State flow

CLI args override `Settings` → `App::load_with_settings()` deserializes JSON state and attaches settings. On quit, `App::save()` writes both JSON state and TOML config.

### View navigation

`App::current_view` drives which `draw_*` renders. Views: `Timer`, `TaskList` (default), `Statistics`, `Settings` (popup overlay), `TaskDetails`. `previous_view` tracks return destination from Settings/Details.

### Key event routing

`handle_key_event` dispatches by `InputMode` first (Editing mode captures all keys for task-name input), then by `current_view` in Normal mode.

### Timer logic

Each tick in `TimerState::Running` subtracts elapsed time from `app.time_remaining` and adds to active task's `time_spent`. On expiry, `app.next_mode()` cycles Pomodoro → ShortBreak (or LongBreak every 4th pomodoro) → Pomodoro, auto-restarting if a task is active.

## Release

GitHub Actions triggers on published release. Linux targets use `cross` (Docker-based). Artifacts are `.tar.gz` binaries uploaded to the GitHub Release + `cargo publish` to crates.io via `CARGO_REGISTRY_TOKEN`.

Linux cross-compile requires `libasound2-dev` and `libdbus-1-dev` — pre-build steps configured in `Cargo.toml` under `[workspace.metadata.cross.target.*]`.

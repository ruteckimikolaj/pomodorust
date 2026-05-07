# pomodorust — task backlog

## Bugs

- [x] **Stale `active_task_index` after task completion** (`app.rs:260`)
  Completing a task leaves `active_task_index` pointing at the now-completed task.
  `toggle_timer` silently no-ops because of the `!task.completed` guard. Timer appears broken
  until user navigates away. Fix: clear or advance `active_task_index` in `complete_active_task`.

- [x] **`next_setting` / `previous_setting` hardcode row count** (`app.rs:400,407`)
  Both wrap at `% 5` / `4`. Adding or removing a settings row silently breaks keyboard nav.
  Fix: derive the count from the actual number of settings rows.

- [x] **`get_data_path` / `get_config_path` ignore XDG env vars** (`app.rs:10`)
  Manually appends `.local/share` and `.config` to `home_dir()`.
  The `directories` crate (already a dep) provides `ProjectDirs::from()` which correctly
  reads `XDG_DATA_HOME` and `XDG_CONFIG_HOME` on Linux.

- [x] **No way to delete active (non-completed) tasks**
  Stats view has `d` to delete completed tasks. Active tasks have no delete key —
  a mistyped task name can't be removed without completing it first.

---

## Code quality

- [x] **Split render functions out of `main.rs`** (`main.rs` is 776 lines)
  `draw_timer`, `draw_task_list`, `draw_statistics`, `draw_task_details` could move to
  `src/ui.rs` (or `src/ui/` module). `draw_settings` already lives in `settings.rs` as
  the odd one out — unify the pattern either way.

- [x] **Separate UI state from business state in `App`**
  `settings_selection`, `completed_task_list_state`, `previous_view`, `input_mode`,
  `current_input` are pure UI cursor/input state mixed into the persisted `App` struct.
  Introduce a `UiState` struct for these so the persistence boundary is explicit.

- [x] **`settings_selection` should not persist to JSON**
  It's a UI cursor; surviving restart is surprising. Mark `#[serde(skip)]`.

- [x] **Extract repeated duration-clamping logic in `modify_setting`** (`app.rs:411`)
  The Pomodoro / Short Break / Long Break arms each repeat the same
  `(current as i64 + delta).max(1)` pattern. Extract `fn bump_duration_mins`.

- [x] **Long break interval hardcoded to 4** (`app.rs:224`)
  `% 4` is buried in `next_mode`. Should be a `Settings` field so it can be
  adjusted in the settings view alongside the other timer params.

---

## New features
- [x] **Pomodoro history sparkline in Statistics view**
  ratatui ships a `Sparkline` widget with zero new deps. Show last N days of
  daily pomodoro counts above the completed task list.

- [x] **Search / filter active task list**
  `/` key enters filter mode; typing narrows the list. Useful once the task list grows. Filter input field is replacing bottom bar with the shortcuts while used. Filter completed tasks in Statistics view as well.

- [x] **Rename / edit task name**
  No way to fix a typo without delete + recreate. Add `e` key in TaskList view to
  re-enter Editing mode with the current task name pre-filled in `current_input`.

- [x] **Configurable long-break interval** (ties to bug above)
  Add `long_break_interval: u32` to `Settings` (default 4). Expose in settings view.

- [x] **Skip current timer segment**
  Add `s` key in Timer view (or `n` for "next") to jump to the next mode immediately,
  the same as natural expiry — plays sound, shows notification, advances `next_mode`.

- [x] **Daily statistics panel**
  Show today's pomodoro count and focused time in the Statistics view summary block.
  Filter by `creation_date` >= start of today (already have `chrono`). maybe some nice chart ?

- [x] **Task notes / description field** (ratatui-textarea modal)
  Add optional `notes: Option<String>` to `Task`. Show and edit in TaskDetails view.
  Allows tracking what was done in a session. we could consider using it for the search feature as well. and maybe use https://github.com/ratatui/ratatui-textarea for multi-line input.
- [ ] **Grouping / projects**
  Add optional `project: Option<String>` to `Task`. Allow filtering by project in TaskList and Statistics views. Maybe add a "Projects" view that shows stats by project and lists tasks grouped by project.


---

## Nice-to-haves
- [ ] **Grouping tasks by day in statistics view**
  Instead of one long list of completed tasks, group them by day with a header for each day. This would make it easier to see daily patterns and find specific completed tasks.

- [ ] **Export statistics to CSV**
  Single keybind (e.g. `x`) in Statistics view. Write completed tasks as CSV to
  `~/.local/share/pomodorust/export.csv`. Fields: name, created, completed,
  time_spent_secs, pomodoros.

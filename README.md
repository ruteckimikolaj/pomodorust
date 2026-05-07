![Version](https://img.shields.io/badge/version-0.2.0-blue)
![https://spdx.org/licenses/CC-BY-NC-SA-4.0.json](https://img.shields.io/badge/License-CC%20%7C%20BY--NC--SA%204.0-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-blue)
![Rust Version](https://img.shields.io/badge/rust-1.70.0-blue)
![https://crates.io/crates/pomodorust](https://img.shields.io/crates/v/pomodorust)
![GitHub Repo Stars](https://img.shields.io/github/stars/ruteckimikolaj/pomodorust?style=social)

# Pomodorust 🍅

A minimalist, powerful, terminal-based Pomodoro timer written in Rust to help you stay focused and productive.

## ✨ Features

- **Classic Pomodoro Workflow** — Cycle through focused work sessions, short breaks, and long breaks with configurable durations and a configurable long-break interval.
- **Task Management** — Create, rename, reorder, complete, and delete tasks. Assign tasks to projects using `@tag` syntax.
- **Task Notes** — Attach multi-line notes to any task. Edit with a full-screen modal editor (`Shift+E`).
- **Search & Filter** — Press `/` in any view to filter tasks by name, notes, or project tag in real time.
- **Statistics** — Weekly bar chart, daily and all-time summary, and a searchable list of completed tasks with per-task details (time focused, pomodoros, dates).
- **Four Color Themes** — Default, Dracula, Solarized, Nord. Switchable from the settings panel.
- **Desktop Notifications** — Native desktop notifications when a timer segment ends.
- **SQLite Persistence** — All tasks and app state are stored in a local SQLite database (`~/.local/share/pomodorust/pomodorust.db`). Settings persist separately as TOML (`~/.config/pomodorust/config.toml`).
- **Cross-Platform** — Runs on macOS and Linux.

## 📸 Screenshots

![](/assets/all-gif.webp)

## 📦 Installation

### Using Cargo

```shell
cargo install pomodorust
```

## 🚀 Usage

### Command-Line Arguments

Override timer durations at launch (in minutes):

```shell
pomodorust -p 25 -s 5 -l 15
```

| Argument                 | Alias | Description                     |
| ------------------------ | ----- | ------------------------------- |
| `--pomodoro-duration`    | `-p`  | Pomodoro duration in minutes    |
| `--short-break-duration` | `-s`  | Short break duration in minutes |
| `--long-break-duration`  | `-l`  | Long break duration in minutes  |

### In-App Controls

Controls are context-sensitive and shown at the bottom of each view.

**Global**

| Key | Action |
| --- | ------ |
| `o` | Open settings panel |
| `q` | Quit |

**Task List**

| Key | Action |
| --- | ------ |
| `↑` / `k`, `↓` / `j` | Navigate tasks |
| `Shift+↑` / `K`, `Shift+↓` / `J` | Reorder selected task |
| `n` | New task (supports `@project` tag, e.g. `Buy milk @work`) |
| `e` | Rename selected task |
| `Shift+E` | Edit notes for selected task |
| `Enter` | Toggle task complete / incomplete |
| `d` | Delete selected task |
| `/` | Enter filter mode — type to narrow list by name, notes, or `@project` |
| `Esc` | Clear filter / cancel input |
| `Space` | Start / pause timer |
| `Tab` | Switch to Statistics view |

**Timer**

| Key | Action |
| --- | ------ |
| `Space` | Start / pause timer |
| `n` | Skip to next segment |
| `Tab` | Switch to Task List view |

**Statistics**

| Key | Action |
| --- | ------ |
| `↑` / `k`, `↓` / `j` | Navigate completed tasks |
| `/` | Filter completed tasks by name, notes, or `@project` |
| `Enter` | View task details |
| `d` / `Delete` | Delete selected task |
| `Tab` | Switch to Timer view |

**Task Details**

| Key | Action |
| --- | ------ |
| `Shift+E` | Edit notes |
| `Enter` / `Esc` | Back |

**Settings**

| Key | Action |
| --- | ------ |
| `↑` / `k`, `↓` / `j` | Select setting |
| `←` / `h`, `→` / `l` | Decrease / increase value |
| `Tab` | Close settings |

### Projects

Append `@tag` anywhere in a task name to assign it to a project:

```
Write report @work
Buy groceries @personal
Fix login bug @work
```

The tag is stripped from the display name and shown as a coloured badge. Filter by `@work` or just `work` in any search field.

### Data & Config Locations

| File | Purpose |
| ---- | ------- |
| `~/.local/share/pomodorust/pomodorust.db` | Tasks and app state (SQLite) |
| `~/.config/pomodorust/config.toml` | Timer durations, theme, notification settings |

On first launch after upgrading from an older version, existing `state.json` data is automatically migrated to SQLite.

## ❤️ Contributing

Contributions, bug reports, and feature suggestions are welcome.

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/your-feature`).
3. Make your changes and commit (`git commit -m 'Add feature'`).
4. Push and open a Pull Request.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![https://spdx.org/licenses/CC-BY-NC-SA-4.0.json](https://img.shields.io/badge/License-CC%20%7C%20BY--NC--SA%204.0-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)
![Rust Version](https://img.shields.io/badge/rust-1.70.0-blue)
![https://crates.io/crates/pomodorust](https://img.shields.io/crates/v/pomodorust)
![Homebrew](https://img.shields.io/badge/homebrew-coming%20soon-orange)
![GitHub Repo Stars](https://img.shields.io/github/stars/ruteckimikolaj/pomodorust?style=social)

# Pomodorust 🍅

A minimalist, powerful, terminal-based Pomodoro timer written in Rust to help you stay focused and productive. This project was born out of a desire to learn Rust and create a practical tool for daily use.

## ✨ Features

- **Classic Pomodoro Workflow**: Cycle through focused work sessions, short breaks, and long breaks.
- **Task Management**: Create, complete, and archive tasks to track your work.
- **Task Prioritization**: Reorder your active tasks to focus on what's most important.
- **Detailed Statistics**: View details for completed tasks, including creation/completion dates and total time spent.
- **Customizable Timers**: Set custom durations for pomodoro, short break, and long break sessions via command-line arguments.
- **Color Themes**: Personalize your experience with built-in themes (Default, Dracula, Solarized, Nord).
- **Desktop Notifications**: Get native desktop notifications when a timer finishes.
- **Cross-Platform**: Built with Rust, it runs on macOS, Linux, and Windows.

## 📸 Screenshots
![](/assets/all-gif.webp)

## 📦 Installation

### Using Cargo

If you have the Rust toolchain installed, you can install `pomodorust` directly from crates.io:

```shell
cargo install pomodorust
```

### Using Homebrew (macOS)

*Coming soon! Once the project is published, you will be able to install it with:*

```shell
brew install pomodorust
```

## 🚀 Usage

### Command-Line Arguments

You can start the application with custom timer durations (in minutes):

```shell
pomodorust -p 25 -s 5 -l 15
```

| **Argument**             | **Alias** | **Description**                 |
| ------------------------ | --------- | ------------------------------- |
| `--pomodoro-duration`    | `-p`      | Pomodoro duration in minutes    |
| `--short-break-duration` | `-s`      | Short break duration in minutes |
| `--long-break-duration`  | `-l`      | Long break duration in minutes  |

### In-App Controls

The application is controlled entirely with your keyboard. The controls are context-aware and displayed at the bottom of each view.

**Global**

- `o`: Open the settings panel.
- `q`: Quit the application.

**Task List View**

- `↑`/`k` & `↓`/`j`: Navigate tasks.
- `Shift` + `↑`/`K` & `Shift` + `↓`/`J`: Move/reorder the selected task.
- `n`: Create a new task.
- `Enter`: Mark the selected task as complete/incomplete.
- `Space`: Start the timer for the selected task.
- `Tab`: Switch to the Statistics view.

**Statistics View**

- `↑`/`k` & `↓`/`j`: Navigate completed tasks.
- `Enter`: View details for the selected task.
- `d` / `Delete`: Delete the selected completed task.
- `Tab`: Switch to the Timer view.

## ❤️ Contributing

This is my first project in Rust, and I'm passionate about making it better! I welcome all forms of contributions, from feature suggestions and bug reports to code improvements and pull requests.

If you have ideas on how to improve the code, make it more idiomatic, or enhance its performance, please don't hesitate to open an issue or a pull request. Your feedback is incredibly valuable.

1. **Fork the repository.**
2. **Create a new branch** (`git checkout -b feature/your-feature-name`).
3. **Make your changes.**
4. **Commit your changes** (`git commit -m 'Add some amazing feature'`).
5. **Push to the branch** (`git push origin feature/your-feature-name`).
6. **Open a Pull Request.**

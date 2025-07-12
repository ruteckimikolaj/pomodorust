use crate::settings::{ColorTheme, Settings};
use chrono::{DateTime, Utc};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Helper function to get the path for the state file.
pub fn get_data_path() -> Option<PathBuf> {
    if let Some(user_dirs) = UserDirs::new() {
        let mut path = user_dirs.home_dir().to_path_buf();
        path.push(".local");
        path.push("share");
        path.push("pomodorust");
        path.push("state.json");
        return Some(path);
    }
    None
}

/// Helper function to get the path for the config file.
pub fn get_config_path() -> Option<PathBuf> {
    if let Some(user_dirs) = UserDirs::new() {
        let mut path = user_dirs.home_dir().to_path_buf();
        path.push(".config");
        path.push("pomodorust");
        path.push("config.toml");
        return Some(path);
    }
    None
}

/// Represents a single task for the Pomodoro timer.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub name: String,
    pub completed: bool,
    pub pomodoros: u32,
    pub time_spent: Duration,
    pub creation_date: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
}

impl Task {
    /// Creates a new task with a given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            completed: false,
            pomodoros: 0,
            time_spent: Duration::from_secs(0),
            creation_date: Utc::now(),
            completion_date: None,
        }
    }
}

/// Represents the different timer modes in the Pomodoro technique.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Mode {
    #[default]
    Pomodoro,
    ShortBreak,
    LongBreak,
}

impl Mode {
    /// Returns the duration of the timer mode based on settings.
    pub fn duration(&self, settings: &Settings) -> Duration {
        match self {
            Mode::Pomodoro => settings.pomodoro_duration,
            Mode::ShortBreak => settings.short_break_duration,
            Mode::LongBreak => settings.long_break_duration,
        }
    }

    /// Returns the title of the timer mode.
    pub fn title(&self) -> &'static str {
        match self {
            Mode::Pomodoro => "Pomodoro",
            Mode::ShortBreak => "Short Break",
            Mode::LongBreak => "Long Break",
        }
    }
}

/// Represents the current state of the timer.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum TimerState {
    #[default]
    Paused,
    Running,
}

/// Represents the different views of the application.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum View {
    Timer,
    #[default]
    TaskList,
    Statistics,
    Settings,
    TaskDetails,
}

/// Represents the different input modes.
#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// The main application state.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct App {
    pub mode: Mode,
    pub state: TimerState,
    pub time_remaining: Duration,
    pub pomodoros_completed_total: u32,
    #[serde(skip)]
    pub should_quit: bool,
    pub current_view: View,
    #[serde(skip)]
    pub previous_view: View,
    pub tasks: Vec<Task>,
    pub active_task_index: Option<usize>,
    #[serde(skip)]
    pub input_mode: InputMode,
    #[serde(skip)]
    pub current_input: String,
    pub completed_task_list_state: Option<usize>,
    #[serde(skip)]
    pub settings: Settings,
    pub settings_selection: usize,
}

impl Default for App {
    fn default() -> Self {
        let settings = Settings::default();
        Self {
            mode: Mode::Pomodoro,
            state: TimerState::Paused,
            time_remaining: settings.pomodoro_duration,
            pomodoros_completed_total: 0,
            should_quit: false,
            current_view: View::TaskList,
            previous_view: View::TaskList,
            tasks: vec![],
            active_task_index: None,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            completed_task_list_state: None,
            settings,
            settings_selection: 0,
        }
    }
}

impl App {
    /// Loads an App instance from a file, or creates a new one.
    pub fn load_with_settings(settings: Settings) -> Self {
        let mut app: App = if let Some(path) = get_data_path() {
            fs::read_to_string(path)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        } else {
            App::default()
        };

        app.settings = settings;
        app.time_remaining = app.mode.duration(&app.settings);
        app
    }

    /// Saves the current state of the app to a file.
    pub fn save(&self) {
        // Save the main app state (tasks, etc.)
        if let Some(path) = get_data_path() {
            if let Some(parent) = path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    if let Ok(json) = serde_json::to_string_pretty(&self) {
                        let _ = fs::write(path, json);
                    }
                }
            }
        }
        // Save the settings
        self.settings.save();
    }

    /// Toggles the timer between running and paused states.
    pub fn toggle_timer(&mut self) {
        if let Some(index) = self.active_task_index {
            if !self.tasks[index].completed {
                match self.state {
                    TimerState::Paused => self.state = TimerState::Running,
                    TimerState::Running => self.state = TimerState::Paused,
                }
            }
        }
    }

    /// Resets the timer to the current mode's full duration.
    pub fn reset_timer(&mut self) {
        self.state = TimerState::Paused;
        self.time_remaining = self.mode.duration(&self.settings);
    }

    /// Switches to the next appropriate timer mode.
    pub fn next_mode(&mut self) -> Mode {
        let previous_mode = self.mode;
        if self.mode == Mode::Pomodoro {
            self.pomodoros_completed_total += 1;
            if let Some(index) = self.active_task_index {
                if let Some(task) = self.tasks.get_mut(index) {
                    task.pomodoros += 1;
                }
            }

            if self.pomodoros_completed_total % 4 == 0 {
                self.mode = Mode::LongBreak;
            } else {
                self.mode = Mode::ShortBreak;
            }
        } else {
            self.mode = Mode::Pomodoro;
        }
        self.reset_timer();
        if let Some(index) = self.active_task_index {
            if !self.tasks[index].completed {
                self.state = TimerState::Running;
            }
        }
        previous_mode
    }

    /// Sets the current mode explicitly.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.reset_timer();
    }

    /// Adds a new task to the list from the current input.
    pub fn submit_task(&mut self) {
        if !self.current_input.is_empty() {
            self.tasks.push(Task::new(self.current_input.clone()));
            self.current_input.clear();
            if self.tasks.len() == 1 {
                self.active_task_index = Some(0);
            }
        }
        self.input_mode = InputMode::Normal;
    }

    /// Toggles the completion status of the active task.
    pub fn complete_active_task(&mut self) {
        if let Some(index) = self.active_task_index {
            if let Some(task) = self.tasks.get_mut(index) {
                task.completed = !task.completed;
                if task.completed {
                    task.completion_date = Some(Utc::now());
                    self.state = TimerState::Paused;
                    self.reset_timer();
                } else {
                    task.completion_date = None;
                }
            }
        }
    }

    /// Moves the selection to the next uncompleted task.
    pub fn next_task(&mut self) {
        let uncompleted_tasks_indices: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.completed)
            .map(|(i, _)| i)
            .collect();

        if uncompleted_tasks_indices.is_empty() {
            self.active_task_index = None;
            return;
        }

        let current_selection = self.active_task_index.unwrap_or(0);
        let next_index_in_uncompleted = uncompleted_tasks_indices
            .iter()
            .position(|&i| i == current_selection)
            .map_or(0, |i| (i + 1) % uncompleted_tasks_indices.len());

        self.active_task_index = Some(uncompleted_tasks_indices[next_index_in_uncompleted]);
    }

    /// Moves the selection to the previous uncompleted task.
    pub fn previous_task(&mut self) {
        let uncompleted_tasks_indices: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.completed)
            .map(|(i, _)| i)
            .collect();

        if uncompleted_tasks_indices.is_empty() {
            self.active_task_index = None;
            return;
        }

        let current_selection = self.active_task_index.unwrap_or(0);
        let current_pos = uncompleted_tasks_indices
            .iter()
            .position(|&i| i == current_selection)
            .unwrap_or(0);
        let next_index_in_uncompleted = if current_pos == 0 {
            uncompleted_tasks_indices.len() - 1
        } else {
            current_pos - 1
        };

        self.active_task_index = Some(uncompleted_tasks_indices[next_index_in_uncompleted]);
    }

    /// Moves the currently active task up in the list.
    pub fn move_active_task_up(&mut self) {
        if let Some(index) = self.active_task_index {
            if index > 0 {
                self.tasks.swap(index, index - 1);
                self.active_task_index = Some(index - 1);
            }
        }
    }

    /// Moves the currently active task down in the list.
    pub fn move_active_task_down(&mut self) {
        if let Some(index) = self.active_task_index {
            if index < self.tasks.len() - 1 {
                self.tasks.swap(index, index + 1);
                self.active_task_index = Some(index + 1);
            }
        }
    }

    // --- Statistics View Methods ---

    pub fn next_completed_task(&mut self) {
        let completed_count = self.tasks.iter().filter(|t| t.completed).count();
        if completed_count == 0 {
            return;
        }
        let i = self.completed_task_list_state.map_or(0, |i| (i + 1) % completed_count);
        self.completed_task_list_state = Some(i);
    }

    pub fn previous_completed_task(&mut self) {
        let completed_count = self.tasks.iter().filter(|t| t.completed).count();
        if completed_count == 0 {
            return;
        }
        let i = self.completed_task_list_state.map_or(0, |i| {
            if i == 0 {
                completed_count - 1
            } else {
                i - 1
            }
        });
        self.completed_task_list_state = Some(i);
    }

    /// Deletes the selected completed task.
    pub fn delete_selected_completed_task(&mut self) {
        if let Some(selected_index) = self.completed_task_list_state {
            let completed_indices: Vec<usize> = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.completed)
                .map(|(i, _)| i)
                .collect();

            if let Some(&task_index_to_delete) = completed_indices.get(selected_index) {
                self.tasks.remove(task_index_to_delete);

                if let Some(active_idx) = self.active_task_index {
                    if active_idx > task_index_to_delete {
                        self.active_task_index = Some(active_idx - 1);
                    }
                }
                self.completed_task_list_state = None;
            }
        }
    }

    // --- Settings View Methods ---
    pub fn next_setting(&mut self) {
        self.settings_selection = (self.settings_selection + 1) % 5; // 5 settings
    }

    pub fn previous_setting(&mut self) {
        if self.settings_selection > 0 {
            self.settings_selection -= 1;
        } else {
            self.settings_selection = 4; // 5 settings, so index is 4
        }
    }

    pub fn modify_setting(&mut self, increase: bool) {
        let delta: i64 = if increase { 1 } else { -1 };
        match self.settings_selection {
            0 => {
                // Pomodoro Duration
                let current = self.settings.pomodoro_duration.as_secs() / 60;
                let new = (current as i64 + delta).max(1);
                self.settings.pomodoro_duration = Duration::from_secs(new as u64 * 60);
            }
            1 => {
                // Short Break
                let current = self.settings.short_break_duration.as_secs() / 60;
                let new = (current as i64 + delta).max(1);
                self.settings.short_break_duration = Duration::from_secs(new as u64 * 60);
            }
            2 => {
                // Long Break
                let current = self.settings.long_break_duration.as_secs() / 60;
                let new = (current as i64 + delta).max(1);
                self.settings.long_break_duration = Duration::from_secs(new as u64 * 60);
            }
            3 => {
                // Theme
                self.settings.theme = match self.settings.theme {
                    ColorTheme::Default => ColorTheme::Dracula,
                    ColorTheme::Dracula => ColorTheme::Solarized,
                    ColorTheme::Solarized => ColorTheme::Nord,
                    ColorTheme::Nord => ColorTheme::Default,
                };
            }
            4 => {
                // Desktop Notifications
                self.settings.desktop_notifications = !self.settings.desktop_notifications;
            }
            _ => {}
        }
        if self.state == TimerState::Paused {
            self.reset_timer();
        }
    }
}

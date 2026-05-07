use crate::settings::{ColorTheme, Settings};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const SETTINGS_ROW_COUNT: usize = 6;

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "pomodorust")
}

pub fn get_data_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.data_local_dir().join("state.json"))
}

pub fn get_config_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.config_dir().join("config.toml"))
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

/// The main application state (business logic + persistence).
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
    pub tasks: Vec<Task>,
    pub active_task_index: Option<usize>,
    #[serde(skip)]
    pub settings: Settings,
}

fn bump_duration_mins(d: Duration, delta: i64) -> Duration {
    let mins = (d.as_secs() / 60) as i64;
    Duration::from_secs((mins + delta).max(1) as u64 * 60)
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
            tasks: vec![],
            active_task_index: None,
            settings,
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

            let interval = self.settings.long_break_interval.max(1) as u32;
            if self.pomodoros_completed_total % interval == 0 {
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


    /// Toggles the completion status of the active task.
    pub fn complete_active_task(&mut self) {
        if let Some(index) = self.active_task_index {
            if let Some(task) = self.tasks.get_mut(index) {
                task.completed = !task.completed;
                if task.completed {
                    task.completion_date = Some(Utc::now());
                    self.state = TimerState::Paused;
                    self.reset_timer();
                    self.active_task_index = self.tasks.iter().enumerate()
                        .find(|(_, t)| !t.completed)
                        .map(|(i, _)| i);
                } else {
                    task.completion_date = None;
                }
            }
        }
    }

    /// Deletes the currently selected active (non-completed) task.
    pub fn delete_active_task(&mut self) {
        if let Some(index) = self.active_task_index {
            self.tasks.remove(index);
            self.state = TimerState::Paused;
            self.reset_timer();
            self.active_task_index = self.tasks.iter().enumerate()
                .find(|(_, t)| !t.completed)
                .map(|(i, _)| i);
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
}

/// Transient UI navigation state — not persisted.
pub struct UiState {
    pub settings_selection: usize,
    pub completed_task_list_state: Option<usize>,
    pub previous_view: View,
    pub input_mode: InputMode,
    pub current_input: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            settings_selection: 0,
            completed_task_list_state: None,
            previous_view: View::TaskList,
            input_mode: InputMode::Normal,
            current_input: String::new(),
        }
    }
}

impl UiState {
    pub fn next_setting(&mut self) {
        self.settings_selection = (self.settings_selection + 1) % SETTINGS_ROW_COUNT;
    }

    pub fn previous_setting(&mut self) {
        if self.settings_selection > 0 {
            self.settings_selection -= 1;
        } else {
            self.settings_selection = SETTINGS_ROW_COUNT - 1;
        }
    }

    pub fn modify_setting(&mut self, app: &mut App, increase: bool) {
        let delta: i64 = if increase { 1 } else { -1 };
        match self.settings_selection {
            0 => app.settings.pomodoro_duration = bump_duration_mins(app.settings.pomodoro_duration, delta),
            1 => app.settings.short_break_duration = bump_duration_mins(app.settings.short_break_duration, delta),
            2 => app.settings.long_break_duration = bump_duration_mins(app.settings.long_break_duration, delta),
            3 => {
                app.settings.theme = match app.settings.theme {
                    ColorTheme::Default => ColorTheme::Dracula,
                    ColorTheme::Dracula => ColorTheme::Solarized,
                    ColorTheme::Solarized => ColorTheme::Nord,
                    ColorTheme::Nord => ColorTheme::Default,
                };
            }
            4 => app.settings.desktop_notifications = !app.settings.desktop_notifications,
            5 => {
                let current = app.settings.long_break_interval as i64;
                app.settings.long_break_interval = (current + delta).max(1) as u32;
            }
            _ => {}
        }
        if app.state == TimerState::Paused {
            app.reset_timer();
        }
    }

    pub fn next_completed_task(&mut self, app: &App) {
        let count = app.tasks.iter().filter(|t| t.completed).count();
        if count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| (i + 1) % count);
        self.completed_task_list_state = Some(i);
    }

    pub fn previous_completed_task(&mut self, app: &App) {
        let count = app.tasks.iter().filter(|t| t.completed).count();
        if count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| {
            if i == 0 { count - 1 } else { i - 1 }
        });
        self.completed_task_list_state = Some(i);
    }

    pub fn delete_selected_completed_task(&mut self, app: &mut App) {
        if let Some(selected) = self.completed_task_list_state {
            let completed_indices: Vec<usize> = app.tasks.iter().enumerate()
                .filter(|(_, t)| t.completed)
                .map(|(i, _)| i)
                .collect();
            if let Some(&idx) = completed_indices.get(selected) {
                app.tasks.remove(idx);
                if let Some(active) = app.active_task_index {
                    if active > idx {
                        app.active_task_index = Some(active - 1);
                    }
                }
                self.completed_task_list_state = None;
            }
        }
    }

    pub fn submit_task(&mut self, app: &mut App) {
        if !self.current_input.is_empty() {
            app.tasks.push(Task::new(self.current_input.clone()));
            self.current_input.clear();
            if app.tasks.len() == 1 {
                app.active_task_index = Some(0);
            }
        }
        self.input_mode = InputMode::Normal;
    }
}

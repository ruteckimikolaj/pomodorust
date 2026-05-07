use crate::settings::Settings;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub mod ui_state;
pub use ui_state::UiState;

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "pomodorust")
}

pub fn get_data_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.data_local_dir().join("state.json"))
}

pub fn get_db_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.data_local_dir().join("pomodorust.db"))
}

pub fn get_config_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.config_dir().join("config.toml"))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub name: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    pub completed: bool,
    pub pomodoros: u32,
    pub time_spent: Duration,
    pub creation_date: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(name: String, project: Option<String>) -> Self {
        Self {
            name,
            notes: None,
            project,
            completed: false,
            pomodoros: 0,
            time_spent: Duration::from_secs(0),
            creation_date: Utc::now(),
            completion_date: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Mode {
    #[default]
    Pomodoro,
    ShortBreak,
    LongBreak,
}

impl Mode {
    pub fn duration(&self, settings: &Settings) -> Duration {
        match self {
            Mode::Pomodoro => settings.pomodoro_duration,
            Mode::ShortBreak => settings.short_break_duration,
            Mode::LongBreak => settings.long_break_duration,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Mode::Pomodoro => "Pomodoro",
            Mode::ShortBreak => "Short Break",
            Mode::LongBreak => "Long Break",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum TimerState {
    #[default]
    Paused,
    Running,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum View {
    Timer,
    #[default]
    TaskList,
    Statistics,
    Settings,
    TaskDetails,
}

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
    Filtering,
    EditingNotes,
}

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

pub(super) fn bump_duration_mins(d: Duration, delta: i64) -> Duration {
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
    pub fn load_with_settings(settings: Settings) -> Self {
        if let Some(db_path) = get_db_path() {
            if let Some(parent) = db_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let is_new_db = !db_path.exists();
            if let Ok(mut conn) = crate::db::open_and_init(&db_path) {
                // One-time migration from legacy JSON on first run
                if is_new_db {
                    if let Some(legacy) = Self::try_load_json() {
                        let _ = crate::db::save_to(&mut conn, &legacy);
                        let mut app = legacy;
                        app.settings = settings;
                        app.time_remaining = app.mode.duration(&app.settings);
                        return app;
                    }
                }
                let s = crate::db::load_from(&conn);
                let time_remaining = s.time_remaining_secs
                    .map(Duration::from_secs)
                    .unwrap_or_else(|| s.mode.duration(&settings));
                return App {
                    mode: s.mode,
                    state: TimerState::Paused,
                    time_remaining,
                    pomodoros_completed_total: s.pomodoros_total,
                    should_quit: false,
                    current_view: s.current_view,
                    tasks: s.tasks,
                    active_task_index: s.active_task_index,
                    settings,
                };
            }
        }
        let mut app = App::default();
        app.settings = settings;
        app.time_remaining = app.mode.duration(&app.settings);
        app
    }

    fn try_load_json() -> Option<Self> {
        let path = get_data_path()?;
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) {
        if let Some(db_path) = get_db_path() {
            if let Some(parent) = db_path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    if let Ok(mut conn) = crate::db::open_and_init(&db_path) {
                        let _ = crate::db::save_to(&mut conn, self);
                    }
                }
            }
        }
        self.settings.save();
    }

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

    pub fn reset_timer(&mut self) {
        self.state = TimerState::Paused;
        self.time_remaining = self.mode.duration(&self.settings);
    }

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

    pub fn skip_segment(&mut self) -> Mode {
        let previous_mode = self.mode;
        if self.mode == Mode::Pomodoro {
            let interval = self.settings.long_break_interval.max(1) as u32;
            if (self.pomodoros_completed_total + 1) % interval == 0 {
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

    pub fn next_task(&mut self) {
        let indices: Vec<usize> = self.tasks.iter().enumerate()
            .filter(|(_, t)| !t.completed)
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() { self.active_task_index = None; return; }
        let cur = self.active_task_index.unwrap_or(0);
        let next = indices.iter().position(|&i| i == cur)
            .map_or(0, |p| (p + 1) % indices.len());
        self.active_task_index = Some(indices[next]);
    }

    pub fn previous_task(&mut self) {
        let indices: Vec<usize> = self.tasks.iter().enumerate()
            .filter(|(_, t)| !t.completed)
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() { self.active_task_index = None; return; }
        let cur = self.active_task_index.unwrap_or(0);
        let pos = indices.iter().position(|&i| i == cur).unwrap_or(0);
        let prev = if pos == 0 { indices.len() - 1 } else { pos - 1 };
        self.active_task_index = Some(indices[prev]);
    }

    pub fn move_active_task_up(&mut self) {
        if let Some(index) = self.active_task_index {
            if index > 0 {
                self.tasks.swap(index, index - 1);
                self.active_task_index = Some(index - 1);
            }
        }
    }

    pub fn move_active_task_down(&mut self) {
        if let Some(index) = self.active_task_index {
            if index < self.tasks.len() - 1 {
                self.tasks.swap(index, index + 1);
                self.active_task_index = Some(index + 1);
            }
        }
    }
}

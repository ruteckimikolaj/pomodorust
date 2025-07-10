use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Helper function to get the path for the state file.
fn get_data_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "pomodorust", "Pomodorust") {
        let mut path = proj_dirs.config_dir().to_path_buf();
        path.push("state.json");
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
}

impl Task {
    /// Creates a new task with a given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            completed: false,
            pomodoros: 0,
            time_spent: Duration::from_secs(0),
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
    /// Returns the duration of the timer mode.
    pub fn duration(&self) -> Duration {
        match self {
            Mode::Pomodoro => Duration::from_secs(25 * 60),
            Mode::ShortBreak => Duration::from_secs(5 * 60),
            Mode::LongBreak => Duration::from_secs(15 * 60),
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
#[derive(Serialize, Deserialize, Default)]
pub enum View {
    Timer,
    #[default]
    TaskList,
    Statistics,
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
    pub should_quit: bool,
    pub current_view: View,
    pub tasks: Vec<Task>,
    pub active_task_index: Option<usize>,
    #[serde(skip)]
    pub input_mode: InputMode,
    #[serde(skip)]
    pub current_input: String,
    pub completed_task_list_state: Option<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Pomodoro,
            state: TimerState::Paused,
            time_remaining: Mode::Pomodoro.duration(),
            pomodoros_completed_total: 0,
            should_quit: false,
            current_view: View::TaskList,
            tasks: vec![],
            active_task_index: None,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            completed_task_list_state: None,
        }
    }
}

impl App {
    /// Creates a new App instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads an App instance from a file, or creates a new one.
    pub fn load_or_new() -> Self {
        if let Some(path) = get_data_path() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(mut app) = serde_json::from_str::<App>(&data) {
                    app.input_mode = InputMode::Normal;
                    app.current_input = String::new();
                    app.should_quit = false;
                    return app;
                }
            }
        }
        App::new()
    }

    /// Saves the current state of the app to a file.
    pub fn save(&self) {
        if let Some(path) = get_data_path() {
            if let Some(parent) = path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    if let Ok(json) = serde_json::to_string_pretty(self) {
                        let _ = fs::write(path, json);
                    }
                }
            }
        }
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
        self.time_remaining = self.mode.duration();
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
                    self.state = TimerState::Paused;
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

    // --- Statistics View Methods ---

    /// Moves selection down in the completed tasks list.
    pub fn next_completed_task(&mut self) {
        let completed_count = self.tasks.iter().filter(|t| t.completed).count();
        if completed_count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| (i + 1) % completed_count);
        self.completed_task_list_state = Some(i);
    }

    /// Moves selection up in the completed tasks list.
    pub fn previous_completed_task(&mut self) {
        let completed_count = self.tasks.iter().filter(|t| t.completed).count();
        if completed_count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| {
            if i == 0 { completed_count - 1 } else { i - 1 }
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

            if let Some(task_index_to_delete) = completed_indices.get(selected_index) {
                self.tasks.remove(*task_index_to_delete);
                // After deletion, reset selection
                self.completed_task_list_state = None;
            }
        }
    }
}

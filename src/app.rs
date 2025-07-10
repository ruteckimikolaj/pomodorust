use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a single task for the Pomodoro timer.
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mode {
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimerState {
    Paused,
    Running,
}

/// Represents the different views of the application.
#[derive(Serialize, Deserialize)]
pub enum View {
    Timer,
    TaskList,
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
}

impl App {
    /// Creates a new App instance.
    pub fn new() -> Self {
        Self {
            mode: Mode::Pomodoro,
            state: TimerState::Paused,
            time_remaining: Mode::Pomodoro.duration(),
            pomodoros_completed_total: 0,
            should_quit: false,
            current_view: View::TaskList, // Start in task list view
            tasks: vec![],
            active_task_index: None,
            input_mode: InputMode::Normal,
            current_input: String::new(),
        }
    }

    /// Toggles the timer between running and paused states.
    pub fn toggle_timer(&mut self) {
        // Timer can only run if an active task is not completed
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
        // Automatically start the next session if there's an active, uncompleted task
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
            // If this is the first task, select it
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
        let current_pos = uncompleted_tasks_indices.iter().position(|&i| i == current_selection).unwrap_or(0);
        let next_index_in_uncompleted = if current_pos == 0 {
            uncompleted_tasks_indices.len() - 1
        } else {
            current_pos - 1
        };

        self.active_task_index = Some(uncompleted_tasks_indices[next_index_in_uncompleted]);
    }
}

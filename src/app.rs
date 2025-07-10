use std::time::Duration;

/// Represents a single task for the Pomodoro timer.
#[derive(Clone)]
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
#[derive(Clone, Copy, PartialEq)]
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
#[derive(Clone, Copy, PartialEq)]
pub enum TimerState {
    Paused,
    Running,
}

/// Represents the different views of the application.
pub enum View {
    Timer,
    TaskList,
    // We can add more views like statistics later
}

/// The main application state.
pub struct App {
    pub mode: Mode,
    pub state: TimerState,
    pub time_remaining: Duration,
    pub pomodoros_completed_total: u32,
    pub should_quit: bool,
    pub current_view: View,
    pub tasks: Vec<Task>,
    pub active_task_index: Option<usize>,
    // This will be used for inputting new task names in a future iteration
    pub input_mode: bool, 
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
            current_view: View::Timer,
            tasks: vec![],
            active_task_index: None,
            input_mode: false,
            current_input: String::new(),
        }
    }
    
    /// Toggles the timer between running and paused states.
    pub fn toggle_timer(&mut self) {
        // Timer can only run if a task is active
        if self.active_task_index.is_some() {
             match self.state {
                TimerState::Paused => self.state = TimerState::Running,
                TimerState::Running => self.state = TimerState::Paused,
            }
        }
    }

    /// Resets the timer to the current mode's full duration.
    pub fn reset_timer(&mut self) {
        self.state = TimerState::Paused;
        self.time_remaining = self.mode.duration();
    }

    /// Switches to the next appropriate timer mode.
    pub fn next_mode(&mut self) {
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
        // Automatically start the next session
        self.state = TimerState::Running;
    }

    /// Sets the current mode explicitly.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.reset_timer();
    }

    /// Adds a new task to the list.
    pub fn add_task(&mut self, name: String) {
        self.tasks.push(Task::new(name));
    }

    /// Toggles the completion status of the active task.
    pub fn complete_active_task(&mut self) {
        if let Some(index) = self.active_task_index {
            if let Some(task) = self.tasks.get_mut(index) {
                task.completed = !task.completed;
                // If we are completing the active task, pause the timer
                if task.completed {
                    self.state = TimerState::Paused;
                }
            }
        }
    }

    /// Moves the selection in the task list down.
    pub fn next_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.active_task_index {
            Some(i) => {
                if i >= self.tasks.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.active_task_index = Some(i);
    }

    /// Moves the selection in the task list up.
    pub fn previous_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.active_task_index {
            Some(i) => {
                if i == 0 { self.tasks.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.active_task_index = Some(i);
    }
}

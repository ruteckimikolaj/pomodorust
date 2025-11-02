use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents the different timer modes in the Pomodoro technique.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default, Debug)]
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default, Debug)]
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
#[derive(Default, Debug)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

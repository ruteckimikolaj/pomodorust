use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

use super::{App, InputMode, Task, TimerState, View, bump_duration_mins};
use crate::settings::ColorTheme;

const SETTINGS_ROW_COUNT: usize = 6;

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

use ratatui_textarea::TextArea;

use super::{App, InputMode, Task, TimerState, View, bump_duration_mins};
use crate::settings::ColorTheme;

/// Splits `"Buy milk @work"` → `("Buy milk", Some("work"))`.
/// The `@tag` can appear anywhere; it is stripped from the name.
pub fn parse_project(input: &str) -> (String, Option<String>) {
    if let Some(at) = input.rfind('@') {
        let rest = &input[at + 1..];
        let end = rest.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .unwrap_or(rest.len());
        if end > 0 {
            let project = rest[..end].to_string();
            let name = format!("{}{}", &input[..at], &rest[end..]).trim().to_string();
            if !name.is_empty() {
                return (name, Some(project));
            }
        }
    }
    (input.trim().to_string(), None)
}

pub fn task_matches_filter(task: &Task, filter: &str) -> bool {
    task.name.to_lowercase().contains(filter)
        || task.notes.as_deref().map_or(false, |n| n.to_lowercase().contains(filter))
        || task.project.as_deref().map_or(false, |p| {
            let tag = format!("@{}", p.to_lowercase());
            tag.contains(filter) || p.to_lowercase().contains(filter)
        })
}

const SETTINGS_ROW_COUNT: usize = 6;

pub struct UiState {
    pub settings_selection: usize,
    pub completed_task_list_state: Option<usize>,
    pub previous_view: View,
    pub input_mode: InputMode,
    pub current_input: String,
    pub filter_input: String,
    pub editing_task_index: Option<usize>,
    pub notes_textarea: Option<TextArea<'static>>,
    pub editing_notes_task_index: Option<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            settings_selection: 0,
            completed_task_list_state: None,
            previous_view: View::TaskList,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            filter_input: String::new(),
            editing_task_index: None,
            notes_textarea: None,
            editing_notes_task_index: None,
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

    fn filtered_completed_count(&self, app: &App) -> usize {
        let filter = self.filter_input.to_lowercase();
        app.tasks.iter()
            .filter(|t| t.completed && (filter.is_empty() || task_matches_filter(t, &filter)))
            .count()
    }

    pub fn next_completed_task(&mut self, app: &App) {
        let count = self.filtered_completed_count(app);
        if count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| (i + 1) % count);
        self.completed_task_list_state = Some(i);
    }

    pub fn previous_completed_task(&mut self, app: &App) {
        let count = self.filtered_completed_count(app);
        if count == 0 { return; }
        let i = self.completed_task_list_state.map_or(0, |i| {
            if i == 0 { count - 1 } else { i - 1 }
        });
        self.completed_task_list_state = Some(i);
    }

    pub fn delete_selected_completed_task(&mut self, app: &mut App) {
        if let Some(selected) = self.completed_task_list_state {
            let filter = self.filter_input.to_lowercase();
            let completed_indices: Vec<usize> = app.tasks.iter().enumerate()
                .filter(|(_, t)| t.completed && (filter.is_empty() || task_matches_filter(t, &filter)))
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

    fn open_notes_for_task(&mut self, idx: usize, app: &App) {
        if let Some(task) = app.tasks.get(idx) {
            let lines: Vec<String> = task.notes.as_deref()
                .unwrap_or("")
                .lines()
                .map(|l| l.to_owned())
                .collect();
            let mut textarea = if lines.is_empty() {
                TextArea::default()
            } else {
                TextArea::new(lines)
            };
            textarea.set_placeholder_text("Type your notes here…");
            self.notes_textarea = Some(textarea);
            self.editing_notes_task_index = Some(idx);
            self.input_mode = InputMode::EditingNotes;
        }
    }

    // Open notes editor for the selected completed task (called from TaskDetails)
    pub fn start_edit_notes(&mut self, app: &App) {
        if let Some(selected) = self.completed_task_list_state {
            let filter = self.filter_input.to_lowercase();
            if let Some(idx) = app.tasks.iter().enumerate()
                .filter(|(_, t)| t.completed && (filter.is_empty() || task_matches_filter(t, &filter)))
                .nth(selected)
                .map(|(i, _)| i)
            {
                self.open_notes_for_task(idx, app);
            }
        }
    }

    // Open notes editor for the active task (called from TaskList)
    pub fn start_edit_notes_active(&mut self, app: &App) {
        if let Some(idx) = app.active_task_index {
            self.open_notes_for_task(idx, app);
        }
    }

    pub fn submit_notes(&mut self, app: &mut App) {
        if let (Some(textarea), Some(idx)) = (self.notes_textarea.take(), self.editing_notes_task_index.take()) {
            if let Some(task) = app.tasks.get_mut(idx) {
                let text = textarea.lines().join("\n");
                task.notes = if text.trim().is_empty() { None } else { Some(text) };
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn cancel_notes(&mut self) {
        self.notes_textarea = None;
        self.editing_notes_task_index = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn next_filtered_task(&mut self, app: &mut App) {
        let filter = self.filter_input.to_lowercase();
        if filter.is_empty() { app.next_task(); return; }
        let indices: Vec<usize> = app.tasks.iter().enumerate()
            .filter(|(_, t)| !t.completed && task_matches_filter(t, &filter))
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() { return; }
        let cur = app.active_task_index.unwrap_or(usize::MAX);
        let next = indices.iter().position(|&i| i == cur)
            .map_or(0, |p| (p + 1) % indices.len());
        app.active_task_index = Some(indices[next]);
    }

    pub fn previous_filtered_task(&mut self, app: &mut App) {
        let filter = self.filter_input.to_lowercase();
        if filter.is_empty() { app.previous_task(); return; }
        let indices: Vec<usize> = app.tasks.iter().enumerate()
            .filter(|(_, t)| !t.completed && task_matches_filter(t, &filter))
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() { return; }
        let cur = app.active_task_index.unwrap_or(usize::MAX);
        let pos = indices.iter().position(|&i| i == cur).unwrap_or(0);
        let prev = if pos == 0 { indices.len() - 1 } else { pos - 1 };
        app.active_task_index = Some(indices[prev]);
    }

    pub fn start_rename(&mut self, app: &App) {
        if let Some(idx) = app.active_task_index {
            if let Some(task) = app.tasks.get(idx) {
                if !task.completed {
                    self.editing_task_index = Some(idx);
                    self.current_input = match &task.project {
                        Some(p) => format!("{} @{}", task.name, p),
                        None => task.name.clone(),
                    };
                    self.input_mode = InputMode::Editing;
                }
            }
        }
    }

    pub fn submit_task(&mut self, app: &mut App) {
        if let Some(idx) = self.editing_task_index.take() {
            if !self.current_input.is_empty() {
                let (name, project) = parse_project(&self.current_input);
                if let Some(task) = app.tasks.get_mut(idx) {
                    task.name = name;
                    task.project = project;
                }
            }
            self.current_input.clear();
            self.input_mode = InputMode::Normal;
        } else {
            if !self.current_input.is_empty() {
                let (name, project) = parse_project(&self.current_input);
                app.tasks.push(Task::new(name, project));
                self.current_input.clear();
                if app.tasks.len() == 1 {
                    app.active_task_index = Some(0);
                }
            }
            self.input_mode = InputMode::Normal;
        }
    }
}

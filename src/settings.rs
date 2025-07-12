use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};

use crate::app::{get_config_path, App};
use crate::theme::Theme;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorTheme {
    #[default]
    Default,
    Dracula,
    Solarized,
    Nord,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerializableSettings {
    pomodoro_duration_mins: u64,
    short_break_duration_mins: u64,
    long_break_duration_mins: u64,
    theme: ColorTheme,
    desktop_notifications: bool,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub pomodoro_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub theme: ColorTheme,
    pub desktop_notifications: bool,
}

impl From<SerializableSettings> for Settings {
    fn from(s: SerializableSettings) -> Self {
        Self {
            pomodoro_duration: Duration::from_secs(s.pomodoro_duration_mins * 60),
            short_break_duration: Duration::from_secs(s.short_break_duration_mins * 60),
            long_break_duration: Duration::from_secs(s.long_break_duration_mins * 60),
            theme: s.theme,
            desktop_notifications: s.desktop_notifications,
        }
    }
}

impl From<&Settings> for SerializableSettings {
    fn from(s: &Settings) -> Self {
        Self {
            pomodoro_duration_mins: s.pomodoro_duration.as_secs() / 60,
            short_break_duration_mins: s.short_break_duration.as_secs() / 60,
            long_break_duration_mins: s.long_break_duration.as_secs() / 60,
            theme: s.theme,
            desktop_notifications: s.desktop_notifications,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pomodoro_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            theme: ColorTheme::Default,
            desktop_notifications: true,
        }
    }
}

impl Settings {
    /// Loads settings from the config file, or creates a default one.
    pub fn load() -> Self {
        if let Some(path) = get_config_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(serializable) = toml::from_str::<SerializableSettings>(&content) {
                    return serializable.into();
                }
            }
        }
        // If loading fails or path doesn't exist, create default and save it.
        let default_settings = Settings::default();
        default_settings.save();
        default_settings
    }

    /// Saves the current settings to the config file.
    pub fn save(&self) {
        if let Some(path) = get_config_path() {
            if let Some(parent) = path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    let serializable = SerializableSettings::from(self);
                    if let Ok(toml_string) = toml::to_string_pretty(&serializable) {
                        let _ = fs::write(path, toml_string);
                    }
                }
            }
        }
    }
}

/// A helper function to create a centered rect using up certain percentages of the available rect.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_settings(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let area = centered_rect(60, 50, frame.area());

    let settings_block = Block::default()
        .title(" ⚙ SETTINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(theme.accent_color).bg(theme.base_bg))
        .title_alignment(Alignment::Center);
    
    let inner_area = settings_block.inner(area);

    // Create a layout to include a footer for help text
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .margin(1)
        .split(inner_area);

    let rows = vec![
        Row::new(vec![
            Cell::from("Pomodoro Duration"),
            Cell::from(format!("< {} mins >", app.settings.pomodoro_duration.as_secs() / 60)),
        ]),
        Row::new(vec![
            Cell::from("Short Break"),
            Cell::from(format!("< {} mins >", app.settings.short_break_duration.as_secs() / 60)),
        ]),
        Row::new(vec![
            Cell::from("Long Break"),
            Cell::from(format!("< {} mins >", app.settings.long_break_duration.as_secs() / 60)),
        ]),
        Row::new(vec![
            Cell::from("Color Theme"),
            Cell::from(format!("< {:?} >", app.settings.theme)),
        ]),
        Row::new(vec![
            Cell::from("Desktop Notifications"),
            Cell::from(format!("< {} >", if app.settings.desktop_notifications { "On" } else { "Off" })),
        ]),
    ].into_iter().map(|r| r.height(1).style(Style::default().fg(theme.base_fg))).collect::<Vec<Row>>();

    let mut table_state = TableState::default();
    table_state.select(Some(app.settings_selection));

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
        .row_highlight_style(Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    // Render the popup
    frame.render_widget(Clear, area); // This clears the area before rendering the popup
    frame.render_widget(settings_block, area);
    frame.render_stateful_widget(table, inner_layout[0], &mut table_state);

    // Render the help text in the footer
    let help_text = " [↑/↓] Navigate | [←/→] Change | [Tab] Back ";
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.help_text_fg));
    frame.render_widget(help_paragraph, inner_layout[1]);
}

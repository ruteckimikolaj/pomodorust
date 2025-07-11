use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::app::App;
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
pub struct Settings {
    pub pomodoro_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub theme: ColorTheme,
    pub desktop_notifications: bool,
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

pub fn draw_settings(frame: &mut Frame, app: &mut App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4)])
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title("⚙️ Settings")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    let settings_list = vec![
        format!(
            "Pomodoro Duration:      < {} mins >",
            app.settings.pomodoro_duration.as_secs() / 60
        ),
        format!(
            "Short Break Duration:   < {} mins >",
            app.settings.short_break_duration.as_secs() / 60
        ),
        format!(
            "Long Break Duration:    < {} mins >",
            app.settings.long_break_duration.as_secs() / 60
        ),
        format!("Color Theme:            < {:?} >", app.settings.theme),
        format!(
            "Desktop Notifications:  < {} >",
            if app.settings.desktop_notifications {
                "On"
            } else {
                "Off"
            }
        ),
    ];

    let items: Vec<ListItem> = settings_list
        .iter()
        .map(|s| ListItem::new(s.clone()).style(Style::default().fg(theme.base_fg)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.settings_selection));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let help_text = if chunks[2].width > 80 {
        " [Tab] Back to Timer | [↑/↓] Navigate | [←/→] Change Value | [q] Quit "
    } else {
        " [Tab] [↑/↓] [←/→] [q] "
    };
    frame.render_widget(
        Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.help_text_fg)),
            )
            .alignment(Alignment::Center),
        chunks[2],
    );
}

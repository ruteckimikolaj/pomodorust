use ratatui::{prelude::*, widgets::*};

use super::centered_rect;
use crate::app::{App, UiState};
use crate::settings::Theme;

pub fn draw_settings(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let area = centered_rect(60, 50, frame.area());

    let settings_block = Block::default()
        .title(" ⚙ SETTINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(theme.accent_color).bg(theme.base_bg))
        .title_alignment(Alignment::Center);

    let inner_area = settings_block.inner(area);
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .margin(1)
        .split(inner_area);

    let rows: Vec<Row> = vec![
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
        Row::new(vec![
            Cell::from("Long Break Interval"),
            Cell::from(format!("< {} pomodoros >", app.settings.long_break_interval)),
        ]),
    ]
    .into_iter()
    .map(|r| r.height(1).style(Style::default().fg(theme.base_fg)))
    .collect();

    let mut table_state = TableState::default();
    table_state.select(Some(ui.settings_selection));

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
        .row_highlight_style(Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_widget(Clear, area);
    frame.render_widget(settings_block, area);
    frame.render_stateful_widget(table, inner_layout[0], &mut table_state);
    frame.render_widget(
        Paragraph::new(" [↑/↓] Navigate | [←/→] Change | [Tab] Back ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.help_text_fg)),
        inner_layout[1],
    );
}

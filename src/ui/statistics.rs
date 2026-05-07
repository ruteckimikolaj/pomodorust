use std::time::Duration;

use ratatui::{prelude::*, widgets::*};

use crate::app::{App, UiState};
use crate::settings::Theme;

pub fn draw_statistics(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title(" Σ STATISTICS ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    let total_time_spent: Duration = app.tasks.iter().map(|t| t.time_spent).sum();
    let time_spent_formatted = format!(
        "{}h {}m",
        total_time_spent.as_secs() / 3600,
        (total_time_spent.as_secs() % 3600) / 60
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("Total Pomodoros: {}", app.pomodoros_completed_total)),
            Line::from(format!("Total Time Focused: {time_spent_formatted}")),
        ])
        .block(Block::default().borders(Borders::ALL).title("Summary")
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)))
        .alignment(Alignment::Center),
        chunks[1],
    );

    let completed_tasks: Vec<_> = app.tasks.iter().filter(|t| t.completed).collect();
    let mut list_state = ListState::default();
    list_state.select(ui.completed_task_list_state);

    let list_items: Vec<ListItem> = completed_tasks
        .iter()
        .map(|task| {
            let content = format!("{:<40} | {} ●", task.name, task.pomodoros);
            ListItem::new(Line::from(content)).style(Style::default().fg(theme.base_fg))
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Completed & Archived Tasks")
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)))
        .highlight_style(Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[2], &mut list_state);

    let help_text = if chunks[3].width > 80 {
        " [Tab] Timer | [↑/↓] Navigate | [Enter] Details | [d]elete Selected Task | [q] Quit "
    } else {
        " [Tab] [↑/↓] [Ent] [d] [q] "
    };
    frame.render_widget(
        Paragraph::new(help_text)
            .block(Block::default().title("Controls").borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(theme.help_text_fg)))
            .alignment(Alignment::Center),
        chunks[3],
    );
}

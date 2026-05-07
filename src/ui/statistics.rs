use std::time::Duration;

use chrono::Local;
use ratatui::{prelude::*, widgets::*};

use crate::app::{App, UiState};
use crate::settings::Theme;

const SPARKLINE_DAYS: usize = 14;

fn daily_pomodoros(app: &App) -> Vec<u64> {
    let today = Local::now().date_naive();
    let mut counts = vec![0u64; SPARKLINE_DAYS];
    for task in &app.tasks {
        if let Some(completed) = task.completion_date {
            let task_date = completed.with_timezone(&Local).date_naive();
            let days_ago = (today - task_date).num_days();
            if days_ago >= 0 && (days_ago as usize) < SPARKLINE_DAYS {
                let idx = SPARKLINE_DAYS - 1 - days_ago as usize;
                counts[idx] += task.pomodoros as u64;
            }
        }
    }
    counts
}

pub fn draw_statistics(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
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

    let spark_data = daily_pomodoros(app);
    frame.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Last 14 days ")
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
            )
            .data(spark_data.iter().copied())
            .style(Style::default().fg(theme.pomodoro_color)),
        chunks[2],
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
    frame.render_stateful_widget(list, chunks[3], &mut list_state);

    let help_text = if chunks[4].width > 80 {
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
        chunks[4],
    );
}

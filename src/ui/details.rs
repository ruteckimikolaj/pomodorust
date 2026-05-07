use chrono::prelude::*;
use ratatui::{prelude::*, widgets::*};

use crate::app::{App, UiState};
use crate::settings::Theme;

pub fn draw_task_details(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title(" i DETAILS ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::uniform(1))
        .style(Style::default().fg(theme.base_fg).bg(theme.base_bg));
    let inner_area = main_block.inner(chunks[1]);
    frame.render_widget(main_block, chunks[1]);

    if let Some(selected_completed_index) = ui.completed_task_list_state {
        let filter = ui.filter_input.to_lowercase();
        let completed_tasks: Vec<_> = app
            .tasks
            .iter()
            .filter(|t| {
                t.completed && (filter.is_empty() || t.name.to_lowercase().contains(&filter))
            })
            .collect();
        if let Some(task) = completed_tasks.get(selected_completed_index) {
            let created: DateTime<Local> = task.creation_date.into();
            let completed_str = task.completion_date.map_or_else(
                || "N/A".to_string(),
                |dt| {
                    let local_dt: DateTime<Local> = dt.into();
                    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
                },
            );
            let time_spent_formatted = format!(
                "{}h {}m {}s",
                task.time_spent.as_secs() / 3600,
                (task.time_spent.as_secs() % 3600) / 60,
                task.time_spent.as_secs() % 60
            );
            let time_to_complete_str =
                if let (Some(completed), created) = (task.completion_date, task.creation_date) {
                    let d = completed.signed_duration_since(created);
                    format!(
                        "{}d {}h {}m",
                        d.num_days(),
                        d.num_hours() % 24,
                        d.num_minutes() % 60
                    )
                } else {
                    "N/A".to_string()
                };

            let rows = vec![
                Row::new(vec![Cell::from("Task"), Cell::from(task.name.clone())]),
                Row::new(vec![Cell::from("Status"), Cell::from("✓ Completed")])
                    .style(Style::default().fg(theme.running_fg)),
                Row::new(vec![
                    Cell::from("Created"),
                    Cell::from(created.format("%Y-%m-%d %H:%M").to_string()),
                ]),
                Row::new(vec![Cell::from("Completed"), Cell::from(completed_str)]),
                Row::new(vec![
                    Cell::from("Time to Complete"),
                    Cell::from(time_to_complete_str),
                ]),
                Row::new(vec![
                    Cell::from("Time Focused"),
                    Cell::from(time_spent_formatted),
                ]),
                Row::new(vec![
                    Cell::from("Pomodoros"),
                    Cell::from(format!("{} ●", task.pomodoros)),
                ]),
            ];

            frame.render_widget(
                Table::new(rows, [Constraint::Length(20), Constraint::Min(20)])
                    .header(
                        Row::new(vec!["Metric", "Value"])
                            .style(Style::default().add_modifier(Modifier::BOLD)),
                    )
                    .block(
                        Block::default()
                            .title("Statistics")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .style(Style::default().fg(theme.base_fg)),
                    )
                    .column_spacing(2)
                    .style(Style::default().fg(theme.base_fg)),
                inner_area,
            );
        } else {
            frame.render_widget(
                Paragraph::new("Error: Could not find selected task.").alignment(Alignment::Center),
                inner_area,
            );
        }
    } else {
        frame.render_widget(
            Paragraph::new("No task selected.").alignment(Alignment::Center),
            inner_area,
        );
    }

    frame.render_widget(
        Paragraph::new(" [Esc / Enter] Back | [q]uit ")
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

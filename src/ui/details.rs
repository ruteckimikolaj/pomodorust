use chrono::prelude::*;
use ratatui::{prelude::*, widgets::*};

use crate::app::{App, UiState};
use crate::settings::Theme;

const WIDE_THRESHOLD: u16 = 90;

pub fn draw_task_details(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let wide = frame.area().width >= WIDE_THRESHOLD;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(frame.area());

    // Title
    frame.render_widget(
        Block::default()
            .title(" i DETAILS ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    // Help bar
    frame.render_widget(
        Paragraph::new(" [Esc / Enter] Back | [Shift+E] Edit notes | [q]uit ")
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

    let body = chunks[1];

    let Some(selected) = ui.completed_task_list_state else {
        frame.render_widget(
            Paragraph::new("No task selected.")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg))),
            body,
        );
        return;
    };

    let filter = ui.filter_input.to_lowercase();
    let completed: Vec<_> = app.tasks.iter()
        .filter(|t| t.completed && (filter.is_empty()
            || t.name.to_lowercase().contains(&filter)
            || t.notes.as_deref().map_or(false, |n| n.to_lowercase().contains(&filter))))
        .collect();

    let Some(task) = completed.get(selected) else {
        frame.render_widget(
            Paragraph::new("Error: task not found.")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg))),
            body,
        );
        return;
    };

    // Build stats data
    let created: DateTime<Local> = task.creation_date.into();
    let completed_str = task.completion_date.map_or_else(
        || "N/A".to_string(),
        |dt| DateTime::<Local>::from(dt).format("%Y-%m-%d %H:%M:%S").to_string(),
    );
    let time_spent_fmt = format!(
        "{}h {}m {}s",
        task.time_spent.as_secs() / 3600,
        (task.time_spent.as_secs() % 3600) / 60,
        task.time_spent.as_secs() % 60,
    );
    let time_to_complete = task.completion_date.map_or("N/A".to_string(), |c| {
        let d = c.signed_duration_since(task.creation_date);
        format!("{}d {}h {}m", d.num_days(), d.num_hours() % 24, d.num_minutes() % 60)
    });

    let mut rows = vec![
        Row::new(vec![Cell::from("Task"), Cell::from(task.name.clone())]),
        Row::new(vec![Cell::from("Status"), Cell::from("✓ Completed")])
            .style(Style::default().fg(theme.running_fg)),
        Row::new(vec![Cell::from("Created"), Cell::from(created.format("%Y-%m-%d %H:%M").to_string())]),
        Row::new(vec![Cell::from("Completed"), Cell::from(completed_str)]),
        Row::new(vec![Cell::from("Time to Complete"), Cell::from(time_to_complete)]),
        Row::new(vec![Cell::from("Time Focused"), Cell::from(time_spent_fmt)]),
        Row::new(vec![Cell::from("Pomodoros"), Cell::from(format!("{} ●", task.pomodoros))]),
    ];
    if let Some(proj) = &task.project {
        rows.push(Row::new(vec![
            Cell::from("Project"),
            Cell::from(format!("@{}", proj)).style(Style::default().fg(theme.accent_color)),
        ]));
    }

    let row_count = rows.len();
    let stats_table = Table::new(rows, [Constraint::Length(18), Constraint::Min(16)])
        .header(Row::new(vec!["Metric", "Value"]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(
            Block::default()
                .title("Statistics")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        )
        .column_spacing(2)
        .style(Style::default().fg(theme.base_fg));

    let notes_text = task.notes.as_deref().unwrap_or("");
    let notes_hint = if notes_text.is_empty() {
        Line::from(Span::styled("No notes yet. Press [Shift+E] to add.", Style::default().fg(theme.help_text_fg)))
    } else {
        Line::from("")
    };

    let notes_widget = if notes_text.is_empty() {
        Paragraph::new(notes_hint)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Notes")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
            )
    } else {
        Paragraph::new(notes_text)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title("Notes")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
            )
    };

    if wide {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(body);
        frame.render_widget(stats_table, cols[0]);
        frame.render_widget(notes_widget, cols[1]);
    } else {
        // Narrow: stats fixed height, notes takes the rest
        let rows_needed = row_count as u16 + 4; // data rows + header + borders + padding
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(rows_needed), Constraint::Min(0)])
            .split(body);
        frame.render_widget(stats_table, vert[0]);
        frame.render_widget(notes_widget, vert[1]);
    }
}

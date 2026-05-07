use std::time::Duration;

use chrono::{Datelike, Local, Weekday};
use ratatui::{prelude::*, widgets::*};

use crate::app::{App, InputMode, UiState};
use crate::settings::Theme;

// Below this total terminal width, collapse chart and show sparkline underneath
const BARCHART_MIN_WIDTH: u16 = 50;

fn weekday_label(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

// Full Mon–Sun of current ISO week; future days are 0
fn weekly_bar_data(app: &App) -> Vec<(String, u64)> {
    let today = Local::now().date_naive();
    let monday = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let mut counts = [0u64; 7];
    for task in &app.tasks {
        if let Some(completed) = task.completion_date {
            let d = (completed.with_timezone(&Local).date_naive() - monday).num_days();
            if d >= 0 && d < 7 {
                counts[d as usize] += task.pomodoros as u64;
            }
        }
    }
    (0..7)
        .map(|i| {
            let date = monday + chrono::Duration::days(i as i64);
            (weekday_label(date.weekday()).to_string(), counts[i])
        })
        .collect()
}

// Peak daily count over the last 28 days — used as BarChart max
fn four_week_max(app: &App) -> u64 {
    let today = Local::now().date_naive();
    let cutoff = today - chrono::Duration::days(28);
    let mut daily: std::collections::HashMap<chrono::NaiveDate, u64> = Default::default();
    for task in &app.tasks {
        if let Some(completed) = task.completion_date {
            let d = completed.with_timezone(&Local).date_naive();
            if d >= cutoff {
                *daily.entry(d).or_insert(0) += task.pomodoros as u64;
            }
        }
    }
    daily.values().copied().max().unwrap_or(1).max(1)
}

// Last 7 rolling days for the sparkline fallback
fn last7_sparkline(app: &App) -> Vec<u64> {
    let today = Local::now().date_naive();
    let mut counts = [0u64; 7];
    for task in &app.tasks {
        if let Some(completed) = task.completion_date {
            let days_ago = (today - completed.with_timezone(&Local).date_naive()).num_days();
            if days_ago >= 0 && (days_ago as usize) < 7 {
                counts[6 - days_ago as usize] += task.pomodoros as u64;
            }
        }
    }
    counts.to_vec()
}

pub fn draw_statistics(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let wide = frame.area().width >= BARCHART_MIN_WIDTH;

    // Vertical layout differs between wide and narrow modes
    let chunks = if wide {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // title
                Constraint::Length(8),  // summary (left) + barchart (right)
                Constraint::Min(0),     // task list
                Constraint::Length(4),  // help
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // title
                Constraint::Length(8),  // summary full-width
                Constraint::Length(3),  // sparkline
                Constraint::Min(0),     // task list
                Constraint::Length(4),  // help
            ])
            .split(frame.area())
    };

    // Title
    let stats_title = if !ui.filter_input.is_empty() {
        format!(" Σ STATISTICS [/{}] ", ui.filter_input)
    } else {
        " Σ STATISTICS ".to_string()
    };
    frame.render_widget(
        Block::default()
            .title(stats_title)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    // --- Stats data ---
    let today = Local::now().date_naive();
    let today_pomodoros: u64 = app.tasks.iter()
        .filter_map(|t| t.completion_date)
        .filter(|dt| dt.with_timezone(&Local).date_naive() == today)
        .count() as u64;
    let today_time: Duration = app.tasks.iter()
        .filter(|t| t.completion_date.map_or(false, |dt| dt.with_timezone(&Local).date_naive() == today))
        .map(|t| t.time_spent)
        .sum();
    let total_time: Duration = app.tasks.iter().map(|t| t.time_spent).sum();
    let fmt_time = |d: Duration| format!("{}h {}m", d.as_secs() / 3600, (d.as_secs() % 3600) / 60);
    let bold = Style::default().add_modifier(Modifier::BOLD);

    let summary_lines = vec![
        Line::from(Span::styled("Today", bold)),
        Line::from(format!("Pomodoros:    {}", today_pomodoros)),
        Line::from(format!("Time Focused: {}", fmt_time(today_time))),
        Line::from(Span::styled("All Time", bold)),
        Line::from(format!("Pomodoros:    {}", app.pomodoros_completed_total)),
        Line::from(format!("Time Focused: {}", fmt_time(total_time))),
    ];

    if wide {
        // Summary left (38%) + BarChart right (62%)
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(chunks[1]);

        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Summary")
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg));
        let stats_inner = stats_block.inner(top_cols[0]);
        frame.render_widget(stats_block, top_cols[0]);
        frame.render_widget(
            Paragraph::new(summary_lines).alignment(Alignment::Left),
            stats_inner,
        );

        // BarChart — dynamic bar width to fill container
        let bar_data = weekly_bar_data(app);
        let max_val = four_week_max(app);
        let n = 7usize;
        let bar_gap: u16 = 1;
        let inner_w = top_cols[1].width.saturating_sub(2); // minus borders
        let bar_width = (inner_w.saturating_sub(bar_gap * (n as u16 - 1)) / n as u16).max(3);
        let bars: Vec<Bar> = bar_data
            .iter()
            .map(|(label, count)| {
                Bar::default()
                    .label(Line::from(label.clone()))
                    .value(*count)
                    .style(Style::default().fg(theme.pomodoro_color))
            })
            .collect();
        frame.render_widget(
            BarChart::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("This week")
                        .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
                )
                .bar_width(bar_width)
                .bar_gap(bar_gap)
                .max(max_val)
                .value_style(
                    Style::default()
                        .fg(theme.base_bg)
                        .bg(theme.pomodoro_color)
                        .add_modifier(Modifier::BOLD),
                )
                .label_style(Style::default().fg(theme.base_fg))
                .data(BarGroup::default().bars(&bars)),
            top_cols[1],
        );
    } else {
        // Summary full-width
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Summary")
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg));
        let stats_inner = stats_block.inner(chunks[1]);
        frame.render_widget(stats_block, chunks[1]);
        frame.render_widget(
            Paragraph::new(summary_lines).alignment(Alignment::Left),
            stats_inner,
        );

        // Sparkline below summary
        let spark_data = last7_sparkline(app);
        frame.render_widget(
            Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Last 7 days")
                        .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
                )
                .data(spark_data.iter().copied())
                .style(Style::default().fg(theme.pomodoro_color)),
            chunks[2],
        );
    }

    // Indices shift by 1 in narrow mode due to extra sparkline chunk
    let (tasks_idx, help_idx) = if wide { (2, 3) } else { (3, 4) };

    // --- Completed task list ---
    let filter = ui.filter_input.to_lowercase();
    let completed_tasks: Vec<_> = app
        .tasks
        .iter()
        .filter(|t| {
            if !t.completed { return false; }
            if filter.is_empty() { return true; }
            let proj_match = t.project.as_deref().map_or(false, |p| {
                let tag = format!("@{}", p.to_lowercase());
                tag.contains(&filter) || p.to_lowercase().contains(&filter)
            });
            t.name.to_lowercase().contains(&filter)
                || t.notes.as_deref().map_or(false, |n| n.to_lowercase().contains(&filter))
                || proj_match
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(ui.completed_task_list_state);

    let list_items: Vec<ListItem> = completed_tasks
        .iter()
        .map(|task| {
            let mut spans = vec![
                Span::styled(
                    format!("{:<40} | {} ●", task.name, task.pomodoros),
                    Style::default().fg(theme.base_fg),
                ),
            ];
            if let Some(proj) = &task.project {
                spans.push(Span::styled(
                    format!(" @{}", proj),
                    Style::default().fg(theme.accent_color),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let task_list_title = if !filter.is_empty() {
        format!("Completed & Archived Tasks [/{}]", ui.filter_input)
    } else {
        "Completed & Archived Tasks".to_string()
    };
    frame.render_stateful_widget(
        List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(task_list_title)
                    .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> "),
        chunks[tasks_idx],
        &mut list_state,
    );

    // --- Help bar / filter bar ---
    match ui.input_mode {
        InputMode::Filtering => {
            let filter_display = format!("/{}", ui.filter_input);
            frame.render_widget(
                Paragraph::new(filter_display.as_str())
                    .style(Style::default().fg(theme.paused_fg))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title("Filter")
                            .style(Style::default().fg(theme.accent_color)),
                    ),
                chunks[help_idx],
            );
            frame.set_cursor_position((
                chunks[help_idx].x + 1 + 1 + ui.filter_input.len() as u16,
                chunks[help_idx].y + 1,
            ));
            return;
        }
        _ => {}
    }

    let help_text = if chunks[help_idx].width > 80 {
        " [Tab] Timer | [↑/↓] Navigate | [/] Filter | [Enter] Details | [d]elete | [q]uit "
    } else {
        " [Tab] [↑/↓] [/] [Ent] [d] [q] "
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
        chunks[help_idx],
    );
}

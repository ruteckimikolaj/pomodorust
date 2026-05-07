use ratatui::{prelude::*, widgets::*};

use crate::app::{App, InputMode, TimerState, UiState};
use crate::settings::Theme;

pub fn draw_task_list(frame: &mut Frame, app: &App, ui: &UiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(4),
        ])
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title(" ✓ TASKS ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        chunks[0],
    );

    let filter = ui.filter_input.to_lowercase();
    let active_tasks: Vec<_> = app
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            !t.completed && (filter.is_empty() || t.name.to_lowercase().contains(&filter))
        })
        .collect();

    let mut list_state = ListState::default();
    if let Some(active_index) = app.active_task_index {
        if let Some(pos) = active_tasks.iter().position(|(i, _)| *i == active_index) {
            list_state.select(Some(pos));
        }
    }

    let list_title = if !ui.filter_input.is_empty() {
        format!("Active Tasks [/{}]", ui.filter_input)
    } else {
        "Active Tasks".to_string()
    };

    let active_list_items: Vec<ListItem> = active_tasks
        .iter()
        .map(|(i, task)| {
            let running = Some(*i) == app.active_task_index && app.state == TimerState::Running;
            let marker = if running { "▶ " } else { "  " };
            let style = if running {
                Style::default().fg(theme.pomodoro_color)
            } else {
                Style::default().fg(theme.base_fg)
            };
            ListItem::new(Line::from(format!("[ ] {}{}", marker, task.name))).style(style)
        })
        .collect();

    let active_list = List::new(active_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(list_title)
                .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(active_list, chunks[1], &mut list_state);

    let input = Paragraph::new(ui.current_input.as_str())
        .style(match ui.input_mode {
            InputMode::Normal | InputMode::Filtering => Style::default().fg(theme.base_fg),
            InputMode::Editing => Style::default().fg(theme.paused_fg),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("New Task")
                .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
        );
    frame.render_widget(input, chunks[2]);
    if let InputMode::Editing = ui.input_mode {
        frame.set_cursor_position((
            chunks[2].x + ui.current_input.len() as u16 + 1,
            chunks[2].y + 1,
        ));
    }

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
                chunks[3],
            );
            frame.set_cursor_position((
                chunks[3].x + 1 + 1 + ui.filter_input.len() as u16,
                chunks[3].y + 1,
            ));
        }
        _ => {
            let help_text = match ui.input_mode {
                InputMode::Editing => " [Enter] Submit | [Esc] Cancel ",
                _ => {
                    if chunks[3].width > 80 {
                        " [Tab] Stats | [↑/↓] Nav | [Shift+↑/↓] Move | [n]ew | [/] Filter | [Enter] Complete | [d]elete | [q]uit "
                    } else {
                        " [Tab] [↑/↓] [S+↑/↓] [n] [/] [Ent] [d] [q] "
                    }
                }
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
                chunks[3],
            );
        }
    }
}

use chrono::Duration as ChronoDuration;
use ratatui::{prelude::*, widgets::*};

use crate::app::{App, Mode, TimerState};
use crate::settings::Theme;

fn get_char_art(c: char) -> Vec<&'static str> {
    match c {
        '0' => vec!["███", "█ █", "█ █", "█ █", "███"],
        '1' => vec![" █ ", "██ ", " █ ", " █ ", "███"],
        '2' => vec!["███", "  █", "███", "█  ", "███"],
        '3' => vec!["███", "  █", "███", "  █", "███"],
        '4' => vec!["█ █", "█ █", "███", "  █", "  █"],
        '5' => vec!["███", "█  ", "███", "  █", "███"],
        '6' => vec!["███", "█  ", "███", "█ █", "███"],
        '7' => vec!["███", "  █", "  █", "  █", "  █"],
        '8' => vec!["███", "█ █", "███", "█ █", "███"],
        '9' => vec!["███", "█ █", "███", "  █", "███"],
        ':' => vec!["   ", " █ ", "   ", " █ ", "   "],
        _ => vec!["   ", "   ", "   ", "   ", "   "],
    }
}

fn create_big_text_paragraph<'a>(text: &str, style: Style) -> Paragraph<'a> {
    let mut lines: Vec<Line> = vec![Line::from(""); 5];
    for character in text.chars() {
        let art = get_char_art(character);
        for (i, art_line) in art.iter().enumerate() {
            lines[i].spans.push(Span::styled(*art_line, style));
            lines[i].spans.push(Span::raw(" "));
        }
    }
    Paragraph::new(lines).alignment(Alignment::Center)
}

pub fn draw_timer(frame: &mut Frame, app: &App, theme: &Theme) {
    let (accent_color, mode_bg_color) = match app.mode {
        Mode::Pomodoro => (theme.pomodoro_color, theme.pomodoro_bg),
        Mode::ShortBreak => (theme.short_break_color, theme.short_break_bg),
        Mode::LongBreak => (theme.long_break_color, theme.long_break_bg),
    };

    let base_style = Style::default().bg(theme.base_bg).fg(theme.base_fg);
    let accent_style = Style::default().fg(accent_color);
    let running_style = Style::default().fg(theme.running_fg);
    let paused_style = Style::default().fg(theme.paused_fg);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4)])
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title(" P O M O D O R U S T ")
            .title_alignment(Alignment::Center)
            .style(base_style),
        main_layout[0],
    );

    let timer_block_border_style = if app.state == TimerState::Running {
        accent_style
    } else {
        Style::default().fg(theme.help_text_fg)
    };

    let timer_block = Block::default()
        .title(app.mode.title())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(timer_block_border_style)
        .style(Style::default().bg(mode_bg_color));

    let timer_area = timer_block.inner(main_layout[1]);
    frame.render_widget(timer_block, main_layout[1]);

    let vertical_center_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5), Constraint::Min(1)])
        .split(timer_area);

    let time = ChronoDuration::from_std(app.time_remaining).unwrap_or_else(|_| ChronoDuration::zero());
    let time_text = format!("{:02}:{:02}", time.num_minutes(), time.num_seconds() % 60);
    frame.render_widget(create_big_text_paragraph(&time_text, accent_style), vertical_center_layout[1]);

    let bottom_info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .horizontal_margin(4)
        .split(vertical_center_layout[2]);

    let task_name = app.active_task_index
        .and_then(|i| app.tasks.get(i))
        .map_or("No active task", |t| &t.name);
    frame.render_widget(
        Paragraph::new(task_name)
            .style(accent_style.add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center),
        bottom_info_layout[1],
    );

    let (status_text, status_style) = match app.state {
        TimerState::Running => ("▶ Running", running_style),
        TimerState::Paused => ("⏸ Paused", paused_style),
    };
    frame.render_widget(
        Paragraph::new(status_text).style(status_style).alignment(Alignment::Center),
        bottom_info_layout[2],
    );

    let total_duration = app.mode.duration(&app.settings).as_secs_f64();
    let remaining_duration = app.time_remaining.as_secs_f64();
    let progress_ratio = if total_duration > 0.0 {
        ((total_duration - remaining_duration) / total_duration).clamp(0.0, 1.0)
    } else {
        1.0
    };
    frame.render_widget(
        Gauge::default().gauge_style(accent_style).ratio(progress_ratio),
        bottom_info_layout[3],
    );

    frame.render_widget(
        Paragraph::new(format!("Total Sessions: {}", app.pomodoros_completed_total))
            .style(Style::default().fg(theme.help_text_fg))
            .alignment(Alignment::Center),
        bottom_info_layout[4],
    );

    let help_text = if main_layout[2].width > 80 {
        " [Tab] Tasks | [o] Options | [Space] Start/Pause | [r] Reset | [p/s/l] Change Mode | [q] Quit "
    } else {
        " [Tab] [o] [Spc] [r] [p/s/l] [q] "
    };
    frame.render_widget(
        Paragraph::new(help_text)
            .block(Block::default().title("Controls").borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(theme.help_text_fg)))
            .alignment(Alignment::Center),
        main_layout[2],
    );
}

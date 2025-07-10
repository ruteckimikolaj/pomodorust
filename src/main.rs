use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use chrono::Duration as ChronoDuration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

mod app;
use app::{App, InputMode, Mode, TimerState, View};

/// Main function to run the application.
fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let app = App::new();
    run_app(&mut terminal, app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

/// Sets up the terminal for TUI rendering.
fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restores the terminal to its original state.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// The main application loop.
fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match app.current_view {
                            View::Timer => handle_timer_input(key.code, &mut app),
                            View::TaskList => handle_tasklist_input(key.code, &mut app),
                        },
                        InputMode::Editing => handle_editing_input(key.code, &mut app),
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let TimerState::Running = app.state {
                let elapsed = last_tick.elapsed();
                if let Some(remaining) = app.time_remaining.checked_sub(elapsed) {
                    app.time_remaining = remaining;
                    if let Some(index) = app.active_task_index {
                        if let Some(task) = app.tasks.get_mut(index) {
                            task.time_spent += elapsed;
                        }
                    }
                } else {
                    app.time_remaining = Duration::from_secs(0);
                    app.next_mode();
                }
            }
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Handles key events for the Timer view in Normal mode.
fn handle_timer_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char(' ') => app.toggle_timer(),
        KeyCode::Char('r') => app.reset_timer(),
        KeyCode::Char('p') => app.set_mode(Mode::Pomodoro),
        KeyCode::Char('s') => app.set_mode(Mode::ShortBreak),
        KeyCode::Char('l') => app.set_mode(Mode::LongBreak),
        KeyCode::Tab => app.current_view = View::TaskList,
        _ => {}
    }
}

/// Handles key events for the TaskList view in Normal mode.
fn handle_tasklist_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => app.current_view = View::Timer,
        KeyCode::Char('n') => app.input_mode = InputMode::Editing,
        KeyCode::Down | KeyCode::Char('j') => app.next_task(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_task(),
        KeyCode::Enter => app.complete_active_task(),
        _ => {}
    }
}

/// Handles key events when in Editing mode for task input.
fn handle_editing_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Enter => app.submit_task(),
        KeyCode::Char(c) => app.current_input.push(c),
        KeyCode::Backspace => {
            app.current_input.pop();
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.current_input.clear();
        }
        _ => {}
    }
}

/// Renders the user interface based on the current view.
fn ui(frame: &mut Frame, app: &mut App) {
    match app.current_view {
        View::Timer => draw_timer(frame, app),
        View::TaskList => draw_task_list(frame, app),
    }
}

/// Renders the Timer view.
fn draw_timer(frame: &mut Frame, app: &App) {
    let (accent_color, mode_bg_color) = match app.mode {
        Mode::Pomodoro => (Color::LightRed, Color::Rgb(50, 20, 20)),
        Mode::ShortBreak => (Color::LightGreen, Color::Rgb(20, 50, 20)),
        Mode::LongBreak => (Color::LightBlue, Color::Rgb(20, 20, 50)),
    };

    let base_style = Style::default().bg(Color::Black).fg(Color::Gray);
    let accent_style = Style::default().fg(accent_color);
    let running_style = Style::default().fg(Color::Green);
    let paused_style = Style::default().fg(Color::Yellow);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4)])
        .split(frame.size());

    frame.render_widget(
        Block::default()
            .title(Title::from(" pomodorust ").alignment(Alignment::Center))
            .style(base_style),
        main_layout[0],
    );

    let timer_block_border_style = if app.state == TimerState::Running {
        accent_style
    } else {
        Style::default().fg(Color::DarkGray)
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

    let timer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(45),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ])
        .margin(1)
        .split(timer_area);

    let task_name = app
        .active_task_index
        .and_then(|i| app.tasks.get(i))
        .map_or("No active task", |t| &t.name);

    frame.render_widget(
        Paragraph::new(task_name).style(accent_style).alignment(Alignment::Center),
        timer_layout[0],
    );

    let time = ChronoDuration::from_std(app.time_remaining).unwrap();
    frame.render_widget(
        Paragraph::new(format!("{:02}:{:02}", time.num_minutes(), time.num_seconds() % 60))
            .style(accent_style.add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        timer_layout[1],
    );

    let (status_text, status_style) = match app.state {
        TimerState::Running => ("▶ Running", running_style),
        TimerState::Paused => ("⏸ Paused", paused_style),
    };
    frame.render_widget(
        Paragraph::new(status_text).style(status_style).alignment(Alignment::Center),
        timer_layout[2],
    );

    let total_duration = app.mode.duration().as_secs_f64();
    let remaining_duration = app.time_remaining.as_secs_f64();
    let progress_ratio = if total_duration > 0.0 {
        (total_duration - remaining_duration) / total_duration
    } else {
        1.0
    };
    frame.render_widget(
        Gauge::default()
            .gauge_style(accent_style)
            .ratio(progress_ratio)
            .label(format!("{:.0}%", progress_ratio * 100.0)),
        timer_layout[3],
    );

    frame.render_widget(
        Paragraph::new(format!("🍅 {}", app.pomodoros_completed_total))
            .style(accent_style)
            .alignment(Alignment::Center),
        timer_layout[4],
    );

    let help_text = if main_layout[2].width > 80 {
        " [Tab] Tasks   [Space] Start/Pause   [r] Reset\n[p] Pomodoro   [s] Short Break   [l] Long Break   [q] Quit "
    } else if main_layout[2].width > 40 {
        " [Tab] Tasks   [Space] Start/Pause\n[r] Reset   [p/s/l] Mode   [q] Quit "
    } else {
        "[Tab][Spc][r][p/s/l][q]"
    };
    frame.render_widget(
        Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center),
        main_layout[2],
    );
}

/// Renders the Task List view.
fn draw_task_list(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Input or padding
                Constraint::Length(4), // Help
            ]
            .as_ref(),
        )
        .split(frame.size());

    frame.render_widget(
        Block::default()
            .title(Title::from(" Task List ").alignment(Alignment::Center)),
        chunks[0],
    );

    let mut list_state = ListState::default();
    list_state.select(app.active_task_index);

    let list_items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let completed_marker = if task.completed { "[x]" } else { "[ ]" };
            let running_marker =
                if Some(i) == app.active_task_index && app.state == TimerState::Running {
                    "▶ "
                } else {
                    "  "
                };
            let content = format!("{} {}{}", completed_marker, running_marker, task.name);

            let style = if task.completed {
                Style::default().fg(Color::Green).add_modifier(Modifier::CROSSED_OUT)
            } else if Some(i) == app.active_task_index && app.state == TimerState::Running {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let input = Paragraph::new(app.current_input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("New Task"));
    frame.render_widget(input, chunks[2]);
    if let InputMode::Editing = app.input_mode {
        frame.set_cursor(
            chunks[2].x + app.current_input.len() as u16 + 1,
            chunks[2].y + 1,
        );
    }

    let help_text = match app.input_mode {
        InputMode::Normal => {
            if chunks[3].width > 80 {
                " [Tab] Timer | [↑/↓] Navigate | [n] New Task | [Enter] Complete Task | [q] Quit "
            } else {
                " [Tab] [↑/↓] [n] [Ent] [q] "
            }
        }
        InputMode::Editing => " [Enter] Submit | [Esc] Cancel ",
    };
    frame.render_widget(
        Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center),
        chunks[3],
    );
}

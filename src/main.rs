use std::{
    io::{self, stdout, Stdout},
    panic,
    time::{Duration, Instant},
};

use chrono::Duration as ChronoDuration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify_rust::Notification;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use rodio::{source::SineWave, OutputStream, Sink, Source};

mod app;
mod settings;
use app::{App, InputMode, Mode, TimerState, View};
use settings::draw_settings;

/// Main function to run the application.
fn main() -> io::Result<()> {
    // This panic hook ensures the terminal is restored even if a Rust-level panic occurs.
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = stdout();
        execute!(stdout, LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));

    let mut terminal = setup_terminal()?;
    let mut app = App::load_or_new();
    run_app(&mut terminal, &mut app)?;
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
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    
    // Move audio system to the heap to prevent potential stack overflow.
    let audio_system = OutputStream::try_default().ok().and_then(|(stream, handle)| {
        Sink::try_new(&handle)
            .ok()
            .map(|sink| Box::new((stream, sink))) // Wrap in a Box
    });


    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(key, app);
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
                    let finished_mode = app.next_mode();
                    if let Some(audio) = &audio_system {
                        play_sound(&audio.1, finished_mode);
                    }
                    if app.settings.desktop_notifications {
                        show_desktop_notification(finished_mode, app.mode);
                    }
                }
            }
            last_tick = Instant::now();
        }

        if app.should_quit {
            app.save();
            return Ok(());
        }
    }
}

/// Central key event handler.
fn handle_key_event(key: KeyEvent, app: &mut App) {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return;
    }

    // --- FIX: Prioritize Editing mode to capture all key presses for text input ---
    match app.input_mode {
        InputMode::Editing => {
            handle_editing_input(key.code, app);
        }
        InputMode::Normal => {
            // Global keybindings are only processed in Normal mode.
            if let KeyCode::Char('o') = key.code {
                app.current_view = View::Settings;
                return;
            }

            match app.current_view {
                View::Timer => handle_timer_input(key.code, app),
                View::TaskList => handle_tasklist_input(key.code, app),
                View::Statistics => handle_stats_input(key.code, app),
                View::Settings => handle_settings_input(key.code, app),
            }
        }
    }
}

/// Plays a sound notification based on the mode that just finished.
fn play_sound(sink: &Sink, finished_mode: Mode) {
    let (freq1, freq2, duration) = match finished_mode {
        Mode::Pomodoro => (440.0, 660.0, 150),
        _ => (660.0, 440.0, 150),
    };
    let source1 = SineWave::new(freq1)
        .take_duration(Duration::from_millis(duration))
        .amplify(0.20);
    let source2 = SineWave::new(freq2)
        .take_duration(Duration::from_millis(duration))
        .amplify(0.20);
    sink.append(source1);
    sink.append(source2);
}

/// Shows a desktop notification.
fn show_desktop_notification(finished_mode: Mode, next_mode: Mode) {
    let summary = format!("{} Finished!", finished_mode.title());
    let body = format!("Time for your {}.", next_mode.title());
    let _ = Notification::new()
        .summary(&summary)
        .body(&body)
        .icon("dialog-information")
        .show();
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
        KeyCode::Tab => app.current_view = View::Statistics,
        KeyCode::Char('n') => app.input_mode = InputMode::Editing,
        KeyCode::Down | KeyCode::Char('j') => app.next_task(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_task(),
        KeyCode::Enter => app.complete_active_task(),
        KeyCode::Char(' ') => {
            if app.active_task_index.is_some() {
                app.state = TimerState::Running;
                app.current_view = View::Timer;
            }
        }
        _ => {}
    }
}

/// Handles key events for the Statistics view in Normal mode.
fn handle_stats_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => app.current_view = View::Timer,
        KeyCode::Down | KeyCode::Char('j') => app.next_completed_task(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_completed_task(),
        KeyCode::Char('d') | KeyCode::Delete => app.delete_selected_completed_task(),
        _ => {}
    }
}

/// Handles key events for the Settings view in Normal mode.
fn handle_settings_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => app.current_view = View::Timer,
        KeyCode::Up | KeyCode::Char('k') => app.previous_setting(),
        KeyCode::Down | KeyCode::Char('j') => app.next_setting(),
        KeyCode::Left | KeyCode::Char('h') => app.modify_setting(false),
        KeyCode::Right | KeyCode::Char('l') => app.modify_setting(true),
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
        View::Statistics => draw_statistics(frame, app),
        View::Settings => draw_settings(frame, app),
    }
}

/// Returns a vector of strings representing the ASCII art for a given character.
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

/// Creates a Paragraph widget with large text from a string.
fn create_big_text_paragraph<'a>(text: &str, style: Style) -> Paragraph<'a> {
    let big_text_height = 5;
    let mut lines: Vec<Line> = vec![Line::from(""); big_text_height];

    for character in text.chars() {
        let art = get_char_art(character);
        for (i, art_line) in art.iter().enumerate() {
            lines[i].spans.push(Span::styled(*art_line, style));
            lines[i].spans.push(Span::raw(" ")); // Space between characters
        }
    }
    Paragraph::new(lines).alignment(Alignment::Center)
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
        .split(frame.area());

    frame.render_widget(
        Block::default()
            .title(" 🦀 Pomodorust 🦀 ")
            .title_alignment(Alignment::Center)
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

    // This layout centers the main timer display vertically
    let vertical_center_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top spacer
            Constraint::Length(5), // Big text height
            Constraint::Min(1),    // Bottom area for other info
        ])
        .split(timer_area);

    // The timer text itself
    let time = ChronoDuration::from_std(app.time_remaining).unwrap_or_else(|_| ChronoDuration::zero());
    let time_text = format!(
        "{:02}:{:02}",
        time.num_minutes(),
        time.num_seconds() % 60
    );
    let timer_paragraph = create_big_text_paragraph(&time_text, accent_style);
    frame.render_widget(timer_paragraph, vertical_center_layout[1]);

    // Layout for the bottom info section
    let bottom_info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top spacer
            Constraint::Length(1),      // Task Name
            Constraint::Length(1),      // Status
            Constraint::Length(1),      // Progress Bar
            Constraint::Length(1),      // Total Sessions
        ])
        .horizontal_margin(4) // Indent the smaller info
        .split(vertical_center_layout[2]);

    // Task Name
    let task_name = app
        .active_task_index
        .and_then(|i| app.tasks.get(i))
        .map_or("No active task", |t| &t.name);
    frame.render_widget(
        Paragraph::new(task_name)
            .style(accent_style.add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center),
        bottom_info_layout[1],
    );

    // Status Text
    let (status_text, status_style) = match app.state {
        TimerState::Running => ("▶ Running", running_style),
        TimerState::Paused => ("⏸ Paused", paused_style),
    };
    frame.render_widget(
        Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Center),
        bottom_info_layout[2],
    );

    // Progress Bar
    let total_duration = app.mode.duration(&app.settings).as_secs_f64();
    let remaining_duration = app.time_remaining.as_secs_f64();
    let progress_ratio = if total_duration > 0.0 {
        (total_duration - remaining_duration) / total_duration
    } else {
        1.0
    };
    let progress_bar = Gauge::default()
        .gauge_style(accent_style)
        .ratio(progress_ratio);
    frame.render_widget(progress_bar, bottom_info_layout[3]);

    // Pomodoros Completed
    frame.render_widget(
        Paragraph::new(format!("Total Sessions: {}", app.pomodoros_completed_total))
            .style(Style::default().fg(Color::DarkGray))
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
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(4),
            ]
            .as_ref(),
        )
        .split(frame.area());

    frame.render_widget(
        Block::default().title("✅ Tasks").title_alignment(Alignment::Center),
        chunks[0],
    );

    let (active_tasks, _): (Vec<_>, Vec<_>) =
        app.tasks.iter().enumerate().partition(|(_, t)| !t.completed);

    let mut list_state = ListState::default();
    if let Some(active_index) = app.active_task_index {
        if let Some(pos) = active_tasks.iter().position(|(i, _)| *i == active_index) {
            list_state.select(Some(pos));
        }
    }

    let active_list_items: Vec<ListItem> = active_tasks
        .iter()
        .map(|(i, task)| {
            let running_marker = if Some(*i) == app.active_task_index && app.state == TimerState::Running { "▶ " } else { "  " };
            let content = format!("[ ] {}{}", running_marker, task.name);
            let style = if Some(*i) == app.active_task_index && app.state == TimerState::Running { Style::default().fg(Color::LightRed) } else { Style::default() };
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    let active_list = List::new(active_list_items)
        .block(Block::default().borders(Borders::ALL).title("Active Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(active_list, chunks[1], &mut list_state);

    let input = Paragraph::new(app.current_input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("New Task"));
    frame.render_widget(input, chunks[2]);
    if let InputMode::Editing = app.input_mode {
        frame.set_cursor_position((
            chunks[2].x + app.current_input.len() as u16 + 1,
            chunks[2].y + 1,
        ));
    }

    let help_text = match app.input_mode {
        InputMode::Normal => {
            if chunks[3].width > 80 {
                " [Tab] Stats | [Space] Start | [↑/↓] Navigate | [n] New | [Enter] Complete | [q] Quit "
            } else {
                " [Tab] [Spc] [↑/↓] [n] [Ent] [q] "
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

/// Renders the Statistics view.
fn draw_statistics(frame: &mut Frame, app: &mut App) {
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
        Block::default().title("📊 Statistics").title_alignment(Alignment::Center),
        chunks[0],
    );

    let total_time_spent: Duration = app.tasks.iter().map(|t| t.time_spent).sum();
    let time_spent_formatted = format!(
        "{}h {}m",
        total_time_spent.as_secs() / 3600,
        (total_time_spent.as_secs() % 3600) / 60
    );
    let summary_text = vec![
        Line::from(format!("Total Pomodoros: {}", app.pomodoros_completed_total)),
        Line::from(format!("Total Time Focused: {}", time_spent_formatted)),
    ];
    frame.render_widget(
        Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary"))
            .alignment(Alignment::Center),
        chunks[1],
    );

    let completed_tasks: Vec<_> = app
        .tasks
        .iter()
        .filter(|t| t.completed)
        .collect();

    let mut list_state = ListState::default();
    list_state.select(app.completed_task_list_state);

    let list_items: Vec<ListItem> = completed_tasks
        .iter()
        .map(|task| {
            let pomos = format!("{} 🍅", task.pomodoros);
            let content = format!("{:<40} | {}", task.name, pomos);
            ListItem::new(Line::from(content))
        })
        .collect();
    
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Completed & Archived Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[2], &mut list_state);

    let help_text = if chunks[3].width > 80 {
        " [Tab] Timer | [↑/↓] Navigate | [d]elete Selected Task | [q] Quit "
    } else {
        " [Tab] [↑/↓] [d] [q] "
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

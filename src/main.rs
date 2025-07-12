use std::{
    io::{self, stdout, Stdout},
    panic,
    time::{Duration, Instant},
};

use chrono::{prelude::*, Duration as ChronoDuration};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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
mod theme;
use app::{App, InputMode, Mode, TimerState, View};
use settings::{draw_settings, Settings};
use theme::Theme;

/// A simple Pomodoro timer for your terminal.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Pomodoro duration in minutes.
    #[arg(short = 'p', long)]
    pomodoro_duration: Option<u64>,

    /// Short break duration in minutes.
    #[arg(short = 's', long)]
    short_break_duration: Option<u64>,

    /// Long break duration in minutes.
    #[arg(short = 'l', long)]
    long_break_duration: Option<u64>,
}

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
    
    // Parse command-line arguments.
    let cli = Cli::parse();

    let mut terminal = setup_terminal()?;
    
    // Load settings from config file.
    let mut settings = Settings::load();

    // Override settings from CLI arguments if provided.
    if let Some(duration) = cli.pomodoro_duration {
        settings.pomodoro_duration = Duration::from_secs(duration * 60);
    }
    if let Some(duration) = cli.short_break_duration {
        settings.short_break_duration = Duration::from_secs(duration * 60);
    }
    if let Some(duration) = cli.long_break_duration {
        settings.long_break_duration = Duration::from_secs(duration * 60);
    }
    
    // Load app state with the final settings.
    let mut app = App::load_with_settings(settings);

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

    // Prioritize Editing mode to capture all key presses for text input.
    match app.input_mode {
        InputMode::Editing => {
            handle_editing_input(key, app);
        }
        InputMode::Normal => {
            // Global keybindings are only processed in Normal mode.
            if key.code == KeyCode::Char('o') && key.modifiers == KeyModifiers::NONE {
                app.previous_view = app.current_view;
                app.current_view = View::Settings;
                return;
            }

            match app.current_view {
                View::Timer => handle_timer_input(key, app),
                View::TaskList => handle_tasklist_input(key, app),
                View::Statistics => handle_stats_input(key, app),
                View::Settings => handle_settings_input(key, app),
                View::TaskDetails => handle_task_details_input(key, app),
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
fn handle_timer_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char(' ') => app.toggle_timer(),
        KeyCode::Char('r') => app.reset_timer(),
        KeyCode::Char('p') => app.set_mode(Mode::Pomodoro),
        KeyCode::Char('s') => app.set_mode(Mode::ShortBreak),
        KeyCode::Char('l') => app.set_mode(Mode::LongBreak),
        KeyCode::Tab => {
            app.previous_view = app.current_view;
            app.current_view = View::TaskList;
        }
        _ => {}
    }
}

/// Handles key events for the TaskList view in Normal mode.
fn handle_tasklist_input(key: KeyEvent, app: &mut App) {
    match key {
        // Handle task reordering with Shift modifier
        KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::SHIFT,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('K'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => app.move_active_task_up(),
        KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::SHIFT,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('J'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => app.move_active_task_down(),

        // Handle other keys without modifiers
        KeyEvent { code, .. } => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Tab => {
                app.previous_view = app.current_view;
                app.current_view = View::Statistics;
            }
            KeyCode::Char('n') => app.input_mode = InputMode::Editing,
            KeyCode::Down | KeyCode::Char('j') => app.next_task(),
            KeyCode::Up | KeyCode::Char('k') => app.previous_task(),
            KeyCode::Enter => app.complete_active_task(),
            KeyCode::Char(' ') => {
                if app.active_task_index.is_some() {
                    app.previous_view = app.current_view;
                    app.current_view = View::Timer;
                }
            }
            _ => {}
        },
    }
}

/// Handles key events for the Statistics view in Normal mode.
fn handle_stats_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => {
            app.previous_view = app.current_view;
            app.current_view = View::Timer;
        }
        KeyCode::Down | KeyCode::Char('j') => app.next_completed_task(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_completed_task(),
        KeyCode::Enter => {
            if app.completed_task_list_state.is_some() {
                app.previous_view = app.current_view;
                app.current_view = View::TaskDetails;
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => app.delete_selected_completed_task(),
        _ => {}
    }
}

/// Handles key events for the Settings view in Normal mode.
fn handle_settings_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => app.current_view = app.previous_view,
        KeyCode::Up | KeyCode::Char('k') => app.previous_setting(),
        KeyCode::Down | KeyCode::Char('j') => app.next_setting(),
        KeyCode::Left | KeyCode::Char('h') => app.modify_setting(false),
        KeyCode::Right | KeyCode::Char('l') => app.modify_setting(true),
        _ => {}
    }
}

/// Handles key events for the Task Details view in Normal mode.
fn handle_task_details_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc | KeyCode::Enter => app.current_view = app.previous_view,
        _ => {}
    }
}

/// Handles key events when in Editing mode for task input.
fn handle_editing_input(key: KeyEvent, app: &mut App) {
    match key.code {
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
    let theme = Theme::from_settings(app.settings.theme);
    match app.current_view {
        View::Timer => draw_timer(frame, app, &theme),
        View::TaskList => draw_task_list(frame, app, &theme),
        View::Statistics => draw_statistics(frame, app, &theme),
        View::Settings => draw_settings(frame, app, &theme),
        View::TaskDetails => draw_task_details(frame, app, &theme),
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
fn draw_timer(frame: &mut Frame, app: &App, theme: &Theme) {
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
            Constraint::Percentage(50), // Top Spacer
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
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.help_text_fg)),
            )
            .alignment(Alignment::Center),
        main_layout[2],
    );
}

/// Renders the Task List view.
fn draw_task_list(frame: &mut Frame, app: &mut App, theme: &Theme) {
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
        Block::default()
            .title(" ✓ TASKS ")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
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
            let style = if Some(*i) == app.active_task_index && app.state == TimerState::Running { Style::default().fg(theme.pomodoro_color) } else { Style::default().fg(theme.base_fg) };
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    let active_list = List::new(active_list_items)
        .block(Block::default().borders(Borders::ALL).title("Active Tasks").style(Style::default().fg(theme.base_fg).bg(theme.base_bg)))
        .highlight_style(Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(active_list, chunks[1], &mut list_state);

    let input = Paragraph::new(app.current_input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default().fg(theme.base_fg),
            InputMode::Editing => Style::default().fg(theme.paused_fg),
        })
        .block(Block::default().borders(Borders::ALL).title("New Task").style(Style::default().fg(theme.base_fg).bg(theme.base_bg)));
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
                " [Tab] Stats | [↑/↓] Nav | [Shift+↑/↓] Move | [n] New | [Enter] Complete | [q] Quit "
            } else {
                " [Tab] [↑/↓] [S+↑/↓] [n] [Ent] [q] "
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
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.help_text_fg)),
            )
            .alignment(Alignment::Center),
        chunks[3],
    );
}

/// Renders the Statistics view.
fn draw_statistics(frame: &mut Frame, app: &mut App, theme: &Theme) {
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
        Block::default().title(" Σ STATISTICS ").title_alignment(Alignment::Center).style(Style::default().fg(theme.base_fg).bg(theme.base_bg)),
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
            .block(Block::default().borders(Borders::ALL).title("Summary").style(Style::default().fg(theme.base_fg).bg(theme.base_bg)))
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
            let pomos = format!("{} ●", task.pomodoros);
            let content = format!("{:<40} | {}", task.name, pomos);
            ListItem::new(Line::from(content)).style(Style::default().fg(theme.base_fg))
        })
        .collect();
    
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Completed & Archived Tasks").style(Style::default().fg(theme.base_fg).bg(theme.base_bg)))
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

/// Renders the Task Details view.
fn draw_task_details(frame: &mut Frame, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4)])
        .split(frame.area());

    let title = Block::default()
        .title(" i DETAILS ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(theme.base_fg).bg(theme.base_bg));
    frame.render_widget(title, chunks[0]);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::uniform(1))
        .style(Style::default().fg(theme.base_fg).bg(theme.base_bg));
    let inner_area = main_block.inner(chunks[1]);
    frame.render_widget(main_block, chunks[1]);

    if let Some(selected_completed_index) = app.completed_task_list_state {
        let completed_tasks: Vec<_> = app.tasks.iter().filter(|t| t.completed).collect();
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
            let time_to_complete_str = if let (Some(completed), created) = (task.completion_date, task.creation_date) {
                let duration = completed.signed_duration_since(created);
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let mins = duration.num_minutes() % 60;
                format!("{}d {}h {}m", days, hours, mins)
            } else {
                "N/A".to_string()
            };

            let rows = vec![
                Row::new(vec![Cell::from("Task"), Cell::from(task.name.clone())]),
                Row::new(vec![Cell::from("Status"), Cell::from("✓ Completed")]).style(Style::default().fg(theme.running_fg)),
                Row::new(vec![Cell::from("Created"), Cell::from(created.format("%Y-%m-%d %H:%M").to_string())]),
                Row::new(vec![Cell::from("Completed"), Cell::from(completed_str)]),
                Row::new(vec![Cell::from("Time to Complete"), Cell::from(time_to_complete_str)]),
                Row::new(vec![Cell::from("Time Focused"), Cell::from(time_spent_formatted)]),
                Row::new(vec![Cell::from("Pomodoros"), Cell::from(format!("{} ●", task.pomodoros))]),
            ];

            let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(20)])
                .header(Row::new(vec!["Metric", "Value"]).style(Style::default().add_modifier(Modifier::BOLD)))
                .block(Block::default().title("Statistics").borders(Borders::ALL).style(Style::default().fg(theme.base_fg)))
                .column_spacing(2)
                .style(Style::default().fg(theme.base_fg));

            frame.render_widget(table, inner_area);

        } else {
            let p = Paragraph::new("Error: Could not find selected task.").alignment(Alignment::Center);
            frame.render_widget(p, inner_area);
        }
    } else {
        let p = Paragraph::new("No task selected.").alignment(Alignment::Center);
        frame.render_widget(p, inner_area);
    };

    let help_text = " [Esc / Enter] Back | [q] Quit ";
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
        chunks[2],
    );
}

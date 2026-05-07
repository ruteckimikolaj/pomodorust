use std::{
    io::{self, stdout, Stdout},
    panic,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify_rust::Notification;
use ratatui::prelude::*;
use rodio::{source::SineWave, stream::DeviceSinkBuilder, Player, Source};

mod app;
mod db;
mod settings;
mod ui;
use app::{App, InputMode, Mode, TimerState, UiState, View};
use settings::{Settings, Theme};
use ratatui_textarea::Input;
use ui::{draw_notes_modal, draw_settings, draw_statistics, draw_task_details, draw_task_list, draw_timer};

/// An andvanced Pomodoro timer for your terminal.
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    let mut ui_state = UiState::default();

    let audio_system = DeviceSinkBuilder::open_default_sink()
        .ok()
        .map(|sink| {
            let player = Player::connect_new(sink.mixer());
            Box::new((sink, player))
        });

    loop {
        terminal.draw(|f| ui(f, app, &ui_state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(key, app, &mut ui_state, audio_system.as_deref().map(|b| &b.1));
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

fn handle_key_event(key: KeyEvent, app: &mut App, ui: &mut UiState, player: Option<&Player>) {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return;
    }

    match ui.input_mode {
        InputMode::Editing => handle_editing_input(key, app, ui),
        InputMode::Filtering => handle_filtering_input(key, ui),
        InputMode::EditingNotes => handle_editing_notes_input(key, app, ui),
        InputMode::Normal => {
            if key.code == KeyCode::Char('o') && key.modifiers == KeyModifiers::NONE {
                ui.previous_view = app.current_view;
                app.current_view = View::Settings;
                return;
            }

            match app.current_view {
                View::Timer => handle_timer_input(key, app, ui, player),
                View::TaskList => handle_tasklist_input(key, app, ui),
                View::Statistics => handle_stats_input(key, app, ui),
                View::Settings => handle_settings_input(key, app, ui),
                View::TaskDetails => handle_task_details_input(key, app, ui),
            }
        }
    }
}

/// Plays a sound notification based on the mode that just finished.
fn play_sound(sink: &Player, finished_mode: Mode) {
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

fn handle_timer_input(key: KeyEvent, app: &mut App, ui: &mut UiState, player: Option<&Player>) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char(' ') => app.toggle_timer(),
        KeyCode::Char('r') => app.reset_timer(),
        KeyCode::Char('n') => {
            let finished_mode = app.skip_segment();
            if let Some(p) = player {
                play_sound(p, finished_mode);
            }
            if app.settings.desktop_notifications {
                show_desktop_notification(finished_mode, app.mode);
            }
        }
        KeyCode::Tab => {
            ui.previous_view = app.current_view;
            app.current_view = View::TaskList;
        }
        _ => {}
    }
}

fn handle_tasklist_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key {
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

        KeyEvent { code, .. } => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Tab => {
                ui.previous_view = app.current_view;
                app.current_view = View::Statistics;
            }
            KeyCode::Char('n') => ui.input_mode = InputMode::Editing,
            KeyCode::Char('e') => ui.start_rename(app),
            KeyCode::Char('E') if key.modifiers == KeyModifiers::SHIFT => ui.start_edit_notes_active(app),
            KeyCode::Char('/') => ui.input_mode = InputMode::Filtering,
            KeyCode::Down | KeyCode::Char('j') => ui.next_filtered_task(app),
            KeyCode::Up | KeyCode::Char('k') => ui.previous_filtered_task(app),
            KeyCode::Enter => app.complete_active_task(),
            KeyCode::Char('d') | KeyCode::Delete => app.delete_active_task(),
            KeyCode::Char(' ') => {
                if app.active_task_index.is_some() {
                    ui.previous_view = app.current_view;
                    app.current_view = View::Timer;
                }
            }
            _ => {}
        },
    }
}

fn handle_stats_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => {
            ui.previous_view = app.current_view;
            app.current_view = View::Timer;
        }
        KeyCode::Char('/') => ui.input_mode = InputMode::Filtering,
        KeyCode::Down | KeyCode::Char('j') => ui.next_completed_task(app),
        KeyCode::Up | KeyCode::Char('k') => ui.previous_completed_task(app),
        KeyCode::Enter => {
            if ui.completed_task_list_state.is_some() {
                ui.previous_view = app.current_view;
                app.current_view = View::TaskDetails;
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => ui.delete_selected_completed_task(app),
        _ => {}
    }
}

fn handle_settings_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => app.current_view = ui.previous_view,
        KeyCode::Up | KeyCode::Char('k') => ui.previous_setting(),
        KeyCode::Down | KeyCode::Char('j') => ui.next_setting(),
        KeyCode::Left | KeyCode::Char('h') => ui.modify_setting(app, false),
        KeyCode::Right | KeyCode::Char('l') => ui.modify_setting(app, true),
        _ => {}
    }
}

fn handle_task_details_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('E') if key.modifiers == KeyModifiers::SHIFT => ui.start_edit_notes(app),
        KeyCode::Esc | KeyCode::Enter => app.current_view = ui.previous_view,
        _ => {}
    }
}

fn handle_editing_notes_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key {
        // Ctrl+S — save
        KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL, .. } => {
            ui.submit_notes(app);
        }
        // Esc — cancel
        KeyEvent { code: KeyCode::Esc, .. } => {
            ui.cancel_notes();
        }
        // Everything else goes to the textarea
        _ => {
            if let Some(textarea) = &mut ui.notes_textarea {
                textarea.input(Input::from(key));
            }
        }
    }
}

fn handle_filtering_input(key: KeyEvent, ui: &mut UiState) {
    match key.code {
        KeyCode::Char(c) => ui.filter_input.push(c),
        KeyCode::Backspace => { ui.filter_input.pop(); }
        KeyCode::Esc => {
            ui.input_mode = InputMode::Normal;
            ui.filter_input.clear();
        }
        KeyCode::Enter => ui.input_mode = InputMode::Normal,
        _ => {}
    }
}

fn handle_editing_input(key: KeyEvent, app: &mut App, ui: &mut UiState) {
    match key.code {
        KeyCode::Enter => ui.submit_task(app),
        KeyCode::Char(c) => ui.current_input.push(c),
        KeyCode::Backspace => { ui.current_input.pop(); }
        KeyCode::Esc => {
            ui.input_mode = InputMode::Normal;
            ui.current_input.clear();
            ui.editing_task_index = None;
        }
        _ => {}
    }
}

fn ui(frame: &mut Frame, app: &App, ui_state: &UiState) {
    let theme = Theme::from_settings(app.settings.theme);
    match app.current_view {
        View::Timer => draw_timer(frame, app, &theme),
        View::TaskList => draw_task_list(frame, app, ui_state, &theme),
        View::Statistics => draw_statistics(frame, app, ui_state, &theme),
        View::Settings => draw_settings(frame, app, ui_state, &theme),
        View::TaskDetails => draw_task_details(frame, app, ui_state, &theme),
    }
    if matches!(ui_state.input_mode, InputMode::EditingNotes) {
        draw_notes_modal(frame, ui_state, &theme);
    }
}


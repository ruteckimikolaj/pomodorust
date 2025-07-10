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

/// Represents the different timer modes in the Pomodoro technique.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Pomodoro,
    ShortBreak,
    LongBreak,
}

impl Mode {
    /// Returns the duration of the timer mode.
    fn duration(&self) -> Duration {
        match self {
            Mode::Pomodoro => Duration::from_secs(25 * 60),
            Mode::ShortBreak => Duration::from_secs(5 * 60),
            Mode::LongBreak => Duration::from_secs(15 * 60),
        }
    }

    /// Returns the title of the timer mode.
    fn title(&self) -> &'static str {
        match self {
            Mode::Pomodoro => "Pomodoro",
            Mode::ShortBreak => "Short Break",
            Mode::LongBreak => "Long Break",
        }
    }
}

/// Represents the current state of the timer.
#[derive(Clone, Copy, PartialEq)]
enum TimerState {
    Paused,
    Running,
}

/// The main application state.
struct App {
    mode: Mode,
    state: TimerState,
    time_remaining: Duration,
    last_tick: Instant,
    pomodoros_completed: u32,
    should_quit: bool,
}

impl App {
    /// Creates a new App instance.
    fn new() -> Self {
        Self {
            mode: Mode::Pomodoro,
            state: TimerState::Paused,
            time_remaining: Mode::Pomodoro.duration(),
            last_tick: Instant::now(),
            pomodoros_completed: 0,
            should_quit: false,
        }
    }

    /// Updates the application state on each tick.
    fn on_tick(&mut self) {
        if let TimerState::Running = self.state {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_tick);
            self.last_tick = now;

            if let Some(remaining) = self.time_remaining.checked_sub(elapsed) {
                self.time_remaining = remaining;
            } else {
                self.time_remaining = Duration::from_secs(0);
                self.next_mode();
            }
        }
    }
    
    /// Toggles the timer between running and paused states.
    fn toggle_timer(&mut self) {
        match self.state {
            TimerState::Paused => {
                self.state = TimerState::Running;
                self.last_tick = Instant::now();
            }
            TimerState::Running => self.state = TimerState::Paused,
        }
    }

    /// Resets the timer to the current mode's full duration.
    fn reset_timer(&mut self) {
        self.state = TimerState::Paused;
        self.time_remaining = self.mode.duration();
    }

    /// Switches to the next appropriate timer mode.
    fn next_mode(&mut self) {
        if self.mode == Mode::Pomodoro {
            self.pomodoros_completed += 1;
            if self.pomodoros_completed % 4 == 0 {
                self.mode = Mode::LongBreak;
            } else {
                self.mode = Mode::ShortBreak;
            }
        } else {
            self.mode = Mode::Pomodoro;
        }
        self.reset_timer();
        // Automatically start the next session
        self.state = TimerState::Running;
        self.last_tick = Instant::now();
    }

    /// Sets the current mode explicitly.
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.reset_timer();
    }
}

/// Main function to run the application.
fn main() -> io::Result<()> {
    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create app and run it
    let app = App::new();
    run_app(&mut terminal, app)?;

    // Restore terminal
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
    let tick_rate = Duration::from_millis(250);
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(app.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char(' ') => app.toggle_timer(),
                        KeyCode::Char('r') => app.reset_timer(),
                        KeyCode::Char('p') => app.set_mode(Mode::Pomodoro),
                        KeyCode::Char('s') => app.set_mode(Mode::ShortBreak),
                        KeyCode::Char('l') => app.set_mode(Mode::LongBreak),
                        _ => {}
                    }
                }
            }
        }

        app.on_tick();
        if app.should_quit {
            return Ok(());
        }
    }
}

/// Renders the user interface.
fn ui(frame: &mut Frame, app: &App) {
    // === STYLING ===
    let (accent_color, mode_bg_color) = match app.mode {
        Mode::Pomodoro => (Color::LightRed, Color::Rgb(50, 20, 20)),
        Mode::ShortBreak => (Color::LightGreen, Color::Rgb(20, 50, 20)),
        Mode::LongBreak => (Color::LightBlue, Color::Rgb(20, 20, 50)),
    };

    let base_style = Style::default().bg(Color::Black).fg(Color::Gray);
    let accent_style = Style::default().fg(accent_color);
    let running_style = Style::default().fg(Color::Green);
    let paused_style = Style::default().fg(Color::Yellow);
    let key_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    // === LAYOUT ===
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(frame.size());

    // === BLOCKS ===

    // --- Title Block ---
    let title_block = Block::default()
        .title(block::Title::from("Pomodorust").alignment(Alignment::Center))
        .style(base_style);
    frame.render_widget(title_block, main_layout[0]);

    // --- Main Timer Block ---
    let timer_block_border_style = if matches!(app.state, TimerState::Running) {
        Style::default().fg(accent_color)
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

    // --- Help/Instructions Block ---
    let help_block = Block::default()
        .title("Controls")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(base_style);
    frame.render_widget(help_block, main_layout[2]);

    // === WIDGETS INSIDE BLOCKS ===

    // --- Widgets inside Timer Block ---
    let timer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
        .margin(1)
        .split(timer_area);

    // Timer Text
    let time = ChronoDuration::from_std(app.time_remaining).expect("valid duration");
    let time_text = format!(
        "{:02}:{:02}",
        time.num_minutes(),
        time.num_seconds() % 60
    );
    let timer_paragraph = Paragraph::new(time_text)
        .style(accent_style.add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(timer_paragraph, timer_layout[0]);

    // Status Text
    let (status_text, status_style) = match app.state {
        TimerState::Running => ("▶ Running", running_style),
        TimerState::Paused => ("⏸ Paused", paused_style),
    };
    let status_paragraph = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Center);
    frame.render_widget(status_paragraph, timer_layout[1]);

    // Progress Bar
    let total_duration = app.mode.duration().as_secs_f64();
    let remaining_duration = app.time_remaining.as_secs_f64();
    let progress_ratio = if total_duration > 0.0 {
        (total_duration - remaining_duration) / total_duration
    } else { 1.0 };
    let progress_bar = Gauge::default()
        .gauge_style(accent_style)
        .ratio(progress_ratio)
        .label(format!("{:.0}%", progress_ratio * 100.0));
    frame.render_widget(progress_bar, timer_layout[2]);

    // Pomodoros Completed
    let sessions_text = format!("🍅 {}", app.pomodoros_completed);
    let sessions_paragraph = Paragraph::new(sessions_text)
        .style(accent_style)
        .alignment(Alignment::Center);
    frame.render_widget(sessions_paragraph, timer_layout[3]);

    // --- Widgets inside Help Block ---
    let help_layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .vertical_margin(1)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_layout[2]);
    
    let left_help_text = Text::from(vec![
        Line::from(vec![
            Span::styled("p", key_style),
            Span::raw("omodoro "),
            Span::styled("s", key_style),
            Span::raw("hort break "),
            Span::styled("l", key_style),
            Span::raw("ong break"),
        ]),
        Line::from(vec![
            Span::styled("q", key_style),
            Span::raw("uit"),
        ]),
    ]);
    
    let right_help_text = Text::from(vec![
        Line::from(vec![
            Span::styled("<space>", key_style),
            Span::raw(" start/pause"),
        ]),
        Line::from(vec![
            Span::styled("r", key_style),
            Span::raw("eset timer"),
        ]),
    ]);

    let left_para = Paragraph::new(left_help_text).alignment(Alignment::Left);
    let right_para = Paragraph::new(right_help_text).alignment(Alignment::Right);

    frame.render_widget(left_para, help_layout[0]);
    frame.render_widget(right_para, help_layout[1]);
}

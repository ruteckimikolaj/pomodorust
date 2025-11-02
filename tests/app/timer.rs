use pomodorust::app::{
    types::{Mode, TimerState},
    App,
};
use std::time::Duration;

#[test]
fn test_toggle_timer() {
    let mut app = App::new();
    app.current_input = "test".to_string();
    app.submit_task();
    app.toggle_timer();
    assert_eq!(app.state, TimerState::Running);
    app.toggle_timer();
    assert_eq!(app.state, TimerState::Paused);
}

#[test]
fn test_reset_timer() {
    let mut app = App::new();
    app.current_input = "test".to_string();
    app.submit_task();
    app.toggle_timer();
    app.time_remaining = Duration::from_secs(0);
    app.reset_timer();
    assert_eq!(app.state, TimerState::Paused);
    assert_eq!(
        app.time_remaining,
        app.mode.duration(&app.settings)
    );
}

#[test]
fn test_next_mode() {
    let mut app = App::new();
    app.current_input = "test".to_string();
    app.submit_task();
    app.next_mode();
    assert_eq!(app.mode, Mode::ShortBreak);
    app.next_mode();
    assert_eq!(app.mode, Mode::Pomodoro);
    app.pomodoros_completed_total = 3;
    app.next_mode();
    assert_eq!(app.mode, Mode::LongBreak);
}

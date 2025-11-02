use pomodorust::settings::{ColorTheme, Settings};
use std::time::Duration;

#[test]
fn test_settings_default() {
    let settings = Settings::default();
    assert_eq!(settings.pomodoro_duration, Duration::from_secs(25 * 60));
    assert_eq!(settings.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(settings.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(settings.theme, ColorTheme::Default);
}

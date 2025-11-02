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

#[test]
fn test_settings_save_load() {
    let mut settings = Settings::default();
    settings.pomodoro_duration = Duration::from_secs(30 * 60);
    settings.theme = ColorTheme::Dracula;

    let dir = tempfile::tempdir().unwrap();
    let _file_path = dir.path().join("config.toml");

    // Override the config path for testing purposes
    std::env::set_var(
        "XDG_CONFIG_HOME",
        dir.path().parent().unwrap().to_str().unwrap(),
    );
    std::env::set_var("HOME", dir.path().parent().unwrap().to_str().unwrap());

    settings.save();

    let loaded_settings = Settings::load();
    assert_eq!(settings.pomodoro_duration, loaded_settings.pomodoro_duration);
    assert_eq!(settings.theme, loaded_settings.theme);
}

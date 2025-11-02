use pomodorust::{settings::{ColorTheme, CustomTheme}, theme::Theme};
use ratatui::style::Color;
use std::collections::HashMap;

#[test]
fn test_theme_from_settings() {
    let custom_themes = HashMap::<String, CustomTheme>::new();
    let dracula_theme = Theme::from_settings(ColorTheme::Dracula, &custom_themes);
    assert_eq!(dracula_theme.pomodoro_color, Color::Rgb(255, 85, 85));

    let default_theme = Theme::from_settings(ColorTheme::Default, &custom_themes);
    assert_eq!(default_theme.pomodoro_color, Color::LightRed);
}

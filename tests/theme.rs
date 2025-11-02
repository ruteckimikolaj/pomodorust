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

use pomodorust::theme;

#[test]
fn test_parse_color() {
    assert_eq!(theme::parse_color("#ff0000"), Color::Rgb(255, 0, 0));
    assert_eq!(theme::parse_color("lightred"), Color::LightRed);
    assert_eq!(theme::parse_color("invalid"), Color::Reset);
}

#[test]
fn test_from_custom() {
    let mut custom_themes = HashMap::new();
    custom_themes.insert(
        "MyCoolTheme".to_string(),
        CustomTheme {
            pomodoro_color: "#ff0000".to_string(),
            short_break_color: "lightgreen".to_string(),
            long_break_color: "#0000ff".to_string(),
            pomodoro_bg: "#331111".to_string(),
            short_break_bg: "#113311".to_string(),
            long_break_bg: "#111133".to_string(),
            accent_color: "magenta".to_string(),
            base_fg: "#dddddd".to_string(),
            base_bg: "#111111".to_string(),
            running_fg: "green".to_string(),
            paused_fg: "yellow".to_string(),
            highlight_bg: "#555555".to_string(),
            help_text_fg: "#777777".to_string(),
        },
    );

    let theme = Theme::from_settings(
        ColorTheme::Custom("MyCoolTheme".to_string()),
        &custom_themes,
    );
    assert_eq!(theme.pomodoro_color, Color::Rgb(255, 0, 0));
    assert_eq!(theme.short_break_color, Color::LightGreen);
}

use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};

use crate::app::get_config_path;

pub mod theme;
pub use theme::Theme;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorTheme {
    #[default]
    Default,
    Dracula,
    Solarized,
    Nord,
    GruvboxDark,
    Cyberpunk,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CustomThemeColors {
    pub pomodoro_color: Option<String>,
    pub short_break_color: Option<String>,
    pub long_break_color: Option<String>,
    pub pomodoro_bg: Option<String>,
    pub short_break_bg: Option<String>,
    pub long_break_bg: Option<String>,
    pub accent_color: Option<String>,
    pub base_fg: Option<String>,
    pub base_bg: Option<String>,
    pub running_fg: Option<String>,
    pub paused_fg: Option<String>,
    pub highlight_bg: Option<String>,
    pub help_text_fg: Option<String>,
}

fn default_pomodoro_mins() -> u64 { 25 }
fn default_short_break_mins() -> u64 { 5 }
fn default_long_break_mins() -> u64 { 15 }
fn default_long_break_interval() -> u32 { 4 }
fn default_notifications() -> bool { true }

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerializableSettings {
    #[serde(default = "default_pomodoro_mins")]
    pomodoro_duration_mins: u64,
    #[serde(default = "default_short_break_mins")]
    short_break_duration_mins: u64,
    #[serde(default = "default_long_break_mins")]
    long_break_duration_mins: u64,
    #[serde(default = "default_long_break_interval")]
    long_break_interval: u32,
    #[serde(default)]
    theme: ColorTheme,
    #[serde(default = "default_notifications")]
    desktop_notifications: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_theme: Option<CustomThemeColors>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub pomodoro_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32,
    pub theme: ColorTheme,
    pub desktop_notifications: bool,
    pub custom_theme: Option<CustomThemeColors>,
}

impl From<SerializableSettings> for Settings {
    fn from(s: SerializableSettings) -> Self {
        Self {
            pomodoro_duration: Duration::from_secs(s.pomodoro_duration_mins * 60),
            short_break_duration: Duration::from_secs(s.short_break_duration_mins * 60),
            long_break_duration: Duration::from_secs(s.long_break_duration_mins * 60),
            long_break_interval: s.long_break_interval,
            theme: s.theme,
            desktop_notifications: s.desktop_notifications,
            custom_theme: s.custom_theme,
        }
    }
}

impl From<&Settings> for SerializableSettings {
    fn from(s: &Settings) -> Self {
        Self {
            pomodoro_duration_mins: s.pomodoro_duration.as_secs() / 60,
            short_break_duration_mins: s.short_break_duration.as_secs() / 60,
            long_break_duration_mins: s.long_break_duration.as_secs() / 60,
            long_break_interval: s.long_break_interval,
            theme: s.theme,
            desktop_notifications: s.desktop_notifications,
            custom_theme: s.custom_theme.clone(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pomodoro_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            long_break_interval: 4,
            theme: ColorTheme::Default,
            desktop_notifications: true,
            custom_theme: None,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Some(path) = get_config_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(serializable) = toml::from_str::<SerializableSettings>(&content) {
                    return serializable.into();
                }
            }
        }
        let default_settings = Settings::default();
        default_settings.save();
        default_settings
    }

    pub fn save(&self) {
        if let Some(path) = get_config_path() {
            if let Some(parent) = path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    let serializable = SerializableSettings::from(self);
                    if let Ok(toml_string) = toml::to_string_pretty(&serializable) {
                        let _ = fs::write(path, toml_string);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_with_custom_theme() {
        let toml = r##"
pomodoro_duration_mins = 49
short_break_duration_mins = 6
long_break_duration_mins = 30
long_break_interval = 4
theme = "Default"
desktop_notifications = true

[custom_theme]
pomodoro_color = "#fb4934"
base_bg = "#282828"
"##;
        let s: SerializableSettings = toml::from_str(toml).expect("parse failed");
        assert!(s.custom_theme.is_some(), "custom_theme should be Some");
        let ct = s.custom_theme.unwrap();
        assert_eq!(ct.pomodoro_color.as_deref(), Some("#fb4934"));
        assert!(ct.short_break_color.is_none(), "unset field should be None");
    }

    #[test]
    fn deserialize_without_long_break_interval() {
        let toml = r##"
pomodoro_duration_mins = 49
short_break_duration_mins = 6
long_break_duration_mins = 30
theme = "Default"
desktop_notifications = true

[custom_theme]
base_bg = "#282828"
"##;
        let s: SerializableSettings = toml::from_str(toml).expect("parse should not fail without long_break_interval");
        assert_eq!(s.long_break_interval, 4);
        assert!(s.custom_theme.is_some());
    }
}

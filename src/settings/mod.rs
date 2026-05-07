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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerializableSettings {
    pomodoro_duration_mins: u64,
    short_break_duration_mins: u64,
    long_break_duration_mins: u64,
    long_break_interval: u32,
    theme: ColorTheme,
    desktop_notifications: bool,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub pomodoro_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32,
    pub theme: ColorTheme,
    pub desktop_notifications: bool,
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

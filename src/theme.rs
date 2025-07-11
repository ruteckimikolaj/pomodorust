use crate::settings::ColorTheme;
use ratatui::style::Color;

/// A struct that holds all the colors for a given theme.
pub struct Theme {
    pub name: &'static str,
    pub pomodoro_color: Color,
    pub short_break_color: Color,
    pub long_break_color: Color,
    pub pomodoro_bg: Color,
    pub short_break_bg: Color,
    pub long_break_bg: Color,
    pub accent_color: Color,
    pub base_fg: Color,
    pub base_bg: Color,
    pub running_fg: Color,
    pub paused_fg: Color,
    pub highlight_bg: Color,
    pub help_text_fg: Color,
}

impl Theme {
    /// Creates a Theme based on the ColorTheme enum from settings.
    pub fn from_settings(theme_enum: ColorTheme) -> Self {
        match theme_enum {
            ColorTheme::Default => Self::default(),
            ColorTheme::Dracula => Self::dracula(),
            ColorTheme::Solarized => Self::solarized(),
            ColorTheme::Nord => Self::nord(),
        }
    }

    /// Dracula theme colors.
    pub fn dracula() -> Self {
        Self {
            name: "Dracula",
            pomodoro_color: Color::Rgb(255, 85, 85), // Red
            short_break_color: Color::Rgb(80, 250, 123), // Green
            long_break_color: Color::Rgb(189, 147, 249), // Purple
            pomodoro_bg: Color::Rgb(50, 20, 20),
            short_break_bg: Color::Rgb(20, 50, 20),
            long_break_bg: Color::Rgb(30, 20, 50),
            accent_color: Color::Rgb(255, 121, 198), // Pink
            base_fg: Color::Rgb(248, 248, 242), // Foreground
            base_bg: Color::Rgb(40, 42, 54), // Background
            running_fg: Color::Rgb(80, 250, 123), // Green
            paused_fg: Color::Rgb(255, 184, 108), // Orange
            highlight_bg: Color::Rgb(68, 71, 90), // Selection
            help_text_fg: Color::Rgb(98, 114, 164), // Comment
        }
    }
    
    /// Solarized theme colors.
    pub fn solarized() -> Self {
        Self {
            name: "Solarized",
            pomodoro_color: Color::Rgb(220, 50, 47), // red
            short_break_color: Color::Rgb(133, 153, 0), // green
            long_break_color: Color::Rgb(38, 139, 210), // blue
            pomodoro_bg: Color::Rgb(50, 20, 20),
            short_break_bg: Color::Rgb(20, 50, 20),
            long_break_bg: Color::Rgb(20, 20, 50),
            accent_color: Color::Rgb(211, 54, 130), // magenta
            base_fg: Color::Rgb(131, 148, 150), // base1
            base_bg: Color::Rgb(0, 43, 54), // base03
            running_fg: Color::Rgb(133, 153, 0), // green
            paused_fg: Color::Rgb(181, 137, 0), // yellow
            highlight_bg: Color::Rgb(7, 54, 66), // base02
            help_text_fg: Color::Rgb(88, 110, 117), // base01
        }
    }

    /// Nord theme colors.
    pub fn nord() -> Self {
        Self {
            name: "Nord",
            pomodoro_color: Color::Rgb(191, 97, 106), // nord11
            short_break_color: Color::Rgb(163, 190, 140), // nord14
            long_break_color: Color::Rgb(129, 161, 193), // nord10
            pomodoro_bg: Color::Rgb(50, 25, 25),
            short_break_bg: Color::Rgb(25, 50, 25),
            long_break_bg: Color::Rgb(25, 25, 50),
            accent_color: Color::Rgb(180, 142, 173), // nord15
            base_fg: Color::Rgb(216, 222, 233), // nord4
            base_bg: Color::Rgb(46, 52, 64), // nord0
            running_fg: Color::Rgb(163, 190, 140), // nord14
            paused_fg: Color::Rgb(235, 203, 139), // nord13
            highlight_bg: Color::Rgb(59, 66, 82), // nord1
            help_text_fg: Color::Rgb(76, 86, 106), // nord3
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Default",
            pomodoro_color: Color::LightRed,
            short_break_color: Color::LightGreen,
            long_break_color: Color::LightBlue,
            pomodoro_bg: Color::Rgb(50, 20, 20),
            short_break_bg: Color::Rgb(20, 50, 20),
            long_break_bg: Color::Rgb(20, 20, 50),
            accent_color: Color::LightMagenta,
            base_fg: Color::Gray,
            base_bg: Color::Black,
            running_fg: Color::Green,
            paused_fg: Color::Yellow,
            highlight_bg: Color::DarkGray,
            help_text_fg: Color::DarkGray,
        }
    }
}

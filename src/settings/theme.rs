use ratatui::style::Color;
use super::ColorTheme;

pub struct Theme {
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
    pub fn from_settings(theme_enum: ColorTheme) -> Self {
        match theme_enum {
            ColorTheme::Default => Self::default(),
            ColorTheme::Dracula => Self::dracula(),
            ColorTheme::Solarized => Self::solarized(),
            ColorTheme::Nord => Self::nord(),
        }
    }

    pub fn dracula() -> Self {
        Self {
            pomodoro_color: Color::Rgb(255, 85, 85),
            short_break_color: Color::Rgb(80, 250, 123),
            long_break_color: Color::Rgb(189, 147, 249),
            pomodoro_bg: Color::Rgb(50, 20, 20),
            short_break_bg: Color::Rgb(20, 50, 20),
            long_break_bg: Color::Rgb(30, 20, 50),
            accent_color: Color::Rgb(255, 121, 198),
            base_fg: Color::Rgb(248, 248, 242),
            base_bg: Color::Rgb(40, 42, 54),
            running_fg: Color::Rgb(80, 250, 123),
            paused_fg: Color::Rgb(255, 184, 108),
            highlight_bg: Color::Rgb(68, 71, 90),
            help_text_fg: Color::Rgb(98, 114, 164),
        }
    }

    pub fn solarized() -> Self {
        Self {
            pomodoro_color: Color::Rgb(220, 50, 47),
            short_break_color: Color::Rgb(133, 153, 0),
            long_break_color: Color::Rgb(38, 139, 210),
            pomodoro_bg: Color::Rgb(50, 20, 20),
            short_break_bg: Color::Rgb(20, 50, 20),
            long_break_bg: Color::Rgb(20, 20, 50),
            accent_color: Color::Rgb(211, 54, 130),
            base_fg: Color::Rgb(131, 148, 150),
            base_bg: Color::Rgb(0, 43, 54),
            running_fg: Color::Rgb(133, 153, 0),
            paused_fg: Color::Rgb(181, 137, 0),
            highlight_bg: Color::Rgb(7, 54, 66),
            help_text_fg: Color::Rgb(88, 110, 117),
        }
    }

    pub fn nord() -> Self {
        Self {
            pomodoro_color: Color::Rgb(191, 97, 106),
            short_break_color: Color::Rgb(163, 190, 140),
            long_break_color: Color::Rgb(129, 161, 193),
            pomodoro_bg: Color::Rgb(50, 25, 25),
            short_break_bg: Color::Rgb(25, 50, 25),
            long_break_bg: Color::Rgb(25, 25, 50),
            accent_color: Color::Rgb(180, 142, 173),
            base_fg: Color::Rgb(216, 222, 233),
            base_bg: Color::Rgb(46, 52, 64),
            running_fg: Color::Rgb(163, 190, 140),
            paused_fg: Color::Rgb(235, 203, 139),
            highlight_bg: Color::Rgb(59, 66, 82),
            help_text_fg: Color::Rgb(76, 86, 106),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
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

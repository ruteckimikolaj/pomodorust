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
            // Exact Dracula palette colors
            pomodoro_color:    Color::Rgb(255,  85,  85), // red
            short_break_color: Color::Rgb( 80, 250, 123), // green
            long_break_color:  Color::Rgb(189, 147, 249), // purple
            // Subtle tints of the base bg (40,42,54) — readable, not a black hole
            pomodoro_bg:       Color::Rgb( 58,  38,  42),
            short_break_bg:    Color::Rgb( 36,  52,  42),
            long_break_bg:     Color::Rgb( 42,  38,  62),
            accent_color:      Color::Rgb(255, 121, 198), // pink — Dracula's brand color
            base_fg:           Color::Rgb(248, 248, 242),
            base_bg:           Color::Rgb( 40,  42,  54),
            running_fg:        Color::Rgb( 80, 250, 123),
            paused_fg:         Color::Rgb(255, 184, 108), // orange
            highlight_bg:      Color::Rgb( 68,  71,  90),
            help_text_fg:      Color::Rgb( 98, 114, 164),
        }
    }

    pub fn solarized() -> Self {
        Self {
            // Exact Solarized Dark palette
            pomodoro_color:    Color::Rgb(220,  50,  47), // red
            short_break_color: Color::Rgb(133, 153,   0), // green
            long_break_color:  Color::Rgb( 38, 139, 210), // blue
            // Subtle tints of the base bg (0,43,54)
            pomodoro_bg:       Color::Rgb( 28,  36,  44),
            short_break_bg:    Color::Rgb( 12,  46,  44),
            long_break_bg:     Color::Rgb(  8,  40,  60),
            accent_color:      Color::Rgb(108, 113, 196), // violet — less aggressive than magenta
            base_fg:           Color::Rgb(131, 148, 150), // base0
            base_bg:           Color::Rgb(  0,  43,  54), // base03
            running_fg:        Color::Rgb(133, 153,   0),
            paused_fg:         Color::Rgb(181, 137,   0), // yellow
            highlight_bg:      Color::Rgb(  7,  54,  66), // base02
            help_text_fg:      Color::Rgb( 88, 110, 117), // base01
        }
    }

    pub fn nord() -> Self {
        Self {
            // Exact Nord palette
            pomodoro_color:    Color::Rgb(191,  97, 106), // nord11 red
            short_break_color: Color::Rgb(163, 190, 140), // nord14 green
            long_break_color:  Color::Rgb(129, 161, 193), // nord9 blue
            // Subtle tints of the base bg (46,52,64)
            pomodoro_bg:       Color::Rgb( 60,  46,  50),
            short_break_bg:    Color::Rgb( 44,  56,  48),
            long_break_bg:     Color::Rgb( 44,  50,  68),
            accent_color:      Color::Rgb(136, 192, 208), // nord8 frost — teal, no pink
            base_fg:           Color::Rgb(216, 222, 233), // nord4
            base_bg:           Color::Rgb( 46,  52,  64), // nord0
            running_fg:        Color::Rgb(163, 190, 140),
            paused_fg:         Color::Rgb(235, 203, 139), // nord13 yellow
            highlight_bg:      Color::Rgb( 59,  66,  82), // nord1
            help_text_fg:      Color::Rgb( 76,  86, 106), // nord3
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Warm tomato red — not the terminal's LightRed which renders pink
            pomodoro_color:    Color::Rgb(210,  70,  55),
            short_break_color: Color::Rgb( 75, 175, 100),
            long_break_color:  Color::Rgb( 75, 145, 210),
            // Very subtle tints over near-black bg — just a hint of colour
            pomodoro_bg:       Color::Rgb( 38,  22,  20),
            short_break_bg:    Color::Rgb( 20,  36,  22),
            long_break_bg:     Color::Rgb( 18,  24,  42),
            // Amber/gold accent — warm, neutral, no pink
            accent_color:      Color::Rgb(210, 155,  50),
            base_fg:           Color::Rgb(210, 210, 210),
            base_bg:           Color::Rgb( 18,  18,  22),
            running_fg:        Color::Rgb( 80, 185, 105),
            paused_fg:         Color::Rgb(205, 160,  55),
            // Highlight clearly different from bg, text stays readable
            highlight_bg:      Color::Rgb( 45,  52,  68),
            // Subdued but legible against near-black bg
            help_text_fg:      Color::Rgb(110, 115, 130),
        }
    }
}

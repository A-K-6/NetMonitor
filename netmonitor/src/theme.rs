use serde::{Serialize, Deserialize};
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeType {
    Auto,
    Terminal,
    Default,
    Dracula,
    Solarized,
    Monokai,
    HighContrast,
}

impl ThemeType {
    pub fn all() -> Vec<ThemeType> {
        vec![
            ThemeType::Auto,
            ThemeType::Terminal,
            ThemeType::Default,
            ThemeType::Dracula,
            ThemeType::Solarized,
            ThemeType::Monokai,
            ThemeType::HighContrast,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            ThemeType::Auto => "Auto (System)",
            ThemeType::Terminal => "Terminal Native",
            ThemeType::Default => "Default",
            ThemeType::Dracula => "Dracula",
            ThemeType::Solarized => "Solarized",
            ThemeType::Monokai => "Monokai",
            ThemeType::HighContrast => "High Contrast",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub border_fg: Color,
    pub alert_fg: Color,
    pub upload_fg: Color,
    pub download_fg: Color,
    pub status_fg: Color,
    pub help_fg: Color,
}

impl Theme {
    pub fn from_type(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Auto => {
                Self::from_type(ThemeType::Default)
            }
            ThemeType::Terminal => Theme {
                bg: Color::Reset,
                header_fg: Color::Indexed(6), // Cyan
                row_fg: Color::Reset,         // Default FG
                highlight_fg: Color::Indexed(0), // Black
                highlight_bg: Color::Indexed(7), // White
                border_fg: Color::Indexed(8),  // Gray
                alert_fg: Color::Indexed(1),   // Red
                upload_fg: Color::Indexed(2),  // Green
                download_fg: Color::Indexed(3), // Yellow
                status_fg: Color::Indexed(8),  // Gray
                help_fg: Color::Indexed(5),    // Magenta
            },
            ThemeType::Default => Theme {
                bg: Color::Reset,
                header_fg: Color::Cyan,
                row_fg: Color::White,
                highlight_fg: Color::Black,
                highlight_bg: Color::White,
                border_fg: Color::Gray,
                alert_fg: Color::Red,
                upload_fg: Color::Green,
                download_fg: Color::Yellow,
                status_fg: Color::DarkGray,
                help_fg: Color::Magenta,
            },
            ThemeType::Dracula => Theme {
                bg: Color::Rgb(40, 42, 54),
                header_fg: Color::Rgb(189, 147, 249), // Purple
                row_fg: Color::Rgb(248, 248, 242),    // Foreground
                highlight_fg: Color::Rgb(40, 42, 54), // Background
                highlight_bg: Color::Rgb(98, 114, 164), // Selection
                border_fg: Color::Rgb(68, 71, 90),     // Comment
                alert_fg: Color::Rgb(255, 85, 85),      // Red
                upload_fg: Color::Rgb(80, 250, 123),   // Green
                download_fg: Color::Rgb(241, 250, 140), // Yellow
                status_fg: Color::Rgb(98, 114, 164),    // Comment
                help_fg: Color::Rgb(255, 121, 198),     // Pink
            },
            ThemeType::Solarized => Theme {
                bg: Color::Rgb(0, 43, 54),             // Base03
                header_fg: Color::Rgb(38, 139, 210),   // Blue
                row_fg: Color::Rgb(131, 148, 150),     // Content
                highlight_fg: Color::Rgb(253, 246, 227), // Base3
                highlight_bg: Color::Rgb(7, 54, 66),    // Base02
                border_fg: Color::Rgb(88, 110, 117),   // Base01
                alert_fg: Color::Rgb(220, 50, 47),      // Red
                upload_fg: Color::Rgb(133, 153, 0),     // Green
                download_fg: Color::Rgb(181, 137, 0),   // Yellow
                status_fg: Color::Rgb(101, 123, 131),   // Base00
                help_fg: Color::Rgb(211, 54, 130),     // Magenta
            },
            ThemeType::Monokai => Theme {
                bg: Color::Rgb(39, 40, 34),
                header_fg: Color::Rgb(102, 217, 239), // Cyan
                row_fg: Color::Rgb(248, 248, 242),    // White
                highlight_fg: Color::Rgb(39, 40, 34),  // Blackish
                highlight_bg: Color::Rgb(166, 226, 46), // Green
                border_fg: Color::Rgb(117, 113, 94),   // Gray
                alert_fg: Color::Rgb(249, 38, 114),     // Pink/Red
                upload_fg: Color::Rgb(166, 226, 46),   // Green
                download_fg: Color::Rgb(230, 219, 116), // Yellow
                status_fg: Color::Rgb(117, 113, 94),    // Gray
                help_fg: Color::Rgb(253, 151, 31),      // Orange
            },
            ThemeType::HighContrast => Theme {
                bg: Color::Black,
                header_fg: Color::White,
                row_fg: Color::White,
                highlight_fg: Color::Black,
                highlight_bg: Color::White,
                border_fg: Color::White,
                alert_fg: Color::White, // BOLD will distinguish it
                upload_fg: Color::White,
                download_fg: Color::White,
                status_fg: Color::White,
                help_fg: Color::White,
            },
        }
    }
}

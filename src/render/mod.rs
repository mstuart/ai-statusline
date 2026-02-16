use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorLevel {
    None,
    Basic16,
    Color256,
    TrueColor,
}

#[derive(Debug, Clone)]
pub enum ColorSpec {
    Named(String),
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

pub struct Renderer {
    pub color_level: ColorLevel,
}

impl Renderer {
    pub fn detect(override_level: &str) -> Self {
        let color_level = match override_level {
            "none" => ColorLevel::None,
            "16" => ColorLevel::Basic16,
            "256" => ColorLevel::Color256,
            "truecolor" => ColorLevel::TrueColor,
            _ => Self::detect_color_level(),
        };
        Self { color_level }
    }

    fn detect_color_level() -> ColorLevel {
        if env::var("NO_COLOR").is_ok() {
            return ColorLevel::None;
        }
        if let Ok(ct) = env::var("COLORTERM")
            && (ct == "truecolor" || ct == "24bit")
        {
            return ColorLevel::TrueColor;
        }
        if let Ok(term) = env::var("TERM")
            && term.contains("256color")
        {
            return ColorLevel::Color256;
        }
        ColorLevel::Basic16
    }

    pub fn fg(&self, color: &ColorSpec) -> String {
        match self.color_level {
            ColorLevel::None => String::new(),
            ColorLevel::Basic16 => self.named_fg(color),
            ColorLevel::Color256 => self.ansi256_fg(color),
            ColorLevel::TrueColor => self.truecolor_fg(color),
        }
    }

    pub fn bg(&self, color: &ColorSpec) -> String {
        match self.color_level {
            ColorLevel::None => String::new(),
            ColorLevel::Basic16 => self.named_bg(color),
            ColorLevel::Color256 => self.ansi256_bg(color),
            ColorLevel::TrueColor => self.truecolor_bg(color),
        }
    }

    pub fn bold(&self) -> &str {
        if self.color_level == ColorLevel::None {
            ""
        } else {
            "\x1b[1m"
        }
    }

    pub fn reset(&self) -> &str {
        if self.color_level == ColorLevel::None {
            ""
        } else {
            "\x1b[0m"
        }
    }

    pub fn osc8_link(&self, url: &str, text: &str) -> String {
        if self.color_level == ColorLevel::None {
            text.to_string()
        } else {
            format!("\x1b]8;;{url}\x07{text}\x1b]8;;\x07")
        }
    }

    pub fn parse_color(name: &str) -> ColorSpec {
        match name {
            "black" => ColorSpec::Named("black".into()),
            "red" => ColorSpec::Named("red".into()),
            "green" => ColorSpec::Named("green".into()),
            "yellow" => ColorSpec::Named("yellow".into()),
            "blue" => ColorSpec::Named("blue".into()),
            "magenta" => ColorSpec::Named("magenta".into()),
            "cyan" => ColorSpec::Named("cyan".into()),
            "white" => ColorSpec::Named("white".into()),
            "brightBlack" | "bright_black" => ColorSpec::Named("brightBlack".into()),
            "brightRed" | "bright_red" => ColorSpec::Named("brightRed".into()),
            "brightGreen" | "bright_green" => ColorSpec::Named("brightGreen".into()),
            "brightYellow" | "bright_yellow" => ColorSpec::Named("brightYellow".into()),
            "brightBlue" | "bright_blue" => ColorSpec::Named("brightBlue".into()),
            "brightMagenta" | "bright_magenta" => ColorSpec::Named("brightMagenta".into()),
            "brightCyan" | "bright_cyan" => ColorSpec::Named("brightCyan".into()),
            "brightWhite" | "bright_white" => ColorSpec::Named("brightWhite".into()),
            s if s.starts_with('#') && s.len() == 7 => {
                let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
                ColorSpec::Rgb(r, g, b)
            }
            s if s.parse::<u8>().is_ok() => ColorSpec::Ansi256(s.parse().unwrap()),
            _ => ColorSpec::Named("white".into()),
        }
    }

    fn named_fg(&self, color: &ColorSpec) -> String {
        let code = match color {
            ColorSpec::Named(n) => match n.as_str() {
                "black" => "30",
                "red" => "31",
                "green" => "32",
                "yellow" => "33",
                "blue" => "34",
                "magenta" => "35",
                "cyan" => "36",
                "white" => "37",
                "brightBlack" => "90",
                "brightRed" => "91",
                "brightGreen" => "92",
                "brightYellow" => "93",
                "brightBlue" => "94",
                "brightMagenta" => "95",
                "brightCyan" => "96",
                "brightWhite" => "97",
                _ => "37",
            },
            ColorSpec::Ansi256(n) => return format!("\x1b[38;5;{n}m"),
            ColorSpec::Rgb(r, g, b) => {
                return format!("\x1b[38;5;{}m", Self::rgb_to_256(*r, *g, *b));
            }
        };
        format!("\x1b[{code}m")
    }

    fn named_bg(&self, color: &ColorSpec) -> String {
        let code = match color {
            ColorSpec::Named(n) => match n.as_str() {
                "black" => "40",
                "red" => "41",
                "green" => "42",
                "yellow" => "43",
                "blue" => "44",
                "magenta" => "45",
                "cyan" => "46",
                "white" => "47",
                "brightBlack" | "bgBrightBlack" => "100",
                "brightRed" | "bgBrightRed" => "101",
                "brightGreen" | "bgBrightGreen" => "102",
                "brightYellow" | "bgBrightYellow" => "103",
                "brightBlue" | "bgBrightBlue" => "104",
                "brightMagenta" | "bgBrightMagenta" => "105",
                "brightCyan" | "bgBrightCyan" => "106",
                "brightWhite" | "bgBrightWhite" => "107",
                _ => "40",
            },
            ColorSpec::Ansi256(n) => return format!("\x1b[48;5;{n}m"),
            ColorSpec::Rgb(r, g, b) => {
                return format!("\x1b[48;5;{}m", Self::rgb_to_256(*r, *g, *b));
            }
        };
        format!("\x1b[{code}m")
    }

    fn ansi256_fg(&self, color: &ColorSpec) -> String {
        match color {
            ColorSpec::Ansi256(n) => format!("\x1b[38;5;{n}m"),
            ColorSpec::Rgb(r, g, b) => format!("\x1b[38;5;{}m", Self::rgb_to_256(*r, *g, *b)),
            other => self.named_fg(other),
        }
    }

    fn ansi256_bg(&self, color: &ColorSpec) -> String {
        match color {
            ColorSpec::Ansi256(n) => format!("\x1b[48;5;{n}m"),
            ColorSpec::Rgb(r, g, b) => format!("\x1b[48;5;{}m", Self::rgb_to_256(*r, *g, *b)),
            other => self.named_bg(other),
        }
    }

    fn truecolor_fg(&self, color: &ColorSpec) -> String {
        match color {
            ColorSpec::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
            other => self.ansi256_fg(other),
        }
    }

    fn truecolor_bg(&self, color: &ColorSpec) -> String {
        match color {
            ColorSpec::Rgb(r, g, b) => format!("\x1b[48;2;{r};{g};{b}m"),
            other => self.ansi256_bg(other),
        }
    }

    fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
        if r == g && g == b {
            if r < 8 {
                return 16;
            }
            if r > 248 {
                return 231;
            }
            return ((r as u16 - 8) * 24 / 247 + 232) as u8;
        }
        let ri = (r as u16 * 5 / 255) as u8;
        let gi = (g as u16 * 5 / 255) as u8;
        let bi = (b as u16 * 5 / 255) as u8;
        16 + 36 * ri + 6 * gi + bi
    }
}

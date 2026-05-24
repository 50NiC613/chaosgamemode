use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use ratatui::style::Color;
use serde::Deserialize;

const THEME_RELOAD_RATE: Duration = Duration::from_millis(500);

#[derive(Clone, Copy)]
pub(crate) struct Theme {
    pub(crate) black: Color,
    pub(crate) panel: Color,
    pub(crate) panel_dark: Color,
    pub(crate) foreground: Color,
    pub(crate) cyber_yellow: Color,
    pub(crate) neon_cyan: Color,
    pub(crate) neon_magenta: Color,
    pub(crate) hot_red: Color,
    pub(crate) acid_green: Color,
    pub(crate) orange: Color,
    pub(crate) blue: Color,
    pub(crate) muted: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ThemePreset {
    Cyberpunk,
    Hacker,
    Gruvbox,
    TokyoNight,
    Dracula,
    Nord,
    Solarized,
    Everforest,
    Kanagawa,
    RosePine,
    OneDark,
    Ayu,
    #[default]
    Catppuccin,
}

impl Default for Theme {
    fn default() -> Self {
        ThemePreset::default().theme()
    }
}

#[derive(Default, Deserialize)]
struct ThemeFile {
    preset: Option<String>,
    colors: Option<ThemeColorsFile>,
}

#[derive(Default, Deserialize)]
struct ThemeColorsFile {
    black: Option<String>,
    panel: Option<String>,
    panel_dark: Option<String>,
    foreground: Option<String>,
    cyber_yellow: Option<String>,
    neon_cyan: Option<String>,
    neon_magenta: Option<String>,
    hot_red: Option<String>,
    acid_green: Option<String>,
    orange: Option<String>,
    blue: Option<String>,
    muted: Option<String>,
}

pub(crate) struct ThemeWatcher {
    path: Option<PathBuf>,
    modified: Option<SystemTime>,
    last_check: Instant,
    active_preset: ThemePreset,
}

impl Theme {
    fn from_file(path: &PathBuf) -> Result<LoadedTheme, String> {
        let contents =
            fs::read_to_string(path).map_err(|err| format!("no se pudo leer theme.toml: {err}"))?;
        let theme_file = toml::from_str::<ThemeFile>(&contents)
            .map_err(|err| format!("theme.toml invalido: {err}"))?;
        Ok(Self::from_config(theme_file))
    }

    fn from_config(config: ThemeFile) -> LoadedTheme {
        let preset = config
            .preset
            .as_deref()
            .and_then(ThemePreset::parse)
            .unwrap_or_default();
        let default = preset.theme();
        let Some(colors) = config.colors else {
            return LoadedTheme {
                theme: default,
                preset,
            };
        };

        LoadedTheme {
            theme: Self {
                black: parse_color(colors.black.as_deref()).unwrap_or(default.black),
                panel: parse_color(colors.panel.as_deref()).unwrap_or(default.panel),
                panel_dark: parse_color(colors.panel_dark.as_deref()).unwrap_or(default.panel_dark),
                foreground: parse_color(colors.foreground.as_deref()).unwrap_or(default.foreground),
                cyber_yellow: parse_color(colors.cyber_yellow.as_deref())
                    .unwrap_or(default.cyber_yellow),
                neon_cyan: parse_color(colors.neon_cyan.as_deref()).unwrap_or(default.neon_cyan),
                neon_magenta: parse_color(colors.neon_magenta.as_deref())
                    .unwrap_or(default.neon_magenta),
                hot_red: parse_color(colors.hot_red.as_deref()).unwrap_or(default.hot_red),
                acid_green: parse_color(colors.acid_green.as_deref()).unwrap_or(default.acid_green),
                orange: parse_color(colors.orange.as_deref()).unwrap_or(default.orange),
                blue: parse_color(colors.blue.as_deref()).unwrap_or(default.blue),
                muted: parse_color(colors.muted.as_deref()).unwrap_or(default.muted),
            },
            preset,
        }
    }
}

struct LoadedTheme {
    theme: Theme,
    preset: ThemePreset,
}

impl ThemePreset {
    pub(crate) const ALL: [Self; 13] = [
        Self::Cyberpunk,
        Self::Hacker,
        Self::Gruvbox,
        Self::TokyoNight,
        Self::Dracula,
        Self::Nord,
        Self::Solarized,
        Self::Everforest,
        Self::Kanagawa,
        Self::RosePine,
        Self::OneDark,
        Self::Ayu,
        Self::Catppuccin,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Cyberpunk => "cyberpunk",
            Self::Hacker => "hacker",
            Self::Gruvbox => "gruvbox",
            Self::TokyoNight => "tokyo-night",
            Self::Dracula => "dracula",
            Self::Nord => "nord",
            Self::Solarized => "solarized",
            Self::Everforest => "everforest",
            Self::Kanagawa => "kanagawa",
            Self::RosePine => "rose-pine",
            Self::OneDark => "one-dark",
            Self::Ayu => "ayu",
            Self::Catppuccin => "catppuccin",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Cyberpunk => "Cyberpunk",
            Self::Hacker => "Hacker",
            Self::Gruvbox => "Gruvbox",
            Self::TokyoNight => "Tokyo Night",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::Solarized => "Solarized",
            Self::Everforest => "Everforest",
            Self::Kanagawa => "Kanagawa",
            Self::RosePine => "Rosé Pine",
            Self::OneDark => "One Dark",
            Self::Ayu => "Ayu",
            Self::Catppuccin => "Catppuccin",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "cyberpunk" | "neon" | "neon-gruv" => Some(Self::Cyberpunk),
            "hacker" | "mr-robot" | "mrrobot" | "fsociety" | "terminal-green" => Some(Self::Hacker),
            "gruvbox" | "gruvbox-dark" => Some(Self::Gruvbox),
            "tokyo-night" | "tokyonight" | "tokyo" => Some(Self::TokyoNight),
            "catppuccin" | "catppuccin-mocha" | "mocha" => Some(Self::Catppuccin),
            "dracula" | "dracula-theme" => Some(Self::Dracula),
            "nord" | "nordic" => Some(Self::Nord),
            "solarized" | "solarized-dark" => Some(Self::Solarized),
            "everforest" => Some(Self::Everforest),
            "kanagawa" => Some(Self::Kanagawa),
            "rose-pine" | "rosepine" | "rose" => Some(Self::RosePine),
            "one-dark" | "onedark" | "one" => Some(Self::OneDark),
            "ayu" | "ayu-dark" => Some(Self::Ayu),
            _ => None,
        }
    }

    pub(crate) fn theme(self) -> Theme {
        match self {
            Self::Cyberpunk => Theme {
                black: Color::Rgb(8, 10, 14),
                panel: Color::Rgb(25, 28, 34),
                panel_dark: Color::Rgb(13, 16, 22),
                foreground: Color::Rgb(235, 219, 178),
                cyber_yellow: Color::Rgb(250, 189, 47),
                neon_cyan: Color::Rgb(131, 165, 152),
                neon_magenta: Color::Rgb(211, 134, 155),
                hot_red: Color::Rgb(251, 73, 52),
                acid_green: Color::Rgb(184, 187, 38),
                orange: Color::Rgb(254, 128, 25),
                blue: Color::Rgb(69, 133, 136),
                muted: Color::Rgb(146, 131, 116),
            },
            Self::Hacker => Theme {
                black: Color::Rgb(2, 5, 3),
                panel: Color::Rgb(7, 15, 10),
                panel_dark: Color::Rgb(3, 8, 5),
                foreground: Color::Rgb(207, 255, 216),
                cyber_yellow: Color::Rgb(181, 255, 69),
                neon_cyan: Color::Rgb(83, 255, 141),
                neon_magenta: Color::Rgb(255, 80, 104),
                hot_red: Color::Rgb(255, 59, 48),
                acid_green: Color::Rgb(0, 255, 102),
                orange: Color::Rgb(255, 143, 42),
                blue: Color::Rgb(91, 255, 214),
                muted: Color::Rgb(86, 128, 98),
            },
            Self::Gruvbox => Theme {
                black: Color::Rgb(29, 32, 33),
                panel: Color::Rgb(50, 48, 47),
                panel_dark: Color::Rgb(40, 40, 40),
                foreground: Color::Rgb(235, 219, 178),
                cyber_yellow: Color::Rgb(250, 189, 47),
                neon_cyan: Color::Rgb(142, 192, 124),
                neon_magenta: Color::Rgb(211, 134, 155),
                hot_red: Color::Rgb(251, 73, 52),
                acid_green: Color::Rgb(184, 187, 38),
                orange: Color::Rgb(254, 128, 25),
                blue: Color::Rgb(131, 165, 152),
                muted: Color::Rgb(146, 131, 116),
            },
            Self::TokyoNight => Theme {
                black: Color::Rgb(26, 27, 38),
                panel: Color::Rgb(36, 40, 59),
                panel_dark: Color::Rgb(22, 22, 30),
                foreground: Color::Rgb(192, 202, 245),
                cyber_yellow: Color::Rgb(224, 175, 104),
                neon_cyan: Color::Rgb(125, 207, 255),
                neon_magenta: Color::Rgb(187, 154, 247),
                hot_red: Color::Rgb(247, 118, 142),
                acid_green: Color::Rgb(158, 206, 106),
                orange: Color::Rgb(255, 158, 100),
                blue: Color::Rgb(122, 162, 247),
                muted: Color::Rgb(86, 95, 137),
            },
            Self::Catppuccin => Theme {
                black: Color::Rgb(17, 18, 23),
                panel: Color::Rgb(30, 31, 41),
                panel_dark: Color::Rgb(24, 25, 33),
                foreground: Color::Rgb(205, 214, 244),
                cyber_yellow: Color::Rgb(249, 226, 175),
                neon_cyan: Color::Rgb(148, 226, 213),
                neon_magenta: Color::Rgb(245, 194, 231),
                hot_red: Color::Rgb(243, 139, 168),
                acid_green: Color::Rgb(166, 227, 161),
                orange: Color::Rgb(250, 179, 135),
                blue: Color::Rgb(137, 180, 250),
                muted: Color::Rgb(147, 153, 178),
            },
            Self::Dracula => Theme {
                black: Color::Rgb(40, 42, 54),
                panel: Color::Rgb(56, 58, 70),
                panel_dark: Color::Rgb(33, 34, 44),
                foreground: Color::Rgb(248, 248, 242),
                cyber_yellow: Color::Rgb(241, 250, 140),
                neon_cyan: Color::Rgb(139, 233, 253),
                neon_magenta: Color::Rgb(255, 121, 198),
                hot_red: Color::Rgb(255, 85, 85),
                acid_green: Color::Rgb(80, 250, 123),
                orange: Color::Rgb(255, 184, 108),
                blue: Color::Rgb(189, 147, 249),
                muted: Color::Rgb(98, 114, 164),
            },
            Self::Nord => Theme {
                black: Color::Rgb(46, 52, 64),
                panel: Color::Rgb(59, 66, 82),
                panel_dark: Color::Rgb(37, 41, 50),
                foreground: Color::Rgb(216, 222, 233),
                cyber_yellow: Color::Rgb(235, 203, 139),
                neon_cyan: Color::Rgb(136, 192, 208),
                neon_magenta: Color::Rgb(180, 142, 173),
                hot_red: Color::Rgb(191, 97, 106),
                acid_green: Color::Rgb(163, 190, 140),
                orange: Color::Rgb(208, 135, 112),
                blue: Color::Rgb(129, 161, 193),
                muted: Color::Rgb(76, 86, 106),
            },
            Self::Solarized => Theme {
                black: Color::Rgb(0, 43, 54),
                panel: Color::Rgb(7, 54, 66),
                panel_dark: Color::Rgb(0, 33, 43),
                foreground: Color::Rgb(131, 148, 150),
                cyber_yellow: Color::Rgb(181, 137, 0),
                neon_cyan: Color::Rgb(42, 161, 152),
                neon_magenta: Color::Rgb(211, 54, 130),
                hot_red: Color::Rgb(220, 50, 47),
                acid_green: Color::Rgb(133, 153, 0),
                orange: Color::Rgb(203, 75, 22),
                blue: Color::Rgb(38, 139, 210),
                muted: Color::Rgb(88, 110, 117),
            },
            Self::Everforest => Theme {
                black: Color::Rgb(45, 53, 59),
                panel: Color::Rgb(61, 72, 77),
                panel_dark: Color::Rgb(39, 47, 53),
                foreground: Color::Rgb(211, 198, 170),
                cyber_yellow: Color::Rgb(219, 188, 127),
                neon_cyan: Color::Rgb(131, 192, 146),
                neon_magenta: Color::Rgb(214, 153, 182),
                hot_red: Color::Rgb(230, 126, 128),
                acid_green: Color::Rgb(167, 192, 128),
                orange: Color::Rgb(230, 152, 117),
                blue: Color::Rgb(127, 187, 179),
                muted: Color::Rgb(133, 146, 137),
            },
            Self::Kanagawa => Theme {
                black: Color::Rgb(31, 31, 40),
                panel: Color::Rgb(42, 42, 55),
                panel_dark: Color::Rgb(24, 24, 31),
                foreground: Color::Rgb(220, 215, 186),
                cyber_yellow: Color::Rgb(192, 163, 110),
                neon_cyan: Color::Rgb(127, 180, 202),
                neon_magenta: Color::Rgb(149, 127, 184),
                hot_red: Color::Rgb(195, 64, 67),
                acid_green: Color::Rgb(118, 148, 106),
                orange: Color::Rgb(255, 160, 102),
                blue: Color::Rgb(126, 156, 216),
                muted: Color::Rgb(114, 114, 135),
            },
            Self::RosePine => Theme {
                black: Color::Rgb(25, 23, 36),
                panel: Color::Rgb(31, 29, 46),
                panel_dark: Color::Rgb(19, 17, 29),
                foreground: Color::Rgb(224, 222, 244),
                cyber_yellow: Color::Rgb(246, 193, 119),
                neon_cyan: Color::Rgb(156, 207, 216),
                neon_magenta: Color::Rgb(196, 167, 231),
                hot_red: Color::Rgb(235, 111, 146),
                acid_green: Color::Rgb(49, 116, 143),
                orange: Color::Rgb(235, 188, 186),
                blue: Color::Rgb(156, 207, 216),
                muted: Color::Rgb(110, 106, 134),
            },
            Self::OneDark => Theme {
                black: Color::Rgb(40, 44, 52),
                panel: Color::Rgb(53, 59, 69),
                panel_dark: Color::Rgb(33, 37, 43),
                foreground: Color::Rgb(171, 178, 191),
                cyber_yellow: Color::Rgb(229, 192, 123),
                neon_cyan: Color::Rgb(86, 182, 194),
                neon_magenta: Color::Rgb(198, 120, 221),
                hot_red: Color::Rgb(224, 108, 117),
                acid_green: Color::Rgb(152, 195, 121),
                orange: Color::Rgb(209, 154, 102),
                blue: Color::Rgb(97, 175, 239),
                muted: Color::Rgb(92, 99, 112),
            },
            Self::Ayu => Theme {
                black: Color::Rgb(10, 14, 20),
                panel: Color::Rgb(19, 23, 33),
                panel_dark: Color::Rgb(7, 10, 16),
                foreground: Color::Rgb(179, 177, 173),
                cyber_yellow: Color::Rgb(230, 180, 80),
                neon_cyan: Color::Rgb(149, 230, 203),
                neon_magenta: Color::Rgb(210, 166, 255),
                hot_red: Color::Rgb(255, 51, 51),
                acid_green: Color::Rgb(194, 217, 76),
                orange: Color::Rgb(255, 140, 64),
                blue: Color::Rgb(89, 194, 255),
                muted: Color::Rgb(92, 97, 102),
            },
        }
    }

    fn to_toml(self) -> String {
        let theme = self.theme();
        format!(
            concat!(
                "# Chaos Game Mode live theme\n",
                "# Change preset or edit colors while the TUI is open.\n",
                "# Presets: cyberpunk, hacker, gruvbox, tokyo-night, dracula, nord,\n",
                "#          solarized, everforest, kanagawa, rose-pine, one-dark, ayu, catppuccin.\n",
                "preset = \"{}\"\n\n",
                "[colors]\n",
                "black = \"{}\"\n",
                "panel = \"{}\"\n",
                "panel_dark = \"{}\"\n",
                "foreground = \"{}\"\n",
                "cyber_yellow = \"{}\"\n",
                "neon_cyan = \"{}\"\n",
                "neon_magenta = \"{}\"\n",
                "hot_red = \"{}\"\n",
                "acid_green = \"{}\"\n",
                "orange = \"{}\"\n",
                "blue = \"{}\"\n",
                "muted = \"{}\"\n",
            ),
            self.name(),
            color_to_hex(theme.black),
            color_to_hex(theme.panel),
            color_to_hex(theme.panel_dark),
            color_to_hex(theme.foreground),
            color_to_hex(theme.cyber_yellow),
            color_to_hex(theme.neon_cyan),
            color_to_hex(theme.neon_magenta),
            color_to_hex(theme.hot_red),
            color_to_hex(theme.acid_green),
            color_to_hex(theme.orange),
            color_to_hex(theme.blue),
            color_to_hex(theme.muted),
        )
    }
}

impl ThemeWatcher {
    pub(crate) fn new() -> (Self, Theme, ThemePreset, String) {
        let mut watcher = Self {
            path: find_theme_file(),
            modified: None,
            last_check: Instant::now() - THEME_RELOAD_RATE,
            active_preset: ThemePreset::default(),
        };
        let mut theme = Theme::default();
        let status = watcher.load_theme(&mut theme);
        let active_preset = watcher.active_preset;
        (watcher, theme, active_preset, status)
    }

    pub(crate) fn maybe_reload(&mut self, theme: &mut Theme) -> Option<String> {
        if self.last_check.elapsed() < THEME_RELOAD_RATE {
            return None;
        }
        self.last_check = Instant::now();

        let path = self.path.as_ref()?;
        let modified = fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok();
        if modified.is_some() && modified != self.modified {
            return Some(self.load_theme(theme));
        }
        None
    }

    pub(crate) const fn active_preset(&self) -> ThemePreset {
        self.active_preset
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn apply_preset(&mut self, preset: ThemePreset, theme: &mut Theme) -> String {
        *theme = preset.theme();
        self.active_preset = preset;

        let Some(path) = self.path.as_ref() else {
            return format!(
                "tema activo: {} en memoria; theme.toml no encontrado",
                preset.label()
            );
        };

        match fs::write(path, preset.to_toml()) {
            Ok(()) => {
                self.modified = fs::metadata(path)
                    .and_then(|metadata| metadata.modified())
                    .ok();
                format!("tema activo: {} guardado", preset.label())
            }
            Err(err) => format!("error de tema: {err}"),
        }
    }

    fn load_theme(&mut self, theme: &mut Theme) -> String {
        let Some(path) = self.path.as_ref() else {
            return "tema interno activo; theme.toml no encontrado".to_string();
        };

        self.modified = fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok();

        match Theme::from_file(path) {
            Ok(loaded) => {
                *theme = loaded.theme;
                self.active_preset = loaded.preset;
                format!("tema activo: {}", loaded.preset.label())
            }
            Err(err) => format!("error de tema: {err}"),
        }
    }
}

fn find_theme_file() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("CHAOS_THEME") {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("theme.toml"));
        candidates.push(current_dir.join("tui-rs").join("theme.toml"));
    }
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("theme.toml"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn parse_color(value: Option<&str>) -> Option<Color> {
    let value = value?.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&value[0..2], 16).ok()?;
    let green = u8::from_str_radix(&value[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some(Color::Rgb(red, green, blue))
}

fn color_to_hex(color: Color) -> String {
    match color {
        Color::Rgb(red, green, blue) => format!("#{red:02x}{green:02x}{blue:02x}"),
        _ => "#ffffff".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_should_parse_aliases() {
        assert_eq!(
            ThemePreset::parse("tokyonight"),
            Some(ThemePreset::TokyoNight)
        );
    }

    #[test]
    fn preset_should_parse_hacker_aliases() {
        assert_eq!(ThemePreset::parse("mr-robot"), Some(ThemePreset::Hacker));
        assert_eq!(ThemePreset::parse("fsociety"), Some(ThemePreset::Hacker));
    }

    #[test]
    fn preset_should_parse_new_theme_aliases() {
        assert_eq!(ThemePreset::parse("dracula"), Some(ThemePreset::Dracula));
        assert_eq!(ThemePreset::parse("nord"), Some(ThemePreset::Nord));
        assert_eq!(
            ThemePreset::parse("solarized-dark"),
            Some(ThemePreset::Solarized)
        );
        assert_eq!(
            ThemePreset::parse("everforest"),
            Some(ThemePreset::Everforest)
        );
        assert_eq!(ThemePreset::parse("kanagawa"), Some(ThemePreset::Kanagawa));
        assert_eq!(ThemePreset::parse("rose-pine"), Some(ThemePreset::RosePine));
        assert_eq!(ThemePreset::parse("rosepine"), Some(ThemePreset::RosePine));
        assert_eq!(ThemePreset::parse("one-dark"), Some(ThemePreset::OneDark));
        assert_eq!(ThemePreset::parse("onedark"), Some(ThemePreset::OneDark));
        assert_eq!(ThemePreset::parse("ayu"), Some(ThemePreset::Ayu));
    }

    #[test]
    fn new_presets_should_have_unique_colors() {
        let default = Theme::default();
        for preset in ThemePreset::ALL {
            if preset == ThemePreset::default() {
                continue;
            }
            let theme = preset.theme();
            assert_ne!(
                theme.black,
                default.black,
                "{} black is same as default",
                preset.name()
            );
        }
    }

    #[test]
    fn theme_file_should_use_preset_as_base() {
        let loaded = Theme::from_config(ThemeFile {
            preset: Some("gruvbox".to_string()),
            colors: None,
        });

        assert_eq!(loaded.preset, ThemePreset::Gruvbox);
        assert_eq!(loaded.theme.black, ThemePreset::Gruvbox.theme().black);
    }

    #[test]
    fn theme_file_colors_should_override_preset() {
        let loaded = Theme::from_config(ThemeFile {
            preset: Some("catppuccin".to_string()),
            colors: Some(ThemeColorsFile {
                hot_red: Some("#ff0000".to_string()),
                ..ThemeColorsFile::default()
            }),
        });

        assert_eq!(loaded.theme.hot_red, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn theme_status_should_not_expose_file_path() {
        let file_name = format!(
            "chaosgamemode-theme-{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(file_name);
        fs::write(&path, "preset = \"gruvbox\"\n").expect("theme fixture should be writable");

        let mut watcher = ThemeWatcher {
            path: Some(path.clone()),
            modified: None,
            last_check: Instant::now() - THEME_RELOAD_RATE,
            active_preset: ThemePreset::default(),
        };
        let mut theme = Theme::default();
        let status = watcher.load_theme(&mut theme);

        assert_eq!(status, "tema activo: Gruvbox");
        assert!(
            !status.contains(path.to_string_lossy().as_ref()),
            "theme status should not include local file paths"
        );

        fs::remove_file(path).expect("theme fixture should be removable");
    }
}

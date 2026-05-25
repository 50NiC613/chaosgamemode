use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use crate::i18n::Language;
use crate::theme::Theme;

// ── Re-usable block styles ──────────────────────────────────────────────

pub(super) fn danger_block<'a>(theme: &Theme, title: impl Into<Span<'a>>) -> Block<'a> {
    let title_span: Span<'a> = title.into();
    Block::default()
        .title(Span::styled(
            format!(" {} ", title_span.content.as_ref()),
            Style::new().fg(theme.hot_red).bold().italic(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(theme.hot_red))
        .style(Style::new().fg(theme.foreground).bg(theme.panel_dark))
}

pub(super) fn accent_block<'a>(
    theme: &Theme,
    title: impl Into<Span<'a>>,
    color: Color,
) -> Block<'a> {
    let title_span: Span<'a> = title.into();
    Block::default()
        .title(Span::styled(
            format!(" {} ", title_span.content.as_ref()),
            Style::new().fg(color).bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(color))
        .style(Style::new().fg(theme.foreground).bg(theme.panel_dark))
}

pub(super) fn modal_block<'a>(theme: &Theme, title: impl Into<Span<'a>>) -> Block<'a> {
    let title_span: Span<'a> = title.into();
    Block::default()
        .title(Span::styled(
            format!(" {} ", title_span.content.as_ref()),
            Style::new().fg(theme.cyber_yellow).bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(theme.cyber_yellow))
        .style(Style::new().fg(theme.foreground).bg(theme.black))
}

// ── Keycaps ─────────────────────────────────────────────────────────────

pub(super) fn keycap<'a>(theme: &Theme, key: &'static str) -> Span<'a> {
    Span::styled(
        format!(" {key} "),
        Style::new().fg(theme.black).bg(theme.cyber_yellow).bold(),
    )
}

// ── Metric label/value pairs ───────────────────────────────────────────

pub(super) fn metric_label<'a>(theme: &Theme, label: &'static str) -> Span<'a> {
    Span::styled(format!(" {label:<12} "), Style::new().fg(theme.muted))
}

pub(super) fn metric_value<'a>(value: impl Into<String>, color: Color) -> Span<'a> {
    Span::styled(value.into(), Style::new().fg(color).bold())
}

// ── Status badge ────────────────────────────────────────────────────────

pub(super) fn status_badge<'a>(label: &'static str, color: Color) -> Span<'a> {
    Span::styled(format!(" {label:<6} "), Style::new().fg(color).bold())
}

// ── Row style helpers ───────────────────────────────────────────────────

pub(super) fn selected_row_style(theme: &Theme) -> Style {
    Style::new()
        .fg(theme.cyber_yellow)
        .bg(theme.panel)
        .add_modifier(Modifier::BOLD)
}

// ── Centered rect helper ────────────────────────────────────────────────

pub(super) fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(percent_y),
            Constraint::Fill(1),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Percentage(percent_x),
            Constraint::Fill(1),
        ])
        .split(popup[1])[1]
}

// ── Legacy aliases (preserved for compatibility) ────────────────────────

pub(super) fn status_color(theme: &Theme, value: bool) -> Color {
    if value {
        theme.acid_green
    } else {
        theme.hot_red
    }
}

pub(super) fn metric_color(theme: &Theme, percent: u16) -> Color {
    match percent {
        0..=49 => theme.acid_green,
        50..=79 => theme.cyber_yellow,
        _ => theme.hot_red,
    }
}

pub(super) fn status_line(
    theme: &Theme,
    label: &'static str,
    value: &str,
    good: bool,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {label:<8} "), Style::new().fg(theme.muted)),
        Span::styled(
            value.to_string(),
            Style::new().fg(status_color(theme, good)).bold(),
        ),
    ])
}

pub(super) fn command_line(
    theme: &Theme,
    key: &'static str,
    title: &'static str,
    detail: &'static str,
) -> Line<'static> {
    Line::from(vec![
        keycap(theme, key),
        Span::styled(
            format!(" {title:<18} "),
            Style::new().fg(theme.neon_cyan).bold(),
        ),
        Span::styled(detail, Style::new().fg(theme.muted)),
    ])
}

pub(super) fn bar_line(theme: &Theme, percent: u16, width: usize, color: Color) -> Line<'static> {
    let filled = ((usize::from(percent) * width) / 100).min(width);
    let empty = width.saturating_sub(filled);
    Line::from(vec![
        Span::styled("  ", Style::new()),
        Span::styled("█".repeat(filled), Style::new().fg(color).bold()),
        Span::styled("░".repeat(empty), Style::new().fg(theme.muted)),
    ])
}

pub(super) fn panel_bar_width(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(4)).clamp(12, 52)
}

pub(super) fn localized_steam_status(status: &str, language: Language) -> String {
    if status == "scanning Steam libraries..." {
        return match language {
            Language::Spanish => "escaneando bibliotecas Steam...".to_string(),
            Language::English => "scanning Steam libraries...".to_string(),
        };
    }
    if status == "Steam no encontrado en rutas conocidas" {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => "Steam not found in known paths".to_string(),
        };
    }
    if let Some(count) = status.strip_suffix(" Steam games detected") {
        return match language {
            Language::Spanish => format!("{count} juegos Steam detectados"),
            Language::English => status.to_string(),
        };
    }
    status.to_string()
}

pub(super) fn localized_hardware_status(status: &str, language: Language) -> String {
    if status == "hardware sensors pending" {
        return match language {
            Language::Spanish => "hardware sensors pending".to_string(),
            Language::English => status.to_string(),
        };
    }
    if status == "hardware sensors unavailable" {
        return match language {
            Language::Spanish => "hardware sensors unavailable".to_string(),
            Language::English => status.to_string(),
        };
    }
    if status == "hardware sensors unavailable: powershell" {
        return match language {
            Language::Spanish => "hardware sensors unavailable: powershell".to_string(),
            Language::English => status.to_string(),
        };
    }
    if let Some(backend) = status.strip_prefix("hardware sensors: ") {
        return match language {
            Language::Spanish => format!("hardware sensors: {backend}"),
            Language::English => status.to_string(),
        };
    }
    status.to_string()
}

pub(super) fn localized_frame_status(status: &str, language: Language) -> String {
    if status == "RTSS waiting for Steam game" {
        return match language {
            Language::Spanish => "RTSS esperando juego de Steam".to_string(),
            Language::English => status.to_string(),
        };
    }
    if let Some(process) = status.strip_prefix("RTSS starting ") {
        return match language {
            Language::Spanish => format!("RTSS iniciando captura: {process}"),
            Language::English => status.to_string(),
        };
    }
    if let Some(game) = status.strip_prefix("RTSS resolving ") {
        return match language {
            Language::Spanish => format!("RTSS detectando proceso: {game}"),
            Language::English => format!("RTSS resolving process: {game}"),
        };
    }
    if let Some(process) = status.strip_prefix("RTSS probing ") {
        return match language {
            Language::Spanish => format!("RTSS probando {process}"),
            Language::English => format!("RTSS probing {process}"),
        };
    }
    if let Some(process) = status.strip_prefix("RTSS tracking ") {
        return match language {
            Language::Spanish => format!("RTSS capturando {process}"),
            Language::English => status.to_string(),
        };
    }
    if let Some(process) = status.strip_prefix("RTSS waiting fresh frames for ") {
        return match language {
            Language::Spanish => format!("RTSS esperando frames nuevos de {process}"),
            Language::English => status.to_string(),
        };
    }
    if let Some(process) = status.strip_prefix("RTSS waiting for ") {
        return match language {
            Language::Spanish => format!("RTSS esperando {process}"),
            Language::English => status.to_string(),
        };
    }
    if status == "RTSS waiting for hooked game frames" {
        return match language {
            Language::Spanish => "RTSS esperando frames del juego".to_string(),
            Language::English => status.to_string(),
        };
    }
    if status == "RTSS did not expose an active game target" {
        return match language {
            Language::Spanish => "RTSS no expuso un proceso de juego activo".to_string(),
            Language::English => status.to_string(),
        };
    }
    status.to_string()
}

pub(super) fn localized_frame_probe_status(status: &str, language: Language) -> String {
    match status {
        "RTSS listo" => match language {
            Language::Spanish => "RTSS listo".to_string(),
            Language::English => "RTSS ready".to_string(),
        },
        "RTSS not running; start RivaTuner Statistics Server" => match language {
            Language::Spanish => {
                "RTSS no esta abierto; inicia RivaTuner Statistics Server".to_string()
            }
            Language::English => status.to_string(),
        },
        "RTSS shared memory is not initialized" => match language {
            Language::Spanish => "RTSS shared memory no esta inicializada".to_string(),
            Language::English => status.to_string(),
        },
        "RTSS frame capture is only available on Windows" => match language {
            Language::Spanish => "captura RTSS solo disponible en Windows".to_string(),
            Language::English => status.to_string(),
        },
        _ => status.to_string(),
    }
}

pub(super) fn localized_config_status(status: &str, language: Language) -> String {
    if status == "config.toml no encontrado; usando defaults" {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => "config.toml not found; using defaults".to_string(),
        };
    }
    if status == "config defaults" {
        return match language {
            Language::Spanish => "defaults de config".to_string(),
            Language::English => status.to_string(),
        };
    }
    if let Some(path) = status.strip_prefix("config loaded: ") {
        return match language {
            Language::Spanish => format!("config cargada: {path}"),
            Language::English => status.to_string(),
        };
    }
    if status == "process config: nombre vacio" {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => "process config: empty name".to_string(),
        };
    }
    if let Some(err) = status.strip_prefix("config save error: ") {
        return match language {
            Language::Spanish => format!("error guardando config: {err}"),
            Language::English => status.to_string(),
        };
    }
    if let Some(state) = status.strip_prefix("overlay runtime: ") {
        return match (language, state) {
            (Language::Spanish, "enabled") => "overlay runtime: activo".to_string(),
            (Language::Spanish, "disabled") => "overlay runtime: apagado".to_string(),
            _ => status.to_string(),
        };
    }
    if let Some(layout) = status.strip_prefix("overlay hud: layout ") {
        return match language {
            Language::Spanish => format!("HUD overlay: preset {layout}"),
            Language::English => status.to_string(),
        };
    }
    if let Some(rest) = status.strip_prefix("overlay hud: ") {
        return match language {
            Language::Spanish => format!("HUD overlay: {rest}"),
            Language::English => status.to_string(),
        };
    }
    status.to_string()
}

pub(super) fn localized_theme_status(status: &str, language: Language) -> String {
    if status == "tema interno activo; theme.toml no encontrado" {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => "internal theme active; theme.toml not found".to_string(),
        };
    }
    if let Some(preset) = status
        .strip_prefix("tema activo: ")
        .and_then(|value| value.strip_suffix(" en memoria; theme.toml no encontrado"))
    {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => {
                format!("active theme: {preset} in memory; theme.toml not found")
            }
        };
    }
    if let Some(preset) = status
        .strip_prefix("tema activo: ")
        .and_then(|value| value.strip_suffix(" guardado"))
    {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => format!("active theme: {preset} saved"),
        };
    }
    if let Some(preset) = status.strip_prefix("tema activo: ") {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => format!("active theme: {preset}"),
        };
    }
    if let Some(err) = status.strip_prefix("error de tema: ") {
        return match language {
            Language::Spanish => status.to_string(),
            Language::English => format!("theme error: {err}"),
        };
    }
    status.to_string()
}

pub(super) fn localized_session_source(source: &str, language: Language) -> &'static str {
    match (language, source) {
        (Language::Spanish, "auto-detected") => "auto-detectado",
        (Language::English, "auto-detected") => "auto-detected",
        (_, "manual") => "manual",
        _ => "unknown",
    }
}

pub(super) fn localized_source_value(source: &'static str, language: Language) -> &'static str {
    match (language, source) {
        (Language::Spanish, "none") => "ninguno",
        (Language::English, "none") => "none",
        (Language::Spanish, "bundled") => "incluido",
        (Language::English, "bundled") => "bundled",
        (Language::Spanish, "rtss") => "RTSS shared memory",
        (Language::English, "rtss") => "RTSS shared memory",
        _ => source,
    }
}

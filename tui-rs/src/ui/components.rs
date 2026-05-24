use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

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

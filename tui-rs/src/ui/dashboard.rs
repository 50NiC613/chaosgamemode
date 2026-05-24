use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Paragraph, Sparkline, Wrap},
};

use super::components::*;
use crate::app::App;
use crate::metrics::{
    percent, percent_from_f32, readiness_score, scaled_history, sorted_processes, truncate,
};
use crate::system::ProcessGroup;
use crate::theme::Theme;

pub(super) fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        render_wide_dashboard(frame, app, area);
    } else {
        render_compact_dashboard(frame, app, area);
    }
}

fn render_wide_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(37),
            Constraint::Percentage(31),
            Constraint::Percentage(32),
        ])
        .split(area);

    render_metric_stack(frame, app, columns[0]);
    render_history_panel(frame, app, columns[1]);
    render_process_heat_panel(frame, app, columns[2]);
}

fn render_compact_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_metric_stack(frame, app, columns[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(columns[1]);
    render_history_panel(frame, app, right[0]);
    render_process_heat_panel(frame, app, right[1]);
}

fn render_metric_stack(frame: &mut Frame, app: &App, area: Rect) {
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(4),
        ])
        .split(area);

    render_cpu_gauge(frame, app, left[0]);
    render_ram_gauge(frame, app, left[1]);
    render_gpu_panel(frame, app, left[2]);
    render_readiness_panel(frame, app, left[3]);
    render_status_panel(frame, app, left[4]);
}

fn render_cpu_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let pct = percent_from_f32(app.state.cpu_usage);
    let color = metric_color(theme, pct);
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "LOAD"),
            metric_value(format!("{pct:>3}%"), color),
            Span::styled("  CORES ", Style::new().fg(theme.muted)),
            metric_value(app.state.cpu_cores.to_string(), theme.blue),
        ]),
        bar_line(theme, pct, metric_bar_width(area), color),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(accent_block(theme, "CPU", theme.blue)),
        area,
    );
}

fn render_ram_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let pct = app.state.ram_used_pct();
    let color = metric_color(theme, pct);
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "USED"),
            metric_value(
                format!(
                    "{:.1}/{:.1} GB",
                    app.state.ram_used_gb, app.state.ram_total_gb
                ),
                theme.cyber_yellow,
            ),
            Span::styled("  FREE ", Style::new().fg(theme.muted)),
            metric_value(format!("{}%", app.state.ram_free_pct()), theme.acid_green),
        ]),
        bar_line(theme, pct, metric_bar_width(area), color),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(accent_block(theme, "RAM", theme.neon_magenta)),
        area,
    );
}

fn render_gpu_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let hardware = &app.state.hardware;
    let pct = hardware.gpu_load_pct.unwrap_or(0);
    let color = if hardware.gpu_load_pct.is_some() {
        metric_color(theme, pct)
    } else {
        theme.muted
    };
    let vram_pct = hardware.gpu_vram_used_pct().unwrap_or(0);
    let vram_color = if hardware.gpu_vram_used_pct().is_some() {
        metric_color(theme, vram_pct)
    } else {
        theme.muted
    };

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "LOAD"),
            metric_value(format_optional_pct(hardware.gpu_load_pct), color),
            Span::styled("  VRAM ", Style::new().fg(theme.muted)),
            metric_value(format_vram(hardware), vram_color),
        ]),
        Line::from(vec![
            metric_label(theme, "TEMP"),
            metric_value(
                format_temp(hardware.gpu_temp_c),
                temp_color(theme, hardware.gpu_temp_c),
            ),
            Span::styled("  CPU ", Style::new().fg(theme.muted)),
            metric_value(
                format_temp(hardware.cpu_temp_c),
                temp_color(theme, hardware.cpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "FPS"),
            metric_value(format_fps(app.frame_metrics.fps), theme.cyber_yellow),
            Span::styled("  1%L ", Style::new().fg(theme.muted)),
            metric_value(format_fps(app.frame_metrics.low_1_fps), theme.hot_red),
        ]),
        bar_line(theme, pct, metric_bar_width(area), color),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(accent_block(theme, "GPU", theme.orange)),
        area,
    );
}

fn render_status_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let state = &app.state;
    let lines = vec![
        status_line(
            theme,
            "POWER",
            &state.power_plan,
            state.power_plan == "Alto Rendimiento",
        ),
        status_line(
            theme,
            "DESKTOP",
            if state.explorer_on {
                "explorer active"
            } else {
                "minimal shell"
            },
            !state.explorer_on,
        ),
        status_line(
            theme,
            "STEAM",
            if state.steam_on { "online" } else { "offline" },
            state.steam_on,
        ),
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            metric_value(
                format!(
                    "{}/{} running",
                    state.services_running,
                    app.config.active_profile().services.len()
                ),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "BLOAT"),
            metric_value(
                format!("{:.0} MB", state.total_waste_mb),
                metric_color(theme, percent(state.total_waste_mb, 4_000.0)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SENSORS"),
            metric_value(
                crate::metrics::truncate(&state.hardware.status, 28),
                if state.hardware.gpu_load_pct.is_some() {
                    theme.neon_cyan
                } else {
                    theme.muted
                },
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "FRAMES"),
            metric_value(
                truncate(&app.frame_metrics.status, 28),
                if app.frame_metrics.fps.is_some() {
                    theme.acid_green
                } else {
                    theme.muted
                },
            ),
        ]),
    ];
    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "SYSTEM", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_readiness_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let score = readiness_score(&app.state);
    let status = if score >= 80 {
        "SYSTEM READY"
    } else {
        "NEEDS CLEANUP"
    };
    let color = if score >= 80 {
        theme.acid_green
    } else {
        theme.hot_red
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("     ", Style::new()),
            Span::styled(status, Style::new().fg(color).bold()),
        ]),
        Line::from(vec![
            Span::styled("     ", Style::new()),
            metric_value(format!("{score:03}/100"), theme.cyber_yellow),
        ]),
        bar_line(theme, score, metric_bar_width(area), color),
        Line::from(""),
        Line::from(vec![
            keycap(theme, "SPACE"),
            Span::styled(" preview overdrive", Style::new().fg(theme.muted)),
        ]),
    ];
    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "READINESS", color))
        .alignment(Alignment::Left);
    frame.render_widget(panel, area);
}

fn render_history_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let fps_history = scaled_history(&app.fps_history);
    let gpu_history = scaled_history(&app.gpu_history);
    let ram_history = scaled_history(&app.ram_history);
    let waste_history = scaled_history(&app.waste_history);

    // Use actual data max for CPU sparkline instead of fixed 100
    let cpu_max = app.cpu_history.iter().max().copied().unwrap_or(100).max(1);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    frame.render_widget(
        Sparkline::default()
            .block(accent_block(theme, "CPU TRACE", theme.blue))
            .data(app.cpu_history.iter().copied())
            .max(cpu_max)
            .style(Style::new().fg(theme.blue).bg(theme.panel)),
        rows[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!("FPS TRACE / {}-{}", fps_history.min, fps_history.max),
                theme.cyber_yellow,
            ))
            .data(fps_history.values.iter().copied())
            .max(240)
            .style(Style::new().fg(theme.cyber_yellow).bg(theme.panel)),
        rows[1],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!("GPU TRACE / {}-{}%", gpu_history.min, gpu_history.max),
                theme.orange,
            ))
            .data(gpu_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.orange).bg(theme.panel)),
        rows[2],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!("RAM TRACE / {}-{}%", ram_history.min, ram_history.max),
                theme.neon_magenta,
            ))
            .data(ram_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.neon_magenta).bg(theme.panel)),
        rows[3],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!(
                    "BLOAT TRACE / {}-{}MB",
                    waste_history.min, waste_history.max
                ),
                theme.hot_red,
            ))
            .data(waste_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.hot_red).bg(theme.panel)),
        rows[4],
    );
}

fn format_optional_pct(value: Option<u16>) -> String {
    value.map_or_else(|| "--%".to_string(), |value| format!("{value:>3}%"))
}

fn format_fps(value: Option<f64>) -> String {
    value.map_or_else(
        || "--".to_string(),
        |value| format!("{:>3}", value.round().clamp(0.0, 999.0) as u16),
    )
}

fn format_temp(value: Option<f32>) -> String {
    value.map_or_else(|| "--C".to_string(), |value| format!("{value:.0}C"))
}

fn format_vram(hardware: &crate::hardware::HardwareState) -> String {
    match (hardware.gpu_vram_used_mb, hardware.gpu_vram_total_mb) {
        (Some(used), Some(total)) if total > 0.0 => {
            format!("{:.1}/{:.1} GB", used / 1024.0, total / 1024.0)
        }
        (Some(used), _) => format!("{:.1} GB", used / 1024.0),
        _ => "--".to_string(),
    }
}

fn temp_color(theme: &crate::theme::Theme, value: Option<f32>) -> ratatui::style::Color {
    match value {
        Some(value) if value >= 85.0 => theme.hot_red,
        Some(value) if value >= 72.0 => theme.cyber_yellow,
        Some(_) => theme.acid_green,
        None => theme.muted,
    }
}

fn render_process_heat_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines = Vec::new();
    let sorted = sorted_processes(&app.state);
    let total = app.state.total_waste_mb.max(1.0);
    let max_rows = usize::from(area.height.saturating_sub(4)).max(1);
    let bar_width = process_heat_bar_width(area);

    if sorted.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  \u{f00c} ", Style::new().fg(theme.acid_green).bold()),
            Span::styled("no residual targets  ", Style::new().fg(theme.muted)),
            Span::styled("\u{f0e7}", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(" preview overdrive", Style::new().fg(theme.muted)),
        ]));
    } else {
        // Table-like header with % column
        lines.push(Line::from(vec![
            Span::styled("  NAME                    ", Style::new().fg(theme.muted)),
            Span::styled("   MEMORY   #    %  HEAT", Style::new().fg(theme.muted)),
        ]));
        for (name, group) in sorted.iter().take(max_rows) {
            let pct_of_total = ((group.memory_mb / total) * 100.0).clamp(0.0, 100.0);
            lines.push(process_line_with_pct(
                theme,
                name,
                group,
                app.state.total_waste_mb,
                pct_of_total,
                bar_width,
            ));
        }
        if sorted.len() > max_rows {
            lines.push(Line::from(vec![
                Span::styled("  ... ", Style::new().fg(theme.muted)),
                metric_value(
                    format!("{} more entries  ", sorted.len() - max_rows),
                    theme.muted,
                ),
                Span::styled("\u{f0e7}", Style::new().fg(theme.cyber_yellow).bold()),
                Span::styled(" full preview", Style::new().fg(theme.muted)),
            ]));
        }
    }

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "PROCESS HEATMAP", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn metric_bar_width(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(4)).clamp(12, 52)
}

fn process_heat_bar_width(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(47)).clamp(8, 72)
}

fn process_line_with_pct<'a>(
    theme: &Theme,
    name: &'a str,
    group: &ProcessGroup,
    _total_waste_mb: f64,
    pct_of_total: f64,
    max_bar: usize,
) -> Line<'a> {
    let display_name = truncate(name, 24);
    let bar_len = ((pct_of_total / 100.0) * max_bar as f64).ceil() as usize;
    let bar = "█".repeat(bar_len.max(1));
    let heat = metric_color(theme, crate::metrics::percent(group.memory_mb, 1_500.0));

    Line::from(vec![
        Span::styled(
            format!("  {display_name:<24} "),
            Style::new().fg(theme.muted),
        ),
        Span::styled(
            format!("{:>7.0} MB", group.memory_mb),
            Style::new().fg(heat).bold(),
        ),
        Span::styled(
            format!(" {:>2}x", group.count),
            Style::new().fg(theme.cyber_yellow),
        ),
        Span::styled(format!(" {:>4.0}%", pct_of_total), Style::new().fg(heat)),
        Span::styled(bar, Style::new().fg(heat)),
    ])
}

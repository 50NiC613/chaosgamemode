use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph, Wrap},
};

use super::components::*;
use crate::app::App;
use crate::config::{BoostProfile, process_pattern_from_name};
use crate::metrics::readiness_score;
use crate::system::ProcessGroup;

pub(super) fn render_processes(frame: &mut Frame, app: &App, area: Rect) {
    let sorted = app.visible_processes();

    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(44),
                Constraint::Percentage(36),
                Constraint::Percentage(20),
            ])
            .split(area);

        render_process_list(frame, app, columns[0], &sorted);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(8)])
            .split(columns[1]);
        render_process_detail(frame, app, sorted.get(app.process_selected), middle[0]);
        render_process_policy_panel(frame, app, sorted.get(app.process_selected), middle[1]);
        render_process_summary(frame, app, &sorted, columns[2]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(area);

        render_process_list(frame, app, columns[0], &sorted);
        render_process_detail(frame, app, sorted.get(app.process_selected), columns[1]);
    }
}

fn render_process_list(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    sorted: &[(&String, &ProcessGroup)],
) {
    let theme = &app.theme;
    let visible_rows = area.height.saturating_sub(2).max(1) as usize;
    let start = app
        .process_selected
        .saturating_sub(visible_rows.saturating_sub(1) / 2);
    let end = (start + visible_rows).min(sorted.len());

    let empty_text = if !app.process_filter.is_empty() {
        "No hay procesos que coincidan con el filtro"
    } else if app.show_hidden_processes {
        "No hay procesos ocultos detectados"
    } else {
        "No hay procesos accionables detectados"
    };

    let items: Vec<ListItem> = if sorted.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  \u{f00c} ", Style::new().fg(theme.acid_green).bold()),
            Span::styled(empty_text, Style::new().fg(theme.muted)),
            Span::styled("  ", Style::new()),
            Span::styled("\u{f06e}", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(" hidden view", Style::new().fg(theme.muted)),
        ]))]
    } else {
        sorted[start..end]
            .iter()
            .enumerate()
            .map(|(offset, (name, group))| {
                let index = start + offset;
                let selected = index == app.process_selected;
                let profile = app.config.active_profile();
                let (label, color) = process_status(profile, name, theme);
                let marker = if selected { "\u{f0da}" } else { " " };
                let name_style = if selected {
                    selected_row_style(theme)
                } else {
                    Style::new().fg(theme.foreground)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {marker} "), Style::new().fg(theme.orange).bold()),
                    status_badge(label, color),
                    Span::styled(
                        format!("{:<26}", crate::metrics::truncate(name, 25)),
                        name_style,
                    ),
                    Span::styled(
                        format!("{:>7.0} MB", group.memory_mb),
                        Style::new()
                            .fg(metric_color(theme, percent_from_memory(group)))
                            .bold(),
                    ),
                    Span::styled(
                        format!(" {:>2}x", group.count),
                        Style::new().fg(theme.cyber_yellow),
                    ),
                ]))
            })
            .collect()
    };

    let base_title = if app.show_hidden_processes {
        "HIDDEN BIN"
    } else {
        "PROCESSES"
    };
    let title = process_list_title(app, base_title, sorted.len());
    let list = List::new(items).block(accent_block(theme, title, theme.neon_magenta));
    frame.render_widget(list, area);
}

fn render_process_detail(
    frame: &mut Frame,
    app: &App,
    selected: Option<&(&String, &ProcessGroup)>,
    area: Rect,
) {
    let theme = &app.theme;
    let profile = app.config.active_profile();
    let value_width = panel_value_width(area, 72);
    let lines = if let Some((name, group)) = selected {
        let (label, color) = process_status(profile, name, theme);
        let pattern = process_pattern_from_name(name);
        vec![
            Line::from(vec![
                metric_label(theme, "NAME"),
                metric_value(
                    crate::metrics::truncate(name, value_width),
                    theme.cyber_yellow,
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "PATTERN"),
                Span::styled(
                    crate::metrics::truncate(&pattern, value_width),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "STATUS"),
                status_badge(label, color),
            ]),
            Line::from(vec![
                metric_label(theme, "EXE"),
                Span::styled(
                    group
                        .exe_path
                        .as_deref()
                        .map(|path| crate::metrics::truncate(path, value_width))
                        .unwrap_or_else(|| "unavailable".to_string()),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "VIEW"),
                Span::styled(
                    if app.show_hidden_processes {
                        "hidden"
                    } else {
                        "actionable"
                    },
                    Style::new().fg(theme.neon_cyan),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "MEMORY"),
                metric_value(
                    format!("{:.0} MB / {} instances", group.memory_mb, group.count),
                    metric_color(theme, percent_from_memory(group)),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "CONFIG"),
                Span::styled(
                    crate::metrics::truncate(&app.config.status, value_width),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(""),
            process_commands(theme, app.show_hidden_processes),
        ]
    } else {
        vec![
            Line::from("  No process selected"),
            Line::from(""),
            process_commands(theme, app.show_hidden_processes),
        ]
    };

    let detail = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "DETAIL", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, area);
}

fn render_process_policy_panel(
    frame: &mut Frame,
    app: &App,
    selected: Option<&(&String, &ProcessGroup)>,
    area: Rect,
) {
    let theme = &app.theme;
    let profile = app.config.active_profile();
    let value_width = panel_value_width(area, 64);
    let selected_pattern = selected
        .map(|(name, _)| process_pattern_from_name(name))
        .unwrap_or_else(|| "none".to_string());
    let selected_memory = selected
        .map(|(_, group)| format!("{:.0} MB", group.memory_mb))
        .unwrap_or_else(|| "--".to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "PROFILE"),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "TARGETS"),
            metric_value(profile.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, "KEEP"),
            metric_value(
                profile.protected_processes.len().to_string(),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "HIDDEN"),
            metric_value(
                profile.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SELECTED"),
            metric_value(
                crate::metrics::truncate(&selected_pattern, value_width),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "MEMORY"),
            metric_value(selected_memory, theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "FILTER"),
            Span::styled(
                if app.process_filter.is_empty() {
                    "none".to_string()
                } else {
                    crate::metrics::truncate(&app.process_filter, value_width)
                },
                Style::new().fg(if app.process_filter.is_empty() {
                    theme.muted
                } else {
                    theme.foreground
                }),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "CONFIG"),
            Span::styled(
                crate::metrics::truncate(&app.config.status, value_width),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(""),
        process_commands(theme, app.show_hidden_processes),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "POLICY", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_process_summary(
    frame: &mut Frame,
    app: &App,
    sorted: &[(&String, &ProcessGroup)],
    area: Rect,
) {
    let theme = &app.theme;
    let counts = process_status_counts(app, sorted);
    let visible_memory = sorted.iter().map(|(_, group)| group.memory_mb).sum::<f64>();
    let actionable = app.state.observed_processes.len();
    let hidden = app.state.hidden_processes.len();
    let total_groups = actionable + hidden;
    let rows = area.height.saturating_sub(16).max(3) as usize;

    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, "VISIBLE"),
            metric_value(
                format!("{}/{}", sorted.len(), app.visible_process_total()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "MEMORY"),
            metric_value(format!("{visible_memory:.0} MB"), theme.orange),
        ]),
        Line::from(vec![
            metric_label(theme, "TARGET"),
            metric_value(counts.target.to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, "KEEP"),
            metric_value(counts.keep.to_string(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, "WATCH"),
            metric_value(counts.watch.to_string(), theme.muted),
        ]),
        Line::from(vec![
            metric_label(theme, "HIDDEN"),
            metric_value(counts.hidden.to_string(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, "TOTAL"),
            metric_value(format!("{total_groups} groups"), theme.neon_cyan),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " TOP MEMORY",
            Style::new().fg(theme.muted).bold(),
        )]),
    ];

    for (name, group) in sorted.iter().take(rows) {
        let pct = percent_from_memory(group);
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<18}", crate::metrics::truncate(name, 17)),
                Style::new().fg(theme.foreground),
            ),
            Span::styled(
                format!("{:>6.0} MB", group.memory_mb),
                Style::new().fg(metric_color(theme, pct)).bold(),
            ),
        ]));
    }

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "PROCESS MAP", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn process_list_title(app: &App, base_title: &str, visible_count: usize) -> String {
    if app.process_filter.is_empty() {
        return format!(
            "{base_title} {visible_count}/{}",
            app.visible_process_total()
        );
    }

    let marker = if app.editing_process_filter {
        " EDIT"
    } else {
        ""
    };
    format!(
        "{base_title} {visible_count}/{} /{}{}",
        app.visible_process_total(),
        crate::metrics::truncate(&app.process_filter, 18),
        marker
    )
}

fn process_status<'a>(
    profile: &BoostProfile,
    process_name: &str,
    theme: &crate::theme::Theme,
) -> (&'a str, ratatui::style::Color) {
    if profile.is_hidden_process(process_name) {
        ("HIDDEN", theme.neon_magenta)
    } else if profile.is_protected_process(process_name) {
        ("KEEP", theme.acid_green)
    } else if profile.is_target_process(process_name) {
        ("TARGET", theme.hot_red)
    } else {
        ("WATCH", theme.muted)
    }
}

fn percent_from_memory(group: &ProcessGroup) -> u16 {
    crate::metrics::percent(group.memory_mb, 1_500.0)
}

fn panel_value_width(area: Rect, max_width: usize) -> usize {
    usize::from(area.width.saturating_sub(18)).clamp(24, max_width)
}

#[derive(Default)]
struct ProcessStatusCounts {
    target: usize,
    keep: usize,
    watch: usize,
    hidden: usize,
}

fn process_status_counts(app: &App, processes: &[(&String, &ProcessGroup)]) -> ProcessStatusCounts {
    let profile = app.config.active_profile();
    let mut counts = ProcessStatusCounts::default();

    for (name, _) in processes {
        if profile.is_hidden_process(name) {
            counts.hidden += 1;
        } else if profile.is_protected_process(name) {
            counts.keep += 1;
        } else if profile.is_target_process(name) {
            counts.target += 1;
        } else {
            counts.watch += 1;
        }
    }

    counts
}

fn process_commands(theme: &crate::theme::Theme, hidden_view: bool) -> Line<'static> {
    if hidden_view {
        return Line::from(vec![
            keycap(theme, "H"),
            Span::styled(" unhide ", Style::new().fg(theme.neon_cyan).bold()),
            keycap(theme, "V"),
            Span::styled(" active ", Style::new().fg(theme.cyber_yellow).bold()),
            keycap(theme, "/"),
            Span::styled(" filter", Style::new().fg(theme.foreground).bold()),
        ]);
    }

    Line::from(vec![
        keycap(theme, "P"),
        Span::styled(" keep ", Style::new().fg(theme.neon_cyan).bold()),
        keycap(theme, "T"),
        Span::styled(" target ", Style::new().fg(theme.hot_red).bold()),
        keycap(theme, "N"),
        Span::styled(" neutral ", Style::new().fg(theme.muted)),
        keycap(theme, "H"),
        Span::styled(" hide ", Style::new().fg(theme.neon_magenta).bold()),
        keycap(theme, "V"),
        Span::styled(" hidden ", Style::new().fg(theme.cyber_yellow).bold()),
        keycap(theme, "/"),
        Span::styled(" filter", Style::new().fg(theme.foreground).bold()),
    ])
}

pub(super) fn render_boost(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(38),
                Constraint::Percentage(31),
                Constraint::Percentage(31),
            ])
            .split(area);

        render_boost_actions(frame, app, columns[0]);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(10)])
            .split(columns[1]);
        render_boost_profile(frame, app, middle[0]);
        render_boost_payload(frame, app, middle[1]);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(12),
                Constraint::Min(8),
            ])
            .split(columns[2]);
        render_boost_readiness(frame, app, right[0]);
        render_boost_live_status(frame, app, right[1]);
        render_boost_restore_panel(frame, app, right[2]);
        return;
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    render_boost_actions(frame, app, columns[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(columns[1]);
    render_boost_readiness(frame, app, right[0]);
    render_boost_live_status(frame, app, right[1]);
}

fn render_boost_actions(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let actions = Paragraph::new(Text::from(vec![
        command_line(theme, "1", "Preview Overdrive", "confirm before changes"),
        command_line(
            theme,
            "2",
            "Restore System",
            "restart shell, services, balanced power",
        ),
        command_line(
            theme,
            "R",
            "Refresh Telemetry",
            "pull a fresh system snapshot",
        ),
        command_line(
            theme,
            "TAB",
            "Switch Deck",
            "dashboard, steam, processes, overdrive, system, history",
        ),
        command_line(
            theme,
            "M",
            "Cycle Theme",
            "cyberpunk, gruvbox, tokyo night, catppuccin",
        ),
        command_line(theme, "Q", "Exit", "leave terminal cleanly"),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, "PROFILE"),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "PAYLOAD"),
            metric_value(
                format!("{:.0} MB removable heat", app.state.total_waste_mb),
                theme.hot_red,
            ),
        ]),
    ]))
    .block(accent_block(theme, "OVERDRIVE CONSOLE", theme.cyber_yellow))
    .wrap(Wrap { trim: true });
    frame.render_widget(actions, area);
}

fn render_boost_readiness(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let score = readiness_score(&app.state);
    let readiness = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("  READY ", Style::new().fg(theme.muted)),
            Span::styled(
                format!("{score}%"),
                Style::new().fg(theme.acid_green).bold(),
            ),
        ]),
        bar_line(theme, score, panel_bar_width(area), theme.acid_green),
    ]))
    .block(accent_block(theme, "READINESS", theme.acid_green));
    frame.render_widget(readiness, area);
}

fn render_boost_live_status(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let log = Paragraph::new(Text::from(vec![
        Line::from(vec![
            metric_label(theme, "PROFILE"),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "POWER PLAN"),
            Span::styled(
                app.state.power_plan.as_str(),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            Span::styled(
                if app.state.steam_on {
                    "linked"
                } else {
                    "not linked"
                },
                Style::new().fg(status_color(theme, app.state.steam_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SHELL"),
            Span::styled(
                if app.state.explorer_on {
                    "desktop active"
                } else {
                    "minimal shell"
                },
                Style::new().fg(status_color(theme, !app.state.explorer_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            Span::styled(
                format!(
                    "{}/{}",
                    app.state.services_running,
                    app.config.active_profile().services.len()
                ),
                Style::new().fg(theme.neon_cyan),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "TARGETS"),
            metric_value(
                format!("{} active groups", app.state.processes.len()),
                theme.hot_red,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "WASTE"),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.orange),
        ]),
    ]))
    .block(accent_block(theme, "LIVE STATUS", theme.neon_cyan));
    frame.render_widget(log, area);
}

fn render_boost_profile(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let profile = app.config.active_profile();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "NAME"),
            metric_value(profile.name, theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "MODE"),
            metric_value(app.config.active_profile_name(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, "TARGETS"),
            metric_value(profile.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, "PROTECTED"),
            metric_value(
                profile.protected_processes.len().to_string(),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "HIDDEN"),
            metric_value(
                profile.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            metric_value(profile.services.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "POWER"),
            metric_value(
                if profile.set_high_performance {
                    "high performance"
                } else {
                    "unchanged"
                },
                if profile.set_high_performance {
                    theme.acid_green
                } else {
                    theme.muted
                },
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "EXPLORER"),
            metric_value(
                if profile.kill_explorer {
                    "stop on OD"
                } else {
                    "keep running"
                },
                if profile.kill_explorer {
                    theme.hot_red
                } else {
                    theme.acid_green
                },
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "PROFILE PLAN", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_boost_payload(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let sorted = crate::metrics::sorted_processes(&app.state);
    let rows = area.height.saturating_sub(8).max(3) as usize;
    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, "REMOVABLE"),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, "GROUPS"),
            metric_value(sorted.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            metric_value(
                format!(
                    "{}/{} running",
                    app.state.services_running,
                    app.config.active_profile().services.len()
                ),
                theme.neon_cyan,
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " CURRENT TARGETS",
            Style::new().fg(theme.muted).bold(),
        )]),
    ];

    if sorted.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  clean: ", Style::new().fg(theme.acid_green).bold()),
            Span::styled(
                "no configured targets detected",
                Style::new().fg(theme.muted),
            ),
        ]));
    } else {
        for (name, group) in sorted.iter().take(rows) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<20}", crate::metrics::truncate(name, 19)),
                    Style::new().fg(theme.foreground),
                ),
                Span::styled(
                    format!("{:>6.0} MB", group.memory_mb),
                    Style::new()
                        .fg(metric_color(theme, percent_from_memory(group)))
                        .bold(),
                ),
                Span::styled(
                    format!(" {:>2}x", group.count),
                    Style::new().fg(theme.muted),
                ),
            ]));
        }
    }

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "PAYLOAD PREVIEW", theme.hot_red))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_boost_restore_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let profile = app.config.active_profile();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "RESTORE"),
            metric_value("ready", theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, "POWER"),
            Span::styled("Balanceado", Style::new().fg(theme.cyber_yellow)),
        ]),
        Line::from(vec![
            metric_label(theme, "EXPLORER"),
            Span::styled("restart if closed", Style::new().fg(theme.neon_cyan)),
        ]),
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            metric_value(
                format!("{} configured", profile.services.len()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "NOTE"),
            Span::styled("closed apps stay closed", Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        command_line(theme, "2", "Restore System", "undo overdrive changes"),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "RESTORE PLAN", theme.neon_magenta))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

pub(super) fn render_system(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(34),
                Constraint::Percentage(36),
            ])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(9)])
            .split(columns[0]);
        render_system_telemetry_panel(frame, app, left[0]);
        render_system_runtime_panel(frame, app, left[1]);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(11), Constraint::Min(9)])
            .split(columns[1]);
        render_system_windows_panel(frame, app, middle[0]);
        render_system_services_panel(frame, app, middle[1]);

        render_system_graphics_panel(frame, app, columns[2]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(8)])
            .split(columns[0]);
        render_system_telemetry_panel(frame, app, left[0]);
        render_system_windows_panel(frame, app, left[1]);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(columns[1]);
        render_system_graphics_panel(frame, app, right[0]);
        render_system_services_panel(frame, app, right[1]);
    }
}

fn render_system_telemetry_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let cpu_pct = app.state.cpu_usage.clamp(0.0, 100.0).round() as u16;
    let ram_pct = app.state.ram_used_pct();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "CPU USAGE"),
            metric_value(format!("{cpu_pct}%"), metric_color(theme, cpu_pct)),
            Span::styled(" CORES ", Style::new().fg(theme.muted)),
            metric_value(app.state.cpu_cores.to_string(), theme.cyber_yellow),
        ]),
        bar_line(
            theme,
            cpu_pct,
            panel_bar_width(area),
            metric_color(theme, cpu_pct),
        ),
        Line::from(vec![
            metric_label(theme, "RAM USED"),
            metric_value(
                format!(
                    "{:.1}/{:.1} GB",
                    app.state.ram_used_gb, app.state.ram_total_gb
                ),
                metric_color(theme, ram_pct),
            ),
        ]),
        bar_line(
            theme,
            ram_pct,
            panel_bar_width(area),
            metric_color(theme, ram_pct),
        ),
        Line::from(vec![
            metric_label(theme, "RAM FREE"),
            metric_value(format!("{:.1} GB", app.state.ram_free_gb), theme.acid_green),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "TELEMETRY", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_windows_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "POWER"),
            Span::styled(
                app.state.power_plan.as_str(),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "EXPLORER"),
            Span::styled(
                if app.state.explorer_on {
                    "running"
                } else {
                    "stopped"
                },
                Style::new().fg(status_color(theme, !app.state.explorer_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            Span::styled(
                if app.state.steam_on {
                    "running"
                } else {
                    "closed"
                },
                Style::new().fg(status_color(theme, app.state.steam_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM RAM"),
            metric_value(format!("{:.0} MB", app.state.steam_mb), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME"),
            metric_value(app.theme_preset.label(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, "PROFILE"),
            metric_value(app.config.active_profile_name(), theme.acid_green),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "WINDOWS", theme.neon_magenta))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_services_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let profile = app.config.active_profile();
    let service_pct = crate::metrics::percent(
        app.state.services_running as f64,
        profile.services.len().max(1) as f64,
    );
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "SERVICES"),
            metric_value(
                format!(
                    "{}/{} running",
                    app.state.services_running,
                    profile.services.len()
                ),
                metric_color(theme, service_pct),
            ),
        ]),
        bar_line(
            theme,
            service_pct,
            panel_bar_width(area),
            metric_color(theme, service_pct),
        ),
        Line::from(vec![
            metric_label(theme, "TARGETS"),
            metric_value(app.state.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, "OBSERVED"),
            metric_value(
                app.state.observed_processes.len().to_string(),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "HIDDEN"),
            metric_value(
                app.state.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "WASTE"),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.orange),
        ]),
        Line::from(vec![
            metric_label(theme, "CONFIG"),
            Span::styled(
                crate::metrics::truncate(&app.config.status, 34),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "PROCESS SCAN", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_runtime_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "UPTIME"),
            metric_value(
                crate::metrics::format_duration(app.started_at.elapsed()),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "TELEMETRY"),
            metric_value(
                format!("{} ms", app.config.telemetry.telemetry_rate.as_millis()),
                theme.blue,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "PROCESSES"),
            metric_value(
                format!("{} ms", app.config.telemetry.process_rate.as_millis()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "WINDOWS"),
            metric_value(
                format!("{} ms", app.config.telemetry.platform_rate.as_millis()),
                theme.orange,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM LIB"),
            metric_value(format!("{} games", app.steam.games.len()), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, "HISTORY"),
            metric_value(
                format!("{} lines", app.history_lines.len()),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME FILE"),
            Span::styled(
                app.theme_watcher
                    .path()
                    .map(|path| crate::metrics::truncate(&path.display().to_string(), 34))
                    .unwrap_or_else(|| "internal".to_string()),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "RUNTIME", theme.acid_green))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_graphics_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let gpu_load = app.state.hardware.gpu_load_pct.unwrap_or(0);
    let gpu_color = app
        .state
        .hardware
        .gpu_load_pct
        .map_or(theme.muted, |value| metric_color(theme, value));
    let vram_pct = app.state.hardware.gpu_vram_used_pct().unwrap_or(0);
    let vram_color = app
        .state
        .hardware
        .gpu_vram_used_pct()
        .map_or(theme.muted, |value| metric_color(theme, value));
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "GPU LOAD"),
            metric_value(
                app.state
                    .hardware
                    .gpu_load_pct
                    .map_or_else(|| "--%".to_string(), |value| format!("{value}%")),
                gpu_color,
            ),
        ]),
        bar_line(theme, gpu_load, panel_bar_width(area), gpu_color),
        Line::from(vec![
            metric_label(theme, "GPU VRAM"),
            metric_value(format_system_vram(&app.state.hardware), vram_color),
        ]),
        bar_line(theme, vram_pct, panel_bar_width(area), vram_color),
        Line::from(vec![
            metric_label(theme, "GPU TEMP"),
            metric_value(
                format_system_temp(app.state.hardware.gpu_temp_c),
                system_temp_color(theme, app.state.hardware.gpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "CPU TEMP"),
            metric_value(
                format_system_temp(app.state.hardware.cpu_temp_c),
                system_temp_color(theme, app.state.hardware.cpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "FPS"),
            metric_value(format_frame_fps(app.frame_metrics.fps), theme.cyber_yellow),
            Span::styled("  AVG ", Style::new().fg(theme.muted)),
            metric_value(
                format_frame_fps(app.frame_metrics.average_fps),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "1% LOW"),
            metric_value(format_frame_fps(app.frame_metrics.low_1_fps), theme.hot_red),
            Span::styled("  FRAME ", Style::new().fg(theme.muted)),
            metric_value(
                format_frame_ms(app.frame_metrics.frame_time_ms),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SAMPLES"),
            metric_value(app.frame_metrics.samples.to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "TARGET"),
            Span::styled(
                crate::metrics::truncate(
                    app.frame_metrics
                        .process_name
                        .as_deref()
                        .unwrap_or("no target"),
                    34,
                ),
                Style::new().fg(if app.frame_metrics.fps.is_some() {
                    theme.acid_green
                } else {
                    theme.muted
                }),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SENSORS"),
            Span::styled(
                crate::metrics::truncate(&app.state.hardware.status, 36),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "PMON"),
            Span::styled(
                crate::metrics::truncate(&app.frame_metrics.status, 36),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "GPU / FRAMES", theme.orange))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn format_frame_fps(value: Option<f64>) -> String {
    value.map_or_else(
        || "--".to_string(),
        |value| format!("{:>3}", value.round().clamp(0.0, 999.0) as u16),
    )
}

fn format_frame_ms(value: Option<f64>) -> String {
    value.map_or_else(|| "-- ms".to_string(), |value| format!("{value:.1} ms"))
}

fn format_system_vram(hardware: &crate::hardware::HardwareState) -> String {
    match (hardware.gpu_vram_used_mb, hardware.gpu_vram_total_mb) {
        (Some(used), Some(total)) if total > 0.0 => {
            format!("{:.1}/{:.1} GB", used / 1024.0, total / 1024.0)
        }
        (Some(used), _) => format!("{:.1} GB", used / 1024.0),
        _ => "--".to_string(),
    }
}

fn format_system_temp(value: Option<f32>) -> String {
    value.map_or_else(|| "--C".to_string(), |value| format!("{value:.0}C"))
}

fn system_temp_color(theme: &crate::theme::Theme, value: Option<f32>) -> ratatui::style::Color {
    match value {
        Some(value) if value >= 85.0 => theme.hot_red,
        Some(value) if value >= 72.0 => theme.cyber_yellow,
        Some(_) => theme.acid_green,
        None => theme.muted,
    }
}

pub(super) fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(area);
        render_history_log(frame, app, columns[0]);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(8)])
            .split(columns[1]);
        render_history_meta(frame, app, right[0]);
        render_history_digest(frame, app, right[1]);
        return;
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);

    render_history_log(frame, app, columns[0]);
    render_history_meta(frame, app, columns[1]);
}

pub(super) fn render_settings(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(31),
                Constraint::Percentage(37),
                Constraint::Percentage(32),
            ])
            .split(area);

        render_settings_config(frame, app, columns[0]);
        render_settings_integrations(frame, app, columns[1]);
        render_settings_runtime(frame, app, columns[2]);
        return;
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_settings_config(frame, app, columns[0]);
    render_settings_integrations(frame, app, columns[1]);
}

fn render_settings_config(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let config_path = app
        .config
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "defaults only".to_string());
    let theme_path = app
        .theme_watcher
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "internal theme".to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "PROFILE"),
            metric_value(app.config.active_profile_name(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME"),
            metric_value(app.theme_preset.label(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, "LANGUAGE"),
            metric_value(app.config.ui.language.code(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, "CONFIG"),
            Span::styled(
                crate::metrics::truncate(&config_path, 48),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME FILE"),
            Span::styled(
                crate::metrics::truncate(&theme_path, 48),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, "TELEMETRY"),
            metric_value(
                format!("{} ms", app.config.telemetry.telemetry_rate.as_millis()),
                theme.blue,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "PROCESSES"),
            metric_value(
                format!("{} ms", app.config.telemetry.process_rate.as_millis()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "WINDOWS"),
            metric_value(
                format!("{} ms", app.config.telemetry.platform_rate.as_millis()),
                theme.orange,
            ),
        ]),
        Line::from(""),
        command_line(theme, "M", "Cycle Theme", "persists in theme.toml"),
        command_line(theme, "R", "Probe Tools", "refresh PresentMon detection"),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "SETTINGS", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_settings_runtime(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let config_path = app
        .config
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "defaults only".to_string());
    let theme_path = app
        .theme_watcher
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "internal theme".to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "APP"),
            metric_value("Chaos Performance Monitor", theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME LIVE"),
            Span::styled(
                crate::metrics::truncate(&app.theme_status, 34),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "CONFIG"),
            Span::styled(
                crate::metrics::truncate(&config_path, 36),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "THEME"),
            Span::styled(
                crate::metrics::truncate(&theme_path, 36),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " THEME PRESETS",
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled("  cyberpunk ", Style::new().fg(theme.orange).bold()),
            Span::styled("high contrast neon", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  hacker    ", Style::new().fg(theme.acid_green).bold()),
            Span::styled("black terminal ops", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  gruvbox   ", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled("warm terminal palette", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  tokyo     ", Style::new().fg(theme.blue).bold()),
            Span::styled("cool night palette", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  mocha     ", Style::new().fg(theme.neon_magenta).bold()),
            Span::styled("soft pastel palette", Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " ROADMAP",
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled("  Steam now ", Style::new().fg(theme.acid_green).bold()),
            Span::styled("Epic/manual folders later", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  FPS       ", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(
                "PresentMon Console via winget",
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "RUNTIME / THEMES", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_settings_integrations(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let configured = app
        .config
        .integrations
        .presentmon_exe
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "not set".to_string());
    let resolved = app
        .presentmon_probe
        .path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "not found".to_string());
    let presentmon_color = if app.presentmon_probe.path.is_some() {
        theme.acid_green
    } else {
        theme.hot_red
    };

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "PMON"),
            metric_value(app.presentmon_probe.status.as_str(), presentmon_color),
        ]),
        Line::from(vec![
            metric_label(theme, "SOURCE"),
            metric_value(app.presentmon_probe.source, theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "CONFIG"),
            Span::styled(
                crate::metrics::truncate(&configured, 46),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "RESOLVED"),
            Span::styled(
                crate::metrics::truncate(&resolved, 46),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            metric_value(format!("{} games", app.steam.games.len()), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, "PROVIDER"),
            metric_value("Steam active", theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, "NEXT"),
            Span::styled("manual folders -> Epic later", Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  config.toml: ", Style::new().fg(theme.muted)),
            Span::styled("[integrations]", Style::new().fg(theme.cyber_yellow).bold()),
        ]),
        Line::from(vec![Span::styled(
            r#"  presentmon_exe = "D:\Tools\PresentMon.exe""#,
            Style::new().fg(theme.foreground),
        )]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "INTEGRATIONS", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_history_log(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let viewport_lines = area.height.saturating_sub(2).max(1);
    let max_scroll = history_panel_max_scroll(app.history_lines.len(), viewport_lines);
    let scroll = app.history_scroll.min(max_scroll);
    let title = history_panel_title(
        "HISTORY",
        app.history_lines.len(),
        viewport_lines,
        scroll,
        max_scroll,
    );

    let lines = if app.history_lines.is_empty() {
        vec![
            Line::from(vec![
                Span::styled("  STATUS ", Style::new().fg(theme.blue).bold()),
                Span::styled("No hay historial todavia.", Style::new().fg(theme.muted)),
            ]),
            Line::from(""),
            Line::from(vec![
                metric_label(theme, "LOGGED"),
                Span::styled(
                    "overdrive previews, restore runs, Steam launches, sessions",
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "NEXT"),
                keycap(theme, "1"),
                Span::styled(" preview overdrive  ", Style::new().fg(theme.muted)),
                keycap(theme, "2"),
                Span::styled(" restore", Style::new().fg(theme.muted)),
            ]),
            Line::from(vec![
                metric_label(theme, "PATH"),
                Span::styled(
                    crate::metrics::truncate(&app.history_path.display().to_string(), 58),
                    Style::new().fg(theme.foreground),
                ),
            ]),
        ]
    } else {
        app.history_lines
            .iter()
            .map(|line| styled_history_line(theme, line))
            .collect()
    };

    let log = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, title, theme.cyber_yellow))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(log, area);
}

fn render_history_meta(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, "STATUS"),
            Span::styled(
                crate::metrics::truncate(&app.history_status, 34),
                Style::new().fg(if app.history_lines.is_empty() {
                    theme.muted
                } else {
                    theme.acid_green
                }),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "PATH"),
            Span::styled(
                crate::metrics::truncate(&app.history_path.display().to_string(), 34),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "BUFFER"),
            Span::styled(
                format!("last {} lines", app.history_lines.len()),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(""),
        command_line(theme, "R", "Reload History", "read history.log again"),
        command_line(theme, "UP/DN", "Scroll", "move one line"),
        command_line(theme, "PG", "Page", "jump by block"),
        command_line(theme, "HOME", "Top", "first visible line"),
        command_line(theme, "END", "Bottom", "latest entries"),
    ];

    let meta = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "HISTORY CONTROL", theme.neon_cyan))
        .wrap(Wrap { trim: true });
    frame.render_widget(meta, area);
}

fn render_history_digest(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let last_entry = app
        .history_lines
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| crate::metrics::truncate(line, 42))
        .unwrap_or_else(|| "none yet".to_string());
    let warning_count = app
        .history_lines
        .iter()
        .filter(|line| line.contains("[!]") || line.to_ascii_lowercase().contains("error"))
        .count();
    let session_count = app
        .history_lines
        .iter()
        .filter(|line| line.to_ascii_lowercase().contains("sesion"))
        .count();

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "LINES"),
            metric_value(app.history_lines.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, "WARNINGS"),
            metric_value(
                warning_count.to_string(),
                if warning_count == 0 {
                    theme.acid_green
                } else {
                    theme.hot_red
                },
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SESSIONS"),
            metric_value(session_count.to_string(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, "LAST"),
            Span::styled(last_entry, Style::new().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " HISTORY FEEDS",
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled("  overdrive ", Style::new().fg(theme.orange).bold()),
            Span::styled("what changed before gaming", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  restore   ", Style::new().fg(theme.acid_green).bold()),
            Span::styled("what returned to Windows", Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  sessions  ", Style::new().fg(theme.neon_cyan).bold()),
            Span::styled("game launch and timer notes", Style::new().fg(theme.muted)),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "HISTORY DIGEST", theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn styled_history_line(theme: &crate::theme::Theme, line: &str) -> Line<'static> {
    let style = if line.starts_with("====") {
        Style::new().fg(theme.cyber_yellow).bold()
    } else if line.contains("[!]") || line.contains("error") {
        Style::new().fg(theme.hot_red).bold()
    } else if line.contains("Historial guardado") || line.contains("✓") {
        Style::new().fg(theme.acid_green)
    } else if line.trim().is_empty() {
        Style::new().fg(theme.muted)
    } else {
        Style::new().fg(theme.foreground)
    };

    Line::from(Span::styled(line.to_string(), style))
}

fn history_panel_max_scroll(line_count: usize, viewport_lines: u16) -> u16 {
    line_count
        .saturating_sub(usize::from(viewport_lines))
        .min(usize::from(u16::MAX)) as u16
}

fn history_panel_title(
    base_title: &'static str,
    line_count: usize,
    viewport_lines: u16,
    scroll: u16,
    max_scroll: u16,
) -> String {
    if max_scroll == 0 {
        return base_title.to_string();
    }

    let start = usize::from(scroll) + 1;
    let end = (start + usize::from(viewport_lines) - 1).min(line_count);
    format!("{base_title} {start}-{end}/{line_count}")
}

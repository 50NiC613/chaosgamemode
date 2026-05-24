use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph, Sparkline, Wrap},
};

use super::components::*;
use crate::app::{App, FrameEvent, FrameEventKind};
use crate::config::{BoostProfile, process_pattern_from_name};
use crate::i18n::Language;
use crate::metrics::{readiness_score, scaled_history};
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
    let lang = app.config.ui.language;
    let visible_rows = area.height.saturating_sub(2).max(1) as usize;
    let start = app
        .process_selected
        .saturating_sub(visible_rows.saturating_sub(1) / 2);
    let end = (start + visible_rows).min(sorted.len());

    let empty_text = if !app.process_filter.is_empty() {
        lang.no_process_filter_match()
    } else if app.show_hidden_processes {
        lang.no_hidden_processes()
    } else {
        lang.no_actionable_processes()
    };

    let items: Vec<ListItem> = if sorted.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  \u{f00c} ", Style::new().fg(theme.acid_green).bold()),
            Span::styled(empty_text, Style::new().fg(theme.muted)),
            Span::styled("  ", Style::new()),
            Span::styled("\u{f06e}", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(lang.hidden_view_hint(), Style::new().fg(theme.muted)),
        ]))]
    } else {
        sorted[start..end]
            .iter()
            .enumerate()
            .map(|(offset, (name, group))| {
                let index = start + offset;
                let selected = index == app.process_selected;
                let profile = app.config.active_profile();
                let (label, color) = process_status(profile, name, theme, lang);
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
        lang.panel_hidden_bin()
    } else {
        lang.panel_processes()
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
    let lang = app.config.ui.language;
    let profile = app.config.active_profile();
    let value_width = panel_value_width(area, 72);
    let lines = if let Some((name, group)) = selected {
        let (label, color) = process_status(profile, name, theme, lang);
        let pattern = process_pattern_from_name(name);
        vec![
            Line::from(vec![
                metric_label(theme, lang.label_name()),
                metric_value(
                    crate::metrics::truncate(name, value_width),
                    theme.cyber_yellow,
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_pattern()),
                Span::styled(
                    crate::metrics::truncate(&pattern, value_width),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_status()),
                status_badge(label, color),
            ]),
            Line::from(vec![
                metric_label(theme, "EXE"),
                Span::styled(
                    group
                        .exe_path
                        .as_deref()
                        .map(|path| crate::metrics::truncate(path, value_width))
                        .unwrap_or_else(|| lang.unavailable().to_string()),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_view()),
                Span::styled(
                    if app.show_hidden_processes {
                        lang.view_hidden()
                    } else {
                        lang.view_actionable()
                    },
                    Style::new().fg(theme.neon_cyan),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_memory()),
                metric_value(
                    lang.memory_instances(group.memory_mb, group.count),
                    metric_color(theme, percent_from_memory(group)),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_config()),
                Span::styled(
                    crate::metrics::truncate(
                        &localized_config_status(&app.config.status, lang),
                        value_width,
                    ),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(""),
            process_commands(theme, app.show_hidden_processes, lang),
        ]
    } else {
        vec![
            Line::from(lang.no_process_selected()),
            Line::from(""),
            process_commands(theme, app.show_hidden_processes, lang),
        ]
    };

    let detail = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.panel_detail(), theme.cyber_yellow))
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
    let lang = app.config.ui.language;
    let profile = app.config.active_profile();
    let value_width = panel_value_width(area, 64);
    let selected_pattern = selected
        .map(|(name, _)| process_pattern_from_name(name))
        .unwrap_or_else(|| lang.none().to_string());
    let selected_memory = selected
        .map(|(_, group)| format!("{:.0} MB", group.memory_mb))
        .unwrap_or_else(|| "--".to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.profile()),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_targets()),
            metric_value(profile.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_keep()),
            metric_value(
                profile.protected_processes.len().to_string(),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.hidden()),
            metric_value(
                profile.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_selected()),
            metric_value(
                crate::metrics::truncate(&selected_pattern, value_width),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_memory()),
            metric_value(selected_memory, theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_filter()),
            Span::styled(
                if app.process_filter.is_empty() {
                    lang.none().to_string()
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
            metric_label(theme, lang.label_config()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_config_status(&app.config.status, lang),
                    value_width,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(""),
        process_commands(theme, app.show_hidden_processes, lang),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.panel_policy(), theme.blue))
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
    let lang = app.config.ui.language;
    let counts = process_status_counts(app, sorted);
    let visible_memory = sorted.iter().map(|(_, group)| group.memory_mb).sum::<f64>();
    let actionable = app.state.observed_processes.len();
    let hidden = app.state.hidden_processes.len();
    let total_groups = actionable + hidden;
    let rows = area.height.saturating_sub(16).max(3) as usize;

    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_visible()),
            metric_value(
                format!("{}/{}", sorted.len(), app.visible_process_total()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_memory()),
            metric_value(format!("{visible_memory:.0} MB"), theme.orange),
        ]),
        Line::from(vec![
            metric_label(theme, lang.status_target()),
            metric_value(counts.target.to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_keep()),
            metric_value(counts.keep.to_string(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_watch()),
            metric_value(counts.watch.to_string(), theme.muted),
        ]),
        Line::from(vec![
            metric_label(theme, lang.hidden()),
            metric_value(counts.hidden.to_string(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_total()),
            metric_value(lang.groups_count(total_groups), theme.neon_cyan),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            lang.top_memory_heading(),
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
        .block(accent_block(
            theme,
            lang.panel_process_map(),
            theme.cyber_yellow,
        ))
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
    language: Language,
) -> (&'a str, ratatui::style::Color) {
    if profile.is_hidden_process(process_name) {
        (language.status_hidden(), theme.neon_magenta)
    } else if profile.is_protected_process(process_name) {
        (language.status_keep(), theme.acid_green)
    } else if profile.is_target_process(process_name) {
        (language.status_target(), theme.hot_red)
    } else {
        (language.status_watch(), theme.muted)
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

fn process_commands(
    theme: &crate::theme::Theme,
    hidden_view: bool,
    language: Language,
) -> Line<'static> {
    if hidden_view {
        return Line::from(vec![
            keycap(theme, "H"),
            Span::styled(
                format!(" {} ", language.unhide()),
                Style::new().fg(theme.neon_cyan).bold(),
            ),
            keycap(theme, "V"),
            Span::styled(
                format!(" {} ", language.active()),
                Style::new().fg(theme.cyber_yellow).bold(),
            ),
            keycap(theme, "/"),
            Span::styled(
                format!(" {}", language.filter()),
                Style::new().fg(theme.foreground).bold(),
            ),
        ]);
    }

    Line::from(vec![
        keycap(theme, "P"),
        Span::styled(
            format!(" {} ", language.keep()),
            Style::new().fg(theme.neon_cyan).bold(),
        ),
        keycap(theme, "T"),
        Span::styled(
            format!(" {} ", language.target()),
            Style::new().fg(theme.hot_red).bold(),
        ),
        keycap(theme, "N"),
        Span::styled(
            format!(" {} ", language.neutral()),
            Style::new().fg(theme.muted),
        ),
        keycap(theme, "H"),
        Span::styled(
            format!(" {} ", language.hide()),
            Style::new().fg(theme.neon_magenta).bold(),
        ),
        keycap(theme, "V"),
        Span::styled(
            format!(" {} ", language.hidden()),
            Style::new().fg(theme.cyber_yellow).bold(),
        ),
        keycap(theme, "/"),
        Span::styled(
            format!(" {}", language.filter()),
            Style::new().fg(theme.foreground).bold(),
        ),
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
    let lang = app.config.ui.language;
    let actions = Paragraph::new(Text::from(vec![
        command_line(
            theme,
            "1",
            lang.command_preview_overdrive(),
            lang.command_preview_overdrive_detail(),
        ),
        command_line(
            theme,
            "2",
            lang.command_restore_system(),
            lang.command_restore_system_detail(),
        ),
        command_line(
            theme,
            "R",
            lang.command_refresh_telemetry(),
            lang.command_refresh_telemetry_detail(),
        ),
        command_line(
            theme,
            "TAB",
            lang.command_switch_deck(),
            lang.command_switch_deck_detail(),
        ),
        command_line(
            theme,
            "M",
            lang.command_cycle_theme(),
            lang.command_cycle_theme_detail(),
        ),
        command_line(theme, "Q", lang.command_exit(), lang.command_exit_detail()),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, lang.profile()),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_payload()),
            metric_value(lang.removable_heat(app.state.total_waste_mb), theme.hot_red),
        ]),
    ]))
    .block(accent_block(
        theme,
        lang.panel_overdrive_console(),
        theme.cyber_yellow,
    ))
    .wrap(Wrap { trim: true });
    frame.render_widget(actions, area);
}

fn render_boost_readiness(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let score = readiness_score(&app.state);
    let readiness = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled(
                format!("  {} ", lang.label_ready()),
                Style::new().fg(theme.muted),
            ),
            Span::styled(
                format!("{score}%"),
                Style::new().fg(theme.acid_green).bold(),
            ),
        ]),
        bar_line(theme, score, panel_bar_width(area), theme.acid_green),
    ]))
    .block(accent_block(
        theme,
        lang.panel_readiness(),
        theme.acid_green,
    ));
    frame.render_widget(readiness, area);
}

fn render_boost_live_status(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let log = Paragraph::new(Text::from(vec![
        Line::from(vec![
            metric_label(theme, lang.profile()),
            metric_value(app.config.active_profile_name(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_power_plan()),
            Span::styled(
                display_power_plan(&app.state.power_plan, lang),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            Span::styled(
                if app.state.steam_on {
                    lang.linked()
                } else {
                    lang.not_linked()
                },
                Style::new().fg(status_color(theme, app.state.steam_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_shell()),
            Span::styled(
                if app.state.explorer_on {
                    lang.desktop_active()
                } else {
                    lang.minimal_shell()
                },
                Style::new().fg(status_color(theme, !app.state.explorer_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_services()),
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
            metric_label(theme, lang.label_targets()),
            metric_value(lang.active_groups(app.state.processes.len()), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_bloat()),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.orange),
        ]),
    ]))
    .block(accent_block(
        theme,
        lang.panel_live_status(),
        theme.neon_cyan,
    ));
    frame.render_widget(log, area);
}

fn render_boost_profile(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let profile = app.config.active_profile();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_name()),
            metric_value(profile.name, theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_mode()),
            metric_value(app.config.active_profile_name(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_targets()),
            metric_value(profile.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_protected()),
            metric_value(
                profile.protected_processes.len().to_string(),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.hidden()),
            metric_value(
                profile.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_services()),
            metric_value(profile.services.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_power()),
            metric_value(
                if profile.set_high_performance {
                    lang.high_performance_lower()
                } else {
                    lang.unchanged()
                },
                if profile.set_high_performance {
                    theme.acid_green
                } else {
                    theme.muted
                },
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_explorer()),
            metric_value(
                if profile.kill_explorer {
                    lang.stop_on_overdrive()
                } else {
                    lang.keep_running()
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
        .block(accent_block(theme, lang.panel_profile_plan(), theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_boost_payload(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let sorted = crate::metrics::sorted_processes(&app.state);
    let rows = area.height.saturating_sub(8).max(3) as usize;
    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_removable()),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_groups()),
            metric_value(sorted.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_services()),
            metric_value(
                lang.services_running(
                    app.state.services_running,
                    app.config.active_profile().services.len(),
                ),
                theme.neon_cyan,
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            lang.current_targets_heading(),
            Style::new().fg(theme.muted).bold(),
        )]),
    ];

    if sorted.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                lang.clean_prefix(),
                Style::new().fg(theme.acid_green).bold(),
            ),
            Span::styled(lang.no_configured_targets(), Style::new().fg(theme.muted)),
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
        .block(accent_block(
            theme,
            lang.panel_payload_preview(),
            theme.hot_red,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_boost_restore_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let profile = app.config.active_profile();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_restore()),
            metric_value(lang.ready_lower(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_power()),
            Span::styled(lang.balanced_plan(), Style::new().fg(theme.cyber_yellow)),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_explorer()),
            Span::styled(lang.restart_if_closed(), Style::new().fg(theme.neon_cyan)),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_services()),
            metric_value(
                lang.configured_count(profile.services.len()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_note()),
            Span::styled(lang.closed_apps_stay_closed(), Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        command_line(
            theme,
            "2",
            lang.command_restore_system(),
            lang.undo_overdrive_changes(),
        ),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(
            theme,
            lang.panel_restore_plan(),
            theme.neon_magenta,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

pub(super) fn render_frames(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(28),
                Constraint::Percentage(42),
                Constraint::Percentage(30),
            ])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Min(8),
            ])
            .split(columns[0]);
        render_frames_session_panel(frame, app, left[0]);
        render_frames_probe_panel(frame, app, left[1]);
        render_frames_event_log(frame, app, left[2]);

        let middle = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(columns[1]);
        render_frames_metrics_panel(frame, app, middle[0]);
        render_frames_trace_panel(frame, app, middle[1]);

        render_frames_hardware_panel(frame, app, columns[2]);
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(columns[0]);
        render_frames_session_panel(frame, app, left[0]);
        render_frames_metrics_panel(frame, app, left[1]);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Length(12),
                Constraint::Min(8),
            ])
            .split(columns[1]);
        render_frames_trace_panel(frame, app, right[0]);
        render_frames_probe_panel(frame, app, right[1]);
        render_frames_hardware_panel(frame, app, right[2]);
    }
}

fn render_frames_session_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let mut lines = Vec::new();

    if let Some(session) = &app.session.active {
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_active()),
            metric_value(
                crate::metrics::truncate(&session.name, 32),
                theme.acid_green,
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_time()),
            metric_value(
                crate::metrics::format_duration(session.started_at.elapsed()),
                theme.cyber_yellow,
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, "APPID"),
            metric_value(session.app_id.as_str(), theme.neon_cyan),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_mode()),
            metric_value(
                if session.overdrive {
                    lang.mode_overdrive()
                } else {
                    lang.mode_normal()
                },
                theme.orange,
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_source()),
            metric_value(
                localized_session_source(session.source.as_str(), lang),
                theme.neon_cyan,
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_active()),
            Span::styled(lang.none(), Style::new().fg(theme.muted)),
        ]));
        lines.push(Line::from(lang.launch_game_timer_hint()));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_status()),
            Span::styled(
                app.auto_session_status.as_str(),
                Style::new().fg(theme.muted),
            ),
        ]));
    }
    lines.push(Line::from(vec![
        metric_label(theme, lang.label_overlay()),
        metric_value(overlay_config_label(app, lang), overlay_config_color(app)),
    ]));
    if let Some(alert) = app.performance_alert.as_deref() {
        lines.push(Line::from(vec![
            metric_label(theme, "GUARD"),
            Span::styled(
                crate::metrics::truncate(&localized_performance_alert(alert, lang), 42),
                Style::new().fg(theme.hot_red),
            ),
        ]));
    }
    lines.push(Line::from(vec![
        metric_label(theme, lang.label_status()),
        Span::styled(
            crate::metrics::truncate(&app.overlay_status.message, 42),
            Style::new().fg(overlay_status_color(app)),
        ),
    ]));

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "MANGOHUD / FRAMES", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_frames_metrics_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;

    if !app.has_frame_samples() {
        let status = if app.frame_resolution_active() {
            lang.frames_resolving_target()
        } else if app.frame_capture_active() {
            lang.frames_waiting_samples()
        } else if app.frame_probe.available {
            lang.frames_capture_armed()
        } else {
            lang.frames_idle_status()
        };
        let mut lines = vec![
            Line::from(vec![
                metric_label(theme, lang.label_capture()),
                Span::styled(status, Style::new().fg(theme.cyber_yellow)),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_target()),
                Span::styled(
                    app.frame_metrics
                        .process_name
                        .as_deref()
                        .unwrap_or(lang.no_target()),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_status()),
                Span::styled(
                    crate::metrics::truncate(
                        &localized_frame_status(&app.frame_metrics.status, lang),
                        46,
                    ),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::new()),
                Span::styled(lang.frames_idle_hint(), Style::new().fg(theme.muted)),
            ]),
        ];
        if let Some(alert) = app.performance_alert.as_deref() {
            lines.push(Line::from(vec![
                metric_label(theme, "GUARD"),
                Span::styled(
                    crate::metrics::truncate(&localized_performance_alert(alert, lang), 46),
                    Style::new().fg(theme.hot_red),
                ),
            ]));
        }

        let panel = Paragraph::new(Text::from(lines))
            .block(accent_block(
                theme,
                lang.panel_frames_idle(),
                theme.neon_cyan,
            ))
            .wrap(Wrap { trim: true });
        frame.render_widget(panel, area);
        return;
    }

    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, "FPS"),
            metric_value(format_frame_fps(app.frame_metrics.fps), theme.cyber_yellow),
            Span::styled(
                format!("  {} ", lang.label_average()),
                Style::new().fg(theme.muted),
            ),
            metric_value(
                format_frame_fps(app.frame_metrics.average_fps),
                theme.acid_green,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "1% LOW"),
            metric_value(format_frame_fps(app.frame_metrics.low_1_fps), theme.hot_red),
            Span::styled(
                format!("  {} ", lang.label_frame()),
                Style::new().fg(theme.muted),
            ),
            metric_value(
                format_frame_ms(app.frame_metrics.frame_time_ms),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_samples()),
            metric_value(app.frame_metrics.samples.to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_target()),
            Span::styled(
                crate::metrics::truncate(
                    app.frame_metrics
                        .process_name
                        .as_deref()
                        .unwrap_or(lang.no_target()),
                    40,
                ),
                Style::new().fg(if app.frame_metrics.fps.is_some() {
                    theme.acid_green
                } else {
                    theme.muted
                }),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_status()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_frame_status(&app.frame_metrics.status, lang),
                    46,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];
    if let Some(alert) = app.performance_alert.as_deref() {
        lines.push(Line::from(vec![
            metric_label(theme, "GUARD"),
            Span::styled(
                crate::metrics::truncate(&localized_performance_alert(alert, lang), 46),
                Style::new().fg(theme.hot_red),
            ),
        ]));
    }

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(
            theme,
            lang.panel_frame_metrics(),
            theme.neon_cyan,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_frames_trace_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let gpu_history = scaled_history(&app.gpu_history);
    let cpu_history = scaled_history(&app.cpu_history);
    let ram_history = scaled_history(&app.ram_history);

    if !app.has_frame_samples() {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        let status = if app.frame_resolution_active() {
            lang.frames_resolving_target()
        } else if app.frame_capture_active() {
            lang.frames_waiting_samples()
        } else {
            lang.frames_idle_status()
        };
        let lines = vec![
            Line::from(vec![
                metric_label(theme, lang.label_capture()),
                Span::styled(status, Style::new().fg(theme.muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::new()),
                Span::styled(lang.frames_idle_hint(), Style::new().fg(theme.muted)),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(accent_block(theme, lang.panel_frames_idle(), theme.muted))
                .wrap(Wrap { trim: true }),
            rows[0],
        );
        render_scaled_trace(
            frame,
            app,
            rows[1],
            format!(
                "GPU {} / {}-{}%",
                lang.trace_title(),
                gpu_history.min,
                gpu_history.max
            ),
            &gpu_history.values,
            100,
            theme.orange,
        );
        render_scaled_trace(
            frame,
            app,
            rows[2],
            format!(
                "CPU {} / {}-{}%",
                lang.trace_title(),
                cpu_history.min,
                cpu_history.max
            ),
            &cpu_history.values,
            100,
            theme.blue,
        );
        render_scaled_trace(
            frame,
            app,
            rows[3],
            format!(
                "RAM {} / {}-{}%",
                lang.trace_title(),
                ram_history.min,
                ram_history.max
            ),
            &ram_history.values,
            100,
            theme.neon_magenta,
        );
        return;
    }

    let fps_history = scaled_history(&app.fps_history);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!(
                    "FPS {} / {}-{}",
                    lang.trace_title(),
                    fps_history.min,
                    fps_history.max
                ),
                theme.cyber_yellow,
            ))
            .data(fps_history.values.iter().copied())
            .max(240)
            .style(Style::new().fg(theme.cyber_yellow).bg(theme.panel)),
        rows[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!(
                    "GPU {} / {}-{}%",
                    lang.trace_title(),
                    gpu_history.min,
                    gpu_history.max
                ),
                theme.orange,
            ))
            .data(gpu_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.orange).bg(theme.panel)),
        rows[1],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!(
                    "CPU {} / {}-{}%",
                    lang.trace_title(),
                    cpu_history.min,
                    cpu_history.max
                ),
                theme.blue,
            ))
            .data(cpu_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.blue).bg(theme.panel)),
        rows[2],
    );
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(
                theme,
                format!(
                    "RAM {} / {}-{}%",
                    lang.trace_title(),
                    ram_history.min,
                    ram_history.max
                ),
                theme.neon_magenta,
            ))
            .data(ram_history.values.iter().copied())
            .max(100)
            .style(Style::new().fg(theme.neon_magenta).bg(theme.panel)),
        rows[3],
    );
}

fn render_scaled_trace(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    title: String,
    values: &[u64],
    max: u64,
    color: ratatui::style::Color,
) {
    frame.render_widget(
        Sparkline::default()
            .block(accent_block(&app.theme, title, color))
            .data(values.iter().copied())
            .max(max)
            .style(Style::new().fg(color).bg(app.theme.panel)),
        area,
    );
}

fn render_frames_hardware_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
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
            metric_label(theme, lang.label_gpu_load()),
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
            metric_label(theme, lang.label_gpu_vram()),
            metric_value(format_system_vram(&app.state.hardware), vram_color),
        ]),
        bar_line(theme, vram_pct, panel_bar_width(area), vram_color),
        Line::from(vec![
            metric_label(theme, lang.label_gpu_temp()),
            metric_value(
                format_system_temp(app.state.hardware.gpu_temp_c),
                system_temp_color(theme, app.state.hardware.gpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_cpu_temp()),
            metric_value(
                format_system_temp(app.state.hardware.cpu_temp_c),
                system_temp_color(theme, app.state.hardware.cpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_sensors()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_hardware_status(&app.state.hardware.status, lang),
                    40,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "GPU / SENSORS", theme.orange))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_frames_probe_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let session_value = app
        .session
        .active
        .as_ref()
        .map(|session| crate::metrics::truncate(&session.name, 34))
        .unwrap_or_else(|| lang.none().to_string());
    let session_color = if app.session.active.is_some() {
        theme.acid_green
    } else {
        theme.muted
    };
    let target_value = app
        .frame_metrics
        .process_name
        .as_deref()
        .map(|target| crate::metrics::truncate(target, 34))
        .unwrap_or_else(|| {
            if app.frame_resolution_active() {
                match lang {
                    Language::Spanish => "resolviendo".to_string(),
                    Language::English => "resolving".to_string(),
                }
            } else {
                lang.no_target().to_string()
            }
        });
    let target_color = if app.has_frame_samples() {
        theme.acid_green
    } else if app.frame_resolution_active() || app.frame_capture_active() {
        theme.cyber_yellow
    } else {
        theme.muted
    };
    let frame_status = frame_hook_status(app, lang);
    let overlay_status = localized_overlay_status(&app.overlay_status.message, lang);

    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_rtss()),
            Span::styled(
                localized_frame_probe_status(&app.frame_probe.status, lang),
                Style::new().fg(if app.frame_probe.available {
                    theme.acid_green
                } else {
                    theme.hot_red
                }),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "SESSION"),
            metric_value(session_value, session_color),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_target()),
            metric_value(target_value, target_color),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_capture()),
            Span::styled(
                crate::metrics::truncate(&frame_status, 40),
                Style::new().fg(target_color),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_overlay()),
            Span::styled(
                crate::metrics::truncate(&overlay_status, 40),
                Style::new().fg(overlay_status_color(app)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_source()),
            metric_value(
                localized_source_value(app.frame_probe.source, lang),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_status()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_frame_status(&app.frame_metrics.status, lang),
                    40,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];
    if let Some(alert) = app.performance_alert.as_deref() {
        lines.push(Line::from(vec![
            metric_label(theme, "GUARD"),
            Span::styled(
                crate::metrics::truncate(&localized_performance_alert(alert, lang), 40),
                Style::new().fg(theme.hot_red),
            ),
        ]));
    }
    lines.extend([
        Line::from(""),
        Line::from(vec![
            keycap(theme, "R"),
            Span::styled(format!(" {}  ", lang.probe()), Style::new().fg(theme.muted)),
            keycap(theme, "S-F12"),
            Span::styled(
                format!(" {}  ", lang.overlay()),
                Style::new().fg(theme.muted),
            ),
            keycap(theme, "2"),
            Span::styled(format!(" {}", lang.restore()), Style::new().fg(theme.muted)),
        ]),
    ]);

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "HOOK STATUS", theme.neon_magenta))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_frames_event_log(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let max_lines = area.height.saturating_sub(2).max(1) as usize;
    let lines = if app.frame_events.is_empty() {
        vec![Line::from(Span::styled(
            lang.frames_idle_hint(),
            Style::new().fg(theme.muted),
        ))]
    } else {
        app.frame_events
            .iter()
            .rev()
            .take(max_lines)
            .map(|event| frame_event_line(theme, event, lang))
            .collect()
    };

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, "HOOK LOG", theme.cyber_yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn frame_event_line(
    theme: &crate::theme::Theme,
    event: &FrameEvent,
    lang: Language,
) -> Line<'static> {
    let timestamp = crate::metrics::format_duration(event.elapsed);
    let message = localized_frame_event_message(event, lang);
    Line::from(vec![
        Span::styled(format!("{timestamp} "), Style::new().fg(theme.muted)),
        Span::styled(
            format!("{:<4} ", event.kind.label()),
            Style::new().fg(frame_event_color(theme, event)),
        ),
        Span::styled(
            crate::metrics::truncate(&message, 42),
            Style::new().fg(theme.foreground),
        ),
    ])
}

fn localized_frame_event_message(event: &FrameEvent, lang: Language) -> String {
    match event.kind {
        FrameEventKind::Probe => localized_frame_probe_status(&event.message, lang),
        FrameEventKind::Capture => localized_frame_status(&event.message, lang),
        FrameEventKind::Overlay => localized_overlay_status(&event.message, lang),
        FrameEventKind::Session => localized_session_event(&event.message, lang),
        FrameEventKind::Guard => localized_performance_alert(&event.message, lang),
    }
}

fn localized_session_event(message: &str, lang: Language) -> String {
    if let Some(game) = message.strip_prefix("Steam session active: ") {
        return match lang {
            Language::Spanish => format!("sesion Steam activa: {game}"),
            Language::English => message.to_string(),
        };
    }
    if let Some(game) = message.strip_prefix("Steam session ended: ") {
        return match lang {
            Language::Spanish => format!("sesion Steam cerrada: {game}"),
            Language::English => message.to_string(),
        };
    }
    message.to_string()
}

fn localized_overlay_status(message: &str, lang: Language) -> String {
    match message {
        "RTSS overlay off" => match lang {
            Language::Spanish => "RTSS overlay apagado".to_string(),
            Language::English => message.to_string(),
        },
        "RTSS overlay active" => match lang {
            Language::Spanish => "RTSS overlay activo".to_string(),
            Language::English => message.to_string(),
        },
        "RTSS overlay armed; waiting for Steam game" => match lang {
            Language::Spanish => "RTSS overlay armado; esperando juego Steam".to_string(),
            Language::English => message.to_string(),
        },
        "RTSS overlay is only available on Windows" => match lang {
            Language::Spanish => "RTSS overlay solo esta disponible en Windows".to_string(),
            Language::English => message.to_string(),
        },
        _ => message.to_string(),
    }
}

fn localized_performance_alert(message: &str, lang: Language) -> String {
    match message {
        "Overdrive FPS collapse; press 2 to restore" => match lang {
            Language::Spanish => "Overdrive bajo 10 FPS; pulsa 2 para restaurar".to_string(),
            Language::English => message.to_string(),
        },
        "Overdrive FPS recovered" => match lang {
            Language::Spanish => "FPS recuperados en Overdrive".to_string(),
            Language::English => message.to_string(),
        },
        _ => message.to_string(),
    }
}

fn frame_event_color(theme: &crate::theme::Theme, event: &FrameEvent) -> Color {
    match event.kind {
        FrameEventKind::Probe if event.message == "RTSS listo" => theme.acid_green,
        FrameEventKind::Probe => theme.hot_red,
        FrameEventKind::Session => theme.orange,
        FrameEventKind::Capture if event.message.starts_with("RTSS tracking ") => theme.acid_green,
        FrameEventKind::Capture if event.message.starts_with("RTSS probing ") => theme.cyber_yellow,
        FrameEventKind::Capture => theme.neon_cyan,
        FrameEventKind::Overlay if event.message == "RTSS overlay active" => theme.acid_green,
        FrameEventKind::Overlay if event.message == "RTSS overlay off" => theme.muted,
        FrameEventKind::Overlay => theme.neon_magenta,
        FrameEventKind::Guard if event.message == "Overdrive FPS recovered" => theme.acid_green,
        FrameEventKind::Guard => theme.hot_red,
    }
}

fn frame_hook_status(app: &App, lang: Language) -> String {
    if let Some(fps) = app.frame_metrics.fps {
        return match lang {
            Language::Spanish => format!("live {:.0} FPS", fps),
            Language::English => format!("live {:.0} FPS", fps),
        };
    }

    if app.frame_metrics.samples > 0 {
        return match lang {
            Language::Spanish => format!("{} samples", app.frame_metrics.samples),
            Language::English => format!("{} samples", app.frame_metrics.samples),
        };
    }

    localized_frame_status(&app.frame_metrics.status, lang)
}

fn overlay_config_label(app: &App, lang: Language) -> &'static str {
    if app.config.overlay.enabled {
        lang.enabled()
    } else {
        lang.disabled()
    }
}

fn overlay_config_color(app: &App) -> Color {
    if app.config.overlay.enabled {
        app.theme.acid_green
    } else {
        app.theme.muted
    }
}

fn overlay_status_color(app: &App) -> Color {
    if !app.config.overlay.enabled {
        app.theme.muted
    } else if app.overlay_status.active {
        app.theme.acid_green
    } else {
        app.theme.cyber_yellow
    }
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
    let lang = app.config.ui.language;
    let cpu_pct = app.state.cpu_usage.clamp(0.0, 100.0).round() as u16;
    let ram_pct = app.state.ram_used_pct();
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_cpu_usage()),
            metric_value(format!("{cpu_pct}%"), metric_color(theme, cpu_pct)),
            Span::styled(
                format!(" {} ", lang.label_cores()),
                Style::new().fg(theme.muted),
            ),
            metric_value(app.state.cpu_cores.to_string(), theme.cyber_yellow),
        ]),
        bar_line(
            theme,
            cpu_pct,
            panel_bar_width(area),
            metric_color(theme, cpu_pct),
        ),
        Line::from(vec![
            metric_label(theme, lang.label_ram_used()),
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
            metric_label(theme, lang.label_ram_free()),
            metric_value(format!("{:.1} GB", app.state.ram_free_gb), theme.acid_green),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.label_telemetry(), theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_windows_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_power()),
            Span::styled(
                display_power_plan(&app.state.power_plan, lang),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_explorer()),
            Span::styled(
                if app.state.explorer_on {
                    lang.running()
                } else {
                    lang.stopped()
                },
                Style::new().fg(status_color(theme, !app.state.explorer_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            Span::styled(
                if app.state.steam_on {
                    lang.running()
                } else {
                    lang.closed()
                },
                Style::new().fg(status_color(theme, app.state.steam_on)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "STEAM RAM"),
            metric_value(format!("{:.0} MB", app.state.steam_mb), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme()),
            metric_value(app.theme_preset.label(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, lang.profile()),
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
    let lang = app.config.ui.language;
    let profile = app.config.active_profile();
    let service_pct = crate::metrics::percent(
        app.state.services_running as f64,
        profile.services.len().max(1) as f64,
    );
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_services()),
            metric_value(
                lang.services_running(app.state.services_running, profile.services.len()),
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
            metric_label(theme, lang.label_targets()),
            metric_value(app.state.processes.len().to_string(), theme.hot_red),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_observed()),
            metric_value(
                app.state.observed_processes.len().to_string(),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.hidden()),
            metric_value(
                app.state.hidden_processes.len().to_string(),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_bloat()),
            metric_value(format!("{:.0} MB", app.state.total_waste_mb), theme.orange),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_config()),
            Span::styled(
                crate::metrics::truncate(&localized_config_status(&app.config.status, lang), 34),
                Style::new().fg(theme.muted),
            ),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(
            theme,
            format!("{} {}", lang.label_processes(), lang.scan()),
            theme.cyber_yellow,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_system_runtime_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_uptime()),
            metric_value(
                crate::metrics::format_duration(app.started_at.elapsed()),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_telemetry()),
            metric_value(
                format!("{} ms", app.config.telemetry.telemetry_rate.as_millis()),
                theme.blue,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_processes()),
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
            metric_label(theme, lang.label_steam_lib()),
            metric_value(lang.games_count(app.steam.games.len()), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_history()),
            metric_value(
                lang.lines_count(app.history_lines.len()),
                theme.neon_magenta,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme_file()),
            Span::styled(
                app.theme_watcher
                    .path()
                    .map(|path| crate::metrics::truncate(&path.display().to_string(), 34))
                    .unwrap_or_else(|| lang.internal().to_string()),
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
    let lang = app.config.ui.language;
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
    let mut lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_gpu_load()),
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
            metric_label(theme, lang.label_gpu_vram()),
            metric_value(format_system_vram(&app.state.hardware), vram_color),
        ]),
        bar_line(theme, vram_pct, panel_bar_width(area), vram_color),
        Line::from(vec![
            metric_label(theme, lang.label_gpu_temp()),
            metric_value(
                format_system_temp(app.state.hardware.gpu_temp_c),
                system_temp_color(theme, app.state.hardware.gpu_temp_c),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_cpu_temp()),
            metric_value(
                format_system_temp(app.state.hardware.cpu_temp_c),
                system_temp_color(theme, app.state.hardware.cpu_temp_c),
            ),
        ]),
    ];

    if app.has_frame_samples() {
        lines.extend([
            Line::from(vec![
                metric_label(theme, "FPS"),
                metric_value(format_frame_fps(app.frame_metrics.fps), theme.cyber_yellow),
                Span::styled(
                    format!("  {} ", lang.label_average()),
                    Style::new().fg(theme.muted),
                ),
                metric_value(
                    format_frame_fps(app.frame_metrics.average_fps),
                    theme.acid_green,
                ),
            ]),
            Line::from(vec![
                metric_label(theme, "1% LOW"),
                metric_value(format_frame_fps(app.frame_metrics.low_1_fps), theme.hot_red),
                Span::styled(
                    format!("  {} ", lang.label_frame()),
                    Style::new().fg(theme.muted),
                ),
                metric_value(
                    format_frame_ms(app.frame_metrics.frame_time_ms),
                    theme.neon_cyan,
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_samples()),
                metric_value(app.frame_metrics.samples.to_string(), theme.cyber_yellow),
            ]),
            Line::from(vec![
                metric_label(theme, "TARGET"),
                Span::styled(
                    crate::metrics::truncate(
                        app.frame_metrics
                            .process_name
                            .as_deref()
                            .unwrap_or(lang.no_target()),
                        34,
                    ),
                    Style::new().fg(if app.frame_metrics.fps.is_some() {
                        theme.acid_green
                    } else {
                        theme.muted
                    }),
                ),
            ]),
        ]);
    } else {
        let status = if app.frame_resolution_active() {
            lang.frames_resolving_target()
        } else if app.frame_capture_active() {
            lang.frames_waiting_samples()
        } else {
            lang.frames_idle_status()
        };
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_frames()),
            Span::styled(status, Style::new().fg(theme.muted)),
        ]));
    }

    lines.extend([
        Line::from(vec![
            metric_label(theme, lang.label_sensors()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_hardware_status(&app.state.hardware.status, lang),
                    36,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_rtss()),
            Span::styled(
                crate::metrics::truncate(
                    &localized_frame_status(&app.frame_metrics.status, lang),
                    36,
                ),
                Style::new().fg(theme.muted),
            ),
        ]),
    ]);

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
    let lang = app.config.ui.language;
    let config_path = app
        .config
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| lang.defaults_only().to_string());
    let theme_path = app
        .theme_watcher
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| lang.internal_theme().to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.profile()),
            metric_value(app.config.active_profile_name(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme()),
            metric_value(app.theme_preset.label(), theme.neon_magenta),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_language()),
            metric_value(app.config.ui.language.code(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_config()),
            Span::styled(
                crate::metrics::truncate(&config_path, 48),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme_file()),
            Span::styled(
                crate::metrics::truncate(&theme_path, 48),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, lang.label_telemetry()),
            metric_value(
                format!("{} ms", app.config.telemetry.telemetry_rate.as_millis()),
                theme.blue,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_processes()),
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
            metric_label(theme, lang.label_overlay()),
            metric_value(overlay_config_label(app, lang), overlay_config_color(app)),
        ]),
        Line::from(""),
        command_line(
            theme,
            "M",
            lang.command_cycle_theme(),
            lang.command_theme_persists(),
        ),
        command_line(
            theme,
            "R",
            lang.command_probe_tools(),
            lang.command_probe_tools_detail(),
        ),
        command_line(
            theme,
            "S-F12/O",
            lang.command_toggle_overlay(),
            lang.command_toggle_overlay_detail(),
        ),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.panel_settings(), theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_settings_runtime(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let config_path = app
        .config
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| lang.defaults_only().to_string());
    let theme_path = app
        .theme_watcher
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| lang.internal_theme().to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, "APP"),
            metric_value("Chaos Performance Monitor", theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme_live()),
            Span::styled(
                crate::metrics::truncate(&localized_theme_status(&app.theme_status, lang), 34),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_config()),
            Span::styled(
                crate::metrics::truncate(&config_path, 36),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_theme()),
            Span::styled(
                crate::metrics::truncate(&theme_path, 36),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            lang.theme_presets_heading(),
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled("  cyberpunk ", Style::new().fg(theme.orange).bold()),
            Span::styled(lang.preset_cyberpunk_desc(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  hacker    ", Style::new().fg(theme.acid_green).bold()),
            Span::styled(lang.preset_hacker_desc(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  gruvbox   ", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(lang.preset_gruvbox_desc(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  tokyo     ", Style::new().fg(theme.blue).bold()),
            Span::styled(lang.preset_tokyo_desc(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  mocha     ", Style::new().fg(theme.neon_magenta).bold()),
            Span::styled(lang.preset_mocha_desc(), Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            lang.roadmap_heading(),
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled(
                format!("  {} ", lang.roadmap_steam_now()),
                Style::new().fg(theme.acid_green).bold(),
            ),
            Span::styled(lang.roadmap_steam_later(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  FPS       ", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(lang.roadmap_rtss(), Style::new().fg(theme.muted)),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.panel_runtime_themes(), theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_settings_integrations(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let install_hint = match lang {
        Language::Spanish => "RTSS abierto + OSD enabled",
        Language::English => "RTSS running + OSD enabled",
    };
    let packaging_hint = match lang {
        Language::Spanish => "RTSS es requisito externo; el MSI no incluye backend de FPS",
        Language::English => "RTSS is external; MSI does not bundle a frame backend",
    };
    let rtss_color = if app.frame_probe.available {
        theme.acid_green
    } else {
        theme.hot_red
    };

    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_rtss()),
            metric_value(
                localized_frame_probe_status(&app.frame_probe.status, lang),
                rtss_color,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_source()),
            metric_value(
                localized_source_value(app.frame_probe.source, lang),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_backend()),
            metric_value("RivaTuner Statistics Server", theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, "INSTALL"),
            Span::styled(install_hint, Style::new().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, lang.label_overlay()),
            metric_value(overlay_config_label(app, lang), overlay_config_color(app)),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_backend()),
            metric_value(app.overlay_status.backend.as_str(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_status()),
            Span::styled(
                crate::metrics::truncate(&app.overlay_status.message, 46),
                Style::new().fg(overlay_status_color(app)),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_telemetry()),
            metric_value(
                format!("{} ms", app.config.overlay.update_rate.as_millis()),
                theme.cyber_yellow,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, "HOTKEY"),
            metric_value("Shift+F12", theme.cyber_yellow),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, "STEAM"),
            metric_value(lang.games_count(app.steam.games.len()), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_provider()),
            metric_value(lang.steam_active(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_next()),
            Span::styled(lang.manual_folders_later(), Style::new().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![
            metric_label(theme, "GIT"),
            Span::styled(packaging_hint, Style::new().fg(theme.muted)),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(
            theme,
            lang.panel_integrations(),
            theme.cyber_yellow,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn render_history_log(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let viewport_lines = area.height.saturating_sub(2).max(1);
    let max_scroll = history_panel_max_scroll(app.history_lines.len(), viewport_lines);
    let scroll = app.history_scroll.min(max_scroll);
    let title = history_panel_title(
        lang.panel_history(),
        app.history_lines.len(),
        viewport_lines,
        scroll,
        max_scroll,
    );

    let lines = if app.history_lines.is_empty() {
        vec![
            Line::from(vec![
                Span::styled(
                    format!("  {} ", lang.label_status()),
                    Style::new().fg(theme.blue).bold(),
                ),
                Span::styled(lang.no_history_yet_sentence(), Style::new().fg(theme.muted)),
            ]),
            Line::from(""),
            Line::from(vec![
                metric_label(theme, lang.label_logged()),
                Span::styled(
                    lang.history_logged_detail(),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_next()),
                keycap(theme, "1"),
                Span::styled(lang.preview_overdrive_hint(), Style::new().fg(theme.muted)),
                Span::styled("  ", Style::new().fg(theme.muted)),
                keycap(theme, "2"),
                Span::styled(format!(" {}", lang.restore()), Style::new().fg(theme.muted)),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_path()),
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
    let lang = app.config.ui.language;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_status()),
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
            metric_label(theme, lang.label_path()),
            Span::styled(
                crate::metrics::truncate(&app.history_path.display().to_string(), 34),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_buffer()),
            Span::styled(
                lang.latest_lines(app.history_lines.len()),
                Style::new().fg(theme.cyber_yellow),
            ),
        ]),
        Line::from(""),
        command_line(
            theme,
            "R",
            lang.command_reload_history(),
            lang.command_reload_history_detail(),
        ),
        command_line(
            theme,
            "UP/DN",
            lang.command_scroll(),
            lang.command_scroll_detail(),
        ),
        command_line(theme, "PG", lang.command_page(), lang.command_page_detail()),
        command_line(theme, "HOME", lang.command_top(), lang.command_top_detail()),
        command_line(
            theme,
            "END",
            lang.command_bottom(),
            lang.command_bottom_detail(),
        ),
    ];

    let meta = Paragraph::new(Text::from(lines))
        .block(accent_block(
            theme,
            lang.panel_history_control(),
            theme.neon_cyan,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(meta, area);
}

fn render_history_digest(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let last_entry = app
        .history_lines
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| crate::metrics::truncate(line, 42))
        .unwrap_or_else(|| lang.none_yet().to_string());
    let warning_count = app
        .history_lines
        .iter()
        .filter(|line| line.contains("[!]") || line.to_ascii_lowercase().contains("error"))
        .count();
    let session_count = app
        .history_lines
        .iter()
        .filter(|line| {
            let line = line.to_ascii_lowercase();
            line.contains("sesion") || line.contains("session")
        })
        .count();

    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_lines()),
            metric_value(app.history_lines.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_warnings()),
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
            metric_label(theme, lang.session()),
            metric_value(session_count.to_string(), theme.neon_cyan),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_last()),
            Span::styled(last_entry, Style::new().fg(theme.foreground)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            lang.history_feeds_heading(),
            Style::new().fg(theme.muted).bold(),
        )]),
        Line::from(vec![
            Span::styled("  overdrive ", Style::new().fg(theme.orange).bold()),
            Span::styled(lang.history_feed_overdrive(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  restore   ", Style::new().fg(theme.acid_green).bold()),
            Span::styled(lang.history_feed_restore(), Style::new().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("  sessions  ", Style::new().fg(theme.neon_cyan).bold()),
            Span::styled(lang.history_feed_sessions(), Style::new().fg(theme.muted)),
        ]),
    ];

    let panel = Paragraph::new(Text::from(lines))
        .block(accent_block(theme, lang.panel_history_digest(), theme.blue))
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

fn styled_history_line(theme: &crate::theme::Theme, line: &str) -> Line<'static> {
    let style = if line.starts_with("====") {
        Style::new().fg(theme.cyber_yellow).bold()
    } else if line.contains("[!]") || line.contains("error") {
        Style::new().fg(theme.hot_red).bold()
    } else if line.contains("Historial guardado")
        || line.contains("History saved")
        || line.contains("✓")
    {
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

fn display_power_plan(plan: &str, language: Language) -> String {
    match plan {
        "Alto Rendimiento" => language.high_performance_plan().to_string(),
        "Balanceado" => language.balanced_plan().to_string(),
        other => other.to_string(),
    }
}

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::metrics::{format_duration, truncate};

use super::components::*;

pub(super) fn render_steam(frame: &mut Frame, app: &App, area: Rect) {
    if area.width >= 150 {
        render_steam_wide(frame, app, area);
    } else {
        render_steam_compact(frame, app, area);
    }
}

fn render_steam_wide(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(32),
            Constraint::Percentage(26),
        ])
        .split(area);

    render_steam_library(frame, app, columns[0]);

    let middle = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(8)])
        .split(columns[1]);
    render_steam_selection(frame, app, middle[0]);
    render_session_panel(frame, app, middle[1]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Min(7),
        ])
        .split(columns[2]);
    render_steam_tools_panel(frame, app, right[0]);
    render_steam_runtime_panel(frame, app, right[1]);
    render_steam_library_summary(frame, app, right[2]);
}

fn render_steam_compact(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_steam_library(frame, app, columns[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(6),
        ])
        .split(columns[1]);

    render_steam_selection(frame, app, right[0]);
    render_session_panel(frame, app, right[1]);
    render_steam_tools_panel(frame, app, right[2]);
}

fn render_steam_library(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let visible_rows = area.height.saturating_sub(2).max(1) as usize;
    let start = app
        .steam
        .selected
        .saturating_sub(visible_rows.saturating_sub(1) / 2);
    let end = (start + visible_rows).min(app.steam.games.len());

    let items: Vec<ListItem> = if app.steam.games.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  \u{f11b} ", Style::new().fg(theme.blue).bold()),
            Span::styled(lang.steam_no_games_detected(), Style::new().fg(theme.muted)),
            Span::styled("\u{f002}", Style::new().fg(theme.cyber_yellow).bold()),
            Span::styled(lang.scan_library_hint(), Style::new().fg(theme.muted)),
        ]))]
    } else {
        app.steam.games[start..end]
            .iter()
            .enumerate()
            .map(|(offset, game)| {
                let index = start + offset;
                let selected = index == app.steam.selected;
                let marker = if selected { "\u{f0da}" } else { " " };
                let name_style = if selected {
                    selected_row_style(theme)
                } else {
                    Style::new().fg(theme.foreground).bold()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {marker} "), Style::new().fg(theme.orange).bold()),
                    Span::styled(format!("{:<38}", truncate(&game.name, 37)), name_style),
                    Span::styled(format!("  #{}", game.app_id), Style::new().fg(theme.muted)),
                ]))
            })
            .collect()
    };

    let title = if app.steam.scanning {
        lang.panel_steam_scanning()
    } else {
        lang.panel_steam_library()
    };
    frame.render_widget(
        List::new(items).block(accent_block(theme, title, theme.cyber_yellow)),
        area,
    );
}

fn render_steam_selection(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let lines = if let Some(game) = app.steam.selected_game() {
        vec![
            Line::from(vec![
                metric_label(theme, lang.game_label()),
                metric_value(truncate(&game.name, 34), theme.cyber_yellow),
            ]),
            Line::from(vec![
                metric_label(theme, "APPID"),
                Span::styled(game.app_id.as_str(), Style::new().fg(theme.neon_cyan)),
            ]),
            Line::from(vec![
                metric_label(theme, lang.install_path_label()),
                Span::styled(
                    truncate(&game.install_dir.display().to_string(), 38),
                    Style::new().fg(theme.foreground),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.library_label()),
                Span::styled(
                    truncate(&game.library_dir.display().to_string(), 38),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                keycap(theme, "ENTER"),
                Span::styled(
                    lang.preview_od_launch(),
                    Style::new().fg(theme.neon_cyan).bold(),
                ),
            ]),
            Line::from(vec![
                keycap(theme, "L"),
                Span::styled(
                    lang.launch_normally(),
                    Style::new().fg(theme.neon_cyan).bold(),
                ),
            ]),
            Line::from(vec![
                keycap(theme, "I"),
                Span::styled(
                    format!(" {}  ", capitalize_short(lang.install())),
                    Style::new().fg(theme.acid_green).bold(),
                ),
                keycap(theme, "V"),
                Span::styled(
                    format!(" {}  ", capitalize_short(lang.validate())),
                    Style::new().fg(theme.cyber_yellow).bold(),
                ),
                keycap(theme, "P"),
                Span::styled(
                    format!(" {}", capitalize_short(lang.properties())),
                    Style::new().fg(theme.neon_cyan).bold(),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    format!("  {} ", lang.label_status()),
                    Style::new().fg(theme.muted),
                ),
                Span::styled(
                    localized_steam_status(&app.steam.status, lang),
                    Style::new().fg(theme.hot_red),
                ),
            ]),
            Line::from(vec![
                keycap(theme, "S"),
                Span::styled(
                    lang.scan_steam_library(),
                    Style::new().fg(theme.neon_cyan).bold(),
                ),
            ]),
        ]
    };

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(accent_block(
                theme,
                lang.panel_selected_game(),
                theme.neon_cyan,
            ))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_session_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let mut lines = Vec::new();

    if let Some(session) = &app.session.active {
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_active()),
            Span::styled(
                truncate(&session.name, 30),
                Style::new().fg(theme.acid_green).bold(),
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_time()),
            Span::styled(
                format_duration(session.started_at.elapsed()),
                Style::new().fg(theme.cyber_yellow).bold(),
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, "APPID"),
            Span::styled(session.app_id.as_str(), Style::new().fg(theme.neon_cyan)),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_mode()),
            Span::styled(
                if session.overdrive {
                    lang.mode_overdrive()
                } else {
                    lang.mode_normal()
                },
                Style::new().fg(theme.orange),
            ),
        ]));
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_source()),
            Span::styled(
                localized_session_source(session.source.as_str(), lang),
                Style::new().fg(theme.neon_cyan),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            keycap(theme, "E"),
            Span::styled(lang.end_session(), Style::new().fg(theme.neon_cyan).bold()),
        ]));
    } else {
        lines.push(Line::from(vec![
            metric_label(theme, lang.label_active()),
            Span::styled(lang.none(), Style::new().fg(theme.muted)),
        ]));
        if let Some(last_completed) = &app.session.last_completed {
            lines.push(Line::from(vec![
                metric_label(theme, lang.label_last()),
                Span::styled(
                    truncate(last_completed, 36),
                    Style::new().fg(theme.foreground),
                ),
            ]));
        } else {
            lines.push(Line::from(lang.launch_game_timer_hint()));
        }
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(accent_block(theme, lang.session(), theme.orange)),
        area,
    );
}

fn render_steam_tools_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.library_label()),
            Span::styled(
                if app.steam.scanning {
                    lang.scanning().to_string()
                } else {
                    lang.games_count(app.steam.games.len())
                },
                Style::new().fg(theme.cyber_yellow).bold(),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_status()),
            Span::styled(
                truncate(&localized_steam_status(&app.steam.status, lang), 34),
                Style::new().fg(if app.steam.games.is_empty() {
                    theme.hot_red
                } else {
                    theme.acid_green
                }),
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
            metric_label(theme, "MONITOR"),
            Span::styled(
                truncate(&app.auto_session_status, 34),
                Style::new().fg(theme.neon_cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            keycap(theme, "S"),
            Span::styled(
                lang.scan_libraries(),
                Style::new().fg(theme.neon_cyan).bold(),
            ),
            keycap(theme, "D"),
            Span::styled(
                format!(" {}", capitalize_short(lang.downloads())),
                Style::new().fg(theme.cyber_yellow).bold(),
            ),
        ]),
        Line::from(vec![
            keycap(theme, "I"),
            Span::styled(
                lang.install_selected(),
                Style::new().fg(theme.acid_green).bold(),
            ),
            keycap(theme, "V"),
            Span::styled(
                format!(" {}", capitalize_short(lang.validate())),
                Style::new().fg(theme.cyber_yellow).bold(),
            ),
        ]),
        Line::from(vec![
            keycap(theme, "P"),
            Span::styled(
                lang.properties_action(),
                Style::new().fg(theme.neon_cyan).bold(),
            ),
            keycap(theme, "U"),
            Span::styled(
                lang.uninstall_action(),
                Style::new().fg(theme.hot_red).bold(),
            ),
        ]),
        Line::from(vec![
            keycap(theme, "E"),
            Span::styled(
                lang.end_current_timer(),
                Style::new().fg(theme.neon_cyan).bold(),
            ),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(accent_block(
                theme,
                lang.panel_steam_tools(),
                theme.neon_magenta,
            ))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_steam_runtime_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let running = app.steam.detect_running_game_process(&app.state);
    let lines = if let Some(running) = running {
        vec![
            Line::from(vec![
                metric_label(theme, lang.label_running()),
                metric_value(truncate(&running.process_name, 28), theme.acid_green),
            ]),
            Line::from(vec![
                metric_label(theme, "APPID"),
                Span::styled(running.app_id, Style::new().fg(theme.neon_cyan)),
            ]),
            Line::from(vec![
                metric_label(theme, "EXE"),
                Span::styled(
                    truncate(&running.exe_path, 38),
                    Style::new().fg(theme.muted),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_frames()),
                metric_value(
                    truncate(&localized_frame_status(&app.frame_metrics.status, lang), 32),
                    if app.frame_metrics.fps.is_some() {
                        theme.acid_green
                    } else {
                        theme.muted
                    },
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                metric_label(theme, lang.label_running()),
                Span::styled(lang.none_detected(), Style::new().fg(theme.muted)),
            ]),
            Line::from(vec![
                metric_label(theme, "MONITOR"),
                Span::styled(
                    truncate(&app.auto_session_status, 34),
                    Style::new().fg(theme.neon_cyan),
                ),
            ]),
            Line::from(vec![
                metric_label(theme, lang.label_frames()),
                Span::styled(
                    truncate(&localized_frame_status(&app.frame_metrics.status, lang), 34),
                    Style::new().fg(theme.muted),
                ),
            ]),
        ]
    };

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(accent_block(theme, lang.panel_runtime(), theme.acid_green))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_steam_library_summary(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let selected = app
        .steam
        .selected
        .saturating_add(1)
        .min(app.steam.games.len());
    let library_count = steam_library_count(app);
    let selected_library = app
        .steam
        .selected_game()
        .map(|game| game.library_dir.display().to_string())
        .unwrap_or_else(|| lang.none().to_string());
    let selected_install = app
        .steam
        .selected_game()
        .map(|game| game.install_dir.display().to_string())
        .unwrap_or_else(|| lang.none().to_string());

    let lines = vec![
        Line::from(vec![
            metric_label(theme, lang.label_games()),
            metric_value(app.steam.games.len().to_string(), theme.cyber_yellow),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_libraries()),
            metric_value(library_count.to_string(), theme.acid_green),
        ]),
        Line::from(vec![
            metric_label(theme, lang.label_selected()),
            metric_value(
                format!("{selected}/{}", app.steam.games.len()),
                theme.neon_cyan,
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.library_label()),
            Span::styled(
                truncate(&selected_library, 36),
                Style::new().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            metric_label(theme, lang.install_path_label()),
            Span::styled(
                truncate(&selected_install, 36),
                Style::new().fg(theme.foreground),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            keycap(theme, "UP/DN"),
            Span::styled(lang.browse(), Style::new().fg(theme.neon_cyan).bold()),
            keycap(theme, "ENTER"),
            Span::styled(" OD", Style::new().fg(theme.orange).bold()),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(accent_block(theme, lang.panel_library_index(), theme.blue))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn capitalize_short(value: &'static str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

fn steam_library_count(app: &App) -> usize {
    let mut libraries: Vec<_> = app
        .steam
        .games
        .iter()
        .map(|game| game.library_dir.display().to_string())
        .collect();
    libraries.sort();
    libraries.dedup();
    libraries.len()
}

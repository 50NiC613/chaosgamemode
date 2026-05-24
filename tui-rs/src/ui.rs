mod components;
mod dashboard;
mod pages;
mod steam_panel;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, Tab};
use crate::i18n::Language;
use crate::metrics::{format_duration, readiness_score, truncate};
use crate::theme::ThemePreset;

use components::*;

pub(crate) fn render_monitor(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, app, layout[0]);
    render_tabs(frame, app, layout[1]);

    match app.tab {
        Tab::Dashboard => dashboard::render_dashboard(frame, app, layout[2]),
        Tab::Steam => steam_panel::render_steam(frame, app, layout[2]),
        Tab::Frames => pages::render_frames(frame, app, layout[2]),
        Tab::Processes => pages::render_processes(frame, app, layout[2]),
        Tab::Boost => pages::render_boost(frame, app, layout[2]),
        Tab::System => pages::render_system(frame, app, layout[2]),
        Tab::History => pages::render_history(frame, app, layout[2]),
        Tab::Settings => pages::render_settings(frame, app, layout[2]),
    }

    render_footer(frame, app, layout[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let readiness = readiness_score(&app.state);
    let bar_width = 12;
    let filled = ((usize::from(readiness) * bar_width) / 100).min(bar_width);
    let empty = bar_width.saturating_sub(filled);
    let bar_color = if readiness >= 80 {
        theme.acid_green
    } else if readiness >= 50 {
        theme.cyber_yellow
    } else {
        theme.hot_red
    };

    let header = Line::from(vec![
        Span::styled(" CPM ", Style::new().fg(theme.hot_red).bold()),
        Span::styled("v3 ", Style::new().fg(theme.muted)),
        Span::styled(format!("{} ", lang.ready()), Style::new().fg(theme.muted)),
        Span::styled(
            format!("{readiness:03}%"),
            Style::new().fg(theme.cyber_yellow).bold(),
        ),
        Span::styled(
            format!(" {} ", "█".repeat(filled)),
            Style::new().fg(bar_color).bold(),
        ),
        Span::styled("░".repeat(empty).to_string(), Style::new().fg(theme.muted)),
        Span::styled(
            format!("  {} ", lang.profile()),
            Style::new().fg(theme.muted),
        ),
        Span::styled(
            app.config.active_profile_name(),
            Style::new().fg(theme.acid_green).bold(),
        ),
        Span::styled(
            format!("  {} ", lang.preset()),
            Style::new().fg(theme.muted),
        ),
        Span::styled(
            app.theme_preset.label(),
            Style::new().fg(theme.neon_magenta).bold(),
        ),
        Span::styled(
            format!("  {} ", lang.session()),
            Style::new().fg(theme.muted),
        ),
        Span::styled(
            format_duration(app.started_at.elapsed()),
            Style::new().fg(theme.blue),
        ),
        Span::styled(
            format!(
                "  CPU {:>3.0}%  RAM {:>3}%  FPS {}  WASTE {:.0}MB",
                app.state.cpu_usage,
                app.state.ram_used_pct(),
                app.frame_metrics.fps.map_or_else(
                    || "---".to_string(),
                    |fps| format!("{:>3}", fps.round().clamp(0.0, 999.0) as u16)
                ),
                app.state.total_waste_mb
            ),
            Style::new().fg(theme.foreground),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.muted))
        .style(Style::new().bg(theme.black));

    frame.render_widget(Paragraph::new(header).block(block), area);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let content_width = area.width.saturating_sub(2);

    let spans: Vec<Span> = Tab::nav_slots(content_width)
        .into_iter()
        .map(|slot| {
            let selected = slot.tab == app.tab;
            let label = tab_label_for_width(slot.tab, slot.width, lang);
            let text = centered_text(&label, usize::from(slot.width));
            let style = if selected {
                Style::new()
                    .fg(theme.black)
                    .bg(theme.cyber_yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme.foreground).bg(theme.black)
            };
            Span::styled(text, style)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.muted))
        .style(Style::new().bg(theme.black));

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn tab_label_for_width(tab: Tab, width: u16, language: Language) -> String {
    let icon = tab.icon();
    let full_label = tab.label(language);
    let label_str = if usize::from(width) >= full_label.len() + icon.len() + 2 {
        full_label
    } else {
        tab.compact_label(language)
    };
    format!("{} {}", icon, label_str)
}

fn centered_text(label: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let fitted = if label.len() > width {
        label.chars().take(width).collect::<String>()
    } else {
        label.to_string()
    };
    let padding = width.saturating_sub(fitted.len());
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), fitted, " ".repeat(right))
}

fn tab_breadcrumb(tab: Tab, language: Language) -> String {
    format!("{} {}", tab.icon(), tab.label(language))
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let mut spans: Vec<Span> = Vec::new();

    // Breadcrumb: current tab
    spans.push(Span::styled(
        format!("  \u{f054} {} ", tab_breadcrumb(app.tab, lang)),
        Style::new().fg(theme.blue).bold(),
    ));

    // Contextual keycaps — only for the current tab
    match app.tab {
        Tab::Steam => {
            spans.extend([
                keycap(theme, "ENTER"),
                Span::styled(" OD ", Style::new().fg(theme.muted)),
                keycap(theme, "L"),
                Span::styled(format!(" {} ", lang.launch()), Style::new().fg(theme.muted)),
                keycap(theme, "I"),
                Span::styled(
                    format!(" {} ", lang.install()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "V"),
                Span::styled(
                    format!(" {} ", lang.validate()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "U"),
                Span::styled(
                    format!(" {} ", lang.uninstall()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "P"),
                Span::styled(
                    format!(" {} ", lang.properties()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "D"),
                Span::styled(
                    format!(" {} ", lang.downloads()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "S"),
                Span::styled(format!(" {} ", lang.scan()), Style::new().fg(theme.muted)),
                keycap(theme, "E"),
                Span::styled(format!(" {} ", lang.end()), Style::new().fg(theme.muted)),
            ]);
        }
        Tab::Processes => {
            if app.show_hidden_processes {
                spans.extend([
                    keycap(theme, "H"),
                    Span::styled(format!(" {} ", lang.unhide()), Style::new().fg(theme.muted)),
                    keycap(theme, "V"),
                    Span::styled(format!(" {} ", lang.active()), Style::new().fg(theme.muted)),
                    keycap(theme, "/"),
                    Span::styled(format!(" {} ", lang.filter()), Style::new().fg(theme.muted)),
                ]);
            } else {
                spans.extend([
                    keycap(theme, "P"),
                    Span::styled(format!(" {} ", lang.keep()), Style::new().fg(theme.muted)),
                    keycap(theme, "T"),
                    Span::styled(format!(" {} ", lang.target()), Style::new().fg(theme.muted)),
                    keycap(theme, "N"),
                    Span::styled(
                        format!(" {} ", lang.neutral()),
                        Style::new().fg(theme.muted),
                    ),
                    keycap(theme, "H"),
                    Span::styled(format!(" {} ", lang.hide()), Style::new().fg(theme.muted)),
                    keycap(theme, "V"),
                    Span::styled(format!(" {} ", lang.hidden()), Style::new().fg(theme.muted)),
                    keycap(theme, "/"),
                    Span::styled(format!(" {} ", lang.filter()), Style::new().fg(theme.muted)),
                ]);
            }
            if !app.process_filter.is_empty() || app.editing_process_filter {
                spans.push(Span::styled(
                    format!(
                        " /{}{} ",
                        truncate(&app.process_filter, 20),
                        if app.editing_process_filter { "_" } else { "" }
                    ),
                    Style::new().fg(theme.cyber_yellow).bold(),
                ));
            }
        }
        Tab::History => {
            spans.extend([
                keycap(theme, "R"),
                Span::styled(format!(" {} ", lang.reload()), Style::new().fg(theme.muted)),
                keycap(theme, "↑/↓"),
                Span::styled(format!(" {} ", lang.scroll()), Style::new().fg(theme.muted)),
                keycap(theme, "PG"),
                Span::styled(format!(" {} ", lang.page()), Style::new().fg(theme.muted)),
            ]);
        }
        Tab::Settings => {
            spans.extend([
                keycap(theme, "R"),
                Span::styled(format!(" {} ", lang.probe()), Style::new().fg(theme.muted)),
                keycap(theme, "S-F12"),
                Span::styled(
                    format!(" {} ", lang.overlay()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "M"),
                Span::styled(format!(" {} ", lang.theme()), Style::new().fg(theme.muted)),
            ]);
        }
        Tab::Frames => {
            spans.extend([
                keycap(theme, "R"),
                Span::styled(format!(" {} ", lang.probe()), Style::new().fg(theme.muted)),
                keycap(theme, "S-F12"),
                Span::styled(
                    format!(" {} ", lang.overlay()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "E"),
                Span::styled(format!(" {} ", lang.end()), Style::new().fg(theme.muted)),
            ]);
        }
        _ => {
            spans.extend([
                keycap(theme, "1/SPC"),
                Span::styled(
                    format!(" {} ", lang.preview()),
                    Style::new().fg(theme.muted),
                ),
                keycap(theme, "2"),
                Span::styled(
                    format!(" {} ", lang.restore()),
                    Style::new().fg(theme.muted),
                ),
            ]);
        }
    }

    // Tab navigation + theme + quit (always visible)
    spans.extend([
        keycap(theme, "M"),
        Span::styled(format!(" {} ", lang.theme()), Style::new().fg(theme.muted)),
        keycap(theme, "TAB"),
        Span::styled(format!(" {} ", lang.nav()), Style::new().fg(theme.muted)),
        keycap(theme, "Q"),
        Span::styled(format!(" {} ", lang.exit()), Style::new().fg(theme.muted)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.muted))
        .style(Style::new().bg(theme.black));

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

pub(crate) fn render_output(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    if app.output.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("  \u{f0ca} ", Style::new().fg(theme.blue).bold()),
                Span::styled(lang.output_empty(), Style::new().fg(theme.muted)),
            ]),
            Line::from(vec![
                Span::styled("    ", Style::new()),
                Span::styled(lang.output_hint(), Style::new().fg(theme.foreground)),
            ]),
        ]))
        .block(accent_block(theme, lang.action_log(), theme.cyber_yellow))
        .wrap(Wrap { trim: false });
        frame.render_widget(empty, outer[0]);
    } else {
        let lines: Vec<Line> = app.output.iter().map(|l| Line::from(l.as_str())).collect();
        let viewport_lines = outer[0].height.saturating_sub(2).max(1);
        let max_scroll = panel_max_scroll(app.output.len(), viewport_lines);
        let scroll = app.output_scroll.min(max_scroll);
        let title = panel_title(
            lang.action_log(),
            app.output.len(),
            viewport_lines,
            scroll,
            max_scroll,
        );
        let paragraph = Paragraph::new(Text::from(lines))
            .block(accent_block(theme, title, theme.cyber_yellow))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        frame.render_widget(paragraph, outer[0]);
    }

    let hint = Paragraph::new(Line::from(vec![
        keycap(theme, "ANY KEY"),
        Span::styled(
            format!(" {}  ", lang.return_label()),
            Style::new().fg(theme.foreground),
        ),
        keycap(theme, "↑/↓"),
        Span::styled(
            format!(" {}  ", lang.scroll()),
            Style::new().fg(theme.foreground),
        ),
        keycap(theme, "PG"),
        Span::styled(format!(" {}", lang.page()), Style::new().fg(theme.muted)),
    ]))
    .block(accent_block(theme, lang.ready(), theme.neon_cyan));
    frame.render_widget(hint, outer[1]);
}

pub(crate) fn render_confirm(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let area = frame.area();

    // Render the full monitor underneath as backdrop
    render_monitor(frame, app);

    // Dark overlay
    let overlay = Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    frame.render_widget(Clear, overlay);

    // Centered modal area
    let modal_area = centered_rect(area, 66, 70);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(modal_area);

    // Preview content
    let lines: Vec<Line> = app
        .confirm_lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();
    let viewport_lines = vertical[0].height.saturating_sub(2).max(1);
    let max_scroll = panel_max_scroll(app.confirm_lines.len(), viewport_lines);
    let scroll = app.confirm_scroll.min(max_scroll);
    let title = panel_title(
        format!("\u{f071} {}", lang.confirm_overdrive()),
        app.confirm_lines.len(),
        viewport_lines,
        scroll,
        max_scroll,
    );

    let preview = Paragraph::new(Text::from(lines))
        .block(danger_block(theme, title))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(preview, vertical[0]);

    // Hint bar
    let hint = Paragraph::new(Line::from(vec![
        keycap(theme, "Y/ENTER"),
        Span::styled(
            format!(" {}  ", lang.confirm()),
            Style::new().fg(theme.hot_red).bold(),
        ),
        keycap(theme, "N/ESC"),
        Span::styled(
            format!(" {}  ", lang.cancel()),
            Style::new().fg(theme.foreground),
        ),
        keycap(theme, "↑/↓"),
        Span::styled(
            format!(" {}  ", lang.scroll()),
            Style::new().fg(theme.foreground),
        ),
        keycap(theme, "PG"),
        Span::styled(format!(" {}", lang.page()), Style::new().fg(theme.muted)),
    ]))
    .block(modal_block(theme, lang.safety_check()));
    frame.render_widget(hint, vertical[1]);
}

pub(crate) fn render_theme_menu(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let lang = app.config.ui.language;
    let area = frame.area();

    // Render the full monitor underneath as backdrop
    render_monitor(frame, app);

    // Dark overlay
    let overlay = Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: area.height,
    };
    frame.render_widget(Clear, overlay);

    // Centered modal area
    let modal_area = centered_rect(area, 50, 50);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(modal_area);

    // Theme list
    let mut theme_lines = Vec::new();
    for (i, preset) in ThemePreset::ALL.iter().enumerate() {
        let selected = i == app.theme_menu_selected;
        let active = *preset == app.theme_preset;
        let prefix = if active { " \u{f111} " } else { " \u{f10c} " };
        let fg = if selected {
            theme.cyber_yellow
        } else if active {
            theme.acid_green
        } else {
            theme.foreground
        };
        let extra = if active {
            format!("  ({})", lang.active_suffix())
        } else {
            String::new()
        };
        theme_lines.push(Line::from(Span::styled(
            format!("{prefix}{}{extra}", preset.label()),
            Style::new().fg(fg).bold(),
        )));
    }

    let list = Paragraph::new(Text::from(theme_lines))
        .block(accent_block(
            theme,
            lang.theme().to_ascii_uppercase(),
            theme.neon_cyan,
        ))
        .style(Style::new().bg(theme.panel_dark));
    frame.render_widget(list, vertical[0]);

    // Hint bar
    let hint = Paragraph::new(Line::from(vec![
        keycap(theme, "↑/↓"),
        Span::styled(
            format!(" {}  ", lang.navigate()),
            Style::new().fg(theme.foreground),
        ),
        keycap(theme, "ENTER"),
        Span::styled(
            format!(" {}  ", lang.select()),
            Style::new().fg(theme.acid_green).bold(),
        ),
        keycap(theme, "M"),
        Span::styled(format!(" {}", lang.cancel()), Style::new().fg(theme.muted)),
    ]))
    .block(modal_block(theme, lang.select_theme()));
    frame.render_widget(hint, vertical[1]);
}

fn panel_max_scroll(line_count: usize, viewport_lines: u16) -> u16 {
    line_count
        .saturating_sub(usize::from(viewport_lines))
        .min(usize::from(u16::MAX)) as u16
}

fn panel_title(
    base_title: impl AsRef<str>,
    line_count: usize,
    viewport_lines: u16,
    scroll: u16,
    max_scroll: u16,
) -> String {
    let base_title = base_title.as_ref();
    if max_scroll == 0 {
        return base_title.to_string();
    }

    let start = usize::from(scroll) + 1;
    let end = (start + usize::from(viewport_lines) - 1).min(line_count);
    format!("{base_title} {start}-{end}/{line_count}")
}

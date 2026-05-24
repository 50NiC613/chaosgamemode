use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::ExecutableCommand;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use sysinfo::System;

use crate::config::{AppConfig, BoostProfile, TelemetryConfig};
use crate::history;
use crate::i18n::Language;
use crate::metrics::{
    format_duration, percent_from_f32, push_history, sorted_filtered_hidden_processes,
    sorted_filtered_observed_processes, truncate,
};
use crate::presentmon::{FrameMetrics, PresentMonProbe, probe_presentmon, spawn_frame_capture};
use crate::steam::{
    CompletedSession, SessionState, SteamGame, SteamLibrary, SteamScanResult, install_steam_game,
    launch_steam_game, open_steam_downloads, open_steam_game_properties, spawn_steam_scan,
    uninstall_steam_game, validate_steam_game,
};
use crate::system::{
    ProcessGroup, SystemState, action_lines, activate_chaos_mode, refresh_system_state,
    restore_system,
};
use crate::theme::{Theme, ThemePreset, ThemeWatcher};
use crate::ui::{render_confirm, render_monitor, render_output, render_theme_menu};

const HISTORY_VIEW_LIMIT: usize = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Monitor,
    Confirm,
    Output,
    ThemeMenu,
    Quit,
}

#[derive(Clone)]
enum PendingAction {
    Overdrive,
    LaunchSteamOverdrive(SteamGame),
    UninstallSteamGame(SteamGame),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tab {
    Dashboard,
    Steam,
    Processes,
    Boost,
    System,
    History,
    Settings,
}

#[derive(Clone, Copy)]
pub(crate) struct TabNavSlot {
    pub(crate) tab: Tab,
    pub(crate) start: u16,
    pub(crate) width: u16,
}

impl Tab {
    pub(crate) const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::Steam,
        Self::Processes,
        Self::Boost,
        Self::System,
        Self::History,
        Self::Settings,
    ];

    pub(crate) const fn label(self, language: Language) -> &'static str {
        language.tab_label(self)
    }

    pub(crate) const fn compact_label(self, language: Language) -> &'static str {
        language.tab_compact_label(self)
    }

    pub(crate) const fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "\u{f0e4}",
            Self::Steam => "\u{f1b6}",
            Self::Processes => "\u{f0ae}",
            Self::Boost => "\u{f0e7}",
            Self::System => "\u{f233}",
            Self::History => "\u{f1da}",
            Self::Settings => "\u{f013}",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Dashboard => 0,
            Self::Steam => 1,
            Self::Processes => 2,
            Self::Boost => 3,
            Self::System => 4,
            Self::History => 5,
            Self::Settings => 6,
        }
    }

    pub(crate) fn nav_slots(content_width: u16) -> Vec<TabNavSlot> {
        if content_width == 0 {
            return Vec::new();
        }

        let count = Self::ALL.len() as u16;
        let base_width = content_width / count;
        let extra_width = content_width % count;
        let mut start = 0;
        let mut slots = Vec::with_capacity(Self::ALL.len());

        for (index, tab) in Self::ALL.iter().copied().enumerate() {
            let width = base_width + u16::from((index as u16) < extra_width);
            slots.push(TabNavSlot { tab, start, width });
            start = start.saturating_add(width);
        }

        slots
    }

    fn from_nav_column(col: u16, total_width: u16) -> Option<Self> {
        if col == 0 || col >= total_width.saturating_sub(1) {
            return None;
        }

        let content_col = col - 1;
        Self::nav_slots(total_width.saturating_sub(2))
            .into_iter()
            .find(|slot| {
                content_col >= slot.start && content_col < slot.start.saturating_add(slot.width)
            })
            .map(|slot| slot.tab)
    }

    fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    fn previous(self) -> Self {
        let index = self.index();
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub(crate) struct App {
    screen: Screen,
    pub(crate) tab: Tab,
    pub(crate) output: Vec<String>,
    pub(crate) output_scroll: u16,
    pub(crate) confirm_lines: Vec<String>,
    pub(crate) confirm_scroll: u16,
    pub(crate) state: SystemState,
    pub(crate) config: AppConfig,
    pub(crate) theme: Theme,
    pub(crate) theme_watcher: ThemeWatcher,
    pub(crate) theme_preset: ThemePreset,
    pub(crate) theme_status: String,
    telemetry_rx: Receiver<SystemState>,
    frame_rx: Option<Receiver<FrameMetrics>>,
    frame_target: Option<String>,
    pub(crate) frame_metrics: FrameMetrics,
    pub(crate) presentmon_probe: PresentMonProbe,
    pub(crate) steam: SteamLibrary,
    pub(crate) steam_rx: Receiver<SteamScanResult>,
    pub(crate) session: SessionState,
    pub(crate) auto_session_status: String,
    auto_session_ignore_app_id: Option<String>,
    pub(crate) process_selected: usize,
    pub(crate) show_hidden_processes: bool,
    pub(crate) process_filter: String,
    pub(crate) editing_process_filter: bool,
    pub(crate) history_lines: Vec<String>,
    pub(crate) history_path: PathBuf,
    pub(crate) history_status: String,
    pub(crate) history_scroll: u16,
    pub(crate) cpu_history: Vec<u64>,
    pub(crate) gpu_history: Vec<u64>,
    pub(crate) fps_history: Vec<u64>,
    pub(crate) ram_history: Vec<u64>,
    pub(crate) waste_history: Vec<u64>,
    pub(crate) started_at: Instant,
    pub(crate) theme_menu_selected: usize,
    pending_action: Option<PendingAction>,
}

impl App {
    pub(crate) fn visible_processes(&self) -> Vec<(&String, &ProcessGroup)> {
        if self.show_hidden_processes {
            sorted_filtered_hidden_processes(&self.state, &self.process_filter)
        } else {
            sorted_filtered_observed_processes(&self.state, &self.process_filter)
        }
    }

    pub(crate) fn visible_process_total(&self) -> usize {
        if self.show_hidden_processes {
            self.state.hidden_processes.len()
        } else {
            self.state.observed_processes.len()
        }
    }
}

struct HistoryView {
    path: PathBuf,
    lines: Vec<String>,
    status: String,
}

pub(crate) fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();
    let config = AppConfig::load();
    let state = refresh_system_state(&mut sys, None, true, true, config.active_profile());
    let telemetry_rx =
        spawn_telemetry_thread(config.active_profile().clone(), config.telemetry.clone());
    let steam_rx = spawn_steam_scan();
    let (theme_watcher, theme, theme_preset, theme_status) = ThemeWatcher::new();
    let presentmon_probe = probe_presentmon(config.integrations.presentmon_exe.as_deref());
    let language = config.ui.language;
    let history_view = load_history_view(language);
    let now = Instant::now();

    let mut app = App {
        screen: Screen::Monitor,
        tab: Tab::Dashboard,
        output: Vec::new(),
        output_scroll: 0,
        confirm_lines: Vec::new(),
        confirm_scroll: 0,
        config,
        theme,
        theme_watcher,
        theme_preset,
        theme_status,
        telemetry_rx,
        frame_rx: None,
        frame_target: None,
        frame_metrics: FrameMetrics::idle(),
        presentmon_probe,
        steam: SteamLibrary::loading(),
        steam_rx,
        session: SessionState::default(),
        auto_session_status: language.loading_steam_scan().to_string(),
        auto_session_ignore_app_id: None,
        process_selected: 0,
        show_hidden_processes: false,
        process_filter: String::new(),
        editing_process_filter: false,
        history_lines: history_view.lines,
        history_path: history_view.path,
        history_status: history_view.status,
        history_scroll: 0,
        cpu_history: vec![percent_from_f32(state.cpu_usage).into()],
        gpu_history: vec![state.hardware.gpu_load_pct.unwrap_or(0).into()],
        fps_history: vec![0],
        ram_history: vec![state.ram_used_pct().into()],
        waste_history: vec![state.total_waste_mb.round() as u64],
        state,
        started_at: now,
        theme_menu_selected: 0,
        pending_action: None,
    };

    terminal.clear()?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(DisableMouseCapture)?;
    stdout.execute(LeaveAlternateScreen)?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn load_history_view(language: Language) -> HistoryView {
    match history::read_recent_lines(HISTORY_VIEW_LIMIT) {
        Ok(snapshot) => {
            let visible_lines = snapshot.lines.len();
            let status = language.history_status_loaded(visible_lines, snapshot.total_lines);

            HistoryView {
                path: snapshot.path,
                lines: snapshot.lines,
                status,
            }
        }
        Err(err) => HistoryView {
            path: history::current_path(),
            lines: Vec::new(),
            status: format!("history error: {err}"),
        },
    }
}

fn refresh_history(app: &mut App) {
    let history_view = load_history_view(app.config.ui.language);
    app.history_path = history_view.path;
    app.history_lines = history_view.lines;
    app.history_status = history_view.status;
    clamp_history_scroll(app);
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(33);

    loop {
        drain_telemetry(app);
        drain_steam_scan(app);
        drain_frame_metrics(app);
        sync_frame_monitor(app);
        reload_theme(app);

        terminal.draw(|frame| match app.screen {
            Screen::Monitor => render_monitor(frame, app),
            Screen::Confirm => render_confirm(frame, app),
            Screen::Output => render_output(frame, app),
            Screen::ThemeMenu => render_theme_menu(frame, app),
            Screen::Quit => {}
        })?;

        if app.screen == Screen::Quit {
            break Ok(());
        }

        if event::poll(tick_rate)? {
            let event = event::read()?;
            if handle_event(app, event) {
                break Ok(());
            }
        }
    }
}

fn spawn_telemetry_thread(
    profile: BoostProfile,
    telemetry: TelemetryConfig,
) -> Receiver<SystemState> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || run_telemetry_loop(tx, profile, telemetry));
    rx
}

fn run_telemetry_loop(tx: Sender<SystemState>, profile: BoostProfile, telemetry: TelemetryConfig) {
    let mut sys = System::new_all();
    let mut previous: Option<SystemState> = None;
    let mut last_telemetry = Instant::now() - telemetry.telemetry_rate;
    let mut last_process = Instant::now() - telemetry.process_rate;
    let mut last_platform = Instant::now() - telemetry.platform_rate;

    loop {
        let now = Instant::now();
        let refresh_processes =
            previous.is_none() || now.duration_since(last_process) >= telemetry.process_rate;
        let refresh_platform =
            previous.is_none() || now.duration_since(last_platform) >= telemetry.platform_rate;

        if now.duration_since(last_telemetry) >= telemetry.telemetry_rate {
            let state = refresh_system_state(
                &mut sys,
                previous.as_ref(),
                refresh_processes,
                refresh_platform,
                &profile,
            );
            previous = Some(state.clone());
            if tx.send(state).is_err() {
                break;
            }

            last_telemetry = now;
            if refresh_processes {
                last_process = now;
            }
            if refresh_platform {
                last_platform = now;
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn drain_telemetry(app: &mut App) {
    let mut states = Vec::new();
    while let Ok(state) = app.telemetry_rx.try_recv() {
        states.push(state);
    }
    for state in states {
        apply_system_state(app, state);
    }
}

fn drain_steam_scan(app: &mut App) {
    while let Ok(result) = app.steam_rx.try_recv() {
        app.steam.apply_scan(result);
        app.auto_session_status = app
            .config
            .ui
            .language
            .auto_detect_ready(app.steam.games.len());
        sync_auto_steam_session(app);
    }
}

fn drain_frame_metrics(app: &mut App) {
    let mut metrics = Vec::new();
    if let Some(rx) = &app.frame_rx {
        while let Ok(metric) = rx.try_recv() {
            metrics.push(metric);
        }
    }

    for metric in metrics {
        let fps = metric
            .fps
            .map(|fps| fps.round().clamp(0.0, 999.0) as u64)
            .unwrap_or(0);
        app.frame_metrics = metric;
        push_history(&mut app.fps_history, fps);
    }
}

fn sync_frame_monitor(app: &mut App) {
    let target = app
        .session
        .active_app_id()
        .and_then(|app_id| app.steam.running_process_for_app(app_id, &app.state))
        .map(|running| running.process_name);

    if target == app.frame_target {
        return;
    }

    app.frame_target = target.clone();
    if let Some(process_name) = target {
        app.frame_metrics = FrameMetrics {
            process_name: Some(process_name.clone()),
            status: format!("PresentMon starting {process_name}"),
            ..FrameMetrics::idle()
        };
        app.frame_rx = Some(spawn_frame_capture(
            process_name,
            app.config.integrations.presentmon_exe.clone(),
        ));
    } else {
        app.frame_rx = None;
        app.frame_metrics = FrameMetrics::idle();
        push_history(&mut app.fps_history, 0);
    }
}

fn refresh_presentmon_probe(app: &mut App) {
    app.presentmon_probe = probe_presentmon(app.config.integrations.presentmon_exe.as_deref());
}

fn request_steam_scan(app: &mut App) {
    app.steam = SteamLibrary::loading();
    app.steam_rx = spawn_steam_scan();
}

fn reload_theme(app: &mut App) {
    if let Some(status) = app.theme_watcher.maybe_reload(&mut app.theme) {
        app.theme_preset = app.theme_watcher.active_preset();
        app.theme_status = status;
    }
}

fn apply_system_state(app: &mut App, state: SystemState) {
    app.state = state;
    clamp_process_selection(app);
    push_history(
        &mut app.cpu_history,
        percent_from_f32(app.state.cpu_usage).into(),
    );
    push_history(
        &mut app.gpu_history,
        app.state.hardware.gpu_load_pct.unwrap_or(0).into(),
    );
    push_history(&mut app.ram_history, app.state.ram_used_pct().into());
    push_history(
        &mut app.waste_history,
        app.state.total_waste_mb.round() as u64,
    );
    sync_auto_steam_session(app);
}

fn clamp_process_selection(app: &mut App) {
    let len = visible_process_count(app);
    if len == 0 {
        app.process_selected = 0;
    } else if app.process_selected >= len {
        app.process_selected = len - 1;
    }
}

fn select_next_process(app: &mut App) {
    let len = visible_process_count(app);
    if len > 0 {
        app.process_selected = (app.process_selected + 1) % len;
    }
}

fn select_previous_process(app: &mut App) {
    let len = visible_process_count(app);
    if len > 0 {
        app.process_selected = (app.process_selected + len - 1) % len;
    }
}

fn visible_process_count(app: &App) -> usize {
    app.visible_processes().len()
}

fn selected_process_name(app: &App) -> Option<String> {
    app.visible_processes()
        .get(app.process_selected)
        .map(|(name, _)| (*name).clone())
}

fn restart_telemetry(app: &mut App) {
    app.telemetry_rx = spawn_telemetry_thread(
        app.config.active_profile().clone(),
        app.config.telemetry.clone(),
    );
}

fn toggle_selected_process_protection(app: &mut App) {
    let Some(name) = selected_process_name(app) else {
        return;
    };
    app.config.toggle_protected_process(&name);
    restart_telemetry(app);
}

fn toggle_selected_process_target(app: &mut App) {
    let Some(name) = selected_process_name(app) else {
        return;
    };
    app.config.toggle_target_process(&name);
    restart_telemetry(app);
}

fn neutralize_selected_process(app: &mut App) {
    let Some(name) = selected_process_name(app) else {
        return;
    };
    app.config.neutralize_process(&name);
    restart_telemetry(app);
}

fn toggle_process_hidden_view(app: &mut App) {
    app.show_hidden_processes = !app.show_hidden_processes;
    clamp_process_selection(app);
}

fn start_process_filter(app: &mut App) {
    app.editing_process_filter = true;
    clamp_process_selection(app);
}

fn push_process_filter_char(app: &mut App, value: char) {
    if !value.is_control() {
        app.process_filter.push(value);
        clamp_process_selection(app);
    }
}

fn pop_process_filter_char(app: &mut App) {
    app.process_filter.pop();
    clamp_process_selection(app);
}

fn clear_process_filter(app: &mut App) {
    app.process_filter.clear();
    app.editing_process_filter = false;
    clamp_process_selection(app);
}

fn set_tab(app: &mut App, tab: Tab) {
    app.tab = tab;
    if app.tab == Tab::History {
        refresh_history(app);
    } else if app.tab == Tab::Settings {
        refresh_presentmon_probe(app);
    }
}

fn select_next_tab(app: &mut App) {
    set_tab(app, app.tab.next());
}

fn select_previous_tab(app: &mut App) {
    set_tab(app, app.tab.previous());
}

fn handle_process_filter_input(app: &mut App, code: KeyCode) -> bool {
    if app.screen != Screen::Monitor || app.tab != Tab::Processes || !app.editing_process_filter {
        return false;
    }

    match code {
        KeyCode::Char(value) => push_process_filter_char(app, value),
        KeyCode::Backspace => pop_process_filter_char(app),
        KeyCode::Enter => app.editing_process_filter = false,
        KeyCode::Esc => clear_process_filter(app),
        _ => {}
    }
    true
}

fn toggle_selected_process_visibility(app: &mut App) {
    let Some(name) = selected_process_name(app) else {
        return;
    };
    if app.show_hidden_processes {
        app.config.unhide_process(&name);
    } else {
        app.config.hide_process(&name);
    }
    restart_telemetry(app);
}

fn request_overdrive_confirmation(app: &mut App) {
    app.pending_action = Some(PendingAction::Overdrive);
    app.confirm_lines = overdrive_preview(app, None);
    app.confirm_scroll = 0;
    app.screen = Screen::Confirm;
}

fn request_steam_overdrive_confirmation(app: &mut App) {
    let Some(game) = selected_steam_game_or_warn(app) else {
        return;
    };

    app.confirm_lines = overdrive_preview(app, Some(&game));
    app.confirm_scroll = 0;
    app.pending_action = Some(PendingAction::LaunchSteamOverdrive(game));
    app.screen = Screen::Confirm;
}

fn request_steam_uninstall_confirmation(app: &mut App) {
    let Some(game) = selected_steam_game_or_warn(app) else {
        return;
    };

    app.confirm_lines = steam_uninstall_preview(&game, app.config.ui.language);
    app.confirm_scroll = 0;
    app.pending_action = Some(PendingAction::UninstallSteamGame(game));
    app.screen = Screen::Confirm;
}

fn selected_steam_game_or_warn(app: &mut App) -> Option<SteamGame> {
    let game = app.steam.selected_game().cloned();
    if game.is_none() {
        let language = app.config.ui.language;
        show_output(
            app,
            vec![
                language.no_steam_game_selected().to_string(),
                language.rescan_steam_hint().to_string(),
            ],
        );
    }
    game
}

fn overdrive_preview(app: &App, game: Option<&SteamGame>) -> Vec<String> {
    let language = app.config.ui.language;
    let profile = app.config.active_profile();
    let mut lines = Vec::new();
    lines.push(language.overdrive_preview_title().to_string());
    lines.push(format!(
        "  {}: {}",
        language.profile_label(),
        app.config.active_profile_name()
    ));
    if let Some(game) = game {
        lines.push(format!(
            "  {}: {} (#{}).",
            language.launch_after_label(),
            game.name,
            game.app_id
        ));
    }
    lines.push(format!(
        "  {}: {}.",
        language.configured_processes_label(),
        profile.processes.len()
    ));
    lines.push(format!(
        "  {}: {}.",
        language.protected_processes_label(),
        profile.protected_processes.len()
    ));
    lines.push(format!(
        "  {}: {}.",
        language.hidden_processes_label(),
        profile.hidden_processes.len()
    ));
    lines.push(format!(
        "  {}: {} / {:.0} MB.",
        language.detected_processes_label(),
        app.state.processes.len(),
        app.state.total_waste_mb
    ));
    lines.push(format!(
        "  {}: {}.",
        language.configured_services_label(),
        profile.services.len()
    ));
    lines.push(format!(
        "  Explorer: {}.",
        if profile.kill_explorer {
            language.explorer_will_stop()
        } else {
            language.explorer_kept()
        }
    ));
    lines.push(format!(
        "  {}: {}.",
        language.energy_label(),
        if profile.set_high_performance {
            language.high_performance_plan()
        } else {
            language.no_changes()
        }
    ));
    lines.push(String::new());
    lines.push(language.overdrive_targets_heading().to_string());
    lines.extend(overdrive_target_preview_lines(
        &app.state,
        app.config.ui.language,
    ));

    lines.push(String::new());
    lines.push(language.confirm_hint().to_string());
    lines
}

fn steam_uninstall_preview(game: &SteamGame, language: Language) -> Vec<String> {
    vec![
        language.steam_uninstall_title().to_string(),
        format!(
            "  {}: {} (#{}).",
            language.game_label(),
            game.name,
            game.app_id
        ),
        format!(
            "  {}: {}",
            language.install_path_label(),
            game.install_dir.display()
        ),
        format!(
            "  {}: {}",
            language.library_label(),
            game.library_dir.display()
        ),
        String::new(),
        language.steam_uninstall_safe_1().to_string(),
        language.steam_uninstall_safe_2().to_string(),
        language.steam_uninstall_safe_3().to_string(),
        String::new(),
        language.steam_uninstall_hint().to_string(),
    ]
}

fn overdrive_target_preview_lines(state: &SystemState, language: Language) -> Vec<String> {
    let mut sorted: Vec<_> = state.processes.iter().collect();
    sorted.sort_by(|a, b| b.1.memory_mb.total_cmp(&a.1.memory_mb));

    if sorted.is_empty() {
        return vec![language.no_overdrive_targets().to_string()];
    }

    sorted
        .into_iter()
        .flat_map(|(name, group)| format_target_process_preview(name, group, language))
        .collect()
}

fn format_target_process_preview(
    name: &str,
    group: &ProcessGroup,
    language: Language,
) -> [String; 2] {
    let header = format!(
        "  {:<24} {:>7.0} MB  {:>2}x",
        truncate(name, 24),
        group.memory_mb,
        group.count
    );
    let path = group
        .exe_path
        .as_deref()
        .map(|path| truncate(path, 72))
        .unwrap_or_else(|| language.exe_path_unavailable().to_string());
    [header, format!("    exe: {path}")]
}

fn confirm_pending_action(app: &mut App) {
    match app.pending_action.take() {
        Some(PendingAction::Overdrive) => run_overdrive(app),
        Some(PendingAction::LaunchSteamOverdrive(game)) => {
            launch_steam_game_with_overdrive(app, game);
        }
        Some(PendingAction::UninstallSteamGame(game)) => {
            run_steam_uninstall(app, game);
        }
        None => {
            app.screen = Screen::Monitor;
        }
    }
}

fn max_confirm_scroll(line_count: usize) -> u16 {
    line_count.saturating_sub(1).min(usize::from(u16::MAX)) as u16
}

fn clamp_confirm_scroll(app: &mut App) {
    app.confirm_scroll = app
        .confirm_scroll
        .min(max_confirm_scroll(app.confirm_lines.len()));
}

fn scroll_confirm_up(app: &mut App, amount: u16) {
    app.confirm_scroll = app.confirm_scroll.saturating_sub(amount);
}

fn scroll_confirm_down(app: &mut App, amount: u16) {
    app.confirm_scroll = app
        .confirm_scroll
        .saturating_add(amount)
        .min(max_confirm_scroll(app.confirm_lines.len()));
}

fn reset_confirm_scroll(app: &mut App) {
    app.confirm_scroll = 0;
}

fn end_confirm_scroll(app: &mut App) {
    app.confirm_scroll = max_confirm_scroll(app.confirm_lines.len());
}

fn show_output(app: &mut App, output: Vec<String>) {
    app.output = output;
    app.output_scroll = 0;
    app.screen = Screen::Output;
}

fn append_history_status(log: &mut Vec<String>, result: io::Result<PathBuf>, language: Language) {
    match result {
        Ok(path) => log.push(language.saved_history(&path)),
        Err(err) => log.push(language.history_save_error(&err)),
    }
}

fn append_action_history(
    log: &mut Vec<String>,
    event: &str,
    profile_name: &str,
    language: Language,
) {
    let result = history::append_action(event, profile_name, log);
    append_history_status(log, result, language);
}

fn append_session_history(log: &mut Vec<String>, session: &CompletedSession, language: Language) {
    let result = history::append_session(
        &session.name,
        &session.app_id,
        session.seconds,
        session.overdrive,
        session.source.as_str(),
    );
    append_history_status(log, result, language);
}

fn refresh_history_if_visible(app: &mut App) {
    if app.tab == Tab::History {
        refresh_history(app);
    }
}

fn max_output_scroll(line_count: usize) -> u16 {
    line_count.saturating_sub(1).min(usize::from(u16::MAX)) as u16
}

fn scroll_output_up(app: &mut App, amount: u16) {
    app.output_scroll = app.output_scroll.saturating_sub(amount);
}

fn scroll_output_down(app: &mut App, amount: u16) {
    app.output_scroll = app
        .output_scroll
        .saturating_add(amount)
        .min(max_output_scroll(app.output.len()));
}

fn reset_output_scroll(app: &mut App) {
    app.output_scroll = 0;
}

fn end_output_scroll(app: &mut App) {
    app.output_scroll = max_output_scroll(app.output.len());
}

fn max_history_scroll(line_count: usize) -> u16 {
    line_count.saturating_sub(1).min(usize::from(u16::MAX)) as u16
}

fn clamp_history_scroll(app: &mut App) {
    app.history_scroll = app
        .history_scroll
        .min(max_history_scroll(app.history_lines.len()));
}

fn scroll_history_up(app: &mut App, amount: u16) {
    app.history_scroll = app.history_scroll.saturating_sub(amount);
}

fn scroll_history_down(app: &mut App, amount: u16) {
    app.history_scroll = app
        .history_scroll
        .saturating_add(amount)
        .min(max_history_scroll(app.history_lines.len()));
}

fn reset_history_scroll(app: &mut App) {
    app.history_scroll = 0;
}

fn end_history_scroll(app: &mut App) {
    app.history_scroll = max_history_scroll(app.history_lines.len());
}

fn sync_auto_steam_session(app: &mut App) {
    let language = app.config.ui.language;
    if app.steam.scanning {
        app.auto_session_status = language.loading_steam_scan().to_string();
        return;
    }
    if app.steam.games.is_empty() {
        app.auto_session_status = language.auto_detect_no_games().to_string();
        return;
    }

    let detected = app.steam.detect_running_game(&app.state).cloned();
    if should_ignore_detected_game(app, detected.as_ref()) {
        return;
    }

    if !app.session.active_is_auto_detected() {
        if app.session.active.is_some() {
            app.auto_session_status = language.manual_session_active().to_string();
        } else if let Some(game) = detected {
            start_auto_detected_session(app, game);
        } else {
            app.auto_session_status = language.auto_detect_armed().to_string();
        }
        return;
    }

    let active_app_id = app.session.active_app_id().map(ToOwned::to_owned);
    match (active_app_id.as_deref(), detected) {
        (Some(active_app_id), Some(game)) if active_app_id == game.app_id => {
            app.auto_session_status = language.tracking(&truncate(&game.name, 28));
        }
        (Some(_), Some(game)) => {
            let mut log = finish_active_session(app);
            log.push(String::new());
            start_auto_detected_session_with_log(app, game, log);
        }
        (Some(_), None) => {
            let log = finish_active_session(app);
            app.auto_session_status = language.auto_session_ended().to_string();
            append_background_action(app, "steam_auto_detect_end", log);
        }
        (None, Some(game)) => start_auto_detected_session(app, game),
        (None, None) => {
            app.auto_session_status = language.auto_detect_armed().to_string();
        }
    }
}

fn should_ignore_detected_game(app: &mut App, detected: Option<&SteamGame>) -> bool {
    let Some(ignored_app_id) = app.auto_session_ignore_app_id.as_deref() else {
        return false;
    };

    if detected.is_some_and(|game| game.app_id == ignored_app_id) {
        app.auto_session_status = app.config.ui.language.auto_detect_paused().to_string();
        return true;
    }

    app.auto_session_ignore_app_id = None;
    false
}

fn start_auto_detected_session(app: &mut App, game: SteamGame) {
    start_auto_detected_session_with_log(app, game, Vec::new());
}

fn start_auto_detected_session_with_log(app: &mut App, game: SteamGame, mut log: Vec<String>) {
    let language = app.config.ui.language;
    app.auto_session_ignore_app_id = None;
    app.session.start_detected(&game);
    app.auto_session_status = language.detected(&truncate(&game.name, 28));
    log.push(format!(
        "  {}: {} (#{})",
        language.auto_detected_game(),
        game.name,
        game.app_id
    ));
    log.push(format!("  {}", language.session_started_auto()));
    append_background_action(app, "steam_auto_detect_start", log);
}

fn append_background_action(app: &mut App, event: &str, mut log: Vec<String>) {
    let profile_name = app.config.active_profile_name().to_string();
    append_action_history(&mut log, event, &profile_name, app.config.ui.language);
    refresh_history_if_visible(app);
}

fn run_overdrive(app: &mut App) {
    let language = app.config.ui.language;
    let profile = app.config.active_profile().clone();
    let profile_name = app.config.active_profile_name().to_string();
    let mut output = action_lines(&activate_chaos_mode(&profile, language));
    append_action_history(&mut output, "overdrive", &profile_name, language);
    show_output(app, output);
}

fn run_restore(app: &mut App) {
    let language = app.config.ui.language;
    let profile = app.config.active_profile().clone();
    let profile_name = app.config.active_profile_name().to_string();
    let mut output = action_lines(&restore_system(&profile, language));
    append_action_history(&mut output, "restore", &profile_name, language);
    show_output(app, output);
}

fn launch_selected_steam_game(app: &mut App, overdrive: bool) {
    if overdrive {
        request_steam_overdrive_confirmation(app);
        return;
    }

    let Some(game) = selected_steam_game_or_warn(app) else {
        return;
    };

    launch_steam_game_without_overdrive(app, game);
}

fn run_selected_steam_client_action(
    app: &mut App,
    event: &str,
    title: &str,
    protocol: &str,
    action: fn(&SteamGame) -> bool,
) {
    let Some(game) = selected_steam_game_or_warn(app) else {
        return;
    };

    let language = app.config.ui.language;
    let profile_name = app.config.active_profile_name().to_string();
    let mut log = vec![
        format!("\u{f1b6} {title}: {} (#{})", game.name, game.app_id),
        format!("  {}: {protocol}/{}", language.client_label(), game.app_id),
        format!(
            "  {}: {}",
            language.library_label(),
            game.library_dir.display()
        ),
    ];

    if action(&game) {
        log.push(format!("  \u{f00c} {}", language.steam_uri_sent()));
    } else {
        log.push(format!(
            "  \u{f071} {} {protocol}",
            language.steam_uri_failed()
        ));
    }

    append_action_history(&mut log, event, &profile_name, language);
    show_output(app, log);
}

fn run_steam_install(app: &mut App) {
    let title = app.config.ui.language.steam_install_title();
    run_selected_steam_client_action(
        app,
        "steam_install_requested",
        title,
        "steam://install",
        install_steam_game,
    );
}

fn run_steam_validate(app: &mut App) {
    let title = app.config.ui.language.steam_validate_title();
    run_selected_steam_client_action(
        app,
        "steam_validate_requested",
        title,
        "steam://validate",
        validate_steam_game,
    );
}

fn run_steam_properties(app: &mut App) {
    let title = app.config.ui.language.steam_properties_title();
    run_selected_steam_client_action(
        app,
        "steam_properties_opened",
        title,
        "steam://gameproperties",
        open_steam_game_properties,
    );
}

fn run_steam_downloads(app: &mut App) {
    let language = app.config.ui.language;
    let profile_name = app.config.active_profile_name().to_string();
    let mut log = vec![
        format!("\u{f1b6} {}", language.steam_downloads_title()),
        format!("  {}: steam://open/downloads", language.client_label()),
    ];

    if open_steam_downloads() {
        log.push(format!("  \u{f00c} {}", language.steam_uri_sent()));
    } else {
        log.push(format!(
            "  \u{f071} {} steam://open/downloads",
            language.steam_uri_failed()
        ));
    }

    append_action_history(&mut log, "steam_downloads_opened", &profile_name, language);
    show_output(app, log);
}

fn run_steam_uninstall(app: &mut App, game: SteamGame) {
    let language = app.config.ui.language;
    let profile_name = app.config.active_profile_name().to_string();
    let mut log = vec![
        format!(
            "\u{f1b6} {}: {} (#{})",
            language.steam_uninstall_action_title(),
            game.name,
            game.app_id
        ),
        format!(
            "  {}: steam://uninstall/{}",
            language.client_label(),
            game.app_id
        ),
        format!(
            "  {}: {}",
            language.install_path_label(),
            game.install_dir.display()
        ),
    ];

    if uninstall_steam_game(&game) {
        log.push(format!("  \u{f00c} {}", language.steam_uninstall_opened()));
    } else {
        log.push(format!(
            "  \u{f071} {} steam://uninstall",
            language.steam_uri_failed()
        ));
    }

    append_action_history(
        &mut log,
        "steam_uninstall_requested",
        &profile_name,
        language,
    );
    show_output(app, log);
}

fn launch_steam_game_with_overdrive(app: &mut App, game: SteamGame) {
    let language = app.config.ui.language;
    let mut log = Vec::new();
    if app.session.active.is_some() {
        log.extend(finish_active_session(app));
        log.push(String::new());
    }

    let profile = app.config.active_profile().clone();
    let profile_name = app.config.active_profile_name().to_string();
    log.extend(action_lines(&activate_chaos_mode(&profile, language)));
    log.push(String::new());

    log.push(format!(
        "\u{f11b} {} {} (#{})",
        language.launching(),
        game.name,
        game.app_id
    ));
    if launch_steam_game(&game) {
        log.push(format!("  \u{f00c} {}", language.steam_launch_uri_sent()));
    } else {
        log.push(format!(
            "  \u{f071} {} steam://run",
            language.steam_uri_failed()
        ));
    }

    app.auto_session_ignore_app_id = None;
    app.session.start(&game, true);
    app.auto_session_status = language.manual_session_active().to_string();
    log.push(format!("  \u{f017} {}", language.session_started()));

    append_action_history(&mut log, "steam_overdrive_launch", &profile_name, language);
    show_output(app, log);
}

fn launch_steam_game_without_overdrive(app: &mut App, game: SteamGame) {
    let language = app.config.ui.language;
    let mut log = Vec::new();
    let profile_name = app.config.active_profile_name().to_string();
    if app.session.active.is_some() {
        log.extend(finish_active_session(app));
        log.push(String::new());
    }

    log.push(format!(
        "\u{f11b} {} {} (#{})",
        language.launching(),
        game.name,
        game.app_id
    ));
    if launch_steam_game(&game) {
        log.push(format!("  \u{f00c} {}", language.steam_launch_uri_sent()));
    } else {
        log.push(format!(
            "  \u{f071} {} steam://run",
            language.steam_uri_failed()
        ));
    }

    app.auto_session_ignore_app_id = None;
    app.session.start(&game, false);
    app.auto_session_status = language.manual_session_active().to_string();
    log.push(format!("  \u{f017} {}", language.session_started()));

    append_action_history(&mut log, "steam_launch", &profile_name, language);
    show_output(app, log);
}

fn finish_active_session(app: &mut App) -> Vec<String> {
    let language = app.config.ui.language;
    let mut log = Vec::new();
    if let Some(session) = app.session.stop() {
        log.push(format!(
            "\u{f017} {}: {} / {}",
            language.session_closed_prefix(),
            session.name,
            format_duration(Duration::from_secs(session.seconds))
        ));
        append_session_history(&mut log, &session, language);
    } else {
        log.push(language.no_active_session().to_string());
    }
    log
}

fn finish_active_session_from_user(app: &mut App) -> Vec<String> {
    let ignored_app_id = app.session.active_app_id().map(ToOwned::to_owned);
    let output = finish_active_session(app);
    app.auto_session_ignore_app_id = ignored_app_id;
    app.auto_session_status = if app.auto_session_ignore_app_id.is_some() {
        app.config.ui.language.auto_detect_paused().to_string()
    } else {
        app.config.ui.language.auto_detect_armed().to_string()
    };
    output
}

fn handle_event(app: &mut App, event: Event) -> bool {
    if let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        if handle_process_filter_input(app, key.code) {
            return false;
        }

        match app.screen {
            Screen::Monitor => match key.code {
                KeyCode::Up if app.tab == Tab::History => {
                    scroll_history_up(app, 1);
                }
                KeyCode::Down if app.tab == Tab::History => {
                    scroll_history_down(app, 1);
                }
                KeyCode::PageUp if app.tab == Tab::History => {
                    scroll_history_up(app, 8);
                }
                KeyCode::PageDown if app.tab == Tab::History => {
                    scroll_history_down(app, 8);
                }
                KeyCode::Home if app.tab == Tab::History => {
                    reset_history_scroll(app);
                }
                KeyCode::End if app.tab == Tab::History => {
                    end_history_scroll(app);
                }
                KeyCode::Char('r') | KeyCode::Char('R') if app.tab == Tab::History => {
                    refresh_history(app);
                }
                KeyCode::Char('r') | KeyCode::Char('R') if app.tab == Tab::Settings => {
                    refresh_presentmon_probe(app);
                }
                KeyCode::Up if app.tab == Tab::Steam => {
                    app.steam.select_previous();
                }
                KeyCode::Down if app.tab == Tab::Steam => {
                    app.steam.select_next();
                }
                KeyCode::Enter if app.tab == Tab::Steam => {
                    launch_selected_steam_game(app, true);
                }
                KeyCode::Char(' ') if app.tab == Tab::Steam => {
                    launch_selected_steam_game(app, true);
                }
                KeyCode::Char('l') | KeyCode::Char('L') if app.tab == Tab::Steam => {
                    launch_selected_steam_game(app, false);
                }
                KeyCode::Char('s') | KeyCode::Char('S') if app.tab == Tab::Steam => {
                    request_steam_scan(app);
                }
                KeyCode::Char('e') | KeyCode::Char('E') if app.tab == Tab::Steam => {
                    let output = finish_active_session_from_user(app);
                    show_output(app, output);
                }
                KeyCode::Char('i') | KeyCode::Char('I') if app.tab == Tab::Steam => {
                    run_steam_install(app);
                }
                KeyCode::Char('u') | KeyCode::Char('U') if app.tab == Tab::Steam => {
                    request_steam_uninstall_confirmation(app);
                }
                KeyCode::Char('v') | KeyCode::Char('V') if app.tab == Tab::Steam => {
                    run_steam_validate(app);
                }
                KeyCode::Char('p') | KeyCode::Char('P') if app.tab == Tab::Steam => {
                    run_steam_properties(app);
                }
                KeyCode::Char('d') | KeyCode::Char('D') if app.tab == Tab::Steam => {
                    run_steam_downloads(app);
                }
                KeyCode::Up if app.tab == Tab::Processes => {
                    select_previous_process(app);
                }
                KeyCode::Down if app.tab == Tab::Processes => {
                    select_next_process(app);
                }
                KeyCode::Char('p') | KeyCode::Char('P')
                    if app.tab == Tab::Processes && !app.show_hidden_processes =>
                {
                    toggle_selected_process_protection(app);
                }
                KeyCode::Char('t') | KeyCode::Char('T')
                    if app.tab == Tab::Processes && !app.show_hidden_processes =>
                {
                    toggle_selected_process_target(app);
                }
                KeyCode::Char('n') | KeyCode::Char('N')
                    if app.tab == Tab::Processes && !app.show_hidden_processes =>
                {
                    neutralize_selected_process(app);
                }
                KeyCode::Char('h') | KeyCode::Char('H') if app.tab == Tab::Processes => {
                    toggle_selected_process_visibility(app);
                }
                KeyCode::Char('v') | KeyCode::Char('V') if app.tab == Tab::Processes => {
                    toggle_process_hidden_view(app);
                }
                KeyCode::Char('/') if app.tab == Tab::Processes => {
                    start_process_filter(app);
                }
                KeyCode::Char('1') | KeyCode::Char(' ') => {
                    request_overdrive_confirmation(app);
                }
                KeyCode::Char('2') => {
                    run_restore(app);
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    show_output(
                        app,
                        vec![app.config.ui.language.telemetry_refreshing().to_string()],
                    );
                }
                KeyCode::Char('m') | KeyCode::Char('M') => {
                    app.theme_menu_selected = ThemePreset::ALL
                        .iter()
                        .position(|p| *p == app.theme_preset)
                        .unwrap_or(0);
                    app.screen = Screen::ThemeMenu;
                }
                KeyCode::Tab | KeyCode::Right => {
                    select_next_tab(app);
                }
                KeyCode::BackTab | KeyCode::Left => {
                    select_previous_tab(app);
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.screen = Screen::Quit;
                    return true;
                }
                _ => {}
            },
            Screen::Confirm => match key.code {
                KeyCode::Up => {
                    scroll_confirm_up(app, 1);
                }
                KeyCode::Down => {
                    scroll_confirm_down(app, 1);
                }
                KeyCode::PageUp => {
                    scroll_confirm_up(app, 8);
                }
                KeyCode::PageDown => {
                    scroll_confirm_down(app, 8);
                }
                KeyCode::Home => {
                    reset_confirm_scroll(app);
                }
                KeyCode::End => {
                    end_confirm_scroll(app);
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    clamp_confirm_scroll(app);
                    confirm_pending_action(app);
                }
                KeyCode::Esc
                | KeyCode::Char('q')
                | KeyCode::Char('Q')
                | KeyCode::Char('n')
                | KeyCode::Char('N') => {
                    app.pending_action = None;
                    app.confirm_lines.clear();
                    app.confirm_scroll = 0;
                    app.screen = Screen::Monitor;
                }
                _ => {}
            },
            Screen::Output => match key.code {
                KeyCode::Up => {
                    scroll_output_up(app, 1);
                }
                KeyCode::Down => {
                    scroll_output_down(app, 1);
                }
                KeyCode::PageUp => {
                    scroll_output_up(app, 8);
                }
                KeyCode::PageDown => {
                    scroll_output_down(app, 8);
                }
                KeyCode::Home => {
                    reset_output_scroll(app);
                }
                KeyCode::End => {
                    end_output_scroll(app);
                }
                _ => {
                    app.output_scroll = 0;
                    app.screen = Screen::Monitor;
                }
            },
            Screen::ThemeMenu => match key.code {
                KeyCode::Up => {
                    app.theme_menu_selected = app
                        .theme_menu_selected
                        .saturating_sub(1)
                        .min(ThemePreset::ALL.len() - 1);
                    let preset = ThemePreset::ALL[app.theme_menu_selected];
                    app.theme = preset.theme();
                    app.theme_preset = preset;
                }
                KeyCode::Down => {
                    app.theme_menu_selected =
                        (app.theme_menu_selected + 1) % ThemePreset::ALL.len();
                    let preset = ThemePreset::ALL[app.theme_menu_selected];
                    app.theme = preset.theme();
                    app.theme_preset = preset;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let preset = ThemePreset::ALL[app.theme_menu_selected];
                    app.theme_status = app.theme_watcher.apply_preset(preset, &mut app.theme);
                    app.theme_preset = preset;
                    app.screen = Screen::Monitor;
                }
                KeyCode::Esc
                | KeyCode::Char('q')
                | KeyCode::Char('Q')
                | KeyCode::Char('m')
                | KeyCode::Char('M') => {
                    app.screen = Screen::Monitor;
                }
                _ => {}
            },
            Screen::Quit => {}
        }
    } else if let Event::Mouse(mouse) = event {
        return handle_mouse_event(app, mouse);
    }
    false
}

fn handle_mouse_event(app: &mut App, ev: MouseEvent) -> bool {
    match ev.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let col = ev.column;
            let row = ev.row;

            match app.screen {
                Screen::Monitor => {
                    // Tab row content is at terminal y=4 (layout[1].y=3 + 1 border)
                    if row == 4
                        && let Ok((width, _)) = crossterm::terminal::size()
                        && let Some(tab) = Tab::from_nav_column(col, width)
                    {
                        if tab != app.tab {
                            set_tab(app, tab);
                        }
                        return false;
                    }

                    // Process list click (Processes tab) — content starts at y=7
                    if app.tab == Tab::Processes && row >= 7 {
                        if let Ok((_, h)) = crossterm::terminal::size() {
                            let content_height = h.saturating_sub(9); // header(3)+tabs(3)+footer(3)
                            let visible_rows = (content_height.saturating_sub(2)).max(1);
                            let start = app
                                .process_selected
                                .saturating_sub((visible_rows.saturating_sub(1) / 2) as usize);
                            let click_offset = (row - 7) as usize;
                            let idx = start + click_offset;
                            let sorted = app.visible_processes();
                            if idx < sorted.len() {
                                app.process_selected = idx;
                            }
                        }
                        return false;
                    }

                    // Steam list click — content starts at y=7
                    if app.tab == Tab::Steam && row >= 7 {
                        if let Ok((_, h)) = crossterm::terminal::size() {
                            let content_height = h.saturating_sub(9);
                            let visible_rows = (content_height.saturating_sub(2)).max(1);
                            let start = app
                                .steam
                                .selected
                                .saturating_sub((visible_rows.saturating_sub(1) / 2) as usize);
                            let click_offset = (row - 7) as usize;
                            let idx = start + click_offset;
                            if idx < app.steam.games.len() {
                                app.steam.selected = idx;
                            }
                        }
                        return false;
                    }
                }
                Screen::ThemeMenu => {
                    if let Ok((w, h)) = crossterm::terminal::size() {
                        let modal_x = (w * 25) / 100;
                        let modal_y = (h * 25) / 100;
                        let modal_w = (w * 50) / 100;
                        let modal_h = (h * 50) / 100;

                        // Click inside modal?
                        if col >= modal_x
                            && col < modal_x + modal_w
                            && row >= modal_y
                            && row < modal_y + modal_h
                        {
                            let content_y = modal_y + 1; // after top border
                            let item_offset = row.saturating_sub(content_y) as usize;
                            if item_offset < ThemePreset::ALL.len() {
                                app.theme_menu_selected = item_offset;
                                let preset = ThemePreset::ALL[item_offset];
                                app.theme_status =
                                    app.theme_watcher.apply_preset(preset, &mut app.theme);
                                app.theme_preset = preset;
                                app.screen = Screen::Monitor;
                            } else {
                                // Clicked modal background → close
                                app.screen = Screen::Monitor;
                            }
                        }
                    }
                    return false;
                }
                _ => {}
            }
            false
        }
        MouseEventKind::ScrollUp => {
            handle_mouse_scroll(app, -1);
            false
        }
        MouseEventKind::ScrollDown => {
            handle_mouse_scroll(app, 1);
            false
        }
        _ => false,
    }
}

fn handle_mouse_scroll(app: &mut App, direction: i8) {
    match app.screen {
        Screen::Monitor => match app.tab {
            Tab::History => {
                if direction > 0 {
                    scroll_history_down(app, 3);
                } else {
                    scroll_history_up(app, 3);
                }
            }
            Tab::Processes => {
                if direction > 0 {
                    select_next_process(app);
                } else {
                    select_previous_process(app);
                }
            }
            Tab::Steam => {
                if direction > 0 {
                    app.steam.select_next();
                } else {
                    app.steam.select_previous();
                }
            }
            _ => {}
        },
        Screen::Confirm => {
            if direction > 0 {
                scroll_confirm_down(app, 3);
            } else {
                scroll_confirm_up(app, 3);
            }
        }
        Screen::Output => {
            if direction > 0 {
                scroll_output_down(app, 3);
            } else {
                scroll_output_up(app, 3);
            }
        }
        Screen::ThemeMenu => {
            if direction > 0 {
                let next = app.theme_menu_selected + 1;
                if next < ThemePreset::ALL.len() {
                    app.theme_menu_selected = next;
                }
            } else if direction < 0 {
                app.theme_menu_selected = app.theme_menu_selected.saturating_sub(1);
            }
        }
        Screen::Quit => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_process_preview_should_include_exe_path_when_available() {
        let group = ProcessGroup {
            count: 2,
            memory_mb: 128.0,
            exe_path: Some("C:\\Program Files\\Dropbox\\Dropbox.exe".to_string()),
        };

        let lines = format_target_process_preview("Dropbox.exe", &group, Language::Spanish);

        assert!(lines[1].contains("Dropbox.exe"));
    }

    #[test]
    fn target_process_preview_should_explain_missing_exe_path() {
        let group = ProcessGroup {
            count: 1,
            memory_mb: 64.0,
            exe_path: None,
        };

        let lines = format_target_process_preview("helper.exe", &group, Language::Spanish);

        assert!(lines[1].contains("ruta no disponible"));
    }

    #[test]
    fn max_confirm_scroll_should_leave_empty_preview_at_top() {
        assert_eq!(max_confirm_scroll(0), 0);
    }

    #[test]
    fn max_confirm_scroll_should_stop_at_last_line() {
        assert_eq!(max_confirm_scroll(12), 11);
    }

    #[test]
    fn max_output_scroll_should_leave_empty_log_at_top() {
        assert_eq!(max_output_scroll(0), 0);
    }

    #[test]
    fn max_output_scroll_should_stop_at_last_line() {
        assert_eq!(max_output_scroll(9), 8);
    }

    #[test]
    fn max_history_scroll_should_leave_empty_history_at_top() {
        assert_eq!(max_history_scroll(0), 0);
    }

    #[test]
    fn max_history_scroll_should_stop_at_last_line() {
        assert_eq!(max_history_scroll(4), 3);
    }

    #[test]
    fn nav_slots_should_fill_available_tab_width() {
        let content_width = 186;
        let slots = Tab::nav_slots(content_width);

        assert_eq!(slots.first().map(|slot| slot.start), Some(0));
        assert_eq!(
            slots.last().map(|slot| slot.start + slot.width),
            Some(content_width)
        );
    }

    #[test]
    fn tab_click_hitboxes_should_match_rendered_slots() {
        let total_width = 188;
        for slot in Tab::nav_slots(total_width - 2) {
            let column = 1 + slot.start + (slot.width / 2);

            assert_eq!(Tab::from_nav_column(column, total_width), Some(slot.tab));
        }
    }
}

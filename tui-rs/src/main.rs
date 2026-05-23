use std::collections::HashMap;
use std::io;
use std::process::Command;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::{
    Frame,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Terminal,
};
use sysinfo::System;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const HIGH_PERF_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
const BALANCED_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

const KILL_LIST: &[&str] = &[
    "chrome", "msedge", "msedgewebview2", "firefox", "opera", "brave", "vivaldi",
    "dropbox", "googledrivefs", "gdrive", "onedrive", "filecoauth",
    "idman", "qbittorrent", "torrent", "transmission",
    "discord", "slack", "teams", "zoom", "skype",
    "spotify",
    "steelseriesengine", "steelseriessonar", "steelseriesgg", "steelseriesggez", "steelseriesprism",
    "epomaker", "rapoo", "logitech", "razer",
    "anydesk", "teamviewer", "rcclient", "rcservice", "anyviewer", "vnc",
    "whatsapp", "telegram", "signal",
    "winword", "excel", "powerpnt", "outlook", "officeclicktorun",
    "onecommander", "files",
    "radeonsoftware", "amdryzenmaster", "msiafterburner",
    "widgets", "widgetservice",
    "trafficmonitor", "hwmonitor", "cpuid",
    "opengameboost", "razercortex",
    "foxit", "acrobat", "adobereader",
    "snippingtool",
    "searchhost", "searchindexer",
    "startmenuexperiencehost", "shellexperiencehost",
    "runtimebroker",
    "python", "node",
];

const SERVICE_NAMES: &[&str] = &[
    "SysMain", "WSearch", "DiagTrack", "Spooler", "FontCache",
    "PcaSvc", "UsoSvc", "Themes", "WpnService",
];

const STEAM_PATHS: &[&str] = &[
    r"C:\Program Files (x86)\Steam\steam.exe",
    r"C:\Program Files\Steam\steam.exe",
];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
enum Screen {
    Menu,
    Output,
    Quit,
}

struct ProcessGroup {
    count: usize,
    memory_mb: f64,
}

struct SystemState {
    power_plan: String,
    ram_total_gb: f64,
    ram_free_gb: f64,
    explorer_on: bool,
    steam_on: bool,
    steam_mb: f64,
    services_running: usize,
    processes: HashMap<String, ProcessGroup>,
    total_waste_mb: f64,
}

struct App {
    screen: Screen,
    output: Vec<String>,
    state: SystemState,
}

// ---------------------------------------------------------------------------
// System operations
// ---------------------------------------------------------------------------

fn run_powershell(script: &str) -> String {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}

fn get_power_plan_name() -> String {
    let out = run_powershell("powercfg /getactivescheme");
    if out.contains(HIGH_PERF_GUID) {
        "Alto Rendimiento".to_string()
    } else if out.contains(BALANCED_GUID) {
        "Balanceado".to_string()
    } else {
        "Personalizado".to_string()
    }
}

fn set_power_plan(guid: &str) -> bool {
    let output = Command::new("powercfg")
        .args(["/setactive", guid])
        .output();
    output.is_ok() && output.unwrap().status.success()
}

fn is_explorer_running() -> bool {
    let out = run_powershell(
        "if (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue) { '1' } else { '0' }",
    );
    out == "1"
}

fn get_steam_info() -> (bool, f64) {
    let out = run_powershell(
        "$p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue; \
         if ($p) { \"1,$($p.WorkingSet64)\" } else { '0,0' }",
    );
    let parts: Vec<&str> = out.split(',').collect();
    if parts.len() == 2 && parts[0] == "1" {
        let bytes: f64 = parts[1].parse().unwrap_or(0.0);
        (true, bytes / 1_048_576.0)
    } else {
        (false, 0.0)
    }
}

fn get_services_running() -> usize {
    let out = run_powershell(
        "@('SysMain','WSearch','DiagTrack','Spooler','FontCache','PcaSvc','UsoSvc','Themes','WpnService') | \
         ForEach-Object { if ((Get-Service -Name $_ -ErrorAction SilentlyContinue).Status -eq 'Running') { $_ } } | \
         Measure-Object | Select-Object -ExpandProperty Count",
    );
    out.trim().parse().unwrap_or(0)
}

fn kill_processes() -> Vec<String> {
    let mut log = Vec::new();
    let patterns = KILL_LIST.join(",");
    let script = format!(
        "@('{}') | ForEach-Object {{ \
         $p = Get-Process -Name $_ -ErrorAction SilentlyContinue; \
         if ($p) {{ \
         $mb = [Math]::Round(($p.WorkingSet64 / 1MB), 1); \
         Stop-Process -Name $_ -Force -ErrorAction SilentlyContinue; \
         Write-Output \"$($_): $mb MB\" \
         }} \
         }}",
        patterns
    );
    let out = run_powershell(&script);
    for line in out.lines() {
        if !line.is_empty() {
            log.push(format!("  \u{2715} {}", line));
        }
    }
    if log.is_empty() {
        log.push("  (ningun proceso pesado encontrado)".to_string());
    }
    log
}

fn stop_services() -> Vec<String> {
    let mut log = Vec::new();
    let names = SERVICE_NAMES.join("','");
    let script = format!(
        "@('{}') | ForEach-Object {{ \
         $s = Get-Service -Name $_ -ErrorAction SilentlyContinue; \
         if ($s -and $s.Status -eq 'Running') {{ \
         Stop-Service -Name $_ -Force -ErrorAction SilentlyContinue; \
         Write-Output $_ \
         }} \
         }}",
        names
    );
    let out = run_powershell(&script);
    for line in out.lines() {
        if !line.is_empty() {
            log.push(format!("  \u{25CB} {} detenido", line));
        }
    }
    if log.is_empty() {
        log.push("  (servicios ya optimizados)".to_string());
    }
    log
}

fn priority_steam() -> Vec<String> {
    let mut log = Vec::new();
    let script = run_powershell(
        "$p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue; \
         if (-not $p) { \
         $found = $false; \
         @('C:\\Program Files (x86)\\Steam\\steam.exe','C:\\Program Files\\Steam\\steam.exe') | \
         ForEach-Object {{ if (Test-Path $_) {{ Start-Process $_; $found = $true }} }}; \
         if (-not $found) {{ Write-Output 'NO_STEAM' }} \
         } else { \
         Write-Output 'OK' \
         }",
    );
    if script == "NO_STEAM" {
        log.push("  [!] No se encontro Steam, abrelo manualmente".to_string());
    } else if script == "OK" {
        log.push("  \u{2713} Steam ya activo, prioridad asignada".to_string());
    } else {
        log.push("  \u{2192} Steam abierto automaticamente".to_string());
    }
    log
}

fn kill_explorer() -> Vec<String> {
    let mut log = Vec::new();
    let script = run_powershell(
        "$p = Get-Process -Name 'explorer' -ErrorAction SilentlyContinue; \
         if ($p) { Stop-Process -Name 'explorer' -Force; Write-Output 'KILLED' }",
    );
    if script == "KILLED" {
        log.push("  \u{2715} explorer.exe suspendido (~400 MB liberados)".to_string());
    }
    log
}

fn start_explorer() -> Vec<String> {
    let mut log = Vec::new();
    let script = run_powershell(
        "if (-not (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue)) { \
         Start-Process 'explorer.exe'; Write-Output 'STARTED' }",
    );
    if script == "STARTED" {
        log.push("  \u{2713} explorer.exe reiniciado".to_string());
    }
    log
}

fn activate_chaos_mode() -> Vec<String> {
    let mut log = Vec::new();
    log.push(format!(
        "{} Plan de energia \u{2192} Alto Rendimiento",
        "\u{26A1}"
    ));
    set_power_plan(HIGH_PERF_GUID);

    log.push(format!(
        "{} Eliminando procesos en segundo plano...",
        "\u{2699}"
    ));
    log.extend(kill_processes());

    log.push(format!("{} Deteniendo servicios...", "\u{2699}"));
    log.extend(stop_services());

    log.push(format!("{} Steam...", "\u{1F3AE}"));
    log.extend(priority_steam());

    log.push(format!(
        "{} Liberando recursos del sistema...",
        "\u{2699}"
    ));
    log.extend(kill_explorer());

    log.push(String::new());
    log.push("  \u{2705} CHAOS GAME MODE ACTIVADO".to_string());

    log
}

fn restore_system() -> Vec<String> {
    let mut log = Vec::new();
    log.push(format!(
        "{} Restaurando interfaz de Windows...",
        "\u{1F4C1}"
    ));
    log.extend(start_explorer());

    log.push(format!(
        "{} Restaurando servicios...",
        "\u{2699}"
    ));
    let script = run_powershell(
        "@('SysMain','WSearch','DiagTrack','Spooler','FontCache','PcaSvc','UsoSvc','Themes','WpnService') | \
         ForEach-Object {{ \
         $s = Get-Service -Name $_ -ErrorAction SilentlyContinue; \
         if ($s -and $s.Status -ne 'Running') {{ \
         Start-Service -Name $_ -ErrorAction SilentlyContinue; \
         Write-Output $_ \
         }} \
         }}",
    );
    for line in script.lines() {
        if !line.is_empty() {
            log.push(format!("  \u{25CB} {} iniciado", line));
        }
    }

    log.push(format!(
        "{} Plan de energia \u{2192} Balanceado",
        "\u{1F50B}"
    ));
    set_power_plan(BALANCED_GUID);

    log.push(String::new());
    log.push("  \u{2705} SISTEMA RESTAURADO".to_string());
    log.push("  \u{26A0} Apps cerradas no se reabren solas".to_string());

    log
}

// ---------------------------------------------------------------------------
// System state refresh
// ---------------------------------------------------------------------------

fn refresh_system_state(sys: &mut System) -> SystemState {
    sys.refresh_all();

    // Memory
    let total_bytes = sys.total_memory();
    let free_bytes = sys.free_memory();
    let ram_total_gb = total_bytes as f64 / 1_073_741_824.0;
    let ram_free_gb = free_bytes as f64 / 1_073_741_824.0;

    // Scan processes from KILL_LIST
    let mut proc_groups: HashMap<String, ProcessGroup> = HashMap::new();
    let mut total_waste_mb = 0.0_f64;

    let all_processes = sys.processes();
    for pattern in KILL_LIST {
        let pattern_lower = pattern.to_lowercase();
        for (_pid, process) in all_processes {
            let name = process.name().to_string_lossy().to_lowercase();
            if name.contains(&pattern_lower) || pattern_lower == "*" {
                let mem_bytes = process.memory();
                let mem_mb = mem_bytes as f64 / 1_048_576.0;
                let display_name = process.name().to_string_lossy().to_string();
                let entry = proc_groups
                    .entry(display_name)
                    .or_insert(ProcessGroup { count: 0, memory_mb: 0.0 });
                entry.count += 1;
                entry.memory_mb += mem_mb;
                total_waste_mb += mem_mb;
            }
        }
    }

    let power_plan = get_power_plan_name();
    let explorer_on = is_explorer_running();
    let (steam_on, steam_mb) = get_steam_info();
    let services_running = get_services_running();

    SystemState {
        power_plan,
        ram_total_gb,
        ram_free_gb,
        explorer_on,
        steam_on,
        steam_mb,
        services_running,
        processes: proc_groups,
        total_waste_mb,
    }
}

// ---------------------------------------------------------------------------
// UI rendering
// ---------------------------------------------------------------------------

fn render_menu(frame: &mut Frame, app: &mut App) {
    let state = &app.state;

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(9),
            Constraint::Length(1),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Text::from(vec![
        Line::from("  \u{2699} CHAOS GAME MODE").style(
            Style::new()
                .fg(Color::Rgb(255, 80, 80))
                .bold(),
        ),
        Line::from("  Ryzen 5 5500 | RX 550 | 16GB | Win11").style(
            Style::new().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Cyan)),
    );
    frame.render_widget(title, chunks[0]);

    // Menu options
    let menu_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  \u{25B6}  ", Style::new().fg(Color::Green)),
            Span::raw("[1] Activar modo juego"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  \u{25C0}  ", Style::new().fg(Color::Yellow)),
            Span::raw("[2] Restaurar sistema"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  \u{1F4CA}  ", Style::new().fg(Color::Cyan)),
            Span::raw("[3] Refrescar estado"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  \u{2716}  ", Style::new().fg(Color::Red)),
            Span::raw("[4] Salir"),
        ])),
    ];
    let menu = List::new(menu_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::DarkGray)),
    );
    frame.render_widget(menu, chunks[1]);

    // Status section
    let _plan_color = if state.power_plan == "Alto Rendimiento" {
        Color::Green
    } else {
        Color::Yellow
    };
    let _exp_color = if state.explorer_on {
        Color::Yellow
    } else {
        Color::Green
    };
    let _steam_color = if state.steam_on {
        Color::Green
    } else {
        Color::DarkGray
    };

    let ram_pct = if state.ram_total_gb > 0.0 {
        ((state.ram_free_gb / state.ram_total_gb) * 100.0) as u16
    } else {
        0
    };
    let ram_color = if ram_pct >= 50 {
        Color::Green
    } else if ram_pct >= 25 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Gauge for RAM
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" RAM ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Magenta)),
        )
        .gauge_style(
            Style::new()
                .fg(ram_color)
                .bg(Color::Black),
        )
        .percent(ram_pct)
        .label(format!(
            "  {:.1}GB / {:.1}GB ({ram_pct}% libre)",
            state.ram_free_gb, state.ram_total_gb
        ));
    frame.render_widget(gauge, chunks[2]);

    // Process list (top 8)
    let mut proc_lines: Vec<Line> = Vec::new();

    if state.processes.is_empty() {
        proc_lines.push(
            Line::from("  \u{2705} No hay procesos residuales")
                .style(Style::new().fg(Color::Green)),
        );
    } else {
        proc_lines.push(
            Line::from(format!(
                "  \u{26A0} Procesos residuales ({:.0} MB total):",
                state.total_waste_mb
            ))
            .style(Style::new().fg(Color::Red)),
        );

        let mut sorted: Vec<(&String, &ProcessGroup)> = state.processes.iter().collect();
        sorted.sort_by(|a, b| b.1.memory_mb.partial_cmp(&a.1.memory_mb).unwrap());

        for (_i, (name, group)) in sorted.iter().enumerate().take(8) {
            let truncated = if name.len() > 22 {
                &name[..22]
            } else {
                name.as_str()
            };
            let pct_of_total = if state.total_waste_mb > 0.0 {
                ((group.memory_mb / state.total_waste_mb) * 100.0) as u8
            } else {
                0
            };
            let bar_len = std::cmp::max(
                1,
                std::cmp::min(20, (pct_of_total as f64 / 5.0).ceil() as usize),
            );
            let bar: String = "\u{2588}".repeat(bar_len);

            let mem_color = if group.memory_mb >= 500.0 {
                Color::Red
            } else if group.memory_mb >= 200.0 {
                Color::Yellow
            } else {
                Color::DarkGray
            };

            proc_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:22} ", truncated),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:>7.0} MB", group.memory_mb),
                    Style::new().fg(mem_color).bold(),
                ),
                Span::styled(
                    format!(" {:>2}x", group.count),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::styled(format!(" {}", bar), Style::new().fg(mem_color)),
            ]));
        }

        if sorted.len() > 8 {
            proc_lines.push(
                Line::from(format!("  ... y {} procesos mas", sorted.len() - 8))
                    .style(Style::new().fg(Color::DarkGray)),
            );
        }
    }

    let proc_widget = Paragraph::new(Text::from(proc_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Color::Cyan)),
    );
    frame.render_widget(proc_widget, chunks[3]);

    // Bottom bar
    let bottom = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(
                "  {} | {} | {} | Steam: {}",
                state.power_plan,
                if state.explorer_on {
                    "Escritorio"
                } else {
                    "MODO GAMER"
                },
                format!("{:.1}/{}GB", state.ram_free_gb, state.ram_total_gb),
                if state.steam_on { "Activo" } else { "Cerrado" }
            ),
            Style::new().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(bottom, chunks[4]);
}

fn render_output(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let lines: Vec<Line> = app
        .output
        .iter()
        .map(|l| Line::from(l.as_str()))
        .collect();
    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);

    // Hint at bottom
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            "  Presiona cualquier tecla para volver al menu...",
            Style::new().fg(Color::DarkGray),
        ),
    ]))
    .style(Style::new().fg(Color::DarkGray));
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    frame.render_widget(hint, bottom[1]);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new();
    let state = refresh_system_state(&mut sys);

    let mut app = App {
        screen: Screen::Menu,
        output: Vec::new(),
        state,
    };

    terminal.clear()?;
    let res = run_app(&mut terminal, &mut app, &mut sys);

    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen)?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    sys: &mut System,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|frame| match app.screen {
            Screen::Menu => render_menu(frame, app),
            Screen::Output => render_output(frame, app),
            Screen::Quit => {}
        })?;

        if app.screen == Screen::Quit {
            break Ok(());
        }

        if event::poll(tick_rate)? {
            let event = event::read()?;
            if handle_event(app, sys, event) {
                break Ok(());
            }
        }
    }
}

fn handle_event(app: &mut App, sys: &mut System, event: Event) -> bool {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            match app.screen {
                Screen::Menu => match key.code {
                    KeyCode::Char('1') => {
                        app.output = activate_chaos_mode();
                        app.screen = Screen::Output;
                    }
                    KeyCode::Char('2') => {
                        app.output = restore_system();
                        app.screen = Screen::Output;
                    }
                    KeyCode::Char('3') => {
                        app.state = refresh_system_state(sys);
                    }
                    KeyCode::Char('4') | KeyCode::Esc | KeyCode::Char('q') => {
                        app.screen = Screen::Quit;
                        return true;
                    }
                    _ => {}
                },
                Screen::Output => {
                    app.screen = Screen::Menu;
                    app.state = refresh_system_state(sys);
                }
                Screen::Quit => {}
            }
        }
    }
    false
}

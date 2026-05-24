use crate::app::Tab;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    Spanish,
    English,
}

#[allow(dead_code)]
impl Language {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "es" | "esp" | "spanish" | "espanol" => Some(Self::Spanish),
            "en" | "eng" | "english" | "ingles" => Some(Self::English),
            _ => None,
        }
    }

    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::Spanish => "es",
            Self::English => "en",
        }
    }

    pub(crate) const fn tab_label(self, tab: Tab) -> &'static str {
        match (self, tab) {
            (_, Tab::Dashboard) => "DASHBOARD",
            (_, Tab::Steam) => "STEAM",
            (_, Tab::Frames) => "FRAMES",
            (Self::Spanish, Tab::Processes) => "PROCESOS",
            (Self::English, Tab::Processes) => "PROCESSES",
            (_, Tab::Boost) => "OVERDRIVE",
            (Self::Spanish, Tab::System) => "SISTEMA",
            (Self::English, Tab::System) => "SYSTEM",
            (Self::Spanish, Tab::History) => "HISTORIAL",
            (Self::English, Tab::History) => "HISTORY",
            (Self::Spanish, Tab::Settings) => "AJUSTES",
            (Self::English, Tab::Settings) => "SETTINGS",
        }
    }

    pub(crate) const fn tab_compact_label(self, tab: Tab) -> &'static str {
        match (self, tab) {
            (_, Tab::Dashboard) => "DASH",
            (_, Tab::Steam) => "STEAM",
            (_, Tab::Frames) => "FPS",
            (Self::Spanish, Tab::Processes) => "PROC",
            (Self::English, Tab::Processes) => "PROC",
            (_, Tab::Boost) => "ODRV",
            (Self::Spanish, Tab::System) => "SIS",
            (Self::English, Tab::System) => "SYS",
            (Self::Spanish, Tab::History) => "HIST",
            (Self::English, Tab::History) => "HIST",
            (Self::Spanish, Tab::Settings) => "AJUS",
            (Self::English, Tab::Settings) => "SET",
        }
    }

    pub(crate) const fn ready(self) -> &'static str {
        match self {
            Self::Spanish => "READY",
            Self::English => "READY",
        }
    }

    pub(crate) const fn profile(self) -> &'static str {
        match self {
            Self::Spanish => "PERFIL",
            Self::English => "PROFILE",
        }
    }

    pub(crate) const fn preset(self) -> &'static str {
        match self {
            Self::Spanish => "TEMA",
            Self::English => "PRESET",
        }
    }

    pub(crate) const fn session(self) -> &'static str {
        match self {
            Self::Spanish => "SESION",
            Self::English => "SESSION",
        }
    }

    pub(crate) const fn preview(self) -> &'static str {
        match self {
            Self::Spanish => "preview",
            Self::English => "preview",
        }
    }

    pub(crate) const fn restore(self) -> &'static str {
        match self {
            Self::Spanish => "restaurar",
            Self::English => "restore",
        }
    }

    pub(crate) const fn launch(self) -> &'static str {
        match self {
            Self::Spanish => "lanzar",
            Self::English => "launch",
        }
    }

    pub(crate) const fn install(self) -> &'static str {
        match self {
            Self::Spanish => "instalar",
            Self::English => "install",
        }
    }

    pub(crate) const fn validate(self) -> &'static str {
        match self {
            Self::Spanish => "validar",
            Self::English => "validate",
        }
    }

    pub(crate) const fn uninstall(self) -> &'static str {
        match self {
            Self::Spanish => "desinstalar",
            Self::English => "uninstall",
        }
    }

    pub(crate) const fn properties(self) -> &'static str {
        match self {
            Self::Spanish => "propiedades",
            Self::English => "props",
        }
    }

    pub(crate) const fn downloads(self) -> &'static str {
        match self {
            Self::Spanish => "descargas",
            Self::English => "downloads",
        }
    }

    pub(crate) const fn scan(self) -> &'static str {
        match self {
            Self::Spanish => "scan",
            Self::English => "scan",
        }
    }

    pub(crate) const fn end(self) -> &'static str {
        match self {
            Self::Spanish => "cerrar",
            Self::English => "end",
        }
    }

    pub(crate) const fn keep(self) -> &'static str {
        match self {
            Self::Spanish => "proteger",
            Self::English => "keep",
        }
    }

    pub(crate) const fn target(self) -> &'static str {
        "target"
    }

    pub(crate) const fn neutral(self) -> &'static str {
        "neutral"
    }

    pub(crate) const fn hide(self) -> &'static str {
        match self {
            Self::Spanish => "ocultar",
            Self::English => "hide",
        }
    }

    pub(crate) const fn hidden(self) -> &'static str {
        match self {
            Self::Spanish => "ocultos",
            Self::English => "hidden",
        }
    }

    pub(crate) const fn unhide(self) -> &'static str {
        match self {
            Self::Spanish => "mostrar",
            Self::English => "unhide",
        }
    }

    pub(crate) const fn active(self) -> &'static str {
        match self {
            Self::Spanish => "activos",
            Self::English => "active",
        }
    }

    pub(crate) const fn filter(self) -> &'static str {
        match self {
            Self::Spanish => "filtrar",
            Self::English => "filter",
        }
    }

    pub(crate) const fn reload(self) -> &'static str {
        match self {
            Self::Spanish => "recargar",
            Self::English => "reload",
        }
    }

    pub(crate) const fn scroll(self) -> &'static str {
        "scroll"
    }

    pub(crate) const fn page(self) -> &'static str {
        match self {
            Self::Spanish => "pagina",
            Self::English => "page",
        }
    }

    pub(crate) const fn probe(self) -> &'static str {
        "probe"
    }

    pub(crate) const fn theme(self) -> &'static str {
        match self {
            Self::Spanish => "tema",
            Self::English => "theme",
        }
    }

    pub(crate) const fn nav(self) -> &'static str {
        "nav"
    }

    pub(crate) const fn exit(self) -> &'static str {
        match self {
            Self::Spanish => "salir",
            Self::English => "exit",
        }
    }

    pub(crate) const fn return_label(self) -> &'static str {
        match self {
            Self::Spanish => "volver",
            Self::English => "return",
        }
    }

    pub(crate) const fn action_log(self) -> &'static str {
        match self {
            Self::Spanish => "REGISTRO DE ACCIONES",
            Self::English => "ACTION LOG",
        }
    }

    pub(crate) const fn output_empty(self) -> &'static str {
        match self {
            Self::Spanish => "No hay acciones registradas.",
            Self::English => "No actions recorded.",
        }
    }

    pub(crate) const fn output_hint(self) -> &'static str {
        match self {
            Self::Spanish => "Usa Overdrive (1) o Restore (2) para ver el output.",
            Self::English => "Run Overdrive (1) or Restore (2) to see results.",
        }
    }

    pub(crate) const fn confirm_overdrive(self) -> &'static str {
        match self {
            Self::Spanish => "CONFIRMAR OVERDRIVE",
            Self::English => "CONFIRM OVERDRIVE",
        }
    }

    pub(crate) const fn confirm(self) -> &'static str {
        match self {
            Self::Spanish => "confirmar",
            Self::English => "confirm",
        }
    }

    pub(crate) const fn cancel(self) -> &'static str {
        match self {
            Self::Spanish => "cancelar",
            Self::English => "cancel",
        }
    }

    pub(crate) const fn safety_check(self) -> &'static str {
        match self {
            Self::Spanish => "CONTROL DE SEGURIDAD",
            Self::English => "SAFETY CHECK",
        }
    }

    pub(crate) const fn select_theme(self) -> &'static str {
        match self {
            Self::Spanish => "SELECCIONAR TEMA",
            Self::English => "SELECT THEME",
        }
    }

    pub(crate) const fn navigate(self) -> &'static str {
        match self {
            Self::Spanish => "navegar",
            Self::English => "navigate",
        }
    }

    pub(crate) const fn select(self) -> &'static str {
        match self {
            Self::Spanish => "seleccionar",
            Self::English => "select",
        }
    }

    pub(crate) const fn active_suffix(self) -> &'static str {
        match self {
            Self::Spanish => "activo",
            Self::English => "active",
        }
    }

    pub(crate) const fn loading_steam_scan(self) -> &'static str {
        match self {
            Self::Spanish => "auto-detect esperando scan de Steam",
            Self::English => "auto detect waiting for Steam scan",
        }
    }

    pub(crate) fn auto_detect_ready(self, games: usize) -> String {
        match self {
            Self::Spanish => format!("auto-detect listo: {games} juegos"),
            Self::English => format!("auto detect ready: {games} games"),
        }
    }

    pub(crate) const fn auto_detect_no_games(self) -> &'static str {
        match self {
            Self::Spanish => "auto-detect sin juegos de Steam",
            Self::English => "auto detect unavailable: no Steam games",
        }
    }

    pub(crate) const fn manual_session_active(self) -> &'static str {
        match self {
            Self::Spanish => "sesion manual activa",
            Self::English => "manual session active",
        }
    }

    pub(crate) const fn auto_detect_armed(self) -> &'static str {
        match self {
            Self::Spanish => "auto-detect armado",
            Self::English => "auto detect armed",
        }
    }

    pub(crate) fn tracking(self, name: &str) -> String {
        match self {
            Self::Spanish => format!("siguiendo {name}"),
            Self::English => format!("tracking {name}"),
        }
    }

    pub(crate) fn detected(self, name: &str) -> String {
        match self {
            Self::Spanish => format!("detectado {name}"),
            Self::English => format!("detected {name}"),
        }
    }

    pub(crate) const fn auto_session_ended(self) -> &'static str {
        match self {
            Self::Spanish => "sesion automatica cerrada",
            Self::English => "auto session ended",
        }
    }

    pub(crate) const fn auto_detect_paused(self) -> &'static str {
        match self {
            Self::Spanish => "auto-detect paused: game cerrado",
            Self::English => "auto detect paused for ended game",
        }
    }

    pub(crate) const fn no_steam_game_selected(self) -> &'static str {
        match self {
            Self::Spanish => "  No hay juego seleccionado.",
            Self::English => "  No Steam game selected.",
        }
    }

    pub(crate) const fn rescan_steam_hint(self) -> &'static str {
        match self {
            Self::Spanish => "  Pulsa S en Steam para escanear otra vez.",
            Self::English => "  Press S in the Steam tab to rescan the library.",
        }
    }

    pub(crate) const fn overdrive_preview_title(self) -> &'static str {
        match self {
            Self::Spanish => "PREVIEW OVERDRIVE",
            Self::English => "OVERDRIVE PREVIEW",
        }
    }

    pub(crate) const fn profile_label(self) -> &'static str {
        match self {
            Self::Spanish => "Perfil",
            Self::English => "Profile",
        }
    }

    pub(crate) const fn launch_after_label(self) -> &'static str {
        match self {
            Self::Spanish => "Despues lanza",
            Self::English => "Then launch",
        }
    }

    pub(crate) const fn configured_processes_label(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos en lista",
            Self::English => "Configured processes",
        }
    }

    pub(crate) const fn protected_processes_label(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos protegidos",
            Self::English => "Protected processes",
        }
    }

    pub(crate) const fn hidden_processes_label(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos ocultos",
            Self::English => "Hidden processes",
        }
    }

    pub(crate) const fn detected_processes_label(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos live",
            Self::English => "Processes detected now",
        }
    }

    pub(crate) const fn configured_services_label(self) -> &'static str {
        match self {
            Self::Spanish => "Servicios configurados",
            Self::English => "Configured services",
        }
    }

    pub(crate) const fn explorer_will_stop(self) -> &'static str {
        match self {
            Self::Spanish => "se cierra",
            Self::English => "will stop",
        }
    }

    pub(crate) const fn explorer_kept(self) -> &'static str {
        match self {
            Self::Spanish => "se queda abierto",
            Self::English => "kept running",
        }
    }

    pub(crate) const fn energy_label(self) -> &'static str {
        match self {
            Self::Spanish => "Power plan",
            Self::English => "Power",
        }
    }

    pub(crate) const fn high_performance_plan(self) -> &'static str {
        match self {
            Self::Spanish => "High Performance",
            Self::English => "High Performance",
        }
    }

    pub(crate) const fn balanced_plan(self) -> &'static str {
        match self {
            Self::Spanish => "Balanced",
            Self::English => "Balanced",
        }
    }

    pub(crate) const fn no_changes(self) -> &'static str {
        match self {
            Self::Spanish => "unchanged",
            Self::English => "unchanged",
        }
    }

    pub(crate) const fn overdrive_targets_heading(self) -> &'static str {
        match self {
            Self::Spanish => "Targets de Overdrive si confirmas:",
            Self::English => "Processes Overdrive will try to close now:",
        }
    }

    pub(crate) const fn confirm_hint(self) -> &'static str {
        match self {
            Self::Spanish => "Y/ENTER confirma / N/ESC cancela",
            Self::English => "Y/ENTER confirms / N/ESC cancels",
        }
    }

    pub(crate) const fn no_overdrive_targets(self) -> &'static str {
        match self {
            Self::Spanish => "  (sin targets activos en este profile)",
            Self::English => "  (no target process detected for this profile)",
        }
    }

    pub(crate) const fn exe_path_unavailable(self) -> &'static str {
        match self {
            Self::Spanish => "ruta no disponible",
            Self::English => "path unavailable",
        }
    }

    pub(crate) const fn steam_uninstall_title(self) -> &'static str {
        match self {
            Self::Spanish => "CONFIRMAR DESINSTALACION STEAM",
            Self::English => "STEAM UNINSTALL CONFIRMATION",
        }
    }

    pub(crate) const fn game_label(self) -> &'static str {
        match self {
            Self::Spanish => "Juego",
            Self::English => "Game",
        }
    }

    pub(crate) const fn install_path_label(self) -> &'static str {
        match self {
            Self::Spanish => "Instalacion",
            Self::English => "Install path",
        }
    }

    pub(crate) const fn library_label(self) -> &'static str {
        match self {
            Self::Spanish => "Biblioteca",
            Self::English => "Library",
        }
    }

    pub(crate) const fn steam_uninstall_safe_1(self) -> &'static str {
        match self {
            Self::Spanish => "Esta accion NO borra archivos directamente desde Chaos Game Mode.",
            Self::English => "This action does NOT delete files directly from Chaos Game Mode.",
        }
    }

    pub(crate) const fn steam_uninstall_safe_2(self) -> &'static str {
        match self {
            Self::Spanish => "La app solo abrira la confirmacion oficial de Steam.",
            Self::English => "The app only opens Steam's official confirmation.",
        }
    }

    pub(crate) const fn steam_uninstall_safe_3(self) -> &'static str {
        match self {
            Self::Spanish => "Steam puede cerrar descargas o procesos relacionados.",
            Self::English => "Steam may stop related downloads or processes.",
        }
    }

    pub(crate) const fn steam_uninstall_hint(self) -> &'static str {
        match self {
            Self::Spanish => "Y/ENTER abre steam://uninstall / N/ESC cancela",
            Self::English => "Y/ENTER opens steam://uninstall / N/ESC cancels",
        }
    }

    pub(crate) fn history_status_loaded(self, visible: usize, total: usize) -> String {
        if total == 0 {
            return match self {
                Self::Spanish => "sin historial todavia".to_string(),
                Self::English => "no history yet".to_string(),
            };
        }
        if total > visible {
            return match self {
                Self::Spanish => format!("mostrando ultimas {visible}/{total} lineas"),
                Self::English => format!("showing latest {visible}/{total} lines"),
            };
        }
        match self {
            Self::Spanish => format!("{visible} lineas cargadas"),
            Self::English => format!("{visible} lines loaded"),
        }
    }

    pub(crate) fn history_read_error(self, err: &std::io::Error) -> String {
        match self {
            Self::Spanish => format!("error de historial: {err}"),
            Self::English => format!("history error: {err}"),
        }
    }

    pub(crate) fn saved_history(self, path: &std::path::Path) -> String {
        match self {
            Self::Spanish => format!("  Historial guardado: {}", path.display()),
            Self::English => format!("  History saved: {}", path.display()),
        }
    }

    pub(crate) fn history_save_error(self, err: &std::io::Error) -> String {
        match self {
            Self::Spanish => format!("  [history] no se pudo guardar: {err}"),
            Self::English => format!("  [history] could not save: {err}"),
        }
    }

    pub(crate) const fn telemetry_refreshing(self) -> &'static str {
        match self {
            Self::Spanish => "  Telemetria en segundo plano: actualizando snapshot...",
            Self::English => "  Background telemetry: refreshing snapshot...",
        }
    }

    pub(crate) const fn session_started(self) -> &'static str {
        match self {
            Self::Spanish => "Sesion iniciada",
            Self::English => "Session started",
        }
    }

    pub(crate) const fn session_started_auto(self) -> &'static str {
        match self {
            Self::Spanish => "Sesion iniciada automaticamente",
            Self::English => "Session started automatically",
        }
    }

    pub(crate) const fn no_active_session(self) -> &'static str {
        match self {
            Self::Spanish => "  No hay sesion activa.",
            Self::English => "  No active session.",
        }
    }

    pub(crate) fn completed_session_label(
        self,
        name: &str,
        duration: &str,
        source: &str,
    ) -> String {
        let source = match (self, source) {
            (Self::Spanish, "auto-detected") => "auto-detectado",
            (_, "auto-detected") => "auto-detected",
            (_, "manual") => "manual",
            _ => source,
        };
        match self {
            Self::Spanish => format!("{name} termino tras {duration} ({source})"),
            Self::English => format!("{name} ended after {duration} ({source})"),
        }
    }

    pub(crate) const fn session_closed_prefix(self) -> &'static str {
        match self {
            Self::Spanish => "Sesion cerrada",
            Self::English => "Session closed",
        }
    }

    pub(crate) const fn launching(self) -> &'static str {
        match self {
            Self::Spanish => "Lanzando",
            Self::English => "Launching",
        }
    }

    pub(crate) const fn steam_launch_uri_sent(self) -> &'static str {
        match self {
            Self::Spanish => "Steam launch URI enviado",
            Self::English => "Steam launch URI sent",
        }
    }

    pub(crate) const fn steam_uri_sent(self) -> &'static str {
        match self {
            Self::Spanish => "Steam URI enviado",
            Self::English => "Steam URI sent",
        }
    }

    pub(crate) const fn steam_uninstall_opened(self) -> &'static str {
        match self {
            Self::Spanish => "Confirmacion de Steam abierta",
            Self::English => "Steam confirmation opened",
        }
    }

    pub(crate) const fn steam_uri_failed(self) -> &'static str {
        match self {
            Self::Spanish => "No se pudo invocar",
            Self::English => "Could not invoke",
        }
    }

    pub(crate) const fn steam_install_title(self) -> &'static str {
        match self {
            Self::Spanish => "Instalar desde Steam",
            Self::English => "Install from Steam",
        }
    }

    pub(crate) const fn steam_validate_title(self) -> &'static str {
        match self {
            Self::Spanish => "Validar archivos",
            Self::English => "Validate files",
        }
    }

    pub(crate) const fn steam_properties_title(self) -> &'static str {
        match self {
            Self::Spanish => "Abrir propiedades",
            Self::English => "Open properties",
        }
    }

    pub(crate) const fn steam_downloads_title(self) -> &'static str {
        match self {
            Self::Spanish => "Abriendo descargas de Steam",
            Self::English => "Opening Steam downloads",
        }
    }

    pub(crate) const fn steam_uninstall_action_title(self) -> &'static str {
        match self {
            Self::Spanish => "Desinstalar desde Steam",
            Self::English => "Uninstall from Steam",
        }
    }

    pub(crate) const fn client_label(self) -> &'static str {
        match self {
            Self::Spanish => "Cliente",
            Self::English => "Client",
        }
    }

    pub(crate) const fn auto_detected_game(self) -> &'static str {
        match self {
            Self::Spanish => "Juego Steam detectado automaticamente",
            Self::English => "Auto-detected Steam game",
        }
    }

    pub(crate) const fn system_info_no_processes(self) -> &'static str {
        match self {
            Self::Spanish => "sin targets configurados",
            Self::English => "profile has no configured processes",
        }
    }

    pub(crate) fn system_process_closed(self, line: &str) -> String {
        match self {
            Self::Spanish => format!("app cerrada: {line}"),
            Self::English => format!("process closed: {line}"),
        }
    }

    pub(crate) const fn system_no_heavy_process(self) -> &'static str {
        match self {
            Self::Spanish => "no heavy apps activas",
            Self::English => "no heavy process found",
        }
    }

    pub(crate) const fn system_no_services(self) -> &'static str {
        match self {
            Self::Spanish => "sin services configurados",
            Self::English => "profile has no configured services",
        }
    }

    pub(crate) fn system_service_stopped(self, service: &str) -> String {
        match self {
            Self::Spanish => format!("servicio detenido: {service}"),
            Self::English => format!("service stopped: {service}"),
        }
    }

    pub(crate) fn system_service_started(self, service: &str) -> String {
        match self {
            Self::Spanish => format!("servicio iniciado: {service}"),
            Self::English => format!("service started: {service}"),
        }
    }

    pub(crate) const fn system_services_optimized(self) -> &'static str {
        match self {
            Self::Spanish => "services ya optimizados",
            Self::English => "services already optimized",
        }
    }

    pub(crate) const fn steam_not_found_manual(self) -> &'static str {
        match self {
            Self::Spanish => "no se encontro Steam, abrelo manualmente",
            Self::English => "Steam not found, open it manually",
        }
    }

    pub(crate) const fn steam_already_active(self) -> &'static str {
        match self {
            Self::Spanish => "Steam ya activo, prioridad asignada",
            Self::English => "Steam already active, priority assigned",
        }
    }

    pub(crate) const fn steam_opened(self) -> &'static str {
        match self {
            Self::Spanish => "Steam abierto automaticamente",
            Self::English => "Steam opened automatically",
        }
    }

    pub(crate) const fn explorer_stopped(self) -> &'static str {
        match self {
            Self::Spanish => "explorer.exe suspendido (~400 MB liberados)",
            Self::English => "explorer.exe suspended (~400 MB freed)",
        }
    }

    pub(crate) const fn explorer_started(self) -> &'static str {
        match self {
            Self::Spanish => "explorer.exe reiniciado",
            Self::English => "explorer.exe restarted",
        }
    }

    pub(crate) const fn active_profile_line(self) -> &'static str {
        match self {
            Self::Spanish => "perfil activo",
            Self::English => "active profile",
        }
    }

    pub(crate) const fn power_plan_line(self) -> &'static str {
        match self {
            Self::Spanish => "power plan",
            Self::English => "power plan",
        }
    }

    pub(crate) const fn killing_background_processes(self) -> &'static str {
        match self {
            Self::Spanish => "cerrando background apps",
            Self::English => "closing background processes",
        }
    }

    pub(crate) const fn stopping_services(self) -> &'static str {
        match self {
            Self::Spanish => "pausando services",
            Self::English => "stopping services",
        }
    }

    pub(crate) const fn freeing_system_resources(self) -> &'static str {
        match self {
            Self::Spanish => "freeing RAM y desktop",
            Self::English => "freeing system resources",
        }
    }

    pub(crate) const fn explorer_kept_report(self) -> &'static str {
        match self {
            Self::Spanish => "explorer queda running en este profile",
            Self::English => "explorer remains active in this profile",
        }
    }

    pub(crate) const fn overdrive_activated(self) -> &'static str {
        match self {
            Self::Spanish => "OVERDRIVE ACTIVADO",
            Self::English => "CHAOS GAME MODE ACTIVATED",
        }
    }

    pub(crate) const fn restoring_windows_shell(self) -> &'static str {
        match self {
            Self::Spanish => "restaurando Windows shell",
            Self::English => "restoring Windows shell",
        }
    }

    pub(crate) const fn restoring_services(self) -> &'static str {
        match self {
            Self::Spanish => "restaurando services",
            Self::English => "restoring services",
        }
    }

    pub(crate) const fn system_restored(self) -> &'static str {
        match self {
            Self::Spanish => "SYSTEM RESTORED",
            Self::English => "SYSTEM RESTORED",
        }
    }

    pub(crate) const fn closed_apps_not_reopened(self) -> &'static str {
        match self {
            Self::Spanish => "closed apps no se reabren solas",
            Self::English => "closed apps are not reopened automatically",
        }
    }

    pub(crate) const fn label_load(self) -> &'static str {
        match self {
            Self::Spanish => "LOAD",
            Self::English => "LOAD",
        }
    }

    pub(crate) const fn label_cores(self) -> &'static str {
        match self {
            Self::Spanish => "CORES",
            Self::English => "CORES",
        }
    }

    pub(crate) const fn label_used(self) -> &'static str {
        match self {
            Self::Spanish => "USED",
            Self::English => "USED",
        }
    }

    pub(crate) const fn label_free(self) -> &'static str {
        match self {
            Self::Spanish => "FREE",
            Self::English => "FREE",
        }
    }

    pub(crate) const fn label_temp(self) -> &'static str {
        "TEMP"
    }

    pub(crate) const fn label_power(self) -> &'static str {
        match self {
            Self::Spanish => "PLAN",
            Self::English => "POWER",
        }
    }

    pub(crate) const fn label_desktop(self) -> &'static str {
        match self {
            Self::Spanish => "DESKTOP",
            Self::English => "DESKTOP",
        }
    }

    pub(crate) const fn label_services(self) -> &'static str {
        match self {
            Self::Spanish => "SERVICES",
            Self::English => "SERVICES",
        }
    }

    pub(crate) const fn label_bloat(self) -> &'static str {
        match self {
            Self::Spanish => "BLOAT",
            Self::English => "BLOAT",
        }
    }

    pub(crate) const fn label_sensors(self) -> &'static str {
        match self {
            Self::Spanish => "SENSORS",
            Self::English => "SENSORS",
        }
    }

    pub(crate) const fn label_frames(self) -> &'static str {
        match self {
            Self::Spanish => "FRAMES",
            Self::English => "FRAMES",
        }
    }

    pub(crate) const fn label_name(self) -> &'static str {
        match self {
            Self::Spanish => "NAME",
            Self::English => "NAME",
        }
    }

    pub(crate) const fn label_memory(self) -> &'static str {
        match self {
            Self::Spanish => "MEMORY",
            Self::English => "MEMORY",
        }
    }

    pub(crate) const fn label_heat(self) -> &'static str {
        match self {
            Self::Spanish => "HEAT",
            Self::English => "HEAT",
        }
    }

    pub(crate) const fn label_pattern(self) -> &'static str {
        match self {
            Self::Spanish => "PATTERN",
            Self::English => "PATTERN",
        }
    }

    pub(crate) const fn label_status(self) -> &'static str {
        match self {
            Self::Spanish => "STATUS",
            Self::English => "STATUS",
        }
    }

    pub(crate) const fn label_view(self) -> &'static str {
        match self {
            Self::Spanish => "VIEW",
            Self::English => "VIEW",
        }
    }

    pub(crate) const fn label_config(self) -> &'static str {
        match self {
            Self::Spanish => "CONFIG",
            Self::English => "CONFIG",
        }
    }

    pub(crate) const fn label_targets(self) -> &'static str {
        match self {
            Self::Spanish => "TARGETS",
            Self::English => "TARGETS",
        }
    }

    pub(crate) const fn label_keep(self) -> &'static str {
        match self {
            Self::Spanish => "KEEP",
            Self::English => "KEEP",
        }
    }

    pub(crate) const fn label_watch(self) -> &'static str {
        match self {
            Self::Spanish => "WATCH",
            Self::English => "WATCH",
        }
    }

    pub(crate) const fn label_selected(self) -> &'static str {
        match self {
            Self::Spanish => "SELECTED",
            Self::English => "SELECTED",
        }
    }

    pub(crate) const fn label_visible(self) -> &'static str {
        match self {
            Self::Spanish => "VISIBLE",
            Self::English => "VISIBLE",
        }
    }

    pub(crate) const fn label_total(self) -> &'static str {
        "TOTAL"
    }

    pub(crate) const fn label_filter(self) -> &'static str {
        match self {
            Self::Spanish => "FILTER",
            Self::English => "FILTER",
        }
    }

    pub(crate) const fn label_payload(self) -> &'static str {
        match self {
            Self::Spanish => "PAYLOAD",
            Self::English => "PAYLOAD",
        }
    }

    pub(crate) const fn label_ready(self) -> &'static str {
        match self {
            Self::Spanish => "READY",
            Self::English => "READY",
        }
    }

    pub(crate) const fn label_power_plan(self) -> &'static str {
        match self {
            Self::Spanish => "POWER PLAN",
            Self::English => "POWER PLAN",
        }
    }

    pub(crate) const fn label_shell(self) -> &'static str {
        match self {
            Self::Spanish => "SHELL",
            Self::English => "SHELL",
        }
    }

    pub(crate) const fn label_mode(self) -> &'static str {
        match self {
            Self::Spanish => "MODE",
            Self::English => "MODE",
        }
    }

    pub(crate) const fn label_protected(self) -> &'static str {
        match self {
            Self::Spanish => "PROTECTED",
            Self::English => "PROTECTED",
        }
    }

    pub(crate) const fn label_explorer(self) -> &'static str {
        "EXPLORER"
    }

    pub(crate) const fn label_removable(self) -> &'static str {
        match self {
            Self::Spanish => "CLEANUP",
            Self::English => "REMOVABLE",
        }
    }

    pub(crate) const fn label_groups(self) -> &'static str {
        match self {
            Self::Spanish => "GROUPS",
            Self::English => "GROUPS",
        }
    }

    pub(crate) const fn label_restore(self) -> &'static str {
        match self {
            Self::Spanish => "RESTORE",
            Self::English => "RESTORE",
        }
    }

    pub(crate) const fn label_note(self) -> &'static str {
        match self {
            Self::Spanish => "NOTE",
            Self::English => "NOTE",
        }
    }

    pub(crate) const fn label_cpu_usage(self) -> &'static str {
        match self {
            Self::Spanish => "CPU USAGE",
            Self::English => "CPU USAGE",
        }
    }

    pub(crate) const fn label_ram_used(self) -> &'static str {
        match self {
            Self::Spanish => "RAM USED",
            Self::English => "RAM USED",
        }
    }

    pub(crate) const fn label_ram_free(self) -> &'static str {
        match self {
            Self::Spanish => "RAM FREE",
            Self::English => "RAM FREE",
        }
    }

    pub(crate) const fn label_telemetry(self) -> &'static str {
        match self {
            Self::Spanish => "TELEMETRY",
            Self::English => "TELEMETRY",
        }
    }

    pub(crate) const fn label_observed(self) -> &'static str {
        match self {
            Self::Spanish => "ACTIVOS",
            Self::English => "OBSERVED",
        }
    }

    pub(crate) const fn label_uptime(self) -> &'static str {
        match self {
            Self::Spanish => "UPTIME",
            Self::English => "UPTIME",
        }
    }

    pub(crate) const fn label_processes(self) -> &'static str {
        match self {
            Self::Spanish => "PROCESSES",
            Self::English => "PROCESSES",
        }
    }

    pub(crate) const fn label_steam_lib(self) -> &'static str {
        match self {
            Self::Spanish => "STEAM LIB",
            Self::English => "STEAM LIB",
        }
    }

    pub(crate) const fn label_history(self) -> &'static str {
        match self {
            Self::Spanish => "HISTORY",
            Self::English => "HISTORY",
        }
    }

    pub(crate) const fn label_theme_file(self) -> &'static str {
        match self {
            Self::Spanish => "THEME FILE",
            Self::English => "THEME FILE",
        }
    }

    pub(crate) const fn label_gpu_load(self) -> &'static str {
        match self {
            Self::Spanish => "GPU LOAD",
            Self::English => "GPU LOAD",
        }
    }

    pub(crate) const fn label_gpu_vram(self) -> &'static str {
        "GPU VRAM"
    }

    pub(crate) const fn label_gpu_temp(self) -> &'static str {
        match self {
            Self::Spanish => "TEMP GPU",
            Self::English => "GPU TEMP",
        }
    }

    pub(crate) const fn label_cpu_temp(self) -> &'static str {
        match self {
            Self::Spanish => "TEMP CPU",
            Self::English => "CPU TEMP",
        }
    }

    pub(crate) const fn label_average(self) -> &'static str {
        match self {
            Self::Spanish => "AVG",
            Self::English => "AVG",
        }
    }

    pub(crate) const fn label_frame(self) -> &'static str {
        "FRAME"
    }

    pub(crate) const fn label_samples(self) -> &'static str {
        match self {
            Self::Spanish => "SAMPLES",
            Self::English => "SAMPLES",
        }
    }

    pub(crate) const fn label_theme(self) -> &'static str {
        match self {
            Self::Spanish => "TEMA",
            Self::English => "THEME",
        }
    }

    pub(crate) const fn label_language(self) -> &'static str {
        match self {
            Self::Spanish => "LANGUAGE",
            Self::English => "LANGUAGE",
        }
    }

    pub(crate) const fn label_theme_live(self) -> &'static str {
        match self {
            Self::Spanish => "TEMA LIVE",
            Self::English => "THEME LIVE",
        }
    }

    pub(crate) const fn label_source(self) -> &'static str {
        match self {
            Self::Spanish => "SOURCE",
            Self::English => "SOURCE",
        }
    }

    pub(crate) const fn label_resolved(self) -> &'static str {
        match self {
            Self::Spanish => "RESOLVED",
            Self::English => "RESOLVED",
        }
    }

    pub(crate) const fn label_provider(self) -> &'static str {
        match self {
            Self::Spanish => "PROVIDER",
            Self::English => "PROVIDER",
        }
    }

    pub(crate) const fn label_next(self) -> &'static str {
        match self {
            Self::Spanish => "NEXT",
            Self::English => "NEXT",
        }
    }

    pub(crate) const fn label_logged(self) -> &'static str {
        match self {
            Self::Spanish => "LOGGED",
            Self::English => "LOGGED",
        }
    }

    pub(crate) const fn label_path(self) -> &'static str {
        match self {
            Self::Spanish => "PATH",
            Self::English => "PATH",
        }
    }

    pub(crate) const fn label_buffer(self) -> &'static str {
        "BUFFER"
    }

    pub(crate) const fn label_lines(self) -> &'static str {
        match self {
            Self::Spanish => "LINES",
            Self::English => "LINES",
        }
    }

    pub(crate) const fn label_warnings(self) -> &'static str {
        match self {
            Self::Spanish => "WARNINGS",
            Self::English => "WARNINGS",
        }
    }

    pub(crate) const fn label_last(self) -> &'static str {
        match self {
            Self::Spanish => "LAST",
            Self::English => "LAST",
        }
    }

    pub(crate) const fn label_active(self) -> &'static str {
        match self {
            Self::Spanish => "ACTIVE",
            Self::English => "ACTIVE",
        }
    }

    pub(crate) const fn label_time(self) -> &'static str {
        match self {
            Self::Spanish => "TIME",
            Self::English => "TIME",
        }
    }

    pub(crate) const fn label_running(self) -> &'static str {
        match self {
            Self::Spanish => "RUNNING",
            Self::English => "RUNNING",
        }
    }

    pub(crate) const fn label_games(self) -> &'static str {
        match self {
            Self::Spanish => "GAMES",
            Self::English => "GAMES",
        }
    }

    pub(crate) const fn label_libraries(self) -> &'static str {
        match self {
            Self::Spanish => "LIBRARIES",
            Self::English => "LIBRARIES",
        }
    }

    pub(crate) const fn label_pmon(self) -> &'static str {
        "PMON"
    }

    pub(crate) const fn label_target(self) -> &'static str {
        match self {
            Self::Spanish => "TARGET",
            Self::English => "TARGET",
        }
    }

    pub(crate) const fn label_capture(self) -> &'static str {
        match self {
            Self::Spanish => "CAPTURE",
            Self::English => "CAPTURE",
        }
    }

    pub(crate) const fn label_latency(self) -> &'static str {
        match self {
            Self::Spanish => "LATENCY",
            Self::English => "LATENCY",
        }
    }

    pub(crate) const fn label_monitor(self) -> &'static str {
        "MONITOR"
    }

    pub(crate) const fn panel_frame_history(self) -> &'static str {
        match self {
            Self::Spanish => "HISTORIAL FPS",
            Self::English => "FPS HISTORY",
        }
    }

    pub(crate) const fn panel_frame_metrics(self) -> &'static str {
        match self {
            Self::Spanish => "FPS / LATENCIA",
            Self::English => "FRAME METRICS",
        }
    }

    pub(crate) const fn panel_presentmon(self) -> &'static str {
        "PRESENTMON"
    }

    pub(crate) const fn dashboard_ready_status(self) -> &'static str {
        match self {
            Self::Spanish => "GAME READY",
            Self::English => "SYSTEM READY",
        }
    }

    pub(crate) const fn dashboard_cleanup_status(self) -> &'static str {
        match self {
            Self::Spanish => "NEEDS CLEANUP",
            Self::English => "NEEDS CLEANUP",
        }
    }

    pub(crate) const fn preview_overdrive_hint(self) -> &'static str {
        match self {
            Self::Spanish => " preview overdrive",
            Self::English => " preview overdrive",
        }
    }

    pub(crate) const fn full_preview_hint(self) -> &'static str {
        match self {
            Self::Spanish => " ver preview completo",
            Self::English => " full preview",
        }
    }

    pub(crate) const fn no_residual_targets(self) -> &'static str {
        match self {
            Self::Spanish => "no cleanup targets",
            Self::English => "no residual targets",
        }
    }

    pub(crate) const fn trace_title(self) -> &'static str {
        "TRACE"
    }

    pub(crate) const fn panel_readiness(self) -> &'static str {
        match self {
            Self::Spanish => "READY CHECK",
            Self::English => "READINESS",
        }
    }

    pub(crate) const fn panel_system(self) -> &'static str {
        match self {
            Self::Spanish => "SISTEMA",
            Self::English => "SYSTEM",
        }
    }

    pub(crate) const fn panel_process_heatmap(self) -> &'static str {
        match self {
            Self::Spanish => "PROCESS HEATMAP",
            Self::English => "PROCESS HEATMAP",
        }
    }

    pub(crate) const fn panel_processes(self) -> &'static str {
        match self {
            Self::Spanish => "PROCESOS",
            Self::English => "PROCESSES",
        }
    }

    pub(crate) const fn panel_hidden_bin(self) -> &'static str {
        match self {
            Self::Spanish => "OCULTOS",
            Self::English => "HIDDEN BIN",
        }
    }

    pub(crate) const fn panel_detail(self) -> &'static str {
        match self {
            Self::Spanish => "DETALLE",
            Self::English => "DETAIL",
        }
    }

    pub(crate) const fn panel_policy(self) -> &'static str {
        match self {
            Self::Spanish => "POLITICA",
            Self::English => "POLICY",
        }
    }

    pub(crate) const fn panel_process_map(self) -> &'static str {
        match self {
            Self::Spanish => "PROCESOS",
            Self::English => "PROCESS MAP",
        }
    }

    pub(crate) const fn top_memory_heading(self) -> &'static str {
        match self {
            Self::Spanish => " TOP RAM APPS",
            Self::English => " TOP MEMORY",
        }
    }

    pub(crate) const fn current_targets_heading(self) -> &'static str {
        match self {
            Self::Spanish => " KILL LIST",
            Self::English => " CURRENT TARGETS",
        }
    }

    pub(crate) const fn status_hidden(self) -> &'static str {
        match self {
            Self::Spanish => "OCULTO",
            Self::English => "HIDDEN",
        }
    }

    pub(crate) const fn status_keep(self) -> &'static str {
        match self {
            Self::Spanish => "KEEP",
            Self::English => "KEEP",
        }
    }

    pub(crate) const fn status_target(self) -> &'static str {
        match self {
            Self::Spanish => "TARGET",
            Self::English => "TARGET",
        }
    }

    pub(crate) const fn status_watch(self) -> &'static str {
        match self {
            Self::Spanish => "WATCH",
            Self::English => "WATCH",
        }
    }

    pub(crate) const fn no_process_filter_match(self) -> &'static str {
        match self {
            Self::Spanish => "No hay procesos que coincidan con el filtro",
            Self::English => "No processes match the filter",
        }
    }

    pub(crate) const fn no_hidden_processes(self) -> &'static str {
        match self {
            Self::Spanish => "No hay procesos ocultos detectados",
            Self::English => "No hidden processes detected",
        }
    }

    pub(crate) const fn no_actionable_processes(self) -> &'static str {
        match self {
            Self::Spanish => "No hay targets activos",
            Self::English => "No actionable processes detected",
        }
    }

    pub(crate) const fn hidden_view_hint(self) -> &'static str {
        match self {
            Self::Spanish => " vista ocultos",
            Self::English => " hidden view",
        }
    }

    pub(crate) const fn unavailable(self) -> &'static str {
        match self {
            Self::Spanish => "no disponible",
            Self::English => "unavailable",
        }
    }

    pub(crate) const fn none(self) -> &'static str {
        match self {
            Self::Spanish => "ninguno",
            Self::English => "none",
        }
    }

    pub(crate) const fn no_process_selected(self) -> &'static str {
        match self {
            Self::Spanish => "  No hay proceso seleccionado",
            Self::English => "  No process selected",
        }
    }

    pub(crate) const fn view_hidden(self) -> &'static str {
        match self {
            Self::Spanish => "oculto",
            Self::English => "hidden",
        }
    }

    pub(crate) const fn view_actionable(self) -> &'static str {
        match self {
            Self::Spanish => "target",
            Self::English => "actionable",
        }
    }

    pub(crate) fn memory_instances(self, memory_mb: f64, count: usize) -> String {
        match self {
            Self::Spanish => format!("{memory_mb:.0} MB / {count} instancias"),
            Self::English => format!("{memory_mb:.0} MB / {count} instances"),
        }
    }

    pub(crate) fn groups_count(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} grupos"),
            Self::English => format!("{count} groups"),
        }
    }

    pub(crate) fn services_running(self, running: usize, total: usize) -> String {
        match self {
            Self::Spanish => format!("{running}/{total} activos"),
            Self::English => format!("{running}/{total} running"),
        }
    }

    pub(crate) fn active_groups(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} grupos activos"),
            Self::English => format!("{count} active groups"),
        }
    }

    pub(crate) fn removable_heat(self, memory_mb: f64) -> String {
        match self {
            Self::Spanish => format!("{memory_mb:.0} MB cleanup"),
            Self::English => format!("{memory_mb:.0} MB removable heat"),
        }
    }

    pub(crate) fn configured_count(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} configurados"),
            Self::English => format!("{count} configured"),
        }
    }

    pub(crate) fn games_count(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} juegos"),
            Self::English => format!("{count} games"),
        }
    }

    pub(crate) fn lines_count(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} lineas"),
            Self::English => format!("{count} lines"),
        }
    }

    pub(crate) fn more_entries(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("{count} entradas mas  "),
            Self::English => format!("{count} more entries  "),
        }
    }

    pub(crate) fn latest_lines(self, count: usize) -> String {
        match self {
            Self::Spanish => format!("ultimas {count} lineas"),
            Self::English => format!("last {count} lines"),
        }
    }

    pub(crate) const fn command_preview_overdrive(self) -> &'static str {
        match self {
            Self::Spanish => "Preview Overdrive",
            Self::English => "Preview Overdrive",
        }
    }

    pub(crate) const fn command_preview_overdrive_detail(self) -> &'static str {
        match self {
            Self::Spanish => "confirma antes de cambiar",
            Self::English => "confirm before changes",
        }
    }

    pub(crate) const fn command_restore_system(self) -> &'static str {
        match self {
            Self::Spanish => "Restaurar sistema",
            Self::English => "Restore System",
        }
    }

    pub(crate) const fn command_restore_system_detail(self) -> &'static str {
        match self {
            Self::Spanish => "shell, services y balanced power",
            Self::English => "restart shell, services, balanced power",
        }
    }

    pub(crate) const fn command_refresh_telemetry(self) -> &'static str {
        match self {
            Self::Spanish => "Refrescar telemetria",
            Self::English => "Refresh Telemetry",
        }
    }

    pub(crate) const fn command_refresh_telemetry_detail(self) -> &'static str {
        match self {
            Self::Spanish => "pull fresh snapshot",
            Self::English => "pull a fresh system snapshot",
        }
    }

    pub(crate) const fn command_switch_deck(self) -> &'static str {
        match self {
            Self::Spanish => "Cambiar vista",
            Self::English => "Switch Deck",
        }
    }

    pub(crate) const fn command_switch_deck_detail(self) -> &'static str {
        match self {
            Self::Spanish => "dashboard, steam, frames, procesos, overdrive, sistema, historial",
            Self::English => "dashboard, steam, frames, processes, overdrive, system, history",
        }
    }

    pub(crate) const fn command_cycle_theme(self) -> &'static str {
        match self {
            Self::Spanish => "Cambiar tema",
            Self::English => "Cycle Theme",
        }
    }

    pub(crate) const fn command_cycle_theme_detail(self) -> &'static str {
        match self {
            Self::Spanish => "cyberpunk, hacker, gruvbox, tokyo, mocha",
            Self::English => "cyberpunk, hacker, gruvbox, tokyo, mocha",
        }
    }

    pub(crate) const fn command_exit(self) -> &'static str {
        match self {
            Self::Spanish => "Salir",
            Self::English => "Exit",
        }
    }

    pub(crate) const fn command_exit_detail(self) -> &'static str {
        match self {
            Self::Spanish => "clean terminal exit",
            Self::English => "leave terminal cleanly",
        }
    }

    pub(crate) const fn command_probe_tools(self) -> &'static str {
        match self {
            Self::Spanish => "Detectar herramientas",
            Self::English => "Probe Tools",
        }
    }

    pub(crate) const fn command_probe_tools_detail(self) -> &'static str {
        match self {
            Self::Spanish => "refresca deteccion de PresentMon",
            Self::English => "refresh PresentMon detection",
        }
    }

    pub(crate) const fn command_reload_history(self) -> &'static str {
        match self {
            Self::Spanish => "Recargar historial",
            Self::English => "Reload History",
        }
    }

    pub(crate) const fn command_reload_history_detail(self) -> &'static str {
        match self {
            Self::Spanish => "lee history.log otra vez",
            Self::English => "read history.log again",
        }
    }

    pub(crate) const fn command_scroll(self) -> &'static str {
        match self {
            Self::Spanish => "Scroll",
            Self::English => "Scroll",
        }
    }

    pub(crate) const fn command_scroll_detail(self) -> &'static str {
        match self {
            Self::Spanish => "mueve una linea",
            Self::English => "move one line",
        }
    }

    pub(crate) const fn command_page(self) -> &'static str {
        match self {
            Self::Spanish => "Pagina",
            Self::English => "Page",
        }
    }

    pub(crate) const fn command_page_detail(self) -> &'static str {
        match self {
            Self::Spanish => "salta por bloque",
            Self::English => "jump by block",
        }
    }

    pub(crate) const fn command_top(self) -> &'static str {
        match self {
            Self::Spanish => "Inicio",
            Self::English => "Top",
        }
    }

    pub(crate) const fn command_top_detail(self) -> &'static str {
        match self {
            Self::Spanish => "primera linea visible",
            Self::English => "first visible line",
        }
    }

    pub(crate) const fn command_bottom(self) -> &'static str {
        match self {
            Self::Spanish => "Final",
            Self::English => "Bottom",
        }
    }

    pub(crate) const fn command_bottom_detail(self) -> &'static str {
        match self {
            Self::Spanish => "entradas recientes",
            Self::English => "latest entries",
        }
    }

    pub(crate) const fn command_theme_persists(self) -> &'static str {
        match self {
            Self::Spanish => "se guarda en theme.toml",
            Self::English => "persists in theme.toml",
        }
    }

    pub(crate) const fn panel_overdrive_console(self) -> &'static str {
        match self {
            Self::Spanish => "CONSOLA OVERDRIVE",
            Self::English => "OVERDRIVE CONSOLE",
        }
    }

    pub(crate) const fn panel_live_status(self) -> &'static str {
        match self {
            Self::Spanish => "ESTADO EN VIVO",
            Self::English => "LIVE STATUS",
        }
    }

    pub(crate) const fn panel_profile_plan(self) -> &'static str {
        match self {
            Self::Spanish => "PLAN DE PERFIL",
            Self::English => "PROFILE PLAN",
        }
    }

    pub(crate) const fn panel_payload_preview(self) -> &'static str {
        match self {
            Self::Spanish => "PREVIEW PAYLOAD",
            Self::English => "PAYLOAD PREVIEW",
        }
    }

    pub(crate) const fn panel_restore_plan(self) -> &'static str {
        match self {
            Self::Spanish => "RESTORE PLAN",
            Self::English => "RESTORE PLAN",
        }
    }

    pub(crate) const fn linked(self) -> &'static str {
        match self {
            Self::Spanish => "conectado",
            Self::English => "linked",
        }
    }

    pub(crate) const fn not_linked(self) -> &'static str {
        match self {
            Self::Spanish => "sin conexion",
            Self::English => "not linked",
        }
    }

    pub(crate) const fn desktop_active(self) -> &'static str {
        match self {
            Self::Spanish => "desktop activo",
            Self::English => "desktop active",
        }
    }

    pub(crate) const fn minimal_shell(self) -> &'static str {
        match self {
            Self::Spanish => "minimal shell",
            Self::English => "minimal shell",
        }
    }

    pub(crate) const fn explorer_active(self) -> &'static str {
        match self {
            Self::Spanish => "explorer activo",
            Self::English => "explorer active",
        }
    }

    pub(crate) const fn running(self) -> &'static str {
        match self {
            Self::Spanish => "activo",
            Self::English => "running",
        }
    }

    pub(crate) const fn closed(self) -> &'static str {
        match self {
            Self::Spanish => "cerrado",
            Self::English => "closed",
        }
    }

    pub(crate) const fn stopped(self) -> &'static str {
        match self {
            Self::Spanish => "detenido",
            Self::English => "stopped",
        }
    }

    pub(crate) const fn ready_lower(self) -> &'static str {
        match self {
            Self::Spanish => "ready",
            Self::English => "ready",
        }
    }

    pub(crate) const fn high_performance_lower(self) -> &'static str {
        match self {
            Self::Spanish => "high performance",
            Self::English => "high performance",
        }
    }

    pub(crate) const fn unchanged(self) -> &'static str {
        match self {
            Self::Spanish => "unchanged",
            Self::English => "unchanged",
        }
    }

    pub(crate) const fn stop_on_overdrive(self) -> &'static str {
        match self {
            Self::Spanish => "stop en OD",
            Self::English => "stop on OD",
        }
    }

    pub(crate) const fn keep_running(self) -> &'static str {
        match self {
            Self::Spanish => "keep running",
            Self::English => "keep running",
        }
    }

    pub(crate) const fn clean_prefix(self) -> &'static str {
        match self {
            Self::Spanish => "  clean: ",
            Self::English => "  clean: ",
        }
    }

    pub(crate) const fn no_configured_targets(self) -> &'static str {
        match self {
            Self::Spanish => "sin targets activos",
            Self::English => "no configured targets detected",
        }
    }

    pub(crate) const fn restart_if_closed(self) -> &'static str {
        match self {
            Self::Spanish => "restart si esta cerrado",
            Self::English => "restart if closed",
        }
    }

    pub(crate) const fn closed_apps_stay_closed(self) -> &'static str {
        match self {
            Self::Spanish => "closed apps siguen cerradas",
            Self::English => "closed apps stay closed",
        }
    }

    pub(crate) const fn undo_overdrive_changes(self) -> &'static str {
        match self {
            Self::Spanish => "undo cambios de Overdrive",
            Self::English => "undo overdrive changes",
        }
    }

    pub(crate) const fn no_target(self) -> &'static str {
        match self {
            Self::Spanish => "no target",
            Self::English => "no target",
        }
    }

    pub(crate) const fn defaults_only(self) -> &'static str {
        match self {
            Self::Spanish => "solo defaults",
            Self::English => "defaults only",
        }
    }

    pub(crate) const fn internal_theme(self) -> &'static str {
        match self {
            Self::Spanish => "tema interno",
            Self::English => "internal theme",
        }
    }

    pub(crate) const fn internal(self) -> &'static str {
        match self {
            Self::Spanish => "interno",
            Self::English => "internal",
        }
    }

    pub(crate) const fn not_set(self) -> &'static str {
        match self {
            Self::Spanish => "no configurado",
            Self::English => "not set",
        }
    }

    pub(crate) const fn not_found(self) -> &'static str {
        match self {
            Self::Spanish => "no encontrado",
            Self::English => "not found",
        }
    }

    pub(crate) const fn steam_active(self) -> &'static str {
        match self {
            Self::Spanish => "Steam activo",
            Self::English => "Steam active",
        }
    }

    pub(crate) const fn manual_folders_later(self) -> &'static str {
        match self {
            Self::Spanish => "carpetas manuales -> Epic despues",
            Self::English => "manual folders -> Epic later",
        }
    }

    pub(crate) const fn panel_settings(self) -> &'static str {
        match self {
            Self::Spanish => "AJUSTES",
            Self::English => "SETTINGS",
        }
    }

    pub(crate) const fn panel_runtime_themes(self) -> &'static str {
        match self {
            Self::Spanish => "RUNTIME / TEMAS",
            Self::English => "RUNTIME / THEMES",
        }
    }

    pub(crate) const fn panel_integrations(self) -> &'static str {
        match self {
            Self::Spanish => "INTEGRACIONES",
            Self::English => "INTEGRATIONS",
        }
    }

    pub(crate) const fn theme_presets_heading(self) -> &'static str {
        match self {
            Self::Spanish => " PRESETS DE TEMA",
            Self::English => " THEME PRESETS",
        }
    }

    pub(crate) const fn roadmap_heading(self) -> &'static str {
        " ROADMAP"
    }

    pub(crate) const fn preset_cyberpunk_desc(self) -> &'static str {
        match self {
            Self::Spanish => "neon de alto contraste",
            Self::English => "high contrast neon",
        }
    }

    pub(crate) const fn preset_hacker_desc(self) -> &'static str {
        match self {
            Self::Spanish => "ops de terminal negra",
            Self::English => "black terminal ops",
        }
    }

    pub(crate) const fn preset_gruvbox_desc(self) -> &'static str {
        match self {
            Self::Spanish => "paleta terminal calida",
            Self::English => "warm terminal palette",
        }
    }

    pub(crate) const fn preset_tokyo_desc(self) -> &'static str {
        match self {
            Self::Spanish => "paleta noche fria",
            Self::English => "cool night palette",
        }
    }

    pub(crate) const fn preset_mocha_desc(self) -> &'static str {
        match self {
            Self::Spanish => "pasteles suaves",
            Self::English => "soft pastel palette",
        }
    }

    pub(crate) const fn roadmap_steam_now(self) -> &'static str {
        match self {
            Self::Spanish => "Steam ahora",
            Self::English => "Steam now",
        }
    }

    pub(crate) const fn roadmap_steam_later(self) -> &'static str {
        match self {
            Self::Spanish => "Epic/carpetas manuales despues",
            Self::English => "Epic/manual folders later",
        }
    }

    pub(crate) const fn roadmap_presentmon(self) -> &'static str {
        match self {
            Self::Spanish => "PresentMon Console via winget",
            Self::English => "PresentMon Console via winget",
        }
    }

    pub(crate) const fn panel_history(self) -> &'static str {
        match self {
            Self::Spanish => "HISTORIAL",
            Self::English => "HISTORY",
        }
    }

    pub(crate) const fn panel_history_control(self) -> &'static str {
        match self {
            Self::Spanish => "CONTROL HISTORIAL",
            Self::English => "HISTORY CONTROL",
        }
    }

    pub(crate) const fn panel_history_digest(self) -> &'static str {
        match self {
            Self::Spanish => "RESUMEN HISTORIAL",
            Self::English => "HISTORY DIGEST",
        }
    }

    pub(crate) const fn no_history_yet_sentence(self) -> &'static str {
        match self {
            Self::Spanish => "No hay historial todavia.",
            Self::English => "No history yet.",
        }
    }

    pub(crate) const fn history_logged_detail(self) -> &'static str {
        match self {
            Self::Spanish => "Overdrive previews, restores, Steam launches, sesiones",
            Self::English => "overdrive previews, restore runs, Steam launches, sessions",
        }
    }

    pub(crate) const fn none_yet(self) -> &'static str {
        match self {
            Self::Spanish => "nada todavia",
            Self::English => "none yet",
        }
    }

    pub(crate) const fn history_feeds_heading(self) -> &'static str {
        match self {
            Self::Spanish => " FUENTES DEL HISTORIAL",
            Self::English => " HISTORY FEEDS",
        }
    }

    pub(crate) const fn history_feed_overdrive(self) -> &'static str {
        match self {
            Self::Spanish => "pre-game changes",
            Self::English => "what changed before gaming",
        }
    }

    pub(crate) const fn history_feed_restore(self) -> &'static str {
        match self {
            Self::Spanish => "Windows restore changes",
            Self::English => "what returned to Windows",
        }
    }

    pub(crate) const fn history_feed_sessions(self) -> &'static str {
        match self {
            Self::Spanish => "game launches y timer notes",
            Self::English => "game launch and timer notes",
        }
    }

    pub(crate) const fn steam_no_games_detected(self) -> &'static str {
        match self {
            Self::Spanish => "No hay juegos detectados. ",
            Self::English => "No games detected. ",
        }
    }

    pub(crate) const fn scan_library_hint(self) -> &'static str {
        match self {
            Self::Spanish => " scan library",
            Self::English => " scan library",
        }
    }

    pub(crate) const fn panel_steam_scanning(self) -> &'static str {
        match self {
            Self::Spanish => "STEAM / ESCANEANDO",
            Self::English => "STEAM / SCANNING",
        }
    }

    pub(crate) const fn panel_steam_library(self) -> &'static str {
        match self {
            Self::Spanish => "BIBLIOTECA STEAM",
            Self::English => "STEAM LIBRARY",
        }
    }

    pub(crate) const fn panel_selected_game(self) -> &'static str {
        match self {
            Self::Spanish => "SELECTED GAME",
            Self::English => "SELECTED GAME",
        }
    }

    pub(crate) const fn preview_od_launch(self) -> &'static str {
        match self {
            Self::Spanish => " Preview + OD launch",
            Self::English => " Preview + OD launch",
        }
    }

    pub(crate) const fn launch_normally(self) -> &'static str {
        match self {
            Self::Spanish => " Launch normal",
            Self::English => " Launch normally",
        }
    }

    pub(crate) const fn scan_steam_library(self) -> &'static str {
        match self {
            Self::Spanish => " Scan Steam library",
            Self::English => " Scan Steam library",
        }
    }

    pub(crate) const fn launch_game_timer_hint(self) -> &'static str {
        match self {
            Self::Spanish => "  Lanza un game para arrancar el timer",
            Self::English => "  Launch a game to start a session timer",
        }
    }

    pub(crate) const fn end_session(self) -> &'static str {
        match self {
            Self::Spanish => " Cerrar sesion",
            Self::English => " End Session",
        }
    }

    pub(crate) const fn panel_steam_tools(self) -> &'static str {
        match self {
            Self::Spanish => "HERRAMIENTAS STEAM",
            Self::English => "STEAM TOOLS",
        }
    }

    pub(crate) const fn scanning(self) -> &'static str {
        match self {
            Self::Spanish => "escaneando",
            Self::English => "scanning",
        }
    }

    pub(crate) const fn scan_libraries(self) -> &'static str {
        match self {
            Self::Spanish => " Scan libraries  ",
            Self::English => " Scan libraries  ",
        }
    }

    pub(crate) const fn install_selected(self) -> &'static str {
        match self {
            Self::Spanish => " Instalar seleccionado  ",
            Self::English => " Install selected  ",
        }
    }

    pub(crate) const fn properties_action(self) -> &'static str {
        match self {
            Self::Spanish => " Propiedades  ",
            Self::English => " Properties  ",
        }
    }

    pub(crate) const fn uninstall_action(self) -> &'static str {
        match self {
            Self::Spanish => " Desinstalar",
            Self::English => " Uninstall",
        }
    }

    pub(crate) const fn end_current_timer(self) -> &'static str {
        match self {
            Self::Spanish => " Cerrar timer actual",
            Self::English => " End current timer",
        }
    }

    pub(crate) const fn panel_runtime(self) -> &'static str {
        "RUNTIME"
    }

    pub(crate) const fn none_detected(self) -> &'static str {
        match self {
            Self::Spanish => "ninguno detectado",
            Self::English => "none detected",
        }
    }

    pub(crate) const fn panel_library_index(self) -> &'static str {
        match self {
            Self::Spanish => "INDICE BIBLIOTECA",
            Self::English => "LIBRARY INDEX",
        }
    }

    pub(crate) const fn browse(self) -> &'static str {
        match self {
            Self::Spanish => " navegar  ",
            Self::English => " browse  ",
        }
    }

    pub(crate) const fn mode_overdrive(self) -> &'static str {
        "overdrive"
    }

    pub(crate) const fn mode_normal(self) -> &'static str {
        "normal"
    }
}

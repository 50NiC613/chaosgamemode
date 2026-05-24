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
            Self::Spanish => "LISTO",
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
            Self::Spanish => "Ejecuta Overdrive (1) o Restore (2) para ver resultados.",
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
            Self::Spanish => "auto detect espera escaneo Steam",
            Self::English => "auto detect waiting for Steam scan",
        }
    }

    pub(crate) fn auto_detect_ready(self, games: usize) -> String {
        match self {
            Self::Spanish => format!("auto detect listo: {games} juegos"),
            Self::English => format!("auto detect ready: {games} games"),
        }
    }

    pub(crate) const fn auto_detect_no_games(self) -> &'static str {
        match self {
            Self::Spanish => "auto detect sin juegos de Steam",
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
            Self::Spanish => "auto detect armado",
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
            Self::Spanish => "auto detect pausado para juego cerrado",
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
            Self::Spanish => "  Pulsa S en la pestana Steam para re-escanear la biblioteca.",
            Self::English => "  Press S in the Steam tab to rescan the library.",
        }
    }

    pub(crate) const fn overdrive_preview_title(self) -> &'static str {
        match self {
            Self::Spanish => "PREVIEW DE OVERDRIVE",
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
            Self::Spanish => "Luego lanzara",
            Self::English => "Then launch",
        }
    }

    pub(crate) const fn configured_processes_label(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos configurados",
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
            Self::Spanish => "Procesos detectados ahora",
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
            Self::Spanish => "se cerrara",
            Self::English => "will stop",
        }
    }

    pub(crate) const fn explorer_kept(self) -> &'static str {
        match self {
            Self::Spanish => "se mantiene abierto",
            Self::English => "kept running",
        }
    }

    pub(crate) const fn energy_label(self) -> &'static str {
        match self {
            Self::Spanish => "Energia",
            Self::English => "Power",
        }
    }

    pub(crate) const fn high_performance_plan(self) -> &'static str {
        match self {
            Self::Spanish => "Alto Rendimiento",
            Self::English => "High Performance",
        }
    }

    pub(crate) const fn balanced_plan(self) -> &'static str {
        match self {
            Self::Spanish => "Balanceado",
            Self::English => "Balanced",
        }
    }

    pub(crate) const fn no_changes(self) -> &'static str {
        match self {
            Self::Spanish => "sin cambios",
            Self::English => "unchanged",
        }
    }

    pub(crate) const fn overdrive_targets_heading(self) -> &'static str {
        match self {
            Self::Spanish => "Procesos que Overdrive intentara cerrar ahora:",
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
            Self::Spanish => "  (ningun proceso objetivo detectado en este perfil)",
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
            Self::Spanish => "perfil sin procesos configurados",
            Self::English => "profile has no configured processes",
        }
    }

    pub(crate) fn system_process_closed(self, line: &str) -> String {
        match self {
            Self::Spanish => format!("proceso cerrado: {line}"),
            Self::English => format!("process closed: {line}"),
        }
    }

    pub(crate) const fn system_no_heavy_process(self) -> &'static str {
        match self {
            Self::Spanish => "ningun proceso pesado encontrado",
            Self::English => "no heavy process found",
        }
    }

    pub(crate) const fn system_no_services(self) -> &'static str {
        match self {
            Self::Spanish => "perfil sin servicios configurados",
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
            Self::Spanish => "servicios ya optimizados",
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
            Self::Spanish => "plan de energia",
            Self::English => "power plan",
        }
    }

    pub(crate) const fn killing_background_processes(self) -> &'static str {
        match self {
            Self::Spanish => "eliminando procesos en segundo plano",
            Self::English => "closing background processes",
        }
    }

    pub(crate) const fn stopping_services(self) -> &'static str {
        match self {
            Self::Spanish => "deteniendo servicios",
            Self::English => "stopping services",
        }
    }

    pub(crate) const fn freeing_system_resources(self) -> &'static str {
        match self {
            Self::Spanish => "liberando recursos del sistema",
            Self::English => "freeing system resources",
        }
    }

    pub(crate) const fn explorer_kept_report(self) -> &'static str {
        match self {
            Self::Spanish => "explorer se mantiene activo en este perfil",
            Self::English => "explorer remains active in this profile",
        }
    }

    pub(crate) const fn overdrive_activated(self) -> &'static str {
        match self {
            Self::Spanish => "CHAOS GAME MODE ACTIVADO",
            Self::English => "CHAOS GAME MODE ACTIVATED",
        }
    }

    pub(crate) const fn restoring_windows_shell(self) -> &'static str {
        match self {
            Self::Spanish => "restaurando interfaz de Windows",
            Self::English => "restoring Windows shell",
        }
    }

    pub(crate) const fn restoring_services(self) -> &'static str {
        match self {
            Self::Spanish => "restaurando servicios",
            Self::English => "restoring services",
        }
    }

    pub(crate) const fn system_restored(self) -> &'static str {
        match self {
            Self::Spanish => "SISTEMA RESTAURADO",
            Self::English => "SYSTEM RESTORED",
        }
    }

    pub(crate) const fn closed_apps_not_reopened(self) -> &'static str {
        match self {
            Self::Spanish => "apps cerradas no se reabren solas",
            Self::English => "closed apps are not reopened automatically",
        }
    }
}

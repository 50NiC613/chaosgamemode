use crate::app::Tab;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    Spanish,
    English,
}

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
}

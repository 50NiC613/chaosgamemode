use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::overlay::{OverlayBackend, OverlayConfig};

#[derive(Clone)]
pub(crate) struct AppConfig {
    path: Option<PathBuf>,
    active_profile: ProfileName,
    safe: BoostProfile,
    balanced: BoostProfile,
    aggressive: BoostProfile,
    pub(crate) telemetry: TelemetryConfig,
    pub(crate) overlay: OverlayConfig,
    pub(crate) ui: UiConfig,
    pub(crate) status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProfileName {
    Safe,
    Balanced,
    Aggressive,
}

#[derive(Clone)]
pub(crate) struct BoostProfile {
    pub(crate) name: &'static str,
    pub(crate) processes: Vec<String>,
    pub(crate) protected_processes: Vec<String>,
    pub(crate) hidden_processes: Vec<String>,
    pub(crate) services: Vec<String>,
    pub(crate) set_high_performance: bool,
    pub(crate) prioritize_steam: bool,
    pub(crate) kill_explorer: bool,
}

#[derive(Clone)]
pub(crate) struct TelemetryConfig {
    pub(crate) telemetry_rate: Duration,
    pub(crate) process_rate: Duration,
    pub(crate) platform_rate: Duration,
}

#[derive(Clone, Default)]
pub(crate) struct UiConfig {
    pub(crate) language: Language,
}

#[derive(Default, Deserialize)]
struct ConfigFile {
    active_profile: Option<String>,
    telemetry: Option<TelemetryFile>,
    overlay: Option<OverlayFile>,
    ui: Option<UiFile>,
    profiles: Option<ProfilesFile>,
}

#[derive(Default, Deserialize)]
struct TelemetryFile {
    telemetry_ms: Option<u64>,
    process_ms: Option<u64>,
    platform_ms: Option<u64>,
}

#[derive(Default, Deserialize)]
struct OverlayFile {
    enabled: Option<bool>,
    backend: Option<String>,
    update_ms: Option<u64>,
}

#[derive(Default, Deserialize)]
struct UiFile {
    language: Option<String>,
}

#[derive(Default, Deserialize)]
struct ProfilesFile {
    safe: Option<BoostProfileFile>,
    balanced: Option<BoostProfileFile>,
    aggressive: Option<BoostProfileFile>,
}

#[derive(Default, Deserialize)]
struct BoostProfileFile {
    processes: Option<Vec<String>>,
    protected_processes: Option<Vec<String>>,
    hidden_processes: Option<Vec<String>>,
    services: Option<Vec<String>>,
    set_high_performance: Option<bool>,
    prioritize_steam: Option<bool>,
    kill_explorer: Option<bool>,
}

impl AppConfig {
    pub(crate) fn load() -> Self {
        let Some(path) = find_config_file() else {
            return Self {
                status: "config.toml no encontrado; usando defaults".to_string(),
                ..Self::default()
            };
        };

        match fs::read_to_string(&path)
            .map_err(|err| format!("no se pudo leer config.toml: {err}"))
            .and_then(|contents| {
                toml::from_str::<ConfigFile>(&contents)
                    .map_err(|err| format!("config.toml invalido: {err}"))
            }) {
            Ok(file) => Self::from_file(file, path),
            Err(err) => Self {
                status: format!("config error: {err}; usando defaults"),
                ..Self::default()
            },
        }
    }

    pub(crate) fn active_profile(&self) -> &BoostProfile {
        match self.active_profile {
            ProfileName::Safe => &self.safe,
            ProfileName::Balanced => &self.balanced,
            ProfileName::Aggressive => &self.aggressive,
        }
    }

    pub(crate) fn active_profile_name(&self) -> &'static str {
        self.active_profile.as_str()
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn toggle_protected_process(&mut self, process_name: &str) {
        let pattern = process_pattern_from_name(process_name);
        if pattern.is_empty() {
            self.status = "process config: nombre vacio".to_string();
            return;
        }

        let profile = self.active_profile_mut();
        if profile.has_protected_pattern(&pattern) {
            profile.remove_protected_pattern(&pattern);
            self.persist(format!("process config: {pattern} ya no esta protegido"));
        } else {
            profile.remove_process_pattern(&pattern);
            profile.add_protected_pattern(pattern.clone());
            self.persist(format!("process config: {pattern} protegido"));
        }
    }

    pub(crate) fn toggle_target_process(&mut self, process_name: &str) {
        let pattern = process_pattern_from_name(process_name);
        if pattern.is_empty() {
            self.status = "process config: nombre vacio".to_string();
            return;
        }
        if self.active_profile().is_hidden_process(&pattern) {
            self.status = format!("process config: {pattern} es sistema/oculto");
            return;
        }

        let profile = self.active_profile_mut();
        if profile.has_process_pattern(&pattern) && !profile.has_protected_pattern(&pattern) {
            profile.remove_process_pattern(&pattern);
            self.persist(format!("process config: {pattern} desmarcado"));
        } else {
            profile.remove_protected_pattern(&pattern);
            profile.add_process_pattern(pattern.clone());
            self.persist(format!("process config: {pattern} marcado como objetivo"));
        }
    }

    pub(crate) fn neutralize_process(&mut self, process_name: &str) {
        let pattern = process_pattern_from_name(process_name);
        if pattern.is_empty() {
            self.status = "process config: nombre vacio".to_string();
            return;
        }

        let profile = self.active_profile_mut();
        profile.remove_process_pattern(&pattern);
        profile.remove_protected_pattern(&pattern);
        profile.remove_hidden_pattern_for(&pattern);
        self.persist(format!("process config: {pattern} neutral"));
    }

    pub(crate) fn hide_process(&mut self, process_name: &str) {
        let pattern = process_pattern_from_name(process_name);
        if pattern.is_empty() {
            self.status = "process config: nombre vacio".to_string();
            return;
        }
        if is_builtin_hidden_process(&pattern) {
            self.status = format!("process config: {pattern} ya es oculto del sistema");
            return;
        }

        let profile = self.active_profile_mut();
        profile.remove_process_pattern(&pattern);
        profile.remove_protected_pattern(&pattern);
        profile.add_hidden_pattern(pattern.clone());
        self.persist(format!("process config: {pattern} oculto"));
    }

    pub(crate) fn unhide_process(&mut self, process_name: &str) {
        let pattern = process_pattern_from_name(process_name);
        if pattern.is_empty() {
            self.status = "process config: nombre vacio".to_string();
            return;
        }
        if is_builtin_hidden_process(&pattern) {
            self.status = format!("process config: {pattern} es oculto interno de Windows");
            return;
        }

        let profile = self.active_profile_mut();
        if profile.remove_hidden_pattern_for(&pattern) {
            self.persist(format!("process config: {pattern} visible"));
        } else {
            self.status = format!("process config: {pattern} no estaba oculto");
        }
    }

    pub(crate) fn toggle_overlay_enabled(&mut self) {
        self.overlay.enabled = !self.overlay.enabled;
        let state = if self.overlay.enabled {
            "enabled"
        } else {
            "disabled"
        };
        self.persist(format!("overlay: {state}"));
    }

    fn from_file(file: ConfigFile, path: PathBuf) -> Self {
        let mut config = Self {
            path: Some(path.clone()),
            ..Self::default()
        };
        if let Some(active) = file.active_profile.as_deref().and_then(ProfileName::parse) {
            config.active_profile = active;
        }
        if let Some(telemetry) = file.telemetry {
            config.telemetry = config.telemetry.with_override(telemetry);
        }
        if let Some(overlay) = file.overlay {
            config.overlay = config.overlay.with_override(overlay);
        }
        if let Some(ui) = file.ui {
            config.ui = config.ui.with_override(ui);
        }
        if let Some(profiles) = file.profiles {
            if let Some(safe) = profiles.safe {
                config.safe.apply_override(safe);
            }
            if let Some(balanced) = profiles.balanced {
                config.balanced.apply_override(balanced);
            }
            if let Some(aggressive) = profiles.aggressive {
                config.aggressive.apply_override(aggressive);
            }
        }
        config.status = format!("config loaded: {}", path.display());
        config
    }

    fn active_profile_mut(&mut self) -> &mut BoostProfile {
        match self.active_profile {
            ProfileName::Safe => &mut self.safe,
            ProfileName::Balanced => &mut self.balanced,
            ProfileName::Aggressive => &mut self.aggressive,
        }
    }

    fn persist(&mut self, success_status: String) {
        let Some(path) = self.path.as_ref() else {
            self.status = format!("{success_status}; no hay config.toml para guardar");
            return;
        };

        let contents = match toml::to_string_pretty(&self.to_file()) {
            Ok(contents) => contents,
            Err(err) => {
                self.status = format!("config save error: {err}");
                return;
            }
        };

        match fs::write(path, contents) {
            Ok(()) => {
                self.status = success_status;
            }
            Err(err) => {
                self.status = format!("config save error: {err}");
            }
        }
    }

    fn to_file(&self) -> ConfigFileOut {
        ConfigFileOut {
            active_profile: self.active_profile.as_str().to_string(),
            telemetry: TelemetryFileOut {
                telemetry_ms: self.telemetry.telemetry_rate.as_millis() as u64,
                process_ms: self.telemetry.process_rate.as_millis() as u64,
                platform_ms: self.telemetry.platform_rate.as_millis() as u64,
            },
            overlay: OverlayFileOut {
                enabled: self.overlay.enabled,
                backend: self.overlay.backend.as_str().to_string(),
                update_ms: self.overlay.update_rate.as_millis() as u64,
            },
            ui: UiFileOut {
                language: self.ui.language.code().to_string(),
            },
            profiles: ProfilesFileOut {
                safe: BoostProfileFileOut::from_profile(&self.safe),
                balanced: BoostProfileFileOut::from_profile(&self.balanced),
                aggressive: BoostProfileFileOut::from_profile(&self.aggressive),
            },
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path: None,
            active_profile: ProfileName::Balanced,
            safe: BoostProfile::safe(),
            balanced: BoostProfile::balanced(),
            aggressive: BoostProfile::aggressive(),
            telemetry: TelemetryConfig::default(),
            overlay: OverlayConfig::default(),
            ui: UiConfig::default(),
            status: "config defaults".to_string(),
        }
    }
}

impl ProfileName {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "safe" => Some(Self::Safe),
            "balanced" => Some(Self::Balanced),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Balanced => "balanced",
            Self::Aggressive => "aggressive",
        }
    }
}

impl BoostProfile {
    pub(crate) fn is_protected_process(&self, process_name: &str) -> bool {
        let process_name = process_name.to_ascii_lowercase();
        if is_builtin_protected_process(&process_name) {
            return true;
        }
        self.protected_processes
            .iter()
            .any(|pattern| process_name.contains(&pattern.to_ascii_lowercase()))
    }

    pub(crate) fn is_target_process(&self, process_name: &str) -> bool {
        let process_name = process_name.to_ascii_lowercase();
        if self.is_hidden_process(&process_name) {
            return false;
        }

        self.processes
            .iter()
            .any(|pattern| process_name.contains(&pattern.to_ascii_lowercase()))
            && !self.is_protected_process(&process_name)
    }

    pub(crate) fn is_hidden_process(&self, process_name: &str) -> bool {
        let process_key = process_pattern_from_name(process_name);
        is_builtin_hidden_process(&process_key)
            || self.hidden_processes.iter().any(|pattern| {
                let pattern = process_pattern_from_name(pattern);
                !pattern.is_empty() && process_key.contains(&pattern)
            })
    }

    fn safe() -> Self {
        Self {
            name: "safe",
            processes: strings(&[
                "chrome",
                "msedge",
                "msedgewebview2",
                "dropbox",
                "googledrivefs",
                "gdrive",
                "onedrive",
                "discord",
                "slack",
                "teams",
                "zoom",
                "spotify",
                "whatsapp",
                "telegram",
            ]),
            protected_processes: default_protected_processes(),
            hidden_processes: default_hidden_processes(),
            services: Vec::new(),
            set_high_performance: true,
            prioritize_steam: true,
            kill_explorer: false,
        }
    }

    fn balanced() -> Self {
        Self {
            name: "balanced",
            processes: default_processes(),
            protected_processes: default_protected_processes(),
            hidden_processes: default_hidden_processes(),
            services: default_services(),
            set_high_performance: true,
            prioritize_steam: true,
            kill_explorer: false,
        }
    }

    fn aggressive() -> Self {
        Self {
            name: "aggressive",
            processes: default_processes(),
            protected_processes: default_protected_processes(),
            hidden_processes: default_hidden_processes(),
            services: default_services(),
            set_high_performance: true,
            prioritize_steam: true,
            kill_explorer: true,
        }
    }

    fn apply_override(&mut self, override_file: BoostProfileFile) {
        if let Some(processes) = override_file.processes {
            self.processes = processes;
        }
        if let Some(protected_processes) = override_file.protected_processes {
            self.protected_processes = protected_processes;
        }
        if let Some(hidden_processes) = override_file.hidden_processes {
            self.hidden_processes = hidden_processes;
        }
        if let Some(services) = override_file.services {
            self.services = services;
        }
        if let Some(value) = override_file.set_high_performance {
            self.set_high_performance = value;
        }
        if let Some(value) = override_file.prioritize_steam {
            self.prioritize_steam = value;
        }
        if let Some(value) = override_file.kill_explorer {
            self.kill_explorer = value;
        }
    }

    fn has_process_pattern(&self, pattern: &str) -> bool {
        self.processes
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(pattern))
    }

    fn has_protected_pattern(&self, pattern: &str) -> bool {
        self.protected_processes
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(pattern))
    }

    fn add_process_pattern(&mut self, pattern: String) {
        if !self.has_process_pattern(&pattern) {
            self.processes.push(pattern);
            self.processes.sort();
            self.processes.dedup();
        }
    }

    fn add_protected_pattern(&mut self, pattern: String) {
        if !self.has_protected_pattern(&pattern) {
            self.protected_processes.push(pattern);
            self.protected_processes.sort();
            self.protected_processes.dedup();
        }
    }

    fn add_hidden_pattern(&mut self, pattern: String) {
        if !self.has_hidden_pattern(&pattern) {
            self.hidden_processes.push(pattern);
            self.hidden_processes.sort();
            self.hidden_processes.dedup();
        }
    }

    fn remove_process_pattern(&mut self, pattern: &str) {
        self.processes
            .retain(|entry| !entry.eq_ignore_ascii_case(pattern));
    }

    fn remove_protected_pattern(&mut self, pattern: &str) {
        self.protected_processes
            .retain(|entry| !entry.eq_ignore_ascii_case(pattern));
    }

    fn has_hidden_pattern(&self, pattern: &str) -> bool {
        self.hidden_processes
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(pattern))
    }

    fn remove_hidden_pattern_for(&mut self, process_pattern: &str) -> bool {
        let before = self.hidden_processes.len();
        self.hidden_processes.retain(|entry| {
            let hidden_pattern = process_pattern_from_name(entry);
            hidden_pattern.is_empty()
                || (!process_pattern.contains(&hidden_pattern)
                    && !hidden_pattern.contains(process_pattern))
        });
        self.hidden_processes.len() != before
    }
}

#[derive(Serialize)]
struct ConfigFileOut {
    active_profile: String,
    telemetry: TelemetryFileOut,
    overlay: OverlayFileOut,
    ui: UiFileOut,
    profiles: ProfilesFileOut,
}

#[derive(Serialize)]
struct TelemetryFileOut {
    telemetry_ms: u64,
    process_ms: u64,
    platform_ms: u64,
}

#[derive(Serialize)]
struct OverlayFileOut {
    enabled: bool,
    backend: String,
    update_ms: u64,
}

#[derive(Serialize)]
struct UiFileOut {
    language: String,
}

#[derive(Serialize)]
struct ProfilesFileOut {
    safe: BoostProfileFileOut,
    balanced: BoostProfileFileOut,
    aggressive: BoostProfileFileOut,
}

#[derive(Serialize)]
struct BoostProfileFileOut {
    set_high_performance: bool,
    prioritize_steam: bool,
    kill_explorer: bool,
    protected_processes: Vec<String>,
    hidden_processes: Vec<String>,
    services: Vec<String>,
    processes: Vec<String>,
}

impl BoostProfileFileOut {
    fn from_profile(profile: &BoostProfile) -> Self {
        Self {
            set_high_performance: profile.set_high_performance,
            prioritize_steam: profile.prioritize_steam,
            kill_explorer: profile.kill_explorer,
            protected_processes: profile.protected_processes.clone(),
            hidden_processes: profile.hidden_processes.clone(),
            services: profile.services.clone(),
            processes: profile.processes.clone(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            telemetry_rate: Duration::from_secs(1),
            process_rate: Duration::from_secs(3),
            platform_rate: Duration::from_secs(15),
        }
    }
}

impl TelemetryConfig {
    fn with_override(mut self, override_file: TelemetryFile) -> Self {
        if let Some(ms) = override_file.telemetry_ms {
            self.telemetry_rate = Duration::from_millis(ms.max(250));
        }
        if let Some(ms) = override_file.process_ms {
            self.process_rate = Duration::from_millis(ms.max(1_000));
        }
        if let Some(ms) = override_file.platform_ms {
            self.platform_rate = Duration::from_millis(ms.max(5_000));
        }
        self
    }
}

impl OverlayConfig {
    fn with_override(mut self, override_file: OverlayFile) -> Self {
        if let Some(enabled) = override_file.enabled {
            self.enabled = enabled;
        }
        if let Some(backend) = override_file
            .backend
            .as_deref()
            .and_then(OverlayBackend::parse)
        {
            self.backend = backend;
        }
        if let Some(ms) = override_file.update_ms {
            self.update_rate = Duration::from_millis(ms.clamp(100, 2_000));
        }
        self
    }
}

impl UiConfig {
    fn with_override(mut self, override_file: UiFile) -> Self {
        if let Some(language) = override_file.language.as_deref().and_then(Language::parse) {
            self.language = language;
        }
        self
    }
}

fn find_config_file() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CHAOS_CONFIG") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(path) = seed_installed_config() {
        return Some(path);
    }

    let mut candidates = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("config.toml"));
        candidates.push(current_dir.join("tui-rs").join("config.toml"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn seed_installed_config() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    copy_default_file(
        &exe_dir.join("config.default.toml"),
        &exe_dir.join("config.toml"),
    )
}

fn copy_default_file(default_path: &Path, target_path: &Path) -> Option<PathBuf> {
    if target_path.exists() {
        return target_path.is_file().then(|| target_path.to_path_buf());
    }
    if !default_path.is_file() {
        return None;
    }

    fs::copy(default_path, target_path).ok()?;
    target_path.is_file().then(|| target_path.to_path_buf())
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

pub(crate) fn process_pattern_from_name(process_name: &str) -> String {
    let mut value = process_name.trim().to_ascii_lowercase();
    if let Some(stripped) = value.strip_suffix(".exe") {
        value = stripped.to_string();
    }
    value
}

fn is_builtin_hidden_process(process_key: &str) -> bool {
    matches!(
        process_key,
        "system" | "registry" | "secure system" | "memory compression" | "idle"
    ) || process_key.ends_with("host")
}

fn is_builtin_protected_process(process_name: &str) -> bool {
    let process_key = process_pattern_from_name(process_name);
    [
        "amd",
        "radeon",
        "rtss",
        "rivatuner",
        "msiafterburner",
        "steelseries",
    ]
    .iter()
    .any(|pattern| process_key.contains(pattern))
}

fn default_services() -> Vec<String> {
    strings(&[
        "SysMain",
        "WSearch",
        "DiagTrack",
        "Spooler",
        "FontCache",
        "PcaSvc",
        "UsoSvc",
        "Themes",
        "WpnService",
    ])
}

fn default_protected_processes() -> Vec<String> {
    strings(&[
        "amd",
        "radeon",
        "rtss",
        "rivatuner",
        "msiafterburner",
        "steelseries",
    ])
}

fn default_hidden_processes() -> Vec<String> {
    strings(&[
        "smss",
        "csrss",
        "wininit",
        "winlogon",
        "services",
        "lsass",
        "svchost",
        "taskhostw",
        "dwm",
        "runtimebroker",
        "searchindexer",
        "searchapp",
        "msmpeng",
        "nissrv",
        "securityhealth",
        "mssense",
        "sense",
        "defender",
        "antimalware",
    ])
}

fn default_processes() -> Vec<String> {
    strings(&[
        "chrome",
        "msedge",
        "msedgewebview2",
        "firefox",
        "opera",
        "brave",
        "vivaldi",
        "dropbox",
        "googledrivefs",
        "gdrive",
        "onedrive",
        "filecoauth",
        "idman",
        "qbittorrent",
        "torrent",
        "transmission",
        "discord",
        "slack",
        "teams",
        "zoom",
        "skype",
        "spotify",
        "epomaker",
        "rapoo",
        "logitech",
        "razer",
        "anydesk",
        "teamviewer",
        "rcclient",
        "rcservice",
        "anyviewer",
        "vnc",
        "whatsapp",
        "telegram",
        "signal",
        "winword",
        "excel",
        "powerpnt",
        "outlook",
        "officeclicktorun",
        "onecommander",
        "files",
        "widgets",
        "widgetservice",
        "trafficmonitor",
        "hwmonitor",
        "cpuid",
        "opengameboost",
        "razercortex",
        "foxit",
        "acrobat",
        "adobereader",
        "snippingtool",
        "python",
        "node",
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_name_should_parse_known_profiles() {
        assert_eq!(
            ProfileName::parse("AGGRESSIVE"),
            Some(ProfileName::Aggressive)
        );
    }

    #[test]
    fn telemetry_override_should_clamp_fast_values() {
        let telemetry = TelemetryConfig::default().with_override(TelemetryFile {
            telemetry_ms: Some(1),
            process_ms: Some(1),
            platform_ms: Some(1),
        });

        assert_eq!(telemetry.telemetry_rate, Duration::from_millis(250));
        assert_eq!(telemetry.process_rate, Duration::from_millis(1_000));
        assert_eq!(telemetry.platform_rate, Duration::from_millis(5_000));
    }

    #[test]
    fn overlay_config_should_load_rtss_settings() {
        let config = AppConfig::from_file(
            ConfigFile {
                overlay: Some(OverlayFile {
                    enabled: Some(false),
                    backend: Some("rtss".to_string()),
                    update_ms: Some(75),
                }),
                ..ConfigFile::default()
            },
            PathBuf::from("config.toml"),
        );

        assert!(!config.overlay.enabled);
        assert_eq!(config.overlay.backend, OverlayBackend::Rtss);
        assert_eq!(config.overlay.update_rate, Duration::from_millis(100));
    }

    #[test]
    fn ui_config_should_load_language() {
        let config = AppConfig::from_file(
            ConfigFile {
                ui: Some(UiFile {
                    language: Some("en".to_string()),
                }),
                ..ConfigFile::default()
            },
            PathBuf::from("config.toml"),
        );

        assert_eq!(config.ui.language, Language::English);
    }

    #[test]
    fn process_pattern_should_drop_exe_suffix() {
        assert_eq!(
            process_pattern_from_name("SteelSeriesGG.exe"),
            "steelseriesgg"
        );
    }

    #[test]
    fn hidden_process_should_match_windows_host_suffix() {
        let profile = BoostProfile::balanced();

        assert!(profile.is_hidden_process("SearchHost.exe"));
    }

    #[test]
    fn hidden_process_should_not_be_targeted() {
        let mut profile = BoostProfile::balanced();
        profile.processes.push("searchhost".to_string());

        assert!(!profile.is_target_process("SearchHost.exe"));
    }

    #[test]
    fn hidden_process_should_match_defender_patterns() {
        let profile = BoostProfile::balanced();

        assert!(profile.is_hidden_process("MsMpEng.exe"));
    }

    #[test]
    fn gpu_overlay_tools_should_be_protected_even_when_config_targets_them() {
        let mut profile = BoostProfile::balanced();
        profile.processes.push("radeonsoftware".to_string());
        profile.processes.push("msiafterburner".to_string());

        assert!(profile.is_protected_process("RadeonSoftware.exe"));
        assert!(profile.is_protected_process("MSIAfterburner.exe"));
        assert!(!profile.is_target_process("RadeonSoftware.exe"));
        assert!(!profile.is_target_process("MSIAfterburner.exe"));
    }

    #[test]
    fn remove_hidden_pattern_should_match_selected_process_name() {
        let mut profile = BoostProfile::balanced();

        assert!(profile.remove_hidden_pattern_for("securityhealthsystray"));
    }

    #[test]
    fn copy_default_file_should_seed_missing_config() {
        let dir =
            std::env::temp_dir().join(format!("chaosgamemode-config-seed-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("config seed fixture dir should be writable");
        let default_path = dir.join("config.default.toml");
        let target_path = dir.join("config.toml");
        fs::write(&default_path, "active_profile = \"balanced\"\n")
            .expect("default config fixture should be writable");

        assert_eq!(
            copy_default_file(&default_path, &target_path),
            Some(target_path.clone())
        );
        assert_eq!(
            fs::read_to_string(&target_path).expect("seeded config should be readable"),
            "active_profile = \"balanced\"\n"
        );

        fs::remove_dir_all(dir).expect("config seed fixture dir should be removable");
    }
}

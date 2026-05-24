use std::env;
use std::io;
use std::path::{Path, PathBuf};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::config::AppConfig;
use crate::frames::probe_frame_backend;
use crate::theme::ThemeWatcher;

pub(crate) fn run() -> io::Result<()> {
    println!("{}", build_report().render());
    Ok(())
}

struct DoctorReport {
    version: &'static str,
    exe_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
    config_status: CheckLine,
    theme_status: CheckLine,
    rtss_status: CheckLine,
    rtss_process_status: CheckLine,
    install_files: Vec<CheckLine>,
    notes: Vec<&'static str>,
}

struct CheckLine {
    label: &'static str,
    ok: bool,
    detail: String,
}

impl CheckLine {
    fn ok(label: &'static str, detail: impl Into<String>) -> Self {
        Self {
            label,
            ok: true,
            detail: detail.into(),
        }
    }

    fn warn(label: &'static str, detail: impl Into<String>) -> Self {
        Self {
            label,
            ok: false,
            detail: detail.into(),
        }
    }

    fn render(&self) -> String {
        format!(
            "[{}] {:<18} {}",
            if self.ok { "ok" } else { "warn" },
            self.label,
            self.detail
        )
    }
}

fn build_report() -> DoctorReport {
    let exe_path = env::current_exe().ok();
    let cwd = env::current_dir().ok();
    let install_dir = exe_path.as_ref().and_then(|path| path.parent());
    let config = AppConfig::load();
    let (theme_watcher, _theme, theme_preset, theme_status) = ThemeWatcher::new();
    let frame_probe = probe_frame_backend();
    let rtss_processes = running_rtss_processes();

    DoctorReport {
        version: env!("CARGO_PKG_VERSION"),
        exe_path: exe_path.clone(),
        cwd,
        config_status: match config.path() {
            Some(path) => {
                CheckLine::ok("config", format!("{} ({})", path.display(), config.status))
            }
            None => CheckLine::warn("config", config.status),
        },
        theme_status: match theme_watcher.path() {
            Some(path) => CheckLine::ok(
                "theme",
                format!(
                    "{} ({theme_status}, preset {})",
                    path.display(),
                    theme_preset.name()
                ),
            ),
            None => CheckLine::warn("theme", theme_status),
        },
        rtss_status: if frame_probe.available {
            CheckLine::ok("rtss shared memory", frame_probe.status)
        } else {
            CheckLine::warn("rtss shared memory", frame_probe.status)
        },
        rtss_process_status: if rtss_processes.is_empty() {
            CheckLine::warn("rtss process", "RTSS.exe not detected")
        } else {
            CheckLine::ok("rtss process", rtss_processes.join(", "))
        },
        install_files: install_dir.map_or_else(Vec::new, install_file_checks),
        notes: vec![
            "FPS/frametime require RTSS running with OSD enabled for the game.",
            "Launch a Steam game, open the Frames tab, then use Shift+F12 to toggle the overlay.",
            "If FPS stay unavailable, verify the game profile is not disabled in RTSS.",
        ],
    }
}

impl DoctorReport {
    fn render(&self) -> String {
        let mut lines = vec![
            format!("Chaos Game Mode doctor v{}", self.version),
            format!(
                "exe: {}",
                self.exe_path
                    .as_ref()
                    .map(|path| display_path(path))
                    .unwrap_or_else(|| "unknown".to_string())
            ),
            format!(
                "cwd: {}",
                self.cwd
                    .as_ref()
                    .map(|path| display_path(path))
                    .unwrap_or_else(|| "unknown".to_string())
            ),
            String::new(),
            self.config_status.render(),
            self.theme_status.render(),
            self.rtss_process_status.render(),
            self.rtss_status.render(),
        ];

        if !self.install_files.is_empty() {
            lines.push(String::new());
            lines.push("installed files:".to_string());
            lines.extend(self.install_files.iter().map(CheckLine::render));
        }

        lines.push(String::new());
        lines.push("next:".to_string());
        lines.extend(self.notes.iter().map(|note| format!("- {note}")));
        lines.join("\n")
    }
}

fn install_file_checks(install_dir: &Path) -> Vec<CheckLine> {
    [
        "chaosgamemode.exe",
        "config.toml",
        "theme.toml",
        "config.default.toml",
        "theme.default.toml",
    ]
    .into_iter()
    .map(|file| {
        let path = install_dir.join(file);
        if path.is_file() {
            CheckLine::ok(file, path.display().to_string())
        } else {
            CheckLine::warn(file, format!("missing: {}", path.display()))
        }
    })
    .collect()
}

fn running_rtss_processes() -> Vec<String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_exe(UpdateKind::OnlyIfNotSet),
    );

    let mut processes = sys
        .processes()
        .values()
        .filter_map(|process| {
            let name = process.name().to_string_lossy();
            let normalized = name.to_ascii_lowercase();
            (normalized == "rtss.exe"
                || normalized == "rtss"
                || normalized == "rtsshooksloader64.exe"
                || normalized == "rtsshooksloader64")
                .then(|| name.into_owned())
        })
        .collect::<Vec<_>>();
    processes.sort();
    processes.dedup();
    processes
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_line_should_render_ok_status() {
        assert_eq!(
            CheckLine::ok("config", "config.toml").render(),
            "[ok] config             config.toml"
        );
    }

    #[test]
    fn report_should_include_actionable_next_steps() {
        let report = DoctorReport {
            version: "1.0.0",
            exe_path: None,
            cwd: None,
            config_status: CheckLine::ok("config", "ok"),
            theme_status: CheckLine::ok("theme", "ok"),
            rtss_status: CheckLine::warn("rtss shared memory", "missing"),
            rtss_process_status: CheckLine::warn("rtss process", "missing"),
            install_files: Vec::new(),
            notes: vec!["Launch a Steam game."],
        };

        let rendered = report.render();

        assert!(rendered.contains("Chaos Game Mode doctor v1.0.0"));
        assert!(rendered.contains("Launch a Steam game."));
    }
}

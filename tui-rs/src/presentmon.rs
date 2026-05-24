use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

const FRAME_WINDOW: usize = 600;

#[derive(Clone, Debug)]
pub(crate) struct FrameMetrics {
    pub(crate) process_name: Option<String>,
    pub(crate) fps: Option<f64>,
    pub(crate) average_fps: Option<f64>,
    pub(crate) low_1_fps: Option<f64>,
    pub(crate) frame_time_ms: Option<f64>,
    pub(crate) samples: usize,
    pub(crate) status: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PresentMonProbe {
    pub(crate) path: Option<PathBuf>,
    pub(crate) source: &'static str,
    pub(crate) status: String,
}

impl FrameMetrics {
    pub(crate) fn idle() -> Self {
        Self {
            process_name: None,
            fps: None,
            average_fps: None,
            low_1_fps: None,
            frame_time_ms: None,
            samples: 0,
            status: "PresentMon waiting for Steam game".to_string(),
        }
    }

    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: message.into(),
            ..Self::idle()
        }
    }
}

pub(crate) fn probe_presentmon(configured: Option<&Path>) -> PresentMonProbe {
    if let Some(path) = configured {
        if path.is_file() {
            return PresentMonProbe {
                path: Some(path.to_path_buf()),
                source: "config.toml",
                status: "PresentMon listo desde config".to_string(),
            };
        }
        return PresentMonProbe {
            path: Some(path.to_path_buf()),
            source: "config.toml",
            status: "PresentMon configurado pero no existe".to_string(),
        };
    }

    if let Ok(path) = std::env::var("PRESENTMON_EXE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return PresentMonProbe {
                path: Some(path),
                source: "PRESENTMON_EXE",
                status: "PresentMon listo desde env".to_string(),
            };
        }
        return PresentMonProbe {
            path: Some(path),
            source: "PRESENTMON_EXE",
            status: "PRESENTMON_EXE apunta a una ruta invalida".to_string(),
        };
    }

    if let Some(path) = find_known_presentmon_exe() {
        return PresentMonProbe {
            path: Some(path),
            source: "winget",
            status: "PresentMon listo desde winget".to_string(),
        };
    }

    if let Some(path) = find_command("presentmon.exe")
        .or_else(|| find_command("presentmon"))
        .or_else(|| find_command("PresentMon.exe"))
        .or_else(|| find_command("PresentMon-2.4.1-x64.exe"))
        .or_else(|| find_command("PresentMon-2.3.1-x64.exe"))
    {
        return PresentMonProbe {
            path: Some(path),
            source: "PATH",
            status: "PresentMon listo desde PATH".to_string(),
        };
    }

    PresentMonProbe {
        path: None,
        source: "none",
        status: "PresentMon no encontrado".to_string(),
    }
}

pub(crate) fn spawn_frame_capture(
    process_name: String,
    configured_exe: Option<PathBuf>,
) -> Receiver<FrameMetrics> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = run_frame_capture(&process_name, configured_exe.as_deref(), |metrics| {
            tx.send(metrics).is_ok()
        });
        if let Err(message) = result {
            let _ = tx.send(FrameMetrics::unavailable(message));
        }
    });
    rx
}

fn run_frame_capture(
    process_name: &str,
    configured_exe: Option<&Path>,
    mut publish: impl FnMut(FrameMetrics) -> bool,
) -> Result<(), String> {
    let probe = probe_presentmon(configured_exe);
    let Some(exe) = probe.path else {
        return Err("PresentMon no encontrado; configura integrations.presentmon_exe".to_string());
    };
    if !exe.is_file() {
        return Err(format!("PresentMon path invalido: {}", exe.display()));
    };

    let session_name = format!("ChaosGameMode-{}", sanitize_session_name(process_name));
    let mut child = Command::new(&exe)
        .args([
            "--process_name",
            process_name,
            "--output_stdout",
            "--v2_metrics",
            "--exclude_dropped",
            "--no_console_stats",
            "--terminate_on_proc_exit",
            "--session_name",
            &session_name,
            "--stop_existing_session",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("PresentMon launch error: {err}"))?;

    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        return Err("PresentMon no entrego stdout".to_string());
    };

    let reader = BufReader::new(stdout);
    let mut parser = PresentMonParser::new(process_name);
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                let _ = child.kill();
                return Err(format!("PresentMon read error: {err}"));
            }
        };

        if let Some(metrics) = parser.push_line(&line)
            && !publish(metrics)
        {
            let _ = child.kill();
            return Ok(());
        }
    }

    let _ = child.wait();
    Ok(())
}

fn find_known_presentmon_exe() -> Option<PathBuf> {
    let package_dir = PathBuf::from(std::env::var("LOCALAPPDATA").ok()?)
        .join("Microsoft")
        .join("WinGet")
        .join("Packages")
        .join("Intel.PresentMon.Console_Microsoft.Winget.Source_8wekyb3d8bbwe");
    let direct = package_dir.join("presentmon.exe");
    if direct.is_file() {
        return Some(direct);
    }

    std::fs::read_dir(package_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("presentmon.exe"))
                && path.is_file()
        })
}

fn find_command(name: &str) -> Option<PathBuf> {
    let output = Command::new("where.exe").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn sanitize_session_name(process_name: &str) -> String {
    process_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

struct PresentMonParser {
    process_name: String,
    header: Option<CsvHeader>,
    window: FrameWindow,
}

impl PresentMonParser {
    fn new(process_name: &str) -> Self {
        Self {
            process_name: process_name.to_string(),
            header: None,
            window: FrameWindow::default(),
        }
    }

    fn push_line(&mut self, line: &str) -> Option<FrameMetrics> {
        let fields = split_csv_line(line);
        if fields.len() < 2 {
            return None;
        }

        if CsvHeader::looks_like_header(&fields) {
            self.header = CsvHeader::from_fields(&fields);
            return None;
        }

        let header = self.header.as_ref()?;
        let sample = header.sample_from_fields(&fields)?;
        self.window.push(sample.frame_time_ms);
        Some(self.window.metrics(&self.process_name))
    }
}

#[derive(Clone)]
struct CsvHeader {
    frame_time_index: usize,
}

impl CsvHeader {
    fn looks_like_header(fields: &[String]) -> bool {
        fields.iter().any(|field| {
            let normalized = normalize_header(field);
            normalized == "application" || normalized == "processid"
        }) && fields.iter().any(|field| {
            matches!(
                normalize_header(field).as_str(),
                "msbetweenappstart" | "msbetweenpresents" | "displayedtime"
            )
        })
    }

    fn from_fields(fields: &[String]) -> Option<Self> {
        let frame_time_index = column_index(
            fields,
            &["msbetweenappstart", "msbetweenpresents", "displayedtime"],
        )?;
        Some(Self { frame_time_index })
    }

    fn sample_from_fields(&self, fields: &[String]) -> Option<FrameSample> {
        let frame_time_ms = fields
            .get(self.frame_time_index)
            .and_then(|value| parse_metric(value))?;
        if frame_time_ms <= 0.0 {
            return None;
        }
        Some(FrameSample { frame_time_ms })
    }
}

struct FrameSample {
    frame_time_ms: f64,
}

#[derive(Default)]
struct FrameWindow {
    frame_times_ms: VecDeque<f64>,
}

impl FrameWindow {
    fn push(&mut self, frame_time_ms: f64) {
        if self.frame_times_ms.len() == FRAME_WINDOW {
            self.frame_times_ms.pop_front();
        }
        self.frame_times_ms.push_back(frame_time_ms);
    }

    fn metrics(&self, process_name: &str) -> FrameMetrics {
        let current = self.frame_times_ms.back().copied();
        let average_frame_ms = mean(self.frame_times_ms.iter().copied());
        let low_1_frame_ms = percentile(self.frame_times_ms.iter().copied(), 0.99);

        FrameMetrics {
            process_name: Some(process_name.to_string()),
            fps: current.map(fps_from_frame_ms),
            average_fps: average_frame_ms.map(fps_from_frame_ms),
            low_1_fps: low_1_frame_ms.map(fps_from_frame_ms),
            frame_time_ms: current,
            samples: self.frame_times_ms.len(),
            status: format!("PresentMon tracking {process_name}"),
        }
    }
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(field.trim().to_string());
                field.clear();
            }
            _ => field.push(ch),
        }
    }

    fields.push(field.trim().to_string());
    fields
}

fn column_index(fields: &[String], candidates: &[&str]) -> Option<usize> {
    candidates.iter().find_map(|candidate| {
        fields
            .iter()
            .position(|field| normalize_header(field) == *candidate)
    })
}

fn normalize_header(value: &str) -> String {
    value
        .trim()
        .trim_matches('*')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn parse_metric(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("na") || value.is_empty() {
        return None;
    }
    value.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn fps_from_frame_ms(frame_ms: f64) -> f64 {
    1_000.0 / frame_ms
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut count = 0_usize;
    let mut sum = 0.0_f64;
    for value in values {
        count += 1;
        sum += value;
    }
    (count > 0).then_some(sum / count as f64)
}

fn percentile(values: impl Iterator<Item = f64>, percentile: f64) -> Option<f64> {
    let mut values: Vec<_> = values.collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let index = ((values.len() as f64 * percentile).ceil() as usize).min(values.len() - 1);
    values.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_csv_line_should_keep_quoted_commas() {
        let fields = split_csv_line(r#"Game.exe,123,"Composed: Flip, Independent",16.6"#);

        assert_eq!(fields[2], "Composed: Flip, Independent");
    }

    #[test]
    fn probe_should_report_invalid_configured_path() {
        let path = PathBuf::from(r"Z:\missing\PresentMon.exe");
        let probe = probe_presentmon(Some(&path));

        assert_eq!(probe.source, "config.toml");
        assert!(probe.path.is_some());
        assert!(probe.status.contains("no existe"));
    }

    #[test]
    fn parser_should_compute_fps_from_presentmon_csv() {
        let mut parser = PresentMonParser::new("Game.exe");
        parser.push_line("Application,ProcessID,MsBetweenAppStart");
        let metrics = parser
            .push_line("Game.exe,42,16.6667")
            .expect("frame should produce metrics");

        assert_eq!(metrics.samples, 1);
        assert!(metrics.fps.is_some_and(|fps| (fps - 60.0).abs() < 0.01));
    }

    #[test]
    fn frame_window_should_compute_one_percent_low_from_slow_frames() {
        let mut window = FrameWindow::default();
        for _ in 0..99 {
            window.push(16.6667);
        }
        window.push(33.3333);

        let metrics = window.metrics("Game.exe");

        assert!(
            metrics
                .low_1_fps
                .is_some_and(|fps| (fps - 30.0).abs() < 0.01)
        );
    }

    #[test]
    fn header_should_prefer_app_frame_time() {
        let fields = split_csv_line("Application,MsBetweenPresents,MsBetweenAppStart");
        let header = CsvHeader::from_fields(&fields).expect("header should parse");
        let sample = header
            .sample_from_fields(&split_csv_line("Game.exe,8,20"))
            .expect("sample should parse");

        assert_eq!(sample.frame_time_ms, 20.0);
    }
}

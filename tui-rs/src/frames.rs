use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const FRAME_WINDOW: usize = 600;
const POLL_RATE: Duration = Duration::from_millis(100);
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);
const STALE_AFTER: Duration = Duration::from_secs(2);

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
pub(crate) struct FrameProbe {
    pub(crate) available: bool,
    pub(crate) source: &'static str,
    pub(crate) status: String,
}

#[derive(Clone, Debug)]
struct RtssSample {
    process_name: String,
    frame_time_ms: f64,
    fps: f64,
    rtss_time1: u32,
    rtss_frames: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct RtssDiagnosticEntry {
    slot: u32,
    process_id: u32,
    process_name: Option<String>,
    raw_name_is_path: bool,
    fps: Option<f64>,
    frame_time_ms: Option<f64>,
    time_delta_ms: u32,
    frames: u32,
    status: String,
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
            status: "RTSS waiting for Steam game".to_string(),
        }
    }

    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: message.into(),
            ..Self::idle()
        }
    }
}

fn canonical_rtss_process_name(value: &str) -> Option<String> {
    let trimmed = value
        .trim()
        .trim_matches(|ch| ch == '<' || ch == '>' || ch == '"' || ch == '\'');
    let basename = trimmed.rsplit(['\\', '/']).next().unwrap_or(trimmed).trim();
    (!basename.is_empty()).then(|| basename.to_string())
}

fn process_names_match(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
        || canonical_rtss_process_name(left)
            .zip(canonical_rtss_process_name(right))
            .is_some_and(|(left, right)| left.eq_ignore_ascii_case(&right))
}

pub(crate) fn probe_frame_backend() -> FrameProbe {
    match rtss::RtssFrameReader::open() {
        Ok(_) => FrameProbe {
            available: true,
            source: "rtss",
            status: "RTSS listo".to_string(),
        },
        Err(status) => FrameProbe {
            available: false,
            source: "rtss",
            status,
        },
    }
}

pub(crate) fn run_rtss_dump() -> io::Result<()> {
    println!("{}", rtss_dump_report());
    Ok(())
}

fn rtss_dump_report() -> String {
    match rtss::read_diagnostic_entries() {
        Ok(entries) => render_rtss_dump_report(entries),
        Err(error) => [
            format!("Chaos Game Mode RTSS dump v{}", env!("CARGO_PKG_VERSION")),
            "status: error".to_string(),
            format!("error: {error}"),
            String::new(),
            "next: keep RTSS open, enable OSD for the game, then run this while the game is visible."
                .to_string(),
        ]
        .join("\n"),
    }
}

fn render_rtss_dump_report(entries: Vec<RtssDiagnosticEntry>) -> String {
    let mut lines = vec![
        format!("Chaos Game Mode RTSS dump v{}", env!("CARGO_PKG_VERSION")),
        "status: ok".to_string(),
        format!("entries: {}", entries.len()),
        String::new(),
    ];

    if entries.is_empty() {
        lines.push("No active RTSS app entries found.".to_string());
        lines.push("Run this while the game is open and RTSS OSD is enabled for it.".to_string());
        return lines.join("\n");
    }

    lines.push(
        "slot pid      source name                         fps   ft_ms  frames   dt_ms status"
            .to_string(),
    );
    lines.extend(entries.into_iter().map(rtss_dump_line));
    lines.push(String::new());
    lines.push("If Cyberpunk2077.exe is missing here, RTSS is not exposing the game's FPS to Chaos Game Mode.".to_string());
    lines.push("No local install paths are printed; source=path only means RTSS supplied a full path internally.".to_string());
    lines.join("\n")
}

fn rtss_dump_line(entry: RtssDiagnosticEntry) -> String {
    format!(
        "{:>4} {:>6}  {:<6} {:<28} {:>5} {:>7} {:>7} {:>7} {}",
        entry.slot,
        entry.process_id,
        if entry.raw_name_is_path {
            "path"
        } else {
            "name"
        },
        truncate_dump_field(
            entry.process_name.as_deref().unwrap_or("<invalid-name>"),
            28,
        ),
        format_optional_fps(entry.fps),
        format_optional_ms(entry.frame_time_ms),
        entry.frames,
        entry.time_delta_ms,
        entry.status,
    )
}

fn truncate_dump_field(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut shortened = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    shortened.push('~');
    shortened
}

fn format_optional_fps(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.0}", value.round().clamp(0.0, 999.0)))
        .unwrap_or_else(|| "---".to_string())
}

fn format_optional_ms(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.1}", value.clamp(0.0, 999.9)))
        .unwrap_or_else(|| "--.-".to_string())
}

pub(crate) fn spawn_frame_capture(process_name: String) -> Receiver<FrameMetrics> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut window = FrameWindow::default();
        let mut last_status = Instant::now() - Duration::from_secs(1);

        loop {
            match rtss::read_target_sample(&process_name) {
                Ok(Some(sample)) => {
                    if let Some(metrics) = window.push(sample) {
                        if tx.send(metrics).is_err() {
                            break;
                        }
                    } else if last_status.elapsed() >= Duration::from_secs(1) {
                        last_status = Instant::now();
                        if tx
                            .send(FrameMetrics::unavailable(format!(
                                "RTSS waiting fresh frames for {process_name}"
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Ok(None) => {
                    if last_status.elapsed() >= Duration::from_secs(1) {
                        last_status = Instant::now();
                        if tx
                            .send(FrameMetrics::unavailable(format!(
                                "RTSS waiting for {process_name}"
                            )))
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Err(message) => {
                    if tx.send(FrameMetrics::unavailable(message)).is_err() {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }

            thread::sleep(POLL_RATE);
        }
    });
    rx
}

pub(crate) fn spawn_frame_discovery(excludes: Vec<String>) -> Receiver<FrameMetrics> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        let excludes = excludes
            .into_iter()
            .map(|value| value.to_ascii_lowercase())
            .collect::<Vec<_>>();
        let mut windows: HashMap<String, FrameWindow> = HashMap::new();
        let mut published = false;
        let mut last_status = Instant::now() - Duration::from_secs(1);

        while started.elapsed() < DISCOVERY_TIMEOUT {
            match rtss::read_all_samples() {
                Ok(samples) => {
                    for sample in samples {
                        if excludes
                            .iter()
                            .any(|exclude| sample.process_name.eq_ignore_ascii_case(exclude))
                        {
                            continue;
                        }

                        let window = windows.entry(sample.process_name.clone()).or_default();
                        if let Some(metrics) = window.push(sample) {
                            published = true;
                            if tx.send(metrics).is_err() {
                                return;
                            }
                        }
                    }
                    if !published && last_status.elapsed() >= Duration::from_secs(1) {
                        last_status = Instant::now();
                        if tx
                            .send(FrameMetrics::unavailable(
                                "RTSS waiting for hooked game frames",
                            ))
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                Err(message) => {
                    let _ = tx.send(FrameMetrics::unavailable(message));
                    return;
                }
            }

            thread::sleep(POLL_RATE);
        }

        if !published {
            let _ = tx.send(FrameMetrics::unavailable(
                "RTSS did not expose an active game target",
            ));
        }
    });
    rx
}

#[derive(Default)]
struct FrameWindow {
    frame_times_ms: VecDeque<f64>,
    last_rtss_time1: Option<u32>,
    last_rtss_frames: Option<u32>,
    last_changed_at: Option<Instant>,
}

impl FrameWindow {
    fn push(&mut self, sample: RtssSample) -> Option<FrameMetrics> {
        let changed = self.last_rtss_time1 != Some(sample.rtss_time1)
            || self.last_rtss_frames != Some(sample.rtss_frames);
        if changed {
            self.last_rtss_time1 = Some(sample.rtss_time1);
            self.last_rtss_frames = Some(sample.rtss_frames);
            self.last_changed_at = Some(Instant::now());
            if self.frame_times_ms.len() == FRAME_WINDOW {
                self.frame_times_ms.pop_front();
            }
            self.frame_times_ms.push_back(sample.frame_time_ms);
        } else if self
            .last_changed_at
            .is_some_and(|changed_at| changed_at.elapsed() >= STALE_AFTER)
        {
            return None;
        }

        Some(self.metrics(&sample))
    }

    fn metrics(&self, sample: &RtssSample) -> FrameMetrics {
        let average_frame_ms = mean(self.frame_times_ms.iter().copied());
        let low_1_frame_ms = percentile(self.frame_times_ms.iter().copied(), 0.99);

        FrameMetrics {
            process_name: Some(sample.process_name.clone()),
            fps: Some(sample.fps),
            average_fps: average_frame_ms.map(fps_from_frame_ms),
            low_1_fps: low_1_frame_ms.map(fps_from_frame_ms),
            frame_time_ms: Some(sample.frame_time_ms),
            samples: self.frame_times_ms.len(),
            status: format!("RTSS tracking {}", sample.process_name),
        }
    }
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

#[cfg(windows)]
mod rtss {
    use std::{mem, ptr};

    use super::{RtssDiagnosticEntry, RtssSample};
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND, GetLastError, HANDLE};
    use windows_sys::Win32::System::Memory::{
        FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile,
    };

    const RTSS_MAP_NAME: &str = "RTSSSharedMemoryV2";
    const RTSS_SIGNATURE: u32 = 0x5254_5353;
    const RTSS_VERSION_2_0: u32 = 2 << 16;
    const MAX_PATH: usize = 260;

    #[repr(C)]
    struct RtssHeader {
        signature: u32,
        version: u32,
        app_entry_size: u32,
        app_arr_offset: u32,
        app_arr_size: u32,
        osd_entry_size: u32,
        osd_arr_offset: u32,
        osd_arr_size: u32,
        osd_frame: u32,
    }

    #[repr(C)]
    struct RtssAppEntryPrefix {
        process_id: u32,
        name: [u8; MAX_PATH],
        flags: u32,
        time0: u32,
        time1: u32,
        frames: u32,
        frame_time_us: u32,
        stat_flags: u32,
        stat_time0: u32,
        stat_time1: u32,
        stat_frames: u32,
        stat_count: u32,
        stat_framerate_min: u32,
        stat_framerate_avg: u32,
        stat_framerate_max: u32,
    }

    pub(super) struct RtssFrameReader {
        handle: HANDLE,
        view: MEMORY_MAPPED_VIEW_ADDRESS,
    }

    impl RtssFrameReader {
        pub(super) fn open() -> Result<Self, String> {
            let name = wide_null(RTSS_MAP_NAME);
            let handle = unsafe { OpenFileMappingW(FILE_MAP_READ, 0, name.as_ptr()) };
            if handle.is_null() {
                let error = unsafe { GetLastError() };
                if error == ERROR_FILE_NOT_FOUND {
                    return Err("RTSS not running; start RivaTuner Statistics Server".to_string());
                }
                return Err(format!("RTSS OpenFileMapping failed: {error}"));
            }

            let view = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0) };
            if view.Value.is_null() {
                let error = unsafe { GetLastError() };
                unsafe {
                    CloseHandle(handle);
                }
                return Err(format!("RTSS MapViewOfFile failed: {error}"));
            }

            let reader = Self { handle, view };
            reader.validate()?;
            Ok(reader)
        }

        fn validate(&self) -> Result<(), String> {
            let header = self.header();
            if header.signature != RTSS_SIGNATURE || header.version < RTSS_VERSION_2_0 {
                return Err("RTSS shared memory is not initialized".to_string());
            }
            if header.app_entry_size < mem::size_of::<RtssAppEntryPrefix>() as u32 {
                return Err("RTSS app entry size is unsupported".to_string());
            }
            if header.app_arr_size == 0 || header.app_arr_size > 512 {
                return Err("RTSS app array is unsupported".to_string());
            }
            Ok(())
        }

        fn samples(&self) -> Vec<RtssSample> {
            (0..self.header().app_arr_size)
                .filter_map(|slot| self.sample(slot))
                .collect()
        }

        fn diagnostic_entries(&self) -> Vec<RtssDiagnosticEntry> {
            (0..self.header().app_arr_size)
                .filter_map(|slot| self.diagnostic_entry(slot))
                .collect()
        }

        fn sample(&self, slot: u32) -> Option<RtssSample> {
            let entry =
                unsafe { ptr::read_unaligned(self.entry_ptr(slot) as *const RtssAppEntryPrefix) };
            if entry.process_id == 0 || entry.time1 <= entry.time0 {
                return None;
            }

            let process_name = super::canonical_rtss_process_name(&read_cstr(&entry.name)?)?;
            if !is_valid_name(&process_name) {
                return None;
            }

            let fps = if entry.frame_time_us > 0 {
                1_000_000.0 / f64::from(entry.frame_time_us)
            } else if entry.frames > 0 {
                1_000.0 * f64::from(entry.frames) / f64::from(entry.time1 - entry.time0)
            } else {
                return None;
            };
            if !fps.is_finite() || fps <= 0.0 {
                return None;
            }

            let frame_time_ms = if entry.frame_time_us > 0 {
                f64::from(entry.frame_time_us) / 1_000.0
            } else {
                1_000.0 / fps
            };

            Some(RtssSample {
                process_name,
                frame_time_ms,
                fps,
                rtss_time1: entry.time1,
                rtss_frames: entry.frames,
            })
        }

        fn diagnostic_entry(&self, slot: u32) -> Option<RtssDiagnosticEntry> {
            let entry =
                unsafe { ptr::read_unaligned(self.entry_ptr(slot) as *const RtssAppEntryPrefix) };
            let raw_name = read_cstr(&entry.name).unwrap_or_default();
            if entry.process_id == 0 && raw_name.is_empty() {
                return None;
            }

            let process_name =
                super::canonical_rtss_process_name(&raw_name).filter(|name| is_valid_name(name));
            let (fps, frame_time_ms) = live_frame_values(&entry);
            let time_delta_ms = entry.time1.saturating_sub(entry.time0);
            let status = diagnostic_status(process_name.as_deref(), fps, entry.frames);

            Some(RtssDiagnosticEntry {
                slot,
                process_id: entry.process_id,
                process_name,
                raw_name_is_path: raw_name.contains('\\') || raw_name.contains('/'),
                fps,
                frame_time_ms,
                time_delta_ms,
                frames: entry.frames,
                status,
            })
        }

        fn entry_ptr(&self, slot: u32) -> *const u8 {
            unsafe {
                (self.view.Value as *const u8).add(
                    self.header().app_arr_offset as usize
                        + (slot as usize * self.header().app_entry_size as usize),
                )
            }
        }

        fn header(&self) -> &RtssHeader {
            unsafe { &*(self.view.Value as *const RtssHeader) }
        }
    }

    impl Drop for RtssFrameReader {
        fn drop(&mut self) {
            unsafe {
                UnmapViewOfFile(self.view);
                CloseHandle(self.handle);
            }
        }
    }

    pub(super) fn read_target_sample(process_name: &str) -> Result<Option<RtssSample>, String> {
        let reader = RtssFrameReader::open()?;
        Ok(reader
            .samples()
            .into_iter()
            .find(|sample| super::process_names_match(&sample.process_name, process_name)))
    }

    pub(super) fn read_all_samples() -> Result<Vec<RtssSample>, String> {
        Ok(RtssFrameReader::open()?.samples())
    }

    pub(super) fn read_diagnostic_entries() -> Result<Vec<RtssDiagnosticEntry>, String> {
        Ok(RtssFrameReader::open()?.diagnostic_entries())
    }

    fn live_frame_values(entry: &RtssAppEntryPrefix) -> (Option<f64>, Option<f64>) {
        if entry.time1 <= entry.time0 {
            return (None, None);
        }

        let fps = if entry.frame_time_us > 0 {
            Some(1_000_000.0 / f64::from(entry.frame_time_us))
        } else if entry.frames > 0 {
            Some(1_000.0 * f64::from(entry.frames) / f64::from(entry.time1 - entry.time0))
        } else {
            None
        }
        .filter(|fps| fps.is_finite() && *fps > 0.0);

        let frame_time_ms = fps.map(|fps| {
            if entry.frame_time_us > 0 {
                f64::from(entry.frame_time_us) / 1_000.0
            } else {
                1_000.0 / fps
            }
        });

        (fps, frame_time_ms)
    }

    fn diagnostic_status(process_name: Option<&str>, fps: Option<f64>, frames: u32) -> String {
        match (process_name, fps, frames) {
            (None, _, _) => "invalid-name".to_string(),
            (Some(_), Some(_), _) => "live".to_string(),
            (Some(_), None, 0) => "no-frames".to_string(),
            (Some(_), None, _) => "stale/no-fps".to_string(),
        }
    }

    fn read_cstr(value: &[u8]) -> Option<String> {
        let end = value
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(value.len());
        let value = String::from_utf8_lossy(&value[..end]).trim().to_string();
        (!value.is_empty()).then_some(value)
    }

    fn is_valid_name(value: &str) -> bool {
        let normalized = value
            .trim()
            .trim_matches(|ch| ch == '<' || ch == '>' || ch == '"' || ch == '\'')
            .to_ascii_lowercase();
        !matches!(normalized.as_str(), "" | "unknown" | "unk" | "n/a" | "na")
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod rtss {
    use super::{RtssDiagnosticEntry, RtssSample};

    pub(super) struct RtssFrameReader;

    impl RtssFrameReader {
        pub(super) fn open() -> Result<Self, String> {
            Err("RTSS frame capture is only available on Windows".to_string())
        }
    }

    pub(super) fn read_target_sample(_process_name: &str) -> Result<Option<RtssSample>, String> {
        Err("RTSS frame capture is only available on Windows".to_string())
    }

    pub(super) fn read_all_samples() -> Result<Vec<RtssSample>, String> {
        Err("RTSS frame capture is only available on Windows".to_string())
    }

    pub(super) fn read_diagnostic_entries() -> Result<Vec<RtssDiagnosticEntry>, String> {
        Err("RTSS frame capture is only available on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_window_should_compute_one_percent_low_from_slow_frames() {
        let mut window = FrameWindow::default();
        for frame in 0..99 {
            assert!(
                window
                    .push(RtssSample {
                        process_name: "Game.exe".to_string(),
                        frame_time_ms: 16.6667,
                        fps: 60.0,
                        rtss_time1: frame,
                        rtss_frames: frame,
                    })
                    .is_some()
            );
        }

        let metrics = window
            .push(RtssSample {
                process_name: "Game.exe".to_string(),
                frame_time_ms: 33.3333,
                fps: 30.0,
                rtss_time1: 100,
                rtss_frames: 100,
            })
            .expect("fresh sample should produce metrics");

        assert!(
            metrics
                .low_1_fps
                .is_some_and(|fps| (fps - 30.0).abs() < 0.01)
        );
    }

    #[test]
    fn frame_window_should_drop_stale_samples() {
        let mut window = FrameWindow {
            last_rtss_time1: Some(10),
            last_rtss_frames: Some(10),
            last_changed_at: Some(Instant::now() - Duration::from_secs(3)),
            ..FrameWindow::default()
        };

        assert!(
            window
                .push(RtssSample {
                    process_name: "Game.exe".to_string(),
                    frame_time_ms: 16.6667,
                    fps: 60.0,
                    rtss_time1: 10,
                    rtss_frames: 10,
                })
                .is_none()
        );
    }

    #[test]
    fn canonical_rtss_process_name_should_strip_paths_and_quotes() {
        assert_eq!(
            canonical_rtss_process_name(r#""D:\Steam\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe""#)
                .as_deref(),
            Some("Cyberpunk2077.exe")
        );
    }

    #[test]
    fn process_names_match_should_compare_executable_basename() {
        assert!(process_names_match(
            r"D:\Steam\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe",
            "Cyberpunk2077.exe"
        ));
    }

    #[test]
    fn rtss_dump_report_should_render_entries_without_local_paths() {
        let report = render_rtss_dump_report(vec![RtssDiagnosticEntry {
            slot: 2,
            process_id: 1234,
            process_name: Some("Cyberpunk2077.exe".to_string()),
            raw_name_is_path: true,
            fps: Some(33.4),
            frame_time_ms: Some(29.9),
            time_delta_ms: 10_000,
            frames: 333,
            status: "live".to_string(),
        }]);

        assert!(report.contains("Cyberpunk2077.exe"));
        assert!(report.contains("source=path"));
        assert!(!report.contains("SteamLibrary"));
    }

    #[test]
    fn rtss_dump_report_should_explain_empty_snapshot() {
        let report = render_rtss_dump_report(Vec::new());

        assert!(report.contains("No active RTSS app entries found."));
    }
}

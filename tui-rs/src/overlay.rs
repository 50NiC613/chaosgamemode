use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::metrics::format_duration;

#[derive(Clone)]
pub(crate) struct OverlayConfig {
    pub(crate) enabled: bool,
    pub(crate) backend: OverlayBackend,
    pub(crate) update_rate: Duration,
    pub(crate) hud: OverlayHudConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayBackend {
    Rtss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayLayout {
    Compact,
    MangoHud,
    Debug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayHudField {
    FrameStats,
    Gpu,
    Cpu,
    Ram,
    Waste,
    Session,
    Profile,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverlayHudConfig {
    pub(crate) layout: OverlayLayout,
    pub(crate) show_frame_stats: bool,
    pub(crate) show_gpu: bool,
    pub(crate) show_cpu: bool,
    pub(crate) show_ram: bool,
    pub(crate) show_waste: bool,
    pub(crate) show_session: bool,
    pub(crate) show_profile: bool,
    pub(crate) show_target: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct OverlaySnapshot {
    pub(crate) armed: bool,
    pub(crate) enabled: bool,
    pub(crate) backend: OverlayBackend,
    pub(crate) game_name: Option<String>,
    pub(crate) process_name: Option<String>,
    pub(crate) fps: Option<f64>,
    pub(crate) average_fps: Option<f64>,
    pub(crate) low_1_fps: Option<f64>,
    pub(crate) frame_time_ms: Option<f64>,
    pub(crate) frame_samples: usize,
    pub(crate) frame_status: String,
    pub(crate) performance_alert: Option<String>,
    pub(crate) profile_name: String,
    pub(crate) overdrive: bool,
    pub(crate) session_elapsed: Option<Duration>,
    pub(crate) cpu_usage: f32,
    pub(crate) ram_usage_pct: u16,
    pub(crate) gpu_usage_pct: Option<u16>,
    pub(crate) gpu_temp_c: Option<f32>,
    pub(crate) waste_mb: f64,
    pub(crate) hud: OverlayHudConfig,
}

#[derive(Clone, Debug)]
pub(crate) struct OverlayStatus {
    pub(crate) active: bool,
    pub(crate) backend: OverlayBackend,
    pub(crate) message: String,
}

pub(crate) struct OverlayChannels {
    pub(crate) tx: Sender<OverlaySnapshot>,
    pub(crate) status_rx: Receiver<OverlayStatus>,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: OverlayBackend::Rtss,
            update_rate: Duration::from_millis(100),
            hud: OverlayHudConfig::default(),
        }
    }
}

impl Default for OverlayHudConfig {
    fn default() -> Self {
        Self {
            layout: OverlayLayout::MangoHud,
            show_frame_stats: true,
            show_gpu: true,
            show_cpu: true,
            show_ram: true,
            show_waste: true,
            show_session: true,
            show_profile: true,
            show_target: false,
        }
    }
}

impl OverlayBackend {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rtss" => Some(Self::Rtss),
            _ => None,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Rtss => "rtss",
        }
    }
}

impl OverlayLayout {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value
            .trim()
            .to_ascii_lowercase()
            .replace(['_', '-'], "")
            .as_str()
        {
            "compact" => Some(Self::Compact),
            "mangohud" | "mango" => Some(Self::MangoHud),
            "debug" | "diagnostic" => Some(Self::Debug),
            _ => None,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::MangoHud => "mangohud",
            Self::Debug => "debug",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::MangoHud => "MangoHUD",
            Self::Debug => "Debug",
        }
    }

    pub(crate) const fn next(self) -> Self {
        match self {
            Self::Compact => Self::MangoHud,
            Self::MangoHud => Self::Debug,
            Self::Debug => Self::Compact,
        }
    }
}

impl OverlayHudField {
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::FrameStats => "frame_stats",
            Self::Gpu => "gpu",
            Self::Cpu => "cpu",
            Self::Ram => "ram",
            Self::Waste => "waste",
            Self::Session => "session",
            Self::Profile => "profile",
            Self::Target => "target",
        }
    }
}

impl OverlayHudConfig {
    pub(crate) fn toggle(&mut self, field: OverlayHudField) -> bool {
        let value = match field {
            OverlayHudField::FrameStats => &mut self.show_frame_stats,
            OverlayHudField::Gpu => &mut self.show_gpu,
            OverlayHudField::Cpu => &mut self.show_cpu,
            OverlayHudField::Ram => &mut self.show_ram,
            OverlayHudField::Waste => &mut self.show_waste,
            OverlayHudField::Session => &mut self.show_session,
            OverlayHudField::Profile => &mut self.show_profile,
            OverlayHudField::Target => &mut self.show_target,
        };
        *value = !*value;
        *value
    }

    pub(crate) const fn field_enabled(self, field: OverlayHudField) -> bool {
        match field {
            OverlayHudField::FrameStats => self.show_frame_stats,
            OverlayHudField::Gpu => self.show_gpu,
            OverlayHudField::Cpu => self.show_cpu,
            OverlayHudField::Ram => self.show_ram,
            OverlayHudField::Waste => self.show_waste,
            OverlayHudField::Session => self.show_session,
            OverlayHudField::Profile => self.show_profile,
            OverlayHudField::Target => self.show_target,
        }
    }
}

impl OverlaySnapshot {
    pub(crate) fn disabled(backend: OverlayBackend) -> Self {
        Self {
            armed: false,
            enabled: false,
            backend,
            game_name: None,
            process_name: None,
            fps: None,
            average_fps: None,
            low_1_fps: None,
            frame_time_ms: None,
            frame_samples: 0,
            frame_status: "RTSS overlay off".to_string(),
            performance_alert: None,
            profile_name: "balanced".to_string(),
            overdrive: false,
            session_elapsed: None,
            cpu_usage: 0.0,
            ram_usage_pct: 0,
            gpu_usage_pct: None,
            gpu_temp_c: None,
            waste_mb: 0.0,
            hud: OverlayHudConfig::default(),
        }
    }

    pub(crate) fn armed(backend: OverlayBackend) -> Self {
        Self {
            armed: true,
            ..Self::disabled(backend)
        }
    }
}

impl OverlayStatus {
    pub(crate) fn disabled(backend: OverlayBackend) -> Self {
        Self {
            active: false,
            backend,
            message: "RTSS overlay off".to_string(),
        }
    }
}

pub(crate) fn spawn_overlay_thread(config: OverlayConfig) -> OverlayChannels {
    let (tx, rx) = mpsc::channel();
    let (status_tx, status_rx) = mpsc::channel();
    thread::spawn(move || run_overlay_loop(rx, status_tx, config));
    OverlayChannels { tx, status_rx }
}

fn run_overlay_loop(
    rx: Receiver<OverlaySnapshot>,
    status_tx: Sender<OverlayStatus>,
    config: OverlayConfig,
) {
    let mut rtss = RtssOverlay::new();
    let mut overlay_visible = false;
    let mut last_enabled = false;
    let mut last_status = String::new();
    let mut last_text = String::new();
    let mut last_update = Instant::now() - config.update_rate;

    let _ = status_tx.send(OverlayStatus::disabled(config.backend));

    loop {
        let mut snapshot = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(snapshot) => snapshot,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                if overlay_visible {
                    let _ = rtss.clear();
                }
                break;
            }
        };

        for pending in rx.try_iter() {
            snapshot = pending;
        }

        let enabled_changed = snapshot.enabled != last_enabled;
        let overlay_text = snapshot.enabled.then(|| format_overlay_text(&snapshot));

        if snapshot.enabled && !enabled_changed && last_update.elapsed() < config.update_rate {
            continue;
        }

        if snapshot.enabled
            && !enabled_changed
            && overlay_text.as_deref() == Some(last_text.as_str())
        {
            continue;
        }

        last_update = Instant::now();

        let status = if snapshot.enabled {
            match snapshot.backend {
                OverlayBackend::Rtss => match rtss.update(overlay_text.as_deref().unwrap_or("")) {
                    Ok(()) => {
                        overlay_visible = true;
                        last_enabled = true;
                        last_text = overlay_text.unwrap_or_default();
                        OverlayStatus {
                            active: true,
                            backend: OverlayBackend::Rtss,
                            message: "RTSS overlay active".to_string(),
                        }
                    }
                    Err(message) => {
                        overlay_visible = false;
                        last_enabled = false;
                        last_text.clear();
                        OverlayStatus {
                            active: false,
                            backend: OverlayBackend::Rtss,
                            message,
                        }
                    }
                },
            }
        } else {
            if overlay_visible || last_enabled {
                let _ = rtss.clear();
                overlay_visible = false;
            }
            last_enabled = false;
            last_text.clear();
            if snapshot.armed {
                OverlayStatus {
                    active: false,
                    backend: snapshot.backend,
                    message: "RTSS overlay armed; waiting for Steam game".to_string(),
                }
            } else {
                OverlayStatus::disabled(snapshot.backend)
            }
        };

        if status.message != last_status {
            last_status = status.message.clone();
            let _ = status_tx.send(status);
        }
    }
}

fn format_overlay_text(snapshot: &OverlaySnapshot) -> String {
    match snapshot.hud.layout {
        OverlayLayout::Compact => format_compact_overlay_text(snapshot),
        OverlayLayout::MangoHud => format_mangohud_overlay_text(snapshot),
        OverlayLayout::Debug => format_debug_overlay_text(snapshot),
    }
}

fn format_compact_overlay_text(snapshot: &OverlaySnapshot) -> String {
    let game = snapshot
        .game_name
        .as_deref()
        .map(shorten_game_name)
        .unwrap_or_else(|| "Game".to_string());
    let mode = overlay_mode(snapshot);
    let mut lines = vec![game];

    if let Some(system) = compact_system_line(snapshot) {
        lines.push(system);
    }
    lines.push(overlay_frame_line(snapshot, snapshot.hud.show_frame_stats));
    if let Some(summary) = compact_summary_line(snapshot, mode) {
        lines.push(summary);
    }

    if let Some(alert) = snapshot.performance_alert.as_deref() {
        lines.push(format!("GUARD {}", shorten(alert, 42)));
    }

    join_overlay_lines(lines)
}

fn format_mangohud_overlay_text(snapshot: &OverlaySnapshot) -> String {
    let mut lines = vec![
        snapshot
            .game_name
            .as_deref()
            .map(shorten_game_name)
            .unwrap_or_else(|| "Game".to_string()),
    ];

    if snapshot.hud.show_gpu {
        lines.push(gpu_line(snapshot));
    }
    if snapshot.hud.show_cpu {
        lines.push(cpu_line(snapshot));
    }
    if snapshot.hud.show_ram {
        lines.push(ram_line(snapshot));
    }
    lines.push(overlay_frame_line(snapshot, snapshot.hud.show_frame_stats));

    if let Some(summary) = compact_summary_line(snapshot, overlay_mode(snapshot)) {
        lines.push(summary);
    }
    if snapshot.hud.show_target {
        lines.push(target_line(snapshot));
    }
    if let Some(alert) = snapshot.performance_alert.as_deref() {
        lines.push(format!("GUARD {}", shorten(alert, 42)));
    }

    join_overlay_lines(lines)
}

fn format_debug_overlay_text(snapshot: &OverlaySnapshot) -> String {
    let game = snapshot
        .game_name
        .as_deref()
        .map(shorten_game_name)
        .unwrap_or_else(|| "Game".to_string());
    let mut lines = vec![
        format!("CPM {} {game}", overlay_mode(snapshot)),
        overlay_frame_line(snapshot, true),
    ];

    if let Some(system) = compact_system_line(snapshot) {
        lines.push(system);
    }
    if let Some(summary) = compact_summary_line(snapshot, overlay_mode(snapshot)) {
        lines.push(summary);
    }
    lines.push(target_line(snapshot));
    if let Some(alert) = snapshot.performance_alert.as_deref() {
        lines.push(format!("GUARD {}", shorten(alert, 42)));
    }

    join_overlay_lines(lines)
}

fn overlay_mode(snapshot: &OverlaySnapshot) -> &'static str {
    if snapshot.fps.is_some() {
        "LIVE"
    } else if snapshot.frame_samples > 0 {
        "HOLD"
    } else {
        "SYNC"
    }
}

fn compact_system_line(snapshot: &OverlaySnapshot) -> Option<String> {
    let mut parts = Vec::new();
    if snapshot.hud.show_gpu {
        parts.push(gpu_line(snapshot));
    }
    if snapshot.hud.show_cpu {
        parts.push(cpu_line(snapshot));
    }
    if snapshot.hud.show_ram {
        parts.push(ram_line(snapshot));
    }
    if snapshot.hud.show_waste {
        parts.push(waste_line(snapshot));
    }
    (!parts.is_empty()).then(|| parts.join("  "))
}

fn compact_summary_line(snapshot: &OverlaySnapshot, mode: &str) -> Option<String> {
    let mut parts = Vec::new();
    if snapshot.hud.show_session {
        parts.push(
            snapshot
                .session_elapsed
                .map(format_duration)
                .unwrap_or_else(|| "--:--:--".to_string()),
        );
    }
    let run_mode = if snapshot.overdrive { "ODRV" } else { "NORM" };
    if snapshot.hud.show_profile {
        parts.push(format!(
            "{run_mode} {}",
            shorten(&snapshot.profile_name, 10)
        ));
    }
    parts.push(mode.to_string());
    if snapshot.hud.show_target {
        parts.push(target_line(snapshot));
    }
    (!parts.is_empty()).then(|| parts.join("  "))
}

fn gpu_line(snapshot: &OverlaySnapshot) -> String {
    let gpu = snapshot
        .gpu_usage_pct
        .map(|value| format!("{value:>3}%"))
        .unwrap_or_else(|| " --%".to_string());
    let gpu_temp = snapshot
        .gpu_temp_c
        .map(|value| format!("{value:.0}C"))
        .unwrap_or_else(|| "--C".to_string());
    format!("GPU {gpu} {gpu_temp}")
}

fn cpu_line(snapshot: &OverlaySnapshot) -> String {
    format!(
        "CPU {:>3}%",
        snapshot.cpu_usage.round().clamp(0.0, 100.0) as u16,
    )
}

fn ram_line(snapshot: &OverlaySnapshot) -> String {
    format!("RAM {:>3}%", snapshot.ram_usage_pct)
}

fn waste_line(snapshot: &OverlaySnapshot) -> String {
    let waste = snapshot.waste_mb.round().clamp(0.0, 99_999.0) as u32;
    format!("WASTE {waste}MB")
}

fn target_line(snapshot: &OverlaySnapshot) -> String {
    let target = snapshot
        .process_name
        .as_deref()
        .map(shorten_process_name)
        .unwrap_or_else(|| shorten_overlay_status(&snapshot.frame_status, 24));
    format!("TARGET {target}")
}

fn overlay_frame_line(snapshot: &OverlaySnapshot, show_details: bool) -> String {
    let Some(fps) = snapshot.fps else {
        let target = snapshot
            .process_name
            .as_deref()
            .map(shorten_process_name)
            .unwrap_or_else(|| shorten_overlay_status(&snapshot.frame_status, 24));
        return format!("FPS SYNC  {target}");
    };

    let avg = snapshot.average_fps.or(Some(fps));
    let low = snapshot.low_1_fps.or(Some(fps));
    let frame = snapshot.frame_time_ms;

    if show_details {
        format!(
            "FPS {}  AVG {}  1%L {}  FT {} ms",
            format_fps(Some(fps)),
            format_fps(avg),
            format_fps(low),
            format_ms(frame),
        )
    } else {
        format!("FPS {}  FT {} ms", format_fps(Some(fps)), format_ms(frame))
    }
}

fn join_overlay_lines(lines: Vec<String>) -> String {
    let mut text = lines.join("\n");
    while text.len() >= 255 {
        text.pop();
    }
    text
}

fn format_fps(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:>3}", value.round().clamp(0.0, 999.0) as u16))
        .unwrap_or_else(|| "---".to_string())
}

fn format_ms(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:>4.1}", value.clamp(0.0, 999.9)))
        .unwrap_or_else(|| "--.-".to_string())
}

fn shorten_overlay_status(value: &str, max_chars: usize) -> String {
    let status = value
        .strip_prefix("RTSS ")
        .unwrap_or(value)
        .replace("waiting fresh frames for ", "waiting frames ")
        .replace("waiting for hooked game frames", "waiting hooked frames")
        .replace("waiting for ", "waiting ");
    shorten(&status, max_chars)
}

fn shorten_game_name(value: &str) -> String {
    shorten(value, 28)
}

fn shorten_process_name(value: &str) -> String {
    shorten(value, 24)
}

fn shorten(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut shortened: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    shortened.push('~');
    shortened
}

#[cfg(windows)]
mod rtss {
    use std::ptr;

    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND, GetLastError, HANDLE};
    use windows_sys::Win32::System::Memory::{
        FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, OpenFileMappingW,
        UnmapViewOfFile,
    };

    const RTSS_MAP_NAME: &str = "RTSSSharedMemoryV2";
    const RTSS_SIGNATURE: u32 = 0x5254_5353;
    const RTSS_VERSION_2_0: u32 = 2 << 16;
    const RTSS_VERSION_2_7: u32 = (2 << 16) | 7;
    const OWNER: &str = "ChaosGameMode";
    const OSD_TEXT_SIZE: usize = 256;
    const OSD_OWNER_SIZE: usize = 256;
    const OSD_EX_SIZE: usize = 4096;
    const OSD_OWNER_OFFSET: usize = OSD_TEXT_SIZE;
    const OSD_EX_OFFSET: usize = OSD_TEXT_SIZE + OSD_OWNER_SIZE;

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

    pub(super) struct RtssOverlay {
        slot: Option<u32>,
    }

    struct RtssMapping {
        handle: HANDLE,
        view: MEMORY_MAPPED_VIEW_ADDRESS,
    }

    impl RtssOverlay {
        pub(super) const fn new() -> Self {
            Self { slot: None }
        }

        pub(super) fn update(&mut self, text: &str) -> Result<(), String> {
            let mut mapping = RtssMapping::open()?;
            let slot = mapping.write_osd_slot(self.slot, OWNER, text)?;
            self.slot = Some(slot);
            Ok(())
        }

        pub(super) fn clear(&mut self) -> Result<(), String> {
            let mut mapping = RtssMapping::open()?;
            mapping.clear_owned_slots(OWNER)?;
            self.slot = None;
            Ok(())
        }
    }

    impl RtssMapping {
        fn open() -> Result<Self, String> {
            let name = wide_null(RTSS_MAP_NAME);
            let handle = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, name.as_ptr()) };
            if handle.is_null() {
                let error = unsafe { GetLastError() };
                if error == ERROR_FILE_NOT_FOUND {
                    return Err("RTSS not running; start RivaTuner Statistics Server".to_string());
                }
                return Err(format!("RTSS OpenFileMapping failed: {error}"));
            }

            let view = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, 0) };
            if view.Value.is_null() {
                let error = unsafe { GetLastError() };
                unsafe {
                    CloseHandle(handle);
                }
                return Err(format!("RTSS MapViewOfFile failed: {error}"));
            }

            let mapping = Self { handle, view };
            mapping.validate()?;
            Ok(mapping)
        }

        fn validate(&self) -> Result<(), String> {
            let header = self.header();
            if header.signature != RTSS_SIGNATURE || header.version < RTSS_VERSION_2_0 {
                return Err("RTSS shared memory is not initialized".to_string());
            }
            if header.osd_entry_size < (OSD_TEXT_SIZE + OSD_OWNER_SIZE) as u32 {
                return Err("RTSS OSD entry size is unsupported".to_string());
            }
            if header.osd_arr_size < 2 || header.osd_arr_size > 64 {
                return Err("RTSS OSD slot array is unsupported".to_string());
            }
            Ok(())
        }

        fn write_osd_slot(
            &mut self,
            preferred_slot: Option<u32>,
            owner: &str,
            text: &str,
        ) -> Result<u32, String> {
            let owner_bytes = owner.as_bytes();
            let text_bytes = text.as_bytes();
            let slots = self.search_slots(preferred_slot);

            for slot in slots {
                let entry = self.entry_ptr(slot);
                if self.slot_is_owned_or_empty(entry, owner_bytes) {
                    self.write_owner(entry, owner_bytes);
                    self.write_text(entry, text_bytes);
                    self.header_mut().osd_frame = self.header().osd_frame.wrapping_add(1);
                    return Ok(slot);
                }
            }

            Err("RTSS OSD has no free slot".to_string())
        }

        fn clear_owned_slots(&mut self, owner: &str) -> Result<(), String> {
            let owner_bytes = owner.as_bytes();
            let mut cleared = false;

            for slot in 1..self.header().osd_arr_size {
                let entry = self.entry_ptr(slot);
                if cstr_eq(
                    unsafe { entry.add(OSD_OWNER_OFFSET) },
                    OSD_OWNER_SIZE,
                    owner_bytes,
                ) {
                    unsafe {
                        ptr::write_bytes(entry, 0, self.header().osd_entry_size as usize);
                    }
                    cleared = true;
                }
            }

            if cleared {
                self.header_mut().osd_frame = self.header().osd_frame.wrapping_add(1);
            }
            Ok(())
        }

        fn search_slots(&self, preferred_slot: Option<u32>) -> Vec<u32> {
            let max = self.header().osd_arr_size;
            let mut slots = Vec::with_capacity(max.saturating_sub(1) as usize);
            if let Some(slot) = preferred_slot.filter(|slot| *slot > 0 && *slot < max) {
                slots.push(slot);
            }
            for slot in 1..max {
                if Some(slot) != preferred_slot {
                    slots.push(slot);
                }
            }
            slots
        }

        fn slot_is_owned_or_empty(&self, entry: *mut u8, owner: &[u8]) -> bool {
            let owner_ptr = unsafe { entry.add(OSD_OWNER_OFFSET) };
            cstr_is_empty(owner_ptr, OSD_OWNER_SIZE) || cstr_eq(owner_ptr, OSD_OWNER_SIZE, owner)
        }

        fn write_owner(&self, entry: *mut u8, owner: &[u8]) {
            write_cstr(
                unsafe { entry.add(OSD_OWNER_OFFSET) },
                OSD_OWNER_SIZE,
                owner,
            );
        }

        fn write_text(&self, entry: *mut u8, text: &[u8]) {
            write_cstr(entry, OSD_TEXT_SIZE, text);
            if self.header().version >= RTSS_VERSION_2_7
                && self.header().osd_entry_size as usize >= OSD_EX_OFFSET + OSD_EX_SIZE
            {
                write_cstr(unsafe { entry.add(OSD_EX_OFFSET) }, OSD_EX_SIZE, &[]);
            }
        }

        fn entry_ptr(&self, slot: u32) -> *mut u8 {
            unsafe {
                (self.view.Value as *mut u8).add(
                    self.header().osd_arr_offset as usize
                        + (slot as usize * self.header().osd_entry_size as usize),
                )
            }
        }

        fn header(&self) -> &RtssHeader {
            unsafe { &*(self.view.Value as *const RtssHeader) }
        }

        fn header_mut(&mut self) -> &mut RtssHeader {
            unsafe { &mut *(self.view.Value as *mut RtssHeader) }
        }
    }

    impl Drop for RtssMapping {
        fn drop(&mut self) {
            unsafe {
                UnmapViewOfFile(self.view);
                CloseHandle(self.handle);
            }
        }
    }

    fn write_cstr(destination: *mut u8, capacity: usize, value: &[u8]) {
        unsafe {
            ptr::write_bytes(destination, 0, capacity);
            ptr::copy_nonoverlapping(value.as_ptr(), destination, value.len().min(capacity - 1));
        }
    }

    fn cstr_is_empty(source: *const u8, capacity: usize) -> bool {
        unsafe { (0..capacity).all(|index| *source.add(index) == 0) }
    }

    fn cstr_eq(source: *const u8, capacity: usize, expected: &[u8]) -> bool {
        if expected.len() >= capacity {
            return false;
        }
        unsafe {
            for (index, expected_byte) in expected.iter().enumerate() {
                if *source.add(index) != *expected_byte {
                    return false;
                }
            }
            *source.add(expected.len()) == 0
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(windows)]
use rtss::RtssOverlay;

#[cfg(not(windows))]
struct RtssOverlay;

#[cfg(not(windows))]
impl RtssOverlay {
    const fn new() -> Self {
        Self
    }

    fn update(&mut self, _text: &str) -> Result<(), String> {
        Err("RTSS overlay is only available on Windows".to_string())
    }

    fn clear(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_text_should_include_core_metrics() {
        let text = format_overlay_text(&OverlaySnapshot {
            armed: true,
            enabled: true,
            backend: OverlayBackend::Rtss,
            game_name: Some("Cyberpunk 2077".to_string()),
            process_name: Some("Cyberpunk2077.exe".to_string()),
            fps: Some(61.2),
            average_fps: Some(58.7),
            low_1_fps: Some(44.0),
            frame_time_ms: Some(16.4),
            frame_samples: 42,
            frame_status: "RTSS tracking Cyberpunk2077.exe".to_string(),
            performance_alert: None,
            profile_name: "balanced".to_string(),
            overdrive: true,
            session_elapsed: Some(Duration::from_secs(125)),
            cpu_usage: 22.0,
            ram_usage_pct: 78,
            gpu_usage_pct: Some(91),
            gpu_temp_c: Some(72.0),
            waste_mb: 8840.0,
            hud: OverlayHudConfig::default(),
        });

        assert!(text.contains("Cyberpunk 2077"));
        assert!(text.contains("FPS  61"));
        assert!(text.contains("AVG  59"));
        assert!(text.contains("1%L  44"));
        assert!(text.contains("FT 16.4 ms"));
        assert!(text.contains("00:02:05  ODRV balanced  LIVE"));
        assert!(text.contains("GPU  91% 72C"));
        assert!(text.contains("CPU  22%"));
    }

    #[test]
    fn overlay_text_should_apply_compact_layout_without_optional_details() {
        let mut snapshot = OverlaySnapshot::armed(OverlayBackend::Rtss);
        snapshot.enabled = true;
        snapshot.game_name = Some("Cyberpunk 2077".to_string());
        snapshot.fps = Some(33.0);
        snapshot.frame_time_ms = Some(30.3);
        snapshot.hud.layout = OverlayLayout::Compact;
        snapshot.hud.show_frame_stats = false;
        snapshot.hud.show_waste = false;

        let text = format_overlay_text(&snapshot);

        assert!(text.contains("FPS  33  FT 30.3 ms"));
        assert!(!text.contains("AVG"));
        assert!(!text.contains("WASTE"));
    }

    #[test]
    fn overlay_text_should_include_debug_target_when_enabled() {
        let mut snapshot = OverlaySnapshot::armed(OverlayBackend::Rtss);
        snapshot.enabled = true;
        snapshot.hud.layout = OverlayLayout::Debug;
        snapshot.hud.show_target = true;
        snapshot.process_name = Some("Cyberpunk2077.exe".to_string());

        let text = format_overlay_text(&snapshot);

        assert!(text.contains("TARGET Cyberpunk2077.exe"));
    }

    #[test]
    fn overlay_text_should_show_sync_state_without_empty_metric_labels() {
        let mut snapshot = OverlaySnapshot::armed(OverlayBackend::Rtss);
        snapshot.enabled = true;
        snapshot.game_name = Some("Cyberpunk 2077".to_string());
        snapshot.process_name = Some("Cyberpunk2077.exe".to_string());
        snapshot.frame_status = "RTSS waiting fresh frames for Cyberpunk2077.exe".to_string();

        let text = format_overlay_text(&snapshot);

        assert!(text.contains("Cyberpunk 2077"));
        assert!(text.contains("FPS SYNC"));
        assert!(text.contains("Cyberpunk2077.exe"));
        assert!(!text.contains("AVG ---"));
    }

    #[test]
    fn overlay_text_should_include_performance_alert() {
        let mut snapshot = OverlaySnapshot::armed(OverlayBackend::Rtss);
        snapshot.enabled = true;
        snapshot.performance_alert = Some("Overdrive FPS collapse; press 2 to restore".to_string());

        let text = format_overlay_text(&snapshot);

        assert!(text.contains("GUARD Overdrive FPS collapse"));
    }

    #[test]
    fn overlay_text_should_stay_inside_legacy_rtss_text_slot() {
        let text = format_overlay_text(&OverlaySnapshot {
            armed: true,
            enabled: true,
            backend: OverlayBackend::Rtss,
            game_name: Some("Cyberpunk 2077 Ultimate Phantom Liberty Edition".to_string()),
            process_name: Some("Cyberpunk2077VeryLongProcessName.exe".to_string()),
            fps: Some(61.2),
            average_fps: Some(58.7),
            low_1_fps: Some(44.0),
            frame_time_ms: Some(16.4),
            frame_samples: 600,
            frame_status: "RTSS tracking Cyberpunk2077VeryLongProcessName.exe".to_string(),
            performance_alert: Some(
                "Overdrive FPS collapse; press 2 to restore immediately".to_string(),
            ),
            profile_name: "aggressive".to_string(),
            overdrive: true,
            session_elapsed: Some(Duration::from_secs(3723)),
            cpu_usage: 100.0,
            ram_usage_pct: 99,
            gpu_usage_pct: Some(100),
            gpu_temp_c: Some(88.0),
            waste_mb: 12_345.0,
            hud: OverlayHudConfig::default(),
        });

        assert!(text.len() < 255, "{text}");
    }

    #[test]
    fn shorten_should_keep_rtss_text_bounded() {
        assert_eq!(shorten("abcdefghijklmnopqrstuvwxyz", 8), "abcdefg~");
    }
}

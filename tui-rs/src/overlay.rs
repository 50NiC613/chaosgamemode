use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub(crate) struct OverlayConfig {
    pub(crate) enabled: bool,
    pub(crate) backend: OverlayBackend,
    pub(crate) update_rate: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlayBackend {
    Rtss,
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
    pub(crate) cpu_usage: f32,
    pub(crate) ram_usage_pct: u16,
    pub(crate) gpu_usage_pct: Option<u16>,
    pub(crate) gpu_temp_c: Option<f32>,
    pub(crate) waste_mb: f64,
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
            cpu_usage: 0.0,
            ram_usage_pct: 0,
            gpu_usage_pct: None,
            gpu_temp_c: None,
            waste_mb: 0.0,
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
    let mut last_status = String::new();
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

        if last_update.elapsed() < config.update_rate && snapshot.enabled {
            continue;
        }
        last_update = Instant::now();

        let status = if snapshot.enabled {
            match snapshot.backend {
                OverlayBackend::Rtss => match rtss.update(&format_overlay_text(&snapshot)) {
                    Ok(()) => {
                        overlay_visible = true;
                        OverlayStatus {
                            active: true,
                            backend: OverlayBackend::Rtss,
                            message: "RTSS overlay active".to_string(),
                        }
                    }
                    Err(message) => {
                        overlay_visible = false;
                        OverlayStatus {
                            active: false,
                            backend: OverlayBackend::Rtss,
                            message,
                        }
                    }
                },
            }
        } else {
            if overlay_visible {
                let _ = rtss.clear();
                overlay_visible = false;
            }
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
    let game = snapshot
        .game_name
        .as_deref()
        .map(shorten_game_name)
        .unwrap_or_else(|| "Steam game".to_string());
    let process = snapshot
        .process_name
        .as_deref()
        .map(shorten_process_name)
        .unwrap_or_else(|| "resolving".to_string());

    let fps = format_metric(snapshot.fps, 0, "--");
    let avg = format_metric(snapshot.average_fps, 0, "--");
    let low = format_metric(snapshot.low_1_fps, 0, "--");
    let frame = format_metric(snapshot.frame_time_ms, 1, "--");
    let gpu = snapshot
        .gpu_usage_pct
        .map(|value| format!("{value:>3}%"))
        .unwrap_or_else(|| " --%".to_string());
    let gpu_temp = snapshot
        .gpu_temp_c
        .map(|value| format!("{value:.0}C"))
        .unwrap_or_else(|| "--C".to_string());

    format!(
        "CPM  FPS {fps}  AVG {avg}  1% {low}\nCPU {:>3}%  GPU {gpu} {gpu_temp}  RAM {:>3}%\n{frame} ms  WASTE {:.0} MB  {game}\n{process}",
        snapshot.cpu_usage.round().clamp(0.0, 100.0) as u16,
        snapshot.ram_usage_pct,
        snapshot.waste_mb,
    )
}

fn format_metric(value: Option<f64>, decimals: usize, fallback: &str) -> String {
    value
        .map(|value| match decimals {
            0 => format!("{:>3}", value.round().clamp(0.0, 999.0) as u16),
            _ => format!("{value:>4.1}"),
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn shorten_game_name(value: &str) -> String {
    shorten(value, 30)
}

fn shorten_process_name(value: &str) -> String {
    shorten(value, 28)
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
                write_cstr(unsafe { entry.add(OSD_EX_OFFSET) }, OSD_EX_SIZE, text);
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
            cpu_usage: 22.0,
            ram_usage_pct: 78,
            gpu_usage_pct: Some(91),
            gpu_temp_c: Some(72.0),
            waste_mb: 8840.0,
        });

        assert!(text.contains("FPS  61"));
        assert!(text.contains("Cyberpunk 2077"));
    }

    #[test]
    fn shorten_should_keep_rtss_text_bounded() {
        assert_eq!(shorten("abcdefghijklmnopqrstuvwxyz", 8), "abcdefg~");
    }
}

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Clone, Debug)]
pub(crate) enum HotkeyEvent {
    ToggleOverlay,
    Status(String),
}

pub(crate) fn spawn_hotkey_thread() -> Receiver<HotkeyEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || run_hotkey_loop(tx));
    rx
}

#[cfg(windows)]
fn run_hotkey_loop(tx: Sender<HotkeyEvent>) {
    windows_hotkeys::run(tx);
}

#[cfg(not(windows))]
fn run_hotkey_loop(tx: Sender<HotkeyEvent>) {
    let _ = tx.send(HotkeyEvent::Status(
        "hotkey: Shift+F12 only available on Windows".to_string(),
    ));
}

#[cfg(windows)]
mod windows_hotkeys {
    use std::ptr;

    use super::HotkeyEvent;
    use std::sync::mpsc::Sender;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MOD_SHIFT, RegisterHotKey, UnregisterHotKey, VK_F12,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

    const HOTKEY_ID_OVERLAY: i32 = 0x4347;

    pub(super) fn run(tx: Sender<HotkeyEvent>) {
        if !register_shift_f12() {
            let _ = tx.send(HotkeyEvent::Status(
                "hotkey: Shift+F12 unavailable".to_string(),
            ));
            return;
        }

        let _ = tx.send(HotkeyEvent::Status(
            "hotkey: Shift+F12 registered".to_string(),
        ));
        message_loop(&tx);

        unsafe {
            UnregisterHotKey(ptr::null_mut(), HOTKEY_ID_OVERLAY);
        }
    }

    fn register_shift_f12() -> bool {
        unsafe {
            RegisterHotKey(
                ptr::null_mut(),
                HOTKEY_ID_OVERLAY,
                MOD_SHIFT,
                u32::from(VK_F12),
            ) != 0
        }
    }

    fn message_loop(tx: &Sender<HotkeyEvent>) {
        let mut message = unsafe { std::mem::zeroed::<MSG>() };
        loop {
            let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
            if result <= 0 {
                break;
            }
            if message.message == WM_HOTKEY && message.wParam == HOTKEY_ID_OVERLAY as usize {
                let _ = tx.send(HotkeyEvent::ToggleOverlay);
            }
        }
    }
}

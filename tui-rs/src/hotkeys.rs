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
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::HotkeyEvent;
    use std::sync::mpsc::Sender;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey, UnregisterHotKey, VK_F12,
        VK_LSHIFT, VK_RSHIFT, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW, UnhookWindowsHookEx,
        WH_KEYBOARD_LL, WM_HOTKEY, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    const HOTKEY_ID_OVERLAY: i32 = 0x4347;
    static HOTKEY_TX: OnceLock<Sender<HotkeyEvent>> = OnceLock::new();
    static F12_DOWN: AtomicBool = AtomicBool::new(false);

    pub(super) fn run(tx: Sender<HotkeyEvent>) {
        let _ = HOTKEY_TX.set(tx.clone());

        if let Some(hook) = install_keyboard_hook() {
            let _ = tx.send(HotkeyEvent::Status(
                "hotkey: Shift+F12 global hook registered".to_string(),
            ));
            message_loop(&tx);
            unsafe {
                UnhookWindowsHookEx(hook);
            }
            return;
        }

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
                MOD_SHIFT | MOD_NOREPEAT,
                u32::from(VK_F12),
            ) != 0
        }
    }

    fn install_keyboard_hook() -> Option<windows_sys::Win32::UI::WindowsAndMessaging::HHOOK> {
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), ptr::null_mut(), 0) };
        (!hook.is_null()).then_some(hook)
    }

    unsafe extern "system" fn keyboard_hook(code: i32, wparam: usize, lparam: isize) -> isize {
        if code >= 0 {
            let keyboard = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
            let event = wparam as u32;
            if keyboard.vkCode == u32::from(VK_F12) && is_key_down_event(event) {
                if shift_is_down()
                    && !F12_DOWN.swap(true, Ordering::SeqCst)
                    && let Some(tx) = HOTKEY_TX.get()
                {
                    let _ = tx.send(HotkeyEvent::ToggleOverlay);
                }
            } else if keyboard.vkCode == u32::from(VK_F12) && is_key_up_event(event) {
                F12_DOWN.store(false, Ordering::SeqCst);
            }
        }

        unsafe { CallNextHookEx(ptr::null_mut(), code, wparam, lparam) }
    }

    fn is_key_down_event(event: u32) -> bool {
        event == WM_KEYDOWN || event == WM_SYSKEYDOWN
    }

    fn is_key_up_event(event: u32) -> bool {
        event == WM_KEYUP || event == WM_SYSKEYUP
    }

    fn shift_is_down() -> bool {
        key_is_down(VK_SHIFT) || key_is_down(VK_LSHIFT) || key_is_down(VK_RSHIFT)
    }

    fn key_is_down(key: u16) -> bool {
        unsafe { GetAsyncKeyState(i32::from(key)) < 0 }
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

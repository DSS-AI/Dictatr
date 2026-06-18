//! Low-level Windows keyboard hook for system / multimedia keys.
//!
//! `global-hotkey` (and `RegisterHotKey`) cannot observe keys like
//! `VK_LAUNCH_MAIL` / `VK_VOLUME_UP` / `VK_BROWSER_HOME` — Windows routes
//! them directly to the shell or the default handler app. Installing a
//! `WH_KEYBOARD_LL` hook lets us see them before the OS acts on them, and
//! returning `1` from the hook proc suppresses the default action
//! (e.g. stops Outlook from launching on `LaunchMail`).
//!
//! The hook must live on a thread with a running message loop, so we
//! spin up a dedicated thread that installs the hook and pumps messages.
//! Global state (the vk-code → profile-id map and the event sender) is
//! held in a static because the raw C callback cannot capture a closure.

#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
pub use stub::*;

#[cfg(target_os = "windows")]
mod windows_impl {
    use crate::error::{AppError, Result};
    use crate::hotkey::HotkeyEvent;
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::{Duration, Instant};
    use tokio::sync::mpsc::UnboundedSender;
    use uuid::Uuid;
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    /// Max gap between consecutive `WM_KEYDOWN`s still treated as auto-repeat of
    /// the same hold. Anything longer counts as a fresh press. This makes the
    /// dedup self-healing: if a `WM_KEYUP` is ever lost (Windows can drop it when
    /// the foreground changes mid-press, e.g. while the popup opens), the stale
    /// "still held" state can't swallow the next real press — that press arrives
    /// far later than any auto-repeat. Auto-repeat fires every ~30 ms after the
    /// initial ~250–500 ms delay, so this window comfortably covers it.
    const REPEAT_GAP: Duration = Duration::from_millis(600);

    struct LlState {
        /// vk-code (low byte of DWORD) → profile id
        mapping: HashMap<u32, Uuid>,
        /// Last `WM_KEYDOWN` instant per profile, used to tell a fresh press from
        /// a held-key auto-repeat (see `REPEAT_GAP`). Absent = not held.
        pressed: HashMap<Uuid, Instant>,
        tx: Option<UnboundedSender<HotkeyEvent>>,
    }

    fn state() -> &'static Mutex<LlState> {
        static S: OnceLock<Mutex<LlState>> = OnceLock::new();
        S.get_or_init(|| {
            Mutex::new(LlState {
                mapping: HashMap::new(),
                pressed: HashMap::new(),
                tx: None,
            })
        })
    }

    /// Handle to the running hook; drop it to uninstall and exit the thread.
    pub struct LlHotkeyHook {
        hhook: AtomicIsize,
    }

    impl Drop for LlHotkeyHook {
        fn drop(&mut self) {
            let h = self.hhook.swap(0, Ordering::SeqCst);
            if h != 0 {
                unsafe {
                    UnhookWindowsHookEx(h as *mut core::ffi::c_void);
                }
            }
        }
    }

    /// Install the low-level hook on a dedicated thread.
    ///
    /// `mapping` associates vk-codes with profile ids; only the listed
    /// keys are reported and suppressed, every other key passes through.
    pub fn start(
        mapping: HashMap<u32, Uuid>,
        tx: UnboundedSender<HotkeyEvent>,
    ) -> Result<LlHotkeyHook> {
        {
            let mut s = state().lock();
            s.mapping = mapping;
            s.pressed.clear();
            s.tx = Some(tx);
        }

        // We need the HHOOK returned from SetWindowsHookExW which is only
        // available after the hook thread has run; use a oneshot to relay it.
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<std::result::Result<isize, String>>(1);

        thread::Builder::new()
            .name("dss-ll-hotkey".into())
            .spawn(move || unsafe {
                let hhook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), std::ptr::null_mut(), 0);
                if hhook.is_null() {
                    let _ = ready_tx.send(Err("SetWindowsHookExW failed".into()));
                    return;
                }
                let _ = ready_tx.send(Ok(hhook as isize));

                // Standard Win32 message loop — required for low-level hooks.
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            })
            .map_err(|e| AppError::Config(format!("ll-hook thread spawn failed: {e}")))?;

        let hhook = ready_rx
            .recv()
            .map_err(|e| AppError::Config(format!("ll-hook ready channel: {e}")))?
            .map_err(AppError::Config)?;

        Ok(LlHotkeyHook {
            hhook: AtomicIsize::new(hhook),
        })
    }

    /// Replace the active vk → profile mapping without restarting the hook
    /// thread. Call from the hotkey-owner thread on profile reload.
    pub fn update_mapping(new_mapping: HashMap<u32, Uuid>) {
        let mut s = state().lock();
        s.mapping = new_mapping;
        s.pressed.clear();
    }

    unsafe extern "system" fn hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code != HC_ACTION as i32 {
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }
        let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;
        let is_down = w_param as u32 == WM_KEYDOWN || w_param as u32 == WM_SYSKEYDOWN;
        let is_up = w_param as u32 == WM_KEYUP || w_param as u32 == WM_SYSKEYUP;

        let mut st = state().lock();
        let profile_id = match st.mapping.get(&vk).copied() {
            Some(id) => id,
            None => return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param),
        };

        if is_down {
            let now = Instant::now();
            // Fresh press if we've never seen this key held, or the previous
            // keydown was long enough ago that it can't be auto-repeat (so a
            // lost keyup can't wedge us into "perpetually held").
            let is_fresh = st
                .pressed
                .get(&profile_id)
                .map(|last| now.duration_since(*last) > REPEAT_GAP)
                .unwrap_or(true);
            // Always refresh the timestamp so a continuous hold keeps extending
            // the auto-repeat window.
            st.pressed.insert(profile_id, now);
            if is_fresh {
                if let Some(tx) = st.tx.as_ref() {
                    let _ = tx.send(HotkeyEvent::Pressed(profile_id));
                }
            }
            // swallow — don't let Outlook / calculator / volume pop-up fire
            return 1;
        }
        if is_up {
            if st.pressed.remove(&profile_id).is_some() {
                if let Some(tx) = st.tx.as_ref() {
                    let _ = tx.send(HotkeyEvent::Released(profile_id));
                }
            }
            return 1;
        }
        CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
    }

    /// Parse a multimedia-key name into a Windows virtual-key code.
    /// Returns None for non-multimedia strings (e.g. normal chord combos).
    pub fn parse_vk(name: &str) -> Option<u32> {
        match name.trim() {
            "LaunchMail" => Some(VK_LAUNCH_MAIL as u32),
            "LaunchApp1" => Some(VK_LAUNCH_APP1 as u32),
            "LaunchApp2" => Some(VK_LAUNCH_APP2 as u32),
            "LaunchMediaSelect" => Some(VK_LAUNCH_MEDIA_SELECT as u32),
            "MediaPlayPause" => Some(VK_MEDIA_PLAY_PAUSE as u32),
            "MediaStop" => Some(VK_MEDIA_STOP as u32),
            "MediaNextTrack" => Some(VK_MEDIA_NEXT_TRACK as u32),
            "MediaPrevTrack" => Some(VK_MEDIA_PREV_TRACK as u32),
            "VolumeMute" => Some(VK_VOLUME_MUTE as u32),
            "VolumeDown" => Some(VK_VOLUME_DOWN as u32),
            "VolumeUp" => Some(VK_VOLUME_UP as u32),
            "BrowserBack" => Some(VK_BROWSER_BACK as u32),
            "BrowserForward" => Some(VK_BROWSER_FORWARD as u32),
            "BrowserRefresh" => Some(VK_BROWSER_REFRESH as u32),
            "BrowserStop" => Some(VK_BROWSER_STOP as u32),
            "BrowserSearch" => Some(VK_BROWSER_SEARCH as u32),
            "BrowserFavorites" => Some(VK_BROWSER_FAVORITES as u32),
            "BrowserHome" => Some(VK_BROWSER_HOME as u32),
            "Sleep" => Some(VK_SLEEP as u32),
            _ => None,
        }
    }

    /// Parse a bare function-key name (`F1`..`F24`) into its virtual-key code.
    /// Returns None for anything else — including modifier combos like
    /// `Ctrl+F8`, which keep using RegisterHotKey. Routing bare function keys
    /// through the low-level hook gives clean key-up events (reliable
    /// push-to-talk *and* toggle) and intercepts the key before a focused
    /// console/terminal can consume it (e.g. F8 = console history search).
    pub fn parse_function_vk(name: &str) -> Option<u32> {
        match name.trim() {
            "F1" => Some(VK_F1 as u32),
            "F2" => Some(VK_F2 as u32),
            "F3" => Some(VK_F3 as u32),
            "F4" => Some(VK_F4 as u32),
            "F5" => Some(VK_F5 as u32),
            "F6" => Some(VK_F6 as u32),
            "F7" => Some(VK_F7 as u32),
            "F8" => Some(VK_F8 as u32),
            "F9" => Some(VK_F9 as u32),
            "F10" => Some(VK_F10 as u32),
            "F11" => Some(VK_F11 as u32),
            "F12" => Some(VK_F12 as u32),
            "F13" => Some(VK_F13 as u32),
            "F14" => Some(VK_F14 as u32),
            "F15" => Some(VK_F15 as u32),
            "F16" => Some(VK_F16 as u32),
            "F17" => Some(VK_F17 as u32),
            "F18" => Some(VK_F18 as u32),
            "F19" => Some(VK_F19 as u32),
            "F20" => Some(VK_F20 as u32),
            "F21" => Some(VK_F21 as u32),
            "F22" => Some(VK_F22 as u32),
            "F23" => Some(VK_F23 as u32),
            "F24" => Some(VK_F24 as u32),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn function_keys_map_to_vk() {
            assert_eq!(parse_function_vk("F8"), Some(VK_F8 as u32));
            assert_eq!(parse_function_vk("  F9  "), Some(VK_F9 as u32));
            assert_eq!(parse_function_vk("F24"), Some(VK_F24 as u32));
        }

        #[test]
        fn non_function_keys_are_none() {
            // Modifier combos and plain letters keep using RegisterHotKey.
            assert_eq!(parse_function_vk("Ctrl+F8"), None);
            assert_eq!(parse_function_vk("A"), None);
            assert_eq!(parse_function_vk("Space"), None);
            assert_eq!(parse_function_vk("F25"), None);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod stub {
    use crate::error::Result;
    use crate::hotkey::HotkeyEvent;
    use std::collections::HashMap;
    use tokio::sync::mpsc::UnboundedSender;
    use uuid::Uuid;

    pub struct LlHotkeyHook;

    pub fn start(
        _mapping: HashMap<u32, Uuid>,
        _tx: UnboundedSender<HotkeyEvent>,
    ) -> Result<LlHotkeyHook> {
        Ok(LlHotkeyHook)
    }

    pub fn parse_vk(_name: &str) -> Option<u32> {
        None
    }

    pub fn parse_function_vk(_name: &str) -> Option<u32> {
        None
    }

    pub fn update_mapping(_new: HashMap<u32, Uuid>) {}
}

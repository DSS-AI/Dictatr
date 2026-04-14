use crate::error::{AppError, Result};
#[cfg(not(target_os = "macos"))]
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub struct TextInjector;

impl TextInjector {
    /// Inject the text into the focused field via clipboard paste.
    ///
    /// `enigo.text()` (SendInput with unicode characters) regularly drops
    /// the first and/or last character when the target app is briefly
    /// unfocused or buffers keystrokes, and mishandles some unicode
    /// combinations. Putting the text on the clipboard and sending the
    /// platform paste chord is the robust path every major dictation tool
    /// uses.
    pub fn inject(text: &str) -> Result<()> {
        // Preserve whatever the user had on the clipboard.
        let prev = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok());

        {
            let mut cb = arboard::Clipboard::new()
                .map_err(|e| AppError::Inject(e.to_string()))?;
            cb.set_text(text.to_string())
                .map_err(|e| AppError::Inject(e.to_string()))?;
        }

        // Short delay so the clipboard set actually lands before the paste
        // chord fires — also covers the "target app not yet accepting input"
        // race that causes lost leading characters.
        std::thread::sleep(std::time::Duration::from_millis(60));

        send_paste()?;

        // Give the paste a moment to complete before restoring the
        // clipboard, otherwise the target app might see the restored text.
        // On macOS we keep the transcribed text on the clipboard as a manual
        // fallback — if the synthesised Cmd+V is swallowed (beta OS quirks,
        // racing TCC), the user can still Cmd+V by hand.
        #[cfg(not(target_os = "macos"))]
        if let Some(prev_text) = prev {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(prev_text);
            }
        }
        #[cfg(target_os = "macos")]
        let _ = prev;
        Ok(())
    }

    pub fn clipboard_fallback(text: &str) -> Result<()> {
        let mut cb = arboard::Clipboard::new().map_err(|e| AppError::Inject(e.to_string()))?;
        cb.set_text(text.to_string()).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
fn send_paste() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| AppError::Inject(e.to_string()))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| AppError::Inject(e.to_string()))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| AppError::Inject(e.to_string()))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| AppError::Inject(e.to_string()))?;
    Ok(())
}

// -----------------------------------------------------------------------------
// macOS: direct CGEventPost so we bypass enigo. enigo's macOS backend does not
// reliably deliver Cmd+V on macOS 26 beta (the clipboard lands but the paste
// chord is swallowed). CGEventCreateKeyboardEvent + CGEventPost with the
// command flag set is the officially supported path and the same mechanism
// AppleScript uses under the hood.
// -----------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::c_void;

    pub const KVK_V: u16 = 0x09;
    pub const KVK_COMMAND: u16 = 0x37;
    pub const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;
    pub const CG_SESSION_EVENT_TAP: u32 = 1;
    // kCGEventSourceStateHIDSystemState = 1 (behaves like real hardware)
    pub const CG_EVENT_SOURCE_HID_STATE: u32 = 1;

    pub type CGEventRef = *mut c_void;
    pub type CGEventSourceRef = *mut c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub fn CGEventSourceCreate(state_id: u32) -> CGEventSourceRef;
        pub fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        pub fn CGEventPost(tap: u32, event: CGEventRef);
        pub fn CGEventSetFlags(event: CGEventRef, flags: u64);
        pub fn CFRelease(cf: *const c_void);
        pub fn AXIsProcessTrusted() -> bool;
        pub fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        pub fn CFBooleanGetValue(boolean: *const c_void) -> bool;
        pub static kCFBooleanTrue: *const c_void;
        pub static kCFTypeDictionaryKeyCallBacks: c_void;
        pub static kCFTypeDictionaryValueCallBacks: c_void;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub static kAXTrustedCheckOptionPrompt: *const c_void;
    }
}

/// Prompt the user for Accessibility permission if it isn't granted yet.
/// This triggers the standard macOS dialog ("Dictatr möchte auf Bedienungs-
/// hilfen zugreifen") and — crucially — registers the running process with
/// TCC so that subsequent `AXIsProcessTrusted()` calls actually reflect the
/// user's choice. Without this call, direct-binary launches from a terminal
/// often stay invisible to TCC even after the user ticks the box manually.
#[cfg(target_os = "macos")]
pub fn prompt_accessibility_if_needed() {
    use mac::*;
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let opts = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );
        let trusted = AXIsProcessTrustedWithOptions(opts);
        eprintln!("[inject] accessibility prompt result (trusted now? {}) — if false, grant in System Settings and restart the app", trusted);
        CFRelease(opts);
    }
}

/// Trigger macOS' microphone permission dialog via AVCaptureDevice.
/// cpal opens CoreAudio streams that silently return zero-filled buffers
/// when mic access hasn't been granted — whisper then hallucinates "[Musik]"
/// for what looks like silence. AVCaptureDevice.requestAccess reliably
/// prompts the user and registers the running process in TCC's Microphone
/// list. Requires a non-null Obj-C block for the completion handler — we
/// construct a minimal global block by hand, since adding the `block2`
/// crate just for this one call is overkill.
#[cfg(target_os = "macos")]
pub fn prompt_microphone_if_needed() {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_void};

    #[link(name = "objc", kind = "dylib")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *const c_void;
        fn sel_registerName(name: *const c_char) -> *const c_void;
        fn objc_msgSend();
    }

    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {
        static AVMediaTypeAudio: *const c_void;
    }

    #[link(name = "System", kind = "dylib")]
    extern "C" {
        static _NSConcreteGlobalBlock: c_void;
    }

    // Matches clang's block literal layout (ABI stable).
    #[repr(C)]
    struct BlockDescriptor {
        reserved: usize,
        size: usize,
    }
    #[repr(C)]
    struct Block {
        isa: *const c_void,
        flags: i32,
        reserved: i32,
        invoke: unsafe extern "C" fn(*const Block, bool),
        descriptor: *const BlockDescriptor,
    }
    unsafe extern "C" fn noop(_block: *const Block, _granted: bool) {}

    static DESCRIPTOR: BlockDescriptor = BlockDescriptor {
        reserved: 0,
        size: std::mem::size_of::<Block>(),
    };

    type MsgSendStatus = unsafe extern "C" fn(
        *const c_void,
        *const c_void,
        *const c_void,
    ) -> isize;
    type MsgSendVoid = unsafe extern "C" fn(
        *const c_void,
        *const c_void,
        *const c_void,
        *const c_void,
    );

    // Bring the app to the foreground so macOS actually presents the
    // permission dialog — background agents (tray apps with no visible
    // window) silently swallow these prompts.
    type MsgSendBool = unsafe extern "C" fn(*const c_void, *const c_void, bool);
    unsafe {
        let ns_app_class = objc_getClass(CString::new("NSApplication").unwrap().as_ptr());
        let shared_sel = sel_registerName(CString::new("sharedApplication").unwrap().as_ptr());
        type MsgSendInstance = unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void;
        let msg_instance: MsgSendInstance = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let app = msg_instance(ns_app_class, shared_sel);
        if !app.is_null() {
            let activate_sel = sel_registerName(
                CString::new("activateIgnoringOtherApps:").unwrap().as_ptr(),
            );
            let msg_bool: MsgSendBool = std::mem::transmute(
                objc_msgSend as unsafe extern "C" fn(),
            );
            msg_bool(app, activate_sel, true);
        }
    }

    unsafe {
        let class_name = CString::new("AVCaptureDevice").unwrap();
        let class = objc_getClass(class_name.as_ptr());
        if class.is_null() {
            eprintln!("[inject] AVCaptureDevice class not found — skipping mic prompt");
            return;
        }
        let sel_status = sel_registerName(
            CString::new("authorizationStatusForMediaType:").unwrap().as_ptr(),
        );
        let msg_status: MsgSendStatus = std::mem::transmute(
            objc_msgSend as unsafe extern "C" fn(),
        );
        let status = msg_status(class, sel_status, AVMediaTypeAudio);
        eprintln!("[inject] AVCaptureDevice mic status = {} (0=NotDetermined 1=Restricted 2=Denied 3=Authorized)", status);

        // Only NotDetermined (0) triggers the dialog; otherwise no-op.
        if status != 0 {
            return;
        }

        let block = Block {
            isa: &_NSConcreteGlobalBlock as *const _ as *const c_void,
            // BLOCK_IS_GLOBAL = 1<<28; without BLOCK_HAS_COPY_DISPOSE we
            // don't need copy/dispose helpers for a capture-less function.
            flags: 1 << 28,
            reserved: 0,
            invoke: noop,
            descriptor: &DESCRIPTOR,
        };
        let block_leaked: &'static Block = Box::leak(Box::new(block));

        let sel_request = sel_registerName(
            CString::new("requestAccessForMediaType:completionHandler:").unwrap().as_ptr(),
        );
        let msg_request: MsgSendVoid = std::mem::transmute(
            objc_msgSend as unsafe extern "C" fn(),
        );
        msg_request(
            class,
            sel_request,
            AVMediaTypeAudio,
            block_leaked as *const _ as *const c_void,
        );
        eprintln!("[inject] requested microphone access — dialog should appear");
    }
}

#[cfg(target_os = "macos")]
fn send_paste() -> Result<()> {
    unsafe {
        let trusted = mac::AXIsProcessTrusted();
        eprintln!("[inject] AXIsProcessTrusted = {}", trusted);
        if !trusted {
            return Err(AppError::Inject(
                "Accessibility-Berechtigung nicht aktiv für diesen Build (TCC). \
                 Führe `tccutil reset Accessibility de.dss.dictatr` aus, \
                 dann füge Dictatr.app erneut unter System Settings → \
                 Datenschutz & Sicherheit → Bedienungshilfen hinzu."
                    .into(),
            ));
        }
    }
    cg_event_paste()
}

#[cfg(target_os = "macos")]
fn cg_event_paste() -> Result<()> {
    use mac::*;
    unsafe {
        let src = CGEventSourceCreate(CG_EVENT_SOURCE_HID_STATE);
        // src may be null (the OS returns Ok events anyway when null is fine),
        // but using a HID-system-state source mimics real keyboard input more
        // closely and helps apps that filter "low-level" events.

        let mk = |vk: u16, down: bool, with_cmd: bool| -> CGEventRef {
            let e = CGEventCreateKeyboardEvent(src, vk, down);
            if !e.is_null() && with_cmd {
                CGEventSetFlags(e, CG_EVENT_FLAG_MASK_COMMAND);
            }
            e
        };

        // cmd down → v down (with cmd flag) → v up (with cmd flag) → cmd up.
        let events = [
            mk(KVK_COMMAND, true, false),
            mk(KVK_V, true, true),
            mk(KVK_V, false, true),
            mk(KVK_COMMAND, false, false),
        ];
        for e in events {
            if e.is_null() {
                return Err(AppError::Inject("CGEventCreateKeyboardEvent returned null".into()));
            }
            CGEventPost(CG_SESSION_EVENT_TAP, e);
            CFRelease(e);
        }
        if !src.is_null() {
            CFRelease(src);
        }
    }
    Ok(())
}


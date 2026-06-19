use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Logical size of the Prompt-Manager quick-pick popup. Physical size is
/// derived from this via the primary monitor's scale factor at position time.
const PICKER_W: f64 = 440.0;
const PICKER_H: f64 = 540.0;

/// Show the quick-pick popup, creating it on first use. Loads the same React
/// bundle as the main window (`index.html`); `main.tsx` renders the picker view
/// when it detects the `prompt-picker` window label.
///
/// Deliberately minimal: just (re)show and focus via Tauri's own APIs. Earlier
/// attempts to force the foreground with the Win32 `AttachThreadInput` trick
/// worked for the first couple of opens but corrupted the window's show/focus
/// state after a few cycles, so the popup stopped appearing and needed a second
/// hotkey press. The popup is `always_on_top`, so plain `show()` is enough to
/// make it visible even when another app owns the foreground.
pub fn show(app: &AppHandle) -> tauri::Result<()> {
    // Build a brand-new window every time. Reusing a hidden window and
    // re-showing it proved unreliable: after a couple of show/hide cycles Tauri
    // reports the window visible but it no longer comes to the front, so the
    // popup seemed to need a second hotkey press. Destroying + recreating gives
    // a fresh `focused` window that reliably appears on top.
    if let Some(old) = app.get_webview_window("prompt-picker") {
        let _ = old.destroy();
    }
    let win = WebviewWindowBuilder::new(
        app,
        "prompt-picker",
        WebviewUrl::App("index.html".into()),
    )
    .title("Dictatr Prompt-Manager")
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .inner_size(PICKER_W, PICKER_H)
    .focused(true)
    .visible(false)
    .build()?;
    position_center(&win);
    win.show()?;
    let _ = win.set_focus();
    Ok(())
}

/// Close the popup by destroying it, so the next open starts from a clean slate.
pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("prompt-picker") {
        let _ = win.destroy();
    }
}

/// Center the popup on the primary monitor. Recomputed on every show so monitor
/// reconfiguration (hotplug, scale change) is picked up without app restart.
fn position_center(win: &WebviewWindow) {
    let monitor = match win.primary_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let size = monitor.size();
    let pos = monitor.position();
    let scale = monitor.scale_factor();
    let w = (PICKER_W * scale) as i32;
    let h = (PICKER_H * scale) as i32;
    let x = pos.x + (size.width as i32 - w) / 2;
    let y = pos.y + (size.height as i32 - h) / 2;
    let _ = win.set_position(PhysicalPosition::new(x, y));
}

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Logical size of the recording-indicator overlay. Physical size is derived
/// from this via the primary monitor's scale factor at position time.
const OVERLAY_W: f64 = 320.0;
const OVERLAY_H: f64 = 56.0;
/// Gap between overlay bottom edge and the primary monitor's bottom edge.
const BOTTOM_MARGIN: f64 = 40.0;

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    let win = match app.get_webview_window("overlay") {
        Some(w) => w,
        None => WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
            .title("Dictatr Overlay")
            .decorations(false)
            .always_on_top(true)
            .transparent(true)
            .skip_taskbar(true)
            .inner_size(OVERLAY_W, OVERLAY_H)
            .build()?,
    };
    position_bottom_center(&win);
    win.show()?;
    Ok(())
}

pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
}

/// Position the overlay at the bottom-center of the primary monitor.
/// Recomputed on every show so monitor reconfiguration (hotplug, scale change)
/// is picked up without app restart.
fn position_bottom_center(win: &WebviewWindow) {
    let monitor = match win.primary_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let size = monitor.size();
    let pos = monitor.position();
    let scale = monitor.scale_factor();
    let w = (OVERLAY_W * scale) as i32;
    let h = (OVERLAY_H * scale) as i32;
    let margin = (BOTTOM_MARGIN * scale) as i32;
    let x = pos.x + (size.width as i32 - w) / 2;
    let y = pos.y + size.height as i32 - h - margin;
    let _ = win.set_position(PhysicalPosition::new(x, y));
}

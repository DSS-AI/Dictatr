use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
        return Ok(());
    }
    let _ = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("Dictatr Overlay")
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .skip_taskbar(true)
        .inner_size(220.0, 60.0)
        .build()?;
    Ok(())
}

pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
}

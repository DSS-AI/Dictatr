use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open_settings = MenuItem::with_id(app, "open_settings", "Einstellungen", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_settings, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open_settings" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    // Windows steals focus aggressively; toggling always-on-top
                    // forces the window to the foreground reliably.
                    let _ = win.set_always_on_top(true);
                    let _ = win.set_focus();
                    let _ = win.set_always_on_top(false);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

pub fn set_state_icon(app: &AppHandle, path: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(img) = tauri::image::Image::from_path(path) {
            let _ = tray.set_icon(Some(img));
        }
    }
}

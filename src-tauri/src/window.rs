use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

pub const OVERLAY_HEIGHT: u32 = 220;

pub fn setup_overlay(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .expect("main window must exist");

    if let Some(monitor) = window.primary_monitor()? {
        let size = monitor.size();
        let overlay_height = OVERLAY_HEIGHT.min(size.height);

        window.set_size(PhysicalSize::new(size.width, overlay_height))?;
        window.set_position(PhysicalPosition::new(
            0,
            size.height as i32 - overlay_height as i32,
        ))?;
    }

    window.set_ignore_cursor_events(true)?;
    window.set_skip_taskbar(true)?;
    window.show()?;

    Ok(())
}

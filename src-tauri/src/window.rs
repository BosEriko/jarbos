use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

pub fn setup_overlay(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .expect("main window must exist");

    if let Some(monitor) = window.primary_monitor()? {
        let size = monitor.size();

        window.set_size(PhysicalSize::new(size.width, size.height))?;
        window.set_position(PhysicalPosition::new(0, 0))?;
    }

    window.set_ignore_cursor_events(true)?;
    window.set_skip_taskbar(true)?;
    window.show()?;

    Ok(())
}

mod agent;
mod hit_test;
mod process_monitor;
mod state_manager;
mod window;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, State};

use agent::AgentState;
use hit_test::{AvatarRect, AvatarRects, DragFlag};
use state_manager::{InstanceState, SharedStates};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentStateChangedPayload {
    instance_id: String,
    agent_id: String,
    state: AgentState,
}

#[tauri::command]
fn get_agent_states(states: State<'_, SharedStates>) -> HashMap<String, InstanceState> {
    states.lock().unwrap().clone()
}

#[tauri::command]
fn update_avatar_rects(rects: Vec<AvatarRect>, state: State<'_, AvatarRects>) {
    *state.lock().unwrap() = rects;
}

#[tauri::command]
fn begin_drag(state: State<'_, DragFlag>) {
    state.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn end_drag(state: State<'_, DragFlag>) {
    state.store(false, Ordering::SeqCst);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_agent_states,
            update_avatar_rects,
            begin_drag,
            end_drag
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            window::setup_overlay(app.handle())?;

            let avatar_rects: AvatarRects = Arc::new(Mutex::new(Vec::new()));
            let dragging: DragFlag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let overlay_window = app
                .get_webview_window("main")
                .expect("main window must exist");
            hit_test::spawn(
                overlay_window,
                Arc::clone(&avatar_rects),
                Arc::clone(&dragging),
            );
            app.manage(avatar_rects);
            app.manage(dragging);

            let tray_icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_item])?;
            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&tray_menu)
                .tooltip("jarbos")
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            let agents = agent::default_agents();
            let (tx, rx) = mpsc::channel();
            process_monitor::spawn(agents, tx);

            let app_handle = app.handle().clone();
            let states = state_manager::run(rx, move |instance_id, agent_id, state| {
                let payload = AgentStateChangedPayload {
                    instance_id: instance_id.to_string(),
                    agent_id: agent_id.to_string(),
                    state,
                };
                let _ = app_handle.emit("agent-state-changed", payload);
            });
            app.manage(states);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

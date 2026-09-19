mod agent;
mod process_monitor;
mod state_manager;
mod window;

use std::collections::HashMap;
use std::sync::mpsc;

use serde::Serialize;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, State};

use agent::AgentState;
use state_manager::SharedStates;

#[derive(Debug, Clone, Serialize)]
struct AgentStateChangedPayload {
    id: String,
    state: AgentState,
}

#[tauri::command]
fn get_agent_states(states: State<'_, SharedStates>) -> HashMap<String, AgentState> {
    states.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_agent_states])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            window::setup_overlay(app.handle())?;

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
            let states = state_manager::run(rx, move |id, state| {
                let payload = AgentStateChangedPayload {
                    id: id.to_string(),
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

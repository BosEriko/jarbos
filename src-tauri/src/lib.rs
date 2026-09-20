mod agent;
mod hit_test;
mod process_monitor;
mod state_manager;
mod window;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};

use agent::AgentState;
use hit_test::{AvatarRect, AvatarRects, DragFlag};
use state_manager::{InstanceState, SharedStates};

const AVATAR_SIZE_PRESETS: [(&str, &str, u32); 4] = [
    ("resize_small", "Small", 40),
    ("resize_medium", "Medium", 64),
    ("resize_large", "Large", 88),
    ("resize_xlarge", "X-Large", 112),
];
const DEFAULT_AVATAR_SIZE: u32 = 64;
const DEFAULT_AVATAR_SIZE_ID: &str = "resize_medium";
const SETTINGS_FILE_NAME: &str = "display-settings.json";

fn size_for_id(id: &str) -> u32 {
    AVATAR_SIZE_PRESETS
        .iter()
        .find(|(preset_id, _, _)| *preset_id == id)
        .map(|(_, _, size)| *size)
        .unwrap_or(DEFAULT_AVATAR_SIZE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisplaySettings {
    ducks_visible: bool,
    names_visible: bool,
    avatar_size_id: String,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            ducks_visible: true,
            names_visible: true,
            avatar_size_id: DEFAULT_AVATAR_SIZE_ID.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DisplaySettingsPayload {
    ducks_visible: bool,
    names_visible: bool,
    avatar_size: u32,
}

impl From<&DisplaySettings> for DisplaySettingsPayload {
    fn from(settings: &DisplaySettings) -> Self {
        Self {
            ducks_visible: settings.ducks_visible,
            names_visible: settings.names_visible,
            avatar_size: size_for_id(&settings.avatar_size_id),
        }
    }
}

fn settings_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    let dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(SETTINGS_FILE_NAME))
}

fn load_settings(app: &AppHandle) -> DisplaySettings {
    settings_path(app)
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

fn save_settings(app: &AppHandle, settings: &DisplaySettings) {
    if let Ok(path) = settings_path(app) {
        if let Ok(contents) = serde_json::to_string_pretty(settings) {
            let _ = std::fs::write(path, contents);
        }
    }
}

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

#[tauri::command]
fn get_display_settings(settings: State<'_, Arc<Mutex<DisplaySettings>>>) -> DisplaySettingsPayload {
    DisplaySettingsPayload::from(&*settings.lock().unwrap())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_agent_states,
            update_avatar_rects,
            begin_drag,
            end_drag,
            get_display_settings
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

            let settings: Arc<Mutex<DisplaySettings>> =
                Arc::new(Mutex::new(load_settings(app.handle())));
            app.manage(Arc::clone(&settings));
            let initial_settings = settings.lock().unwrap().clone();

            let tray_icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
            let toggle_visibility_item = MenuItem::with_id(
                app,
                "toggle_visibility",
                if initial_settings.ducks_visible { "Hide Ducks" } else { "Show Ducks" },
                true,
                None::<&str>,
            )?;
            let toggle_names_item = MenuItem::with_id(
                app,
                "toggle_names",
                if initial_settings.names_visible { "Hide Names" } else { "Show Names" },
                true,
                None::<&str>,
            )?;
            let size_items: Vec<CheckMenuItem<_>> = AVATAR_SIZE_PRESETS
                .iter()
                .map(|(id, label, _)| {
                    CheckMenuItem::with_id(
                        app,
                        *id,
                        *label,
                        true,
                        *id == initial_settings.avatar_size_id,
                        None::<&str>,
                    )
                })
                .collect::<tauri::Result<_>>()?;
            let size_item_refs: Vec<&dyn IsMenuItem<_>> =
                size_items.iter().map(|item| item as &dyn IsMenuItem<_>).collect();
            let size_submenu = Submenu::with_items(app, "Duck Size", true, &size_item_refs)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(
                app,
                &[
                    &toggle_visibility_item,
                    &toggle_names_item,
                    &size_submenu,
                    &separator,
                    &quit_item,
                ],
            )?;

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&tray_menu)
                .tooltip("jarbos")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "toggle_visibility" => {
                        let visible = {
                            let mut current = settings.lock().unwrap();
                            current.ducks_visible = !current.ducks_visible;
                            save_settings(app, &current);
                            current.ducks_visible
                        };
                        let _ = toggle_visibility_item
                            .set_text(if visible { "Hide Ducks" } else { "Show Ducks" });
                        let _ = app.emit("toggle-ducks-visibility", visible);
                    }
                    "toggle_names" => {
                        let visible = {
                            let mut current = settings.lock().unwrap();
                            current.names_visible = !current.names_visible;
                            save_settings(app, &current);
                            current.names_visible
                        };
                        let _ = toggle_names_item
                            .set_text(if visible { "Hide Names" } else { "Show Names" });
                        let _ = app.emit("toggle-ducks-names", visible);
                    }
                    id => {
                        if let Some(&(_, _, size)) =
                            AVATAR_SIZE_PRESETS.iter().find(|(preset_id, _, _)| *preset_id == id)
                        {
                            {
                                let mut current = settings.lock().unwrap();
                                current.avatar_size_id = id.to_string();
                                save_settings(app, &current);
                            }
                            for item in &size_items {
                                let _ = item.set_checked(item.id().as_ref() == id);
                            }
                            let _ = app.emit("resize-ducks", size);
                        }
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

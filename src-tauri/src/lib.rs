mod commands;
mod watcher;

use std::sync::Arc;
use tokio::sync::Mutex;

pub use commands::*;

pub fn build_state() -> Arc<Mutex<commands::OmniState>> {
    Arc::new(Mutex::new(commands::OmniState::new()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = build_state();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::detect_file,
            commands::list_targets,
            commands::convert_single,
            commands::get_tool_status,
            commands::get_presets,
            commands::queue_add,
            commands::queue_list
        ])
        .run(tauri::generate_context!())
        .expect("failed to run OmniConvert");
}

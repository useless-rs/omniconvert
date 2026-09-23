use std::sync::Arc;
use tokio::sync::Mutex;

use omni_core::{all_presets, OmniEngine, Preset};

pub struct OmniState {
    pub engine: OmniEngine,
    pub queue: omni_core::JobQueue,
}

impl OmniState {
    pub fn new() -> Self {
        Self { engine: OmniEngine::new(), queue: omni_core::JobQueue::new() }
    }
}

#[tauri::command]
pub async fn detect_file(state: tauri::State<'_, Arc<Mutex<OmniState>>>, path: String) -> Result<omni_core::Detected, String> {
    let st = state.lock().await;
    st.engine.detect(std::path::Path::new(&path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_targets(state: tauri::State<'_, Arc<Mutex<OmniState>>>, from: String) -> Result<Vec<String>, String> {
    Ok(state.lock().await.engine.supported_targets(&from))
}

#[tauri::command]
pub async fn convert_single(state: tauri::State<'_, Arc<Mutex<OmniState>>>, input: String, output: String) -> Result<String, String> {
    let st = state.lock().await;
    st.engine
        .convert_file(std::path::Path::new(&input), std::path::Path::new(&output))
        .await
        .map(|_| output.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tool_status() -> Result<Vec<omni_core::deps::ToolStatus>, String> {
    Ok(omni_core::deps::check_all())
}

#[tauri::command]
pub async fn get_presets() -> Result<Vec<Preset>, String> {
    Ok(all_presets())
}

#[tauri::command]
pub async fn queue_add(state: tauri::State<'_, Arc<Mutex<OmniState>>>, input: String, output: String) -> Result<String, String> {
    Ok(state.lock().await.queue.submit(input, output).await)
}

#[tauri::command]
pub async fn queue_list(state: tauri::State<'_, Arc<Mutex<OmniState>>>) -> Result<Vec<omni_core::Job>, String> {
    Ok(state.lock().await.queue.list().await)
}

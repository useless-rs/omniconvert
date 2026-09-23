use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub from: String,
    pub to: String,
    pub options: serde_json::Value,
}

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset { name: "Audio MP3 320k".into(), from: "wav".into(), to: "mp3".into(), options: serde_json::json!({"codec":"libmp3lame","bitrate":"320k"}) },
        Preset { name: "Video H264 1080p".into(), from: "mkv".into(), to: "mp4".into(), options: serde_json::json!({"codec":"libx264","crf":20}) },
        Preset { name: "Image WebP q80".into(), from: "png".into(), to: "webp".into(), options: serde_json::json!({"quality":80}) },
        Preset { name: "Doc to PDF".into(), from: "docx".into(), to: "pdf".into(), options: serde_json::json!({}) },
        Preset { name: "Data JSON pretty".into(), from: "csv".into(), to: "json".into(), options: serde_json::json!({"pretty":true}) },
        Preset { name: "Bundle ZIP".into(), from: "txt".into(), to: "zip".into(), options: serde_json::json!({}) },
    ]
}

pub fn presets_file() -> PathBuf {
    dirs_path().join("omniconvert").join("presets.json")
}

fn dirs_path() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from).map(|h| h.join(".config")))
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn load_custom() -> Vec<Preset> {
    let p = presets_file();
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_custom(presets: &[Preset]) -> anyhow::Result<()> {
    let p = presets_file();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(presets)?)?;
    Ok(())
}

pub fn all_presets() -> Vec<Preset> {
    let mut v = builtin_presets();
    v.extend(load_custom());
    v
}

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::formats::find_by_extension;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detected {
    pub format_id: String,
    pub mime: String,
    pub confidence: f32,
    pub method: String,
}

/// Detect file type by content (magic bytes) first, extension as fallback.
pub fn detect(path: &Path) -> anyhow::Result<Detected> {
    let bytes = std::fs::read(path).unwrap_or_default();
    // 1. magic bytes via `infer`
    if let Some(kind) = infer::get(&bytes) {
        let mime = kind.mime_type().to_string();
        let by_mime = crate::formats::all_formats()
            .into_iter()
            .find(|f| f.mime == mime);
        if let Some(fmt) = by_mime {
            return Ok(Detected {
                format_id: fmt.id,
                mime,
                confidence: 0.95,
                method: format!("magic-bytes:{}", kind.extension()),
            });
        }
        // unknown mime but we got magic -> generic binary with mime info
        return Ok(Detected {
            format_id: sniff_text_or_binary(path, &bytes),
            mime,
            confidence: 0.7,
            method: "magic-bytes+sniff".into(),
        });
    }
    // 2. image-crate sniff (covers far more than infer for images)
    if image::guess_format(&bytes).is_ok() {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_lowercase();
        if let Some(fmt) = find_by_extension(&ext) {
            return Ok(Detected {
                format_id: fmt.id.clone(),
                mime: fmt.mime.clone(),
                confidence: 0.8,
                method: "image-sniff+extension".into(),
            });
        }
    }
    // 3. text sniff
    Ok(Detected {
        format_id: sniff_text_or_binary(path, &bytes),
        mime: "text/plain".into(),
        confidence: 0.5,
        method: "extension+content-sniff".into(),
    })
}

fn sniff_text_or_binary(path: &Path, bytes: &[u8]) -> String {
    // content looks like known text formats?
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_string();
    let t = head.trim_start();
    if t.starts_with('{') || t.starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(&head).is_ok() {
            return "json".into();
        }
    }
    if t.starts_with('<') {
        if t.contains("<html") {
            return "html".into();
        }
        return "xml".into();
    }
    if t.contains("---") || t.lines().any(|l| l.contains(": ")) {
        // plausible yaml
        if serde_yaml::from_str::<serde_yaml::Value>(&head).is_ok() {
            return "yaml".into();
        }
    }
    if head.contains(',') && head.lines().next().map(|l| l.matches(',').count() >= 1).unwrap_or(false) {
        return "csv".into();
    }
    if is_probably_text(bytes) {
        // honor extension for text-ish
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if let Some(fmt) = find_by_extension(ext) {
                return fmt.id;
            }
        }
        // markdown heuristic
        if head.contains("# ") || head.contains("```") || head.contains("[](") {
            return "md".into();
        }
        return "txt".into();
    }
    // binary fallback honors extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if let Some(fmt) = find_by_extension(ext) {
            return fmt.id;
        }
    }
    "bin".into()
}

fn is_probably_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    let sample = &bytes[..bytes.len().min(4096)];
    let nontext = sample
        .iter()
        .filter(|b| **b == 0 || (**b < 9) || (**b > 13 && **b < 32 && **b != 27))
        .count();
    (nontext as f32) / (sample.len() as f32) < 0.05
}

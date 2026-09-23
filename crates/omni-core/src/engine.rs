use std::path::Path;

use crate::adapters::{external, native_archive, native_image, native_sheet, native_text};
use crate::detect::{detect, Detected};
use crate::error::Result;
use crate::registry::ConversionGraph;

/// The universal engine. New format pairs = new adapter + registry edge.
/// Core never changes per-format.
pub struct OmniEngine {
    graph: ConversionGraph,
}

impl OmniEngine {
    pub fn new() -> Self {
        Self { graph: ConversionGraph::new() }
    }

    pub fn detect(&self, path: &Path) -> anyhow::Result<Detected> {
        detect(path)
    }

    pub fn supported_targets(&self, from: &str) -> Vec<String> {
        self.graph.supported_targets(from)
    }

    pub async fn convert_file(&self, input: &Path, output: &Path) -> Result<()> {
        convert_file(input, output).await
    }
}

impl Default for OmniEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn ext_of(p: &Path) -> String {
    p.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// Route (from,to) to the right adapter: native fast-paths first,
/// external tools with graceful ToolMissing otherwise.
pub async fn convert_file(input: &Path, output: &Path) -> Result<()> {
    // Detect real types by content, not just extension
    let from = detect(input)
        .map(|d| d.format_id)
        .unwrap_or_else(|_| ext_or_bin(input));
    let to_raw = ext_of(output);
    let to = if to_raw.is_empty() { from.clone() } else { to_raw };

    let graph = ConversionGraph::new();
    let edge = graph
        .find_route(&from, &to)
        .cloned()
        .ok_or_else(|| graph.explain_why_not(&from, &to))?;

    // If the edge needs a tool, verify early for a clear error message
    if let Some(tool) = &edge.needs_tool {
        if !crate::deps::check_all().iter().any(|t| &t.binary == tool && t.installed) {
            return Err(graph.explain_why_not(&from, &to));
        }
        return external::convert_external(&edge.via, &from, &to, input, output).await;
    }

    match edge.via.as_str() {
        "native_text" | "native_data" => native_text::convert(&from, &to, input, output).await,
        "native_image" => native_image::convert(&from, &to, input, output).await,
        "native_archive" => native_archive::convert(&from, &to, input, output).await,
        "native_sheet" => native_sheet::convert(&from, &to, input, output).await,
        "copy" => {
            std::fs::copy(input, output)?;
            Ok(())
        }
        other => external::convert_external(other, &from, &to, input, output).await,
    }
}

fn ext_or_bin(p: &Path) -> String {
    let e = ext_of(p);
    if e.is_empty() {
        "bin".into()
    } else {
        e
    }
}

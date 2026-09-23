use std::path::Path;

use crate::error::{ConverterError, Result};

/// Pure-Rust image conversion via the `image` crate (parallel decode).
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if from == to {
        std::fs::copy(input, output)?;
        return Ok(());
    }
    let input = input.to_path_buf();
    let img = tokio::task::spawn_blocking(move || image::open(&input))
        .await
        .map_err(|e| ConverterError::Parse(e.to_string()))?
        .map_err(|e| ConverterError::Parse(format!("cannot decode image: {e}")))?;
    let output = output.to_path_buf();
    tokio::task::spawn_blocking(move || {
        // `image` infers format from the output extension
        img.save(&output)
    })
    .await
    .map_err(|e| ConverterError::Parse(e.to_string()))?
    .map_err(|e| ConverterError::Parse(format!("cannot encode as {to}: {e}")))?;
    Ok(())
}

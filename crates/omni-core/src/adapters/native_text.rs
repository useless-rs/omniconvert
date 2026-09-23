use std::collections::BTreeMap;
use std::path::Path;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

use crate::error::{ConverterError, Result};

fn read_to_string_lossy(p: &Path) -> Result<String> {
    let b = std::fs::read(p)?;
    Ok(String::from_utf8_lossy(&b).to_string())
}

/// Convert between text-ish formats without external tools.
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if from == to {
        std::fs::copy(input, output)?;
        return Ok(());
    }
    // Load source into a serde_json::Value when it is structured,
    // else treat as raw text.
    let value: Option<serde_json::Value> = match from {
        "json" => Some(serde_json::from_str(&read_to_string_lossy(input)?).map_err(|e| {
            ConverterError::Parse(format!("invalid JSON in {}: {e}", input.display()))
        })?),
        "yaml" => {
            let v: serde_yaml::Value = serde_yaml::from_str(&read_to_string_lossy(input)?)
                .map_err(|e| ConverterError::Parse(format!("invalid YAML: {e}")))?;
            Some(serde_json::to_value(v).map_err(|e| ConverterError::Parse(e.to_string()))?)
        }
        "toml" => {
            let v: toml::Value = read_to_string_lossy(input)?
                .parse()
                .map_err(|e: toml::de::Error| ConverterError::Parse(format!("invalid TOML: {e}")))?;
            Some(serde_json::to_value(v).map_err(|e| ConverterError::Parse(e.to_string()))?)
        }
        "xml" => {
            let s = read_to_string_lossy(input)?;
            let v: serde_json::Value = quick_xml::de::from_str(&s)
                .map_err(|e| ConverterError::Parse(format!("invalid XML: {e}")))?;
            Some(v)
        }
        "csv" | "tsv" => {
            let delim = if from == "tsv" { b'\t' } else { b',' };
            let data = std::fs::read(input)?;
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(delim)
                .from_reader(data.as_slice());
            let headers = rdr
                .headers()
                .map_err(|e| ConverterError::Parse(e.to_string()))?
                .clone();
            let mut rows = vec![];
            for rec in rdr.records() {
                let rec = rec.map_err(|e| ConverterError::Parse(e.to_string()))?;
                let mut m = BTreeMap::new();
                for (h, v) in headers.iter().zip(rec.iter()) {
                    m.insert(h.to_string(), serde_json::Value::String(v.to_string()));
                }
                rows.push(serde_json::Value::Object(m.into_iter().collect()));
            }
            Some(serde_json::Value::Array(rows))
        }
        _ => None,
    };

    if let Some(v) = value {
        return write_value(v, to, output).await;
    }

    // raw-text conversions
    let raw = std::fs::read(input)?;
    let text = String::from_utf8_lossy(&raw).to_string();
    match (from, to) {
        (_, "b64") => {
            std::fs::write(output, B64.encode(raw))?;
            Ok(())
        }
        ("b64", _) => {
            let decoded = B64.decode(text.trim()).map_err(|e| ConverterError::Parse(format!("invalid base64: {e}")))?;
            std::fs::write(output, decoded)?;
            Ok(())
        }
        (_, "html") => {
            let esc = text.replace('&', "&amp;").replace('<', "&lt;");
            std::fs::write(output, format!("<!doctype html><html><body><pre>{esc}</pre></body></html>"))?;
            Ok(())
        }
        (_, "md") => {
            std::fs::write(output, format!("# Converted from {from}\n\n```\n{text}\n```\n"))?;
            Ok(())
        }
        (_, "srt") if from == "vtt" => {
            std::fs::write(output, text.replace("WEBVTT\n", "").replace('.', ","))?;
            Ok(())
        }
        (_, "vtt") if from == "srt" => {
            std::fs::write(output, format!("WEBVTT\n\n{}", text.replace(',', ".")))?;
            Ok(())
        }
        _ => {
            // txt/md/srt/vtt/m3u/html passthrough-ish
            std::fs::write(output, text)?;
            Ok(())
        }
    }
}

async fn write_value(v: serde_json::Value, to: &str, output: &Path) -> Result<()> {
    match to {
        "json" => {
            std::fs::write(output, serde_json::to_string_pretty(&v).map_err(|e| ConverterError::Parse(e.to_string()))?)?;
            Ok(())
        }
        "yaml" => {
            std::fs::write(output, serde_yaml::to_string(&v).map_err(|e| ConverterError::Parse(e.to_string()))?)?;
            Ok(())
        }
        "toml" => {
            // TOML requires a table at the root: wrap arrays/scalars.
            let doc = match &v {
                serde_json::Value::Object(_) => v,
                serde_json::Value::Array(a) => serde_json::json!({"items": a}),
                other => serde_json::json!({"value": other}),
            };
            let s = toml::to_string(&doc).map_err(|e| ConverterError::Parse(format!("cannot represent as TOML (needs a table at root): {e}")))?;
            std::fs::write(output, s)?;
            Ok(())
        }
        "xml" => {
            let s = quick_xml::se::to_string(&v).map_err(|e| ConverterError::Parse(e.to_string()))?;
            std::fs::write(output, s)?;
            Ok(())
        }
        "csv" => {
            let rows: Vec<BTreeMap<String, String>> = match &v {
                serde_json::Value::Array(a) => a
                    .iter()
                    .map(|r| {
                        r.as_object()
                            .unwrap_or(&serde_json::Map::new())
                            .iter()
                            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                            .collect()
                    })
                    .collect(),
                serde_json::Value::Object(m) => vec![m
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()],
                _ => vec![],
            };
            let mut wtr = csv::Writer::from_path(output).map_err(|e| ConverterError::Parse(e.to_string()))?;
            let mut headers_written = false;
            for row in rows {
                if !headers_written {
                    let hs: Vec<String> = row.keys().cloned().collect();
                    wtr.write_record(&hs).map_err(|e| ConverterError::Parse(e.to_string()))?;
                    headers_written = true;
                }
                let vs: Vec<String> = row.values().cloned().collect();
                wtr.write_record(&vs).map_err(|e| ConverterError::Parse(e.to_string()))?;
            }
            wtr.flush().map_err(|e| ConverterError::Parse(e.to_string()))?;
            Ok(())
        }
        "txt" | "md" => {
            std::fs::write(output, serde_json::to_string_pretty(&v).map_err(|e| ConverterError::Parse(e.to_string()))?)?;
            Ok(())
        }
        _ => Err(ConverterError::Parse(format!("native_text cannot emit {to}"))),
    }
}

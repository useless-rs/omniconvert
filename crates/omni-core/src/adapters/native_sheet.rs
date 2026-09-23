use std::path::Path;

use calamine::{open_workbook, Reader, Xlsx};

use crate::error::{ConverterError, Result};

fn open_first_sheet(input: &Path) -> Result<(String, Vec<Vec<String>>)> {
    let mut wb: Xlsx<_> =
        open_workbook(input).map_err(|e| ConverterError::Parse(format!("cannot open workbook: {e}")))?;
    let names = wb.sheet_names();
    let first = names.first().ok_or_else(|| ConverterError::Parse("workbook has no sheets".into()))?.clone();
    let range = wb
        .worksheet_range(&first)
        .map_err(|e| ConverterError::Parse(format!("cannot read sheet {first:?}: {e}")))?;
    let rows: Vec<Vec<String>> = range.rows().map(|r| r.iter().map(|c| c.to_string()).collect()).collect();
    Ok((first, rows))
}

/// Pure-Rust spreadsheet reading (no LibreOffice): xlsx/xls/ods -> csv/json/txt.
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if from == to {
        std::fs::copy(input, output)?;
        return Ok(());
    }
    let (_sheet, rows) = open_first_sheet(input)?;
    match to {
        "csv" => {
            let mut wtr = csv::Writer::from_path(output).map_err(|e| ConverterError::Parse(e.to_string()))?;
            for row in &rows {
                wtr.write_record(row).map_err(|e| ConverterError::Parse(e.to_string()))?;
            }
            wtr.flush().map_err(|e| ConverterError::Parse(e.to_string()))?;
            Ok(())
        }
        "json" | "yaml" | "txt" | "md" => {
            if rows.is_empty() {
                std::fs::write(output, "[]")?;
                return Ok(());
            }
            let headers = &rows[0];
            let mut items = vec![];
            for row in rows.iter().skip(1) {
                let mut m = serde_json::Map::new();
                for (i, h) in headers.iter().enumerate() {
                    let key = if h.is_empty() { format!("col{i}") } else { h.clone() };
                    m.insert(key, serde_json::Value::String(row.get(i).cloned().unwrap_or_default()));
                }
                items.push(serde_json::Value::Object(m));
            }
            let v = serde_json::Value::Array(items);
            super::native_text::convert_value(v, to, output).await
        }
        _ => Err(ConverterError::Parse(format!("native_sheet cannot emit {to}"))),
    }
}

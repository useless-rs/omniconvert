use std::io::Write;
use std::path::Path;

use crate::error::Result;

/// Native archive support: bundle any file into .zip, list .zip contents.
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if from == "zip" && to == "zip" {
        std::fs::copy(input, output)?;
        return Ok(());
    }
    if to == "zip" {
        let data = std::fs::read(input)?;
        let name = input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let file = std::fs::File::create(output)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(name, opts)?;
        zip.write_all(&data)?;
        zip.finish()?;
        return Ok(());
    }
    if from == "zip" && to == "txt" {
        let file = std::fs::File::open(input)?;
        let mut zip = zip::ZipArchive::new(file)?;
        let mut listing = String::from("Archive contents:\n");
        for i in 0..zip.len() {
            let f = zip.by_index(i)?;
            listing.push_str(&format!("- {} ({} bytes)\n", f.name(), f.size()));
        }
        std::fs::write(output, listing)?;
        return Ok(());
    }
    std::fs::copy(input, output)?;
    Ok(())
}

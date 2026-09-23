use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{ConverterError, Result};

fn is_archive(f: &str) -> bool {
    matches!(f, "zip" | "tar" | "gz" | "7z")
}

fn file_name(input: &Path) -> String {
    input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string()
}

fn stage_dir() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "omni-arch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn safe_name(raw: &str) -> Option<String> {
    let p = Path::new(raw);
    if p.is_absolute() {
        return None;
    }
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return None;
    }
    let s = raw.replace('\\', "/");
    if s.is_empty() {
        return None;
    }
    Some(s)
}

/// Native archives, no external tools: zip/tar/gz/7z.
/// Semantics: plain file -> archive bundles it; archive -> txt lists
/// (gz -> txt yields the decompressed bytes, a single stream);
/// archive -> archive repacks through a staging dir.
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if from == to {
        std::fs::copy(input, output)?;
        return Ok(());
    }
    if !is_archive(from) && !is_archive(to) {
        return Err(ConverterError::Parse(format!("native_archive cannot do {from}->{to}")));
    }
    let staging = stage_dir()?;
    let result = convert_inner(from, to, input, output, &staging).await;
    let _ = std::fs::remove_dir_all(&staging);
    result
}

async fn convert_inner(from: &str, to: &str, input: &Path, output: &Path, staging: &Path) -> Result<()> {
    if !is_archive(from) {
        pack_single(input, to, output)?;
        return Ok(());
    }
    if to == "txt" {
        if from == "gz" {
            let bytes = gunzip(input)?;
            std::fs::write(output, bytes)?;
            return Ok(());
        }
        let entries = extract_to(from, input, &staging.join("in"))?;
        std::fs::write(output, listing(&entries))?;
        return Ok(());
    }
    let indir = staging.join("in");
    extract_to(from, input, &indir)?;
    pack_dir(&indir, to, output)?;
    Ok(())
}

fn pack_single(input: &Path, to: &str, output: &Path) -> Result<()> {
    match to {
        "zip" => {
            let data = std::fs::read(input)?;
            let file = std::fs::File::create(output)?;
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zip.start_file(file_name(input), opts)?;
            zip.write_all(&data)?;
            zip.finish()?;
            Ok(())
        }
        "tar" => {
            let file = std::fs::File::create(output)?;
            let mut tar = tar::Builder::new(file);
            tar.append_path_with_name(input, file_name(input))
                .map_err(|e| ConverterError::Parse(e.to_string()))?;
            tar.finish().map_err(|e| ConverterError::Parse(e.to_string()))?;
            Ok(())
        }
        "gz" => {
            let mut src = std::fs::File::open(input)?;
            let dst = std::fs::File::create(output)?;
            let mut enc = flate2::write::GzEncoder::new(dst, flate2::Compression::default());
            std::io::copy(&mut src, &mut enc)?;
            enc.finish()?;
            Ok(())
        }
        "7z" => {
            let dir = stage_dir()?;
            let staged = dir.join(file_name(input));
            std::fs::copy(input, &staged)?;
            let r = sevenz_rust2::compress_to_path(&staged, output)
                .map_err(|e| ConverterError::Parse(format!("7z pack failed: {e}")));
            let _ = std::fs::remove_dir_all(&dir);
            r.map(|_| ())
        }
        _ => Err(ConverterError::Parse(format!("native_archive cannot emit {to}"))),
    }
}

fn gunzip(input: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(input)?;
    let mut dec = flate2::read::GzDecoder::new(file);
    let mut bytes = Vec::new();
    dec.read_to_end(&mut bytes).map_err(|e| {
        ConverterError::Parse(format!("invalid gzip {}: {e}", input.display()))
    })?;
    Ok(bytes)
}

fn extract_to(from: &str, input: &Path, dir: &Path) -> Result<Vec<(String, u64)>> {
    std::fs::create_dir_all(dir)?;
    match from {
        "zip" => {
            let file = std::fs::File::open(input)?;
            let mut zip = zip::ZipArchive::new(file)?;
            let mut entries = vec![];
            for i in 0..zip.len() {
                let mut f = zip.by_index(i)?;
                let Some(name) = safe_name(f.name()) else { continue };
                let dest = dir.join(&name);
                if f.is_dir() {
                    std::fs::create_dir_all(&dest)?;
                    continue;
                }
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut bytes = Vec::with_capacity(f.size() as usize);
                f.read_to_end(&mut bytes)?;
                let size = bytes.len() as u64;
                std::fs::write(&dest, bytes)?;
                entries.push((name, size));
            }
            Ok(entries)
        }
        "tar" => {
            let file = std::fs::File::open(input)?;
            let mut tar = tar::Archive::new(file);
            let mut entries = vec![];
            for entry in tar.entries().map_err(|e| ConverterError::Parse(e.to_string()))? {
                let mut entry = entry.map_err(|e| ConverterError::Parse(e.to_string()))?;
                let rel = entry.path().map_err(|e| ConverterError::Parse(e.to_string()))?.to_string_lossy().to_string();
                let Some(name) = safe_name(&rel) else { continue };
                let size = entry.size();
                entry.unpack_in(dir).map_err(|e| ConverterError::Parse(e.to_string()))?;
                entries.push((name, size));
            }
            Ok(entries)
        }
        "gz" => {
            let bytes = gunzip(input)?;
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("content")
                .to_string();
            let size = bytes.len() as u64;
            std::fs::write(dir.join(&stem), bytes)?;
            Ok(vec![(stem, size)])
        }
        "7z" => {
            sevenz_rust2::decompress_file(input, dir)
                .map_err(|e| ConverterError::Parse(format!("7z unpack failed: {e}")))?;
            let mut entries = vec![];
            for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let rel = entry
                        .path()
                        .strip_prefix(dir)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_default();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    entries.push((rel, size));
                }
            }
            entries.sort();
            Ok(entries)
        }
        _ => Err(ConverterError::Parse(format!("native_archive cannot read {from}"))),
    }
}

fn pack_dir(dir: &Path, to: &str, output: &Path) -> Result<()> {
    match to {
        "zip" => {
            let file = std::fs::File::create(output)?;
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let mut files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect();
            files.sort();
            for path in files {
                let rel = path.strip_prefix(dir).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                zip.start_file(rel, opts)?;
                let mut src = std::fs::File::open(&path)?;
                std::io::copy(&mut src, &mut zip)?;
            }
            zip.finish()?;
            Ok(())
        }
        "tar" => {
            let file = std::fs::File::create(output)?;
            let mut tar = tar::Builder::new(file);
            let mut files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect();
            files.sort();
            for path in files {
                let rel = path.strip_prefix(dir).unwrap_or(&path).to_path_buf();
                tar.append_path_with_name(&path, &rel)
                    .map_err(|e| ConverterError::Parse(e.to_string()))?;
            }
            tar.finish().map_err(|e| ConverterError::Parse(e.to_string()))?;
            Ok(())
        }
        "gz" => {
            let files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect();
            if files.len() == 1 {
                let mut src = std::fs::File::open(&files[0])?;
                let dst = std::fs::File::create(output)?;
                let mut enc = flate2::write::GzEncoder::new(dst, flate2::Compression::default());
                std::io::copy(&mut src, &mut enc)?;
                enc.finish()?;
                return Ok(());
            }
            let mut tar_bytes = Vec::new();
            {
                let mut tar = tar::Builder::new(&mut tar_bytes);
                for path in &files {
                    let rel = path.strip_prefix(dir).unwrap_or(path).to_path_buf();
                    let mut header = tar::Header::new_gnu();
                    let data = std::fs::read(path)?;
                    header.set_size(data.len() as u64);
                    header.set_mode(0o644);
                    header.set_cksum();
                    tar.append_data(&mut header, &rel, &data[..])
                        .map_err(|e| ConverterError::Parse(e.to_string()))?;
                }
                tar.finish().map_err(|e| ConverterError::Parse(e.to_string()))?;
            }
            let dst = std::fs::File::create(output)?;
            let mut enc = flate2::write::GzEncoder::new(dst, flate2::Compression::default());
            enc.write_all(&tar_bytes)?;
            enc.finish()?;
            Ok(())
        }
        "7z" => {
            sevenz_rust2::compress_to_path(dir, output)
                .map_err(|e| ConverterError::Parse(format!("7z pack failed: {e}")))
                .map(|_| ())
        }
        _ => Err(ConverterError::Parse(format!("native_archive cannot emit {to}"))),
    }
}

fn listing(entries: &[(String, u64)]) -> String {
    let mut s = String::from("Archive contents:\n");
    for (name, size) in entries {
        s.push_str(&format!("- {name} ({size} bytes)\n"));
    }
    s
}

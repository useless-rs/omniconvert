use std::path::Path;

use crate::error::{ConverterError, Result};

/// Shell out to a best-in-class external tool (ffmpeg, magick, pandoc...).
/// Never panics when the tool is missing: returns ToolMissing with install hint.
pub async fn convert_via(tool: &str, args: &[String]) -> Result<()> {
    let status = tokio::process::Command::new(tool)
        .args(args)
        .status()
        .await
        .map_err(|_| ConverterError::ToolMissing {
            tool: tool.into(),
            install_hint: crate::deps::install_hint_for(tool),
        })?;
    if !status.success() {
        return Err(ConverterError::ToolFailed {
            tool: tool.into(),
            stderr: format!("exit status {status}"),
        });
    }
    Ok(())
}

/// Build a default external command for a (from,to) pair and run it.
pub async fn convert_external(via: &str, from: &str, to: &str, input: &Path, output: &Path) -> Result<()> {
    if which::which(via).is_err() && which::which(tool_binary(via)).is_err() {
        return Err(ConverterError::ToolMissing {
            tool: via.into(),
            install_hint: crate::deps::install_hint_for(via),
        });
    }
    let bin = tool_binary(via);
    let inp = input.to_string_lossy().to_string();
    let out = output.to_string_lossy().to_string();
    let args: Vec<String> = match via {
        "ffmpeg" => vec!["-y".into(), "-i".into(), inp, out],
        "magick" => vec![inp, out],
        "pandoc" => vec![inp, "-o".into(), out],
        "libreoffice" => {
            let outdir = output.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or(".".into());
            vec!["--headless".into(), "--convert-to".into(), to.into(), "--outdir".into(), outdir, inp]
        }
        "7z" => vec!["x".into(), inp, format!("-o{}", output.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or(".".into()))],
        _ => vec![inp, out],
    };
    let _ = (from, to);
    convert_via(bin, &args).await
}

fn tool_binary(via: &str) -> &str {
    match via {
        "magick" => "magick",
        "ghostscript" => "gs",
        "ebook-convert" => "ebook-convert",
        other => other,
    }
}

// tiny `which` reimplementation to avoid another dependency
mod which {
    pub fn which(bin: &str) -> std::result::Result<std::path::PathBuf, ()> {
        let paths = std::env::var_os("PATH").unwrap_or_default();
        for dir in std::env::split_paths(&paths) {
            let p = dir.join(bin);
            if p.is_file() {
                return Ok(p);
            }
            #[cfg(windows)]
            {
                let exe = dir.join(format!("{bin}.exe"));
                if exe.is_file() {
                    return Ok(exe);
                }
            }
        }
        Err(())
    }
}

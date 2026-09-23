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
    if via == "ffmpeg" {
        let args: Vec<String> = vec![
            "-y".into(),
            "-hide_banner".into(),
            "-nostats".into(),
            "-progress".into(),
            "pipe:1".into(),
            "-i".into(),
            inp,
            out,
        ];
        return run_ffmpeg_with_progress(bin, &args).await;
    }
    let args: Vec<String> = match via {
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

async fn run_ffmpeg_with_progress(bin: &str, args: &[String]) -> Result<()> {
    use super::ffmpeg_progress as fp;
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut child = tokio::process::Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| ConverterError::ToolMissing {
            tool: bin.into(),
            install_hint: crate::deps::install_hint_for(bin),
        })?;
    let stderr_lines = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    let child_stderr = child.stderr.take();
    let child_stdout = child.stdout.take();
    let err_task = tokio::spawn({
        let lines = stderr_lines.clone();
        async move {
            if let Some(e) = child_stderr {
                let mut rows = BufReader::new(e).lines();
                while let Ok(Some(line)) = rows.next_line().await {
                    lines.lock().await.push(line);
                }
            }
        }
    });
    let mut total: Option<u64> = None;
    let mut last_pct: Option<u32> = None;
    if let Some(o) = child_stdout {
        let mut rows = BufReader::new(o).lines();
        while let Ok(Some(line)) = rows.next_line().await {
            match fp::parse_progress_line(&line) {
                Some(fp::ProgressEvent::Time(us)) => {
                    if total.is_none() {
                        total = fp::parse_duration(&stderr_lines.lock().await.join("\n"));
                    }
                    if let Some(t) = total {
                        if let Some(p) = fp::percent(us, t).filter(|p| last_pct != Some(*p)) {
                            last_pct = Some(p);
                            eprintln!("omni: {p}% ({}/{})", fp::fmt_hms(us), fp::fmt_hms(t));
                        }
                    }
                }
                Some(fp::ProgressEvent::End) => eprintln!("omni: 100%"),
                None => {}
            }
        }
    }
    let _ = err_task.await;
    let status = child.wait().await.map_err(|e| ConverterError::ToolFailed {
        tool: bin.into(),
        stderr: e.to_string(),
    })?;
    if !status.success() {
        let text = stderr_lines.lock().await.join("\n");
        let tail: Vec<&str> = text.lines().rev().take(5).collect();
        let tail: Vec<&str> = tail.into_iter().rev().collect();
        return Err(ConverterError::ToolFailed {
            tool: bin.into(),
            stderr: format!("exit {status}\n{}", tail.join("\n")),
        });
    }
    Ok(())
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

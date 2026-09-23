use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

/// Watch a folder and auto-convert new files to a target format.
pub async fn watch_folder(dir: &Path, target_ext: &str, out_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    println!("watching {} -> *.{} (out: {})", dir.display(), target_ext, out_dir.display());
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                use notify::EventKind::*;
                if matches!(event.kind, Create(_) | Modify(_)) {
                    for p in event.paths {
                        if p.is_file() {
                            if let Err(e) = convert_dropped(&p, target_ext, out_dir).await {
                                eprintln!("watch error {}: {e}", p.display());
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("watch error: {e}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

pub async fn convert_dropped(input: &Path, target_ext: &str, out_dir: &Path) -> anyhow::Result<PathBuf> {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let out = out_dir.join(format!("{stem}.{target_ext}"));
    omni_core::convert_file(input, &out)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("converted {} -> {}", input.display(), out.display());
    Ok(out)
}

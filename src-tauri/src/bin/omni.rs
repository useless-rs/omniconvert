//! `omni` — CLI for the universal converter (GUI-optional).
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "omni", version, about = "OmniConvert: EVERYTHING -> EVERYTHING")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert one file (format inferred by content + extension)
    Convert { input: PathBuf, output: PathBuf },
    /// Batch-convert many files to one target extension
    Batch { to: String, files: Vec<PathBuf>, #[arg(long)] out_dir: Option<PathBuf> },
    /// Detect a file's real type by magic bytes
    Detect { file: PathBuf },
    /// List supported targets for a format
    Targets { from: String },
    /// Show external-tool status (ffmpeg, magick, pandoc...)
    Tools,
    /// Watch a folder and auto-convert arrivals
    Watch { dir: PathBuf, to: String, #[arg(long)] out_dir: Option<PathBuf> },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let engine = omni_core::OmniEngine::new();
    match cli.cmd {
        Cmd::Convert { input, output } => match engine.convert_file(&input, &output).await {
            Ok(()) => println!("OK {} -> {}", input.display(), output.display()),
            Err(e) => {
                eprintln!("FAILED: {e}");
                std::process::exit(1);
            }
        },
        Cmd::Batch { to, files, out_dir } => {
            let out = out_dir.unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&out)?;
            let mut ok = 0;
            let mut fail = 0;
            let jobs = files.iter().map(|f| {
                let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("file").to_string();
                let dest = out.join(format!("{stem}.{to}"));
                let f = f.clone();
                async move { (f.clone(), omni_core::convert_file(&f, &dest).await, dest) }
            });
            for (src, res, dest) in futures::future::join_all(jobs).await {
                match res {
                    Ok(()) => {
                        println!("OK {} -> {}", src.display(), dest.display());
                        ok += 1;
                    }
                    Err(e) => {
                        eprintln!("FAILED {}: {e}", src.display());
                        fail += 1;
                    }
                }
            }
            println!("{ok} ok, {fail} failed");
            if fail > 0 {
                std::process::exit(1);
            }
        }
        Cmd::Detect { file } => {
            let d = engine.detect(&file)?;
            println!("{}: {} ({}) via {} [{:.0}%]", file.display(), d.format_id, d.mime, d.method, d.confidence * 100.0);
        }
        Cmd::Targets { from } => {
            for t in engine.supported_targets(&from) {
                println!("{t}");
            }
        }
        Cmd::Tools => {
            for t in omni_core::deps::check_all() {
                let mark = if t.installed { "OK  " } else { "MISS" };
                println!("{mark} {:<14} {}", t.binary, if t.installed { t.path.unwrap_or_default() } else { t.install_hint });
            }
        }
        Cmd::Watch { dir, to, out_dir } => {
            let out = out_dir.unwrap_or_else(|| dir.clone());
            // watcher lives in the tauri crate; inline minimal loop here via omni watcher logic
            println!("watching {} -> *.{to} (out: {})", dir.display(), out.display());
            // delegate: dynamic import would need the lib; do a simple notify loop
            watch_loop(&dir, &to, &out).await?;
        }
    }
    Ok(())
}

async fn watch_loop(dir: &PathBuf, to: &str, out: &PathBuf) -> anyhow::Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::time::Duration;
    std::fs::create_dir_all(out)?;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(ev)) => {
                if matches!(ev.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_)) {
                    for p in ev.paths {
                        if p.is_file() {
                            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file").to_string();
                            let dest = out.join(format!("{stem}.{to}"));
                            match omni_core::convert_file(&p, &dest).await {
                                Ok(()) => println!("OK {} -> {}", p.display(), dest.display()),
                                Err(e) => eprintln!("FAILED {}: {e}", p.display()),
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

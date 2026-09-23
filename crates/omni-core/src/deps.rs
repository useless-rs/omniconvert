use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub binary: String,
    pub label: String,
    pub install: String,
    pub handles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub binary: String,
    pub label: String,
    pub installed: bool,
    pub path: Option<String>,
    pub install_hint: String,
}

pub static KNOWN_TOOLS: &[ToolStatic] = &[
    ToolStatic { binary: "ffmpeg", label: "FFmpeg (audio/video)", install: "brew install ffmpeg | apt install ffmpeg | choco install ffmpeg" },
    ToolStatic { binary: "magick", label: "ImageMagick (images)", install: "brew install imagemagick | apt install imagemagick | choco install imagemagick" },
    ToolStatic { binary: "pandoc", label: "Pandoc (documents)", install: "brew install pandoc | apt install pandoc | choco install pandoc" },
    ToolStatic { binary: "libreoffice", label: "LibreOffice (office docs)", install: "brew install --cask libreoffice | apt install libreoffice | choco install libreoffice" },
    ToolStatic { binary: "7z", label: "7-Zip (archives)", install: "brew install p7zip | apt install p7zip-full | choco install 7zip" },
    ToolStatic { binary: "gs", label: "Ghostscript (PDF/PS)", install: "brew install ghostscript | apt install ghostscript | choco install ghostscript" },
    ToolStatic { binary: "inkscape", label: "Inkscape (vector)", install: "brew install --cask inkscape | apt install inkscape | choco install inkscape" },
    ToolStatic { binary: "ebook-convert", label: "Calibre (ebooks)", install: "brew install --cask calibre | apt install calibre | choco install calibre" },
    ToolStatic { binary: "blender", label: "Blender (3D)", install: "brew install --cask blender | apt install blender | choco install blender" },
    ToolStatic { binary: "fontforge", label: "FontForge (fonts)", install: "brew install fontforge | apt install fontforge | choco install fontforge" },
];

pub struct ToolStatic {
    pub binary: &'static str,
    pub label: &'static str,
    pub install: &'static str,
}

pub fn install_hint_for(tool: &str) -> String {
    KNOWN_TOOLS
        .iter()
        .find(|t| t.binary == tool)
        .map(|t| t.install.to_string())
        .unwrap_or_else(|| format!("install '{tool}' via your package manager"))
}

fn is_installed(bin: &str) -> Option<String> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let p = dir.join(bin);
            if p.is_file() {
                return Some(p.to_string_lossy().to_string());
            }
            #[cfg(windows)]
            {
                let exe = dir.join(format!("{bin}.exe"));
                if exe.is_file() {
                    return Some(exe.to_string_lossy().to_string());
                }
            }
            None
        })
    })
}

/// Detect what's installed; gracefully degrade when tools are missing.
pub fn check_all() -> Vec<ToolStatus> {
    KNOWN_TOOLS
        .iter()
        .map(|t| {
            let path = is_installed(t.binary);
            ToolStatus {
                binary: t.binary.into(),
                label: t.label.into(),
                installed: path.is_some(),
                path,
                install_hint: t.install.into(),
            }
        })
        .collect()
}

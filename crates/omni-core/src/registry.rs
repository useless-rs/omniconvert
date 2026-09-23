use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::deps::KNOWN_TOOLS;
use crate::error::ConverterError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub via: String,
    pub needs_tool: Option<String>,
}

#[derive(Debug, Default)]
pub struct ConversionGraph {
    edges: Vec<Edge>,
}

impl ConversionGraph {
    pub fn new() -> Self {
        let mut g = Self { edges: vec![] };
        g.register_native();
        g.register_external();
        g
    }

    fn add(&mut self, from: &str, to: &str, via: &str, needs_tool: Option<&str>) {
        self.edges.push(Edge {
            from: from.into(),
            to: to.into(),
            via: via.into(),
            needs_tool: needs_tool.map(|s| s.into()),
        });
    }

    /// Native (pure-Rust, always available) edges. These actually work end-to-end.
    fn register_native(&mut self) {
        let text = ["txt", "md", "html", "b64", "srt", "vtt", "m3u"];
        for a in text {
            for b in text {
                if a != b {
                    self.add(a, b, "native_text", None);
                }
            }
        }
        for a in ["json", "yaml", "toml", "xml", "csv", "tsv"] {
            for b in ["json", "yaml", "toml", "xml", "csv"] {
                if a != b {
                    self.add(a, b, "native_data", None);
                }
            }
            // data <-> text bridge
            self.add(a, "txt", "native_data", None);
            self.add(a, "md", "native_data", None);
            self.add("txt", a, "native_data", None);
        }
        // identical formats = copy
        for f in crate::formats::all_formats() {
            self.add(&f.id.clone(), &f.id.clone(), "copy", None);
        }
        // zip can be a *target* from anything (bundle single file)
        for f in ["txt", "md", "json", "yaml", "toml", "xml", "csv", "png", "jpg", "bmp"] {
            self.add(f, "zip", "native_archive", None);
        }
        self.add("zip", "txt", "native_archive", None); // list/extract-first info
        // image clique (decode anything `image` reads, encode common)
        for a in ["png", "jpg", "bmp", "ico", "gif", "webp", "tiff"] {
            for b in ["png", "jpg", "bmp", "ico"] {
                if a != b {
                    self.add(a, b, "native_image", None);
                }
            }
        }
        // subtitles
        self.add("srt", "vtt", "native_text", None);
        self.add("vtt", "srt", "native_text", None);
        // spreadsheets, read natively (no LibreOffice needed)
        for a in ["xlsx", "xls", "ods"] {
            for b in ["csv", "json", "yaml", "txt", "md"] {
                self.add(a, b, "native_sheet", None);
            }
        }
    }

    /// External-tool edges. Available only when the tool is installed;
    /// otherwise explain_why_not() produces a clear install message.
    fn register_external(&mut self) {
        let audio = ["mp3", "wav", "flac", "ogg"];
        for a in audio {
            for b in audio {
                if a != b {
                    self.add(a, b, "ffmpeg", Some("ffmpeg"));
                }
            }
        }
        let video = ["mp4", "mkv", "avi", "webm", "mov"];
        for a in video {
            for b in video {
                if a != b {
                    self.add(a, b, "ffmpeg", Some("ffmpeg"));
                }
            }
        }
        // extract audio from video
        for v in video {
            for a in audio {
                self.add(v, a, "ffmpeg", Some("ffmpeg"));
            }
        }
        // images via ImageMagick (broad)
        for a in ["png", "jpg", "gif", "webp", "tiff", "svg", "bmp", "pdf"] {
            for b in ["png", "jpg", "gif", "webp", "tiff", "bmp", "pdf"] {
                if a != b {
                    self.add(a, b, "magick", Some("magick"));
                }
            }
        }
        // documents via pandoc / libreoffice
        for a in ["md", "html", "docx", "epub", "txt", "pdf"] {
            for b in ["md", "html", "docx", "epub", "pdf", "txt"] {
                if a != b {
                    self.add(a, b, "pandoc", Some("pandoc"));
                }
            }
        }
        for a in ["doc", "xls", "xlsx", "pptx", "odt", "pdf", "docx", "html"] {
            for b in ["pdf", "docx", "xlsx", "html", "txt"] {
                if a != b {
                    self.add(a, b, "libreoffice", Some("libreoffice"));
                }
            }
        }
        // archives
        for a in ["zip", "tar", "gz", "7z", "rar"] {
            for b in ["zip", "tar", "gz", "7z"] {
                if a != b {
                    self.add(a, b, "7z", Some("7z"));
                }
            }
        }
        // ebooks
        for a in ["epub", "mobi", "azw3", "pdf", "txt"] {
            for b in ["epub", "mobi", "pdf", "txt"] {
                if a != b {
                    self.add(a, b, "ebook-convert", Some("ebook-convert"));
                }
            }
        }
        // vector/print
        for a in ["svg", "pdf", "eps", "ps"] {
            for b in ["svg", "pdf", "png", "eps"] {
                if a != b {
                    self.add(a, b, "inkscape", Some("inkscape"));
                }
            }
        }
        for a in ["pdf", "ps", "eps"] {
            for b in ["pdf", "ps", "png"] {
                if a != b {
                    self.add(a, b, "ghostscript", Some("gs"));
                }
            }
        }
        // 3d
        for a in ["obj", "fbx", "gltf", "stl", "dae", "ply"] {
            for b in ["obj", "fbx", "gltf", "stl"] {
                if a != b {
                    self.add(a, b, "blender", Some("blender"));
                }
            }
        }
        // fonts
        for a in ["ttf", "otf", "woff", "woff2"] {
            for b in ["ttf", "otf", "woff", "woff2"] {
                if a != b {
                    self.add(a, b, "fontforge", Some("fontforge"));
                }
            }
        }
        // CAD/GIS placeholders (routed to best tool, honest about limits)
        for a in ["dxf", "dwg", "step"] {
            for b in ["obj", "stl", "pdf"] {
                self.add(a, b, "blender", Some("blender"));
            }
        }
    }

    pub fn find_route(&self, from: &str, to: &str) -> Option<&Edge> {
        if from == to {
            return self.edges.iter().find(|e| e.from == from && e.to == to);
        }
        // prefer native edges, then any
        self.edges
            .iter()
            .find(|e| e.from == from && e.to == to && e.needs_tool.is_none())
            .or_else(|| self.edges.iter().find(|e| e.from == from && e.to == to))
    }

    pub fn supported_targets(&self, from: &str) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for e in &self.edges {
            if e.from == from {
                set.insert(e.to.clone());
            }
        }
        let mut v: Vec<String> = set.into_iter().collect();
        v.sort();
        v
    }

    pub fn explain_why_not(&self, from: &str, to: &str) -> ConverterError {
        if let Some(edge) = self.edges.iter().find(|e| e.from == from && e.to == to) {
            if let Some(tool) = &edge.needs_tool {
                let hint = KNOWN_TOOLS
                    .iter()
                    .find(|t| t.binary == *tool)
                    .map(|t| t.install.to_string())
                    .unwrap_or_else(|| format!("install {tool} and retry"));
                return ConverterError::ToolMissing {
                    tool: tool.clone(),
                    install_hint: hint,
                };
            }
        }
        // unknown pair: suggest what's reachable
        let known: HashMap<&str, bool> = HashMap::new();
        let _ = known;
        ConverterError::NotSupported {
            from: from.into(),
            to: to.into(),
            reason: "no adapter registered for this pair yet".into(),
            hint: format!(
                "supported targets from '{from}': [{}]. To add this pair, implement the Adapter trait (see README).",
                self.supported_targets(from).join(", ")
            ),
        }
    }
}

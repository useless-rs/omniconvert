use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    pub id: String,
    pub name: String,
    pub category: String,
    pub extensions: Vec<String>,
    pub mime: String,
    pub description: String,
}

fn f(id: &str, name: &str, cat: &str, exts: &[&str], mime: &str, desc: &str) -> Format {
    Format {
        id: id.into(),
        name: name.into(),
        category: cat.into(),
        extensions: exts.iter().map(|s| s.to_string()).collect(),
        mime: mime.into(),
        description: desc.into(),
    }
}

/// Every format the engine knows about. Designed to grow to "all of them".
pub fn all_formats() -> Vec<Format> {
    vec![
        f("txt", "Plain Text", "document", &["txt"], "text/plain", "UTF-8 plain text"),
        f("md", "Markdown", "document", &["md", "markdown"], "text/markdown", "Markdown text"),
        f("html", "HTML", "document", &["html", "htm"], "text/html", "Hypertext document"),
        f("json", "JSON", "data", &["json"], "application/json", "JSON data"),
        f("yaml", "YAML", "data", &["yaml", "yml"], "application/yaml", "YAML data"),
        f("toml", "TOML", "data", &["toml"], "application/toml", "TOML config"),
        f("xml", "XML", "data", &["xml"], "application/xml", "XML data"),
        f("csv", "CSV", "spreadsheet", &["csv"], "text/csv", "Comma-separated values"),
        f("tsv", "TSV", "spreadsheet", &["tsv"], "text/tab-separated-values", "Tab-separated values"),
        f("png", "PNG Image", "image", &["png"], "image/png", "Portable network graphics"),
        f("jpg", "JPEG Image", "image", &["jpg", "jpeg"], "image/jpeg", "JPEG photo"),
        f("bmp", "BMP Image", "image", &["bmp"], "image/bmp", "Bitmap"),
        f("ico", "ICO Icon", "image", &["ico"], "image/x-icon", "Windows icon"),
        f("gif", "GIF", "image", &["gif"], "image/gif", "Animated image (needs ImageMagick)"),
        f("webp", "WebP", "image", &["webp"], "image/webp", "Web image (needs ImageMagick)"),
        f("tiff", "TIFF", "image", &["tif", "tiff"], "image/tiff", "Tagged image (needs ImageMagick)"),
        f("svg", "SVG Vector", "image", &["svg"], "image/svg+xml", "Vector graphics (needs Inkscape/rsvg)"),
        f("pdf", "PDF", "document", &["pdf"], "application/pdf", "Portable document (needs LibreOffice/Ghostscript)"),
        f("docx", "Word OOXML", "document", &["docx"], "application/vnd.openxmlformats-officedocument.wordprocessingml.document", "Word doc (needs Pandoc/LibreOffice)"),
        f("doc", "Word Legacy", "document", &["doc"], "application/msword", "Legacy Word (needs LibreOffice)"),
        f("xls", "Excel Legacy", "spreadsheet", &["xls"], "application/vnd.ms-excel", "Legacy Excel (needs LibreOffice)"),
        f("xlsx", "Excel OOXML", "spreadsheet", &["xlsx"], "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "Excel workbook (needs LibreOffice)"),
        f("pptx", "PowerPoint", "presentation", &["pptx", "ppt"], "application/vnd.openxmlformats-officedocument.presentationml.presentation", "Slides (needs LibreOffice)"),
        f("epub", "EPUB Ebook", "ebook", &["epub"], "application/epub+zip", "Ebook (needs Pandoc/Calibre)"),
        f("mobi", "MOBI Ebook", "ebook", &["mobi"], "application/x-mobipocket-ebook", "Kindle ebook (needs Calibre)"),
        f("zip", "ZIP Archive", "archive", &["zip"], "application/zip", "Zip archive"),
        f("tar", "TAR Archive", "archive", &["tar"], "application/x-tar", "Tape archive (needs 7-Zip)"),
        f("gz", "GZIP", "archive", &["gz", "tgz"], "application/gzip", "Gzip (needs 7-Zip)"),
        f("7z", "7-Zip", "archive", &["7z"], "application/x-7z-compressed", "7-Zip archive (needs 7-Zip)"),
        f("mp3", "MP3 Audio", "audio", &["mp3"], "audio/mpeg", "MP3 audio (needs FFmpeg)"),
        f("wav", "WAV Audio", "audio", &["wav"], "audio/wav", "Waveform audio (needs FFmpeg)"),
        f("flac", "FLAC Audio", "audio", &["flac"], "audio/flac", "Lossless audio (needs FFmpeg)"),
        f("ogg", "OGG Audio", "audio", &["ogg", "oga"], "audio/ogg", "Ogg audio (needs FFmpeg)"),
        f("mp4", "MP4 Video", "video", &["mp4", "m4v"], "video/mp4", "MP4 video (needs FFmpeg)"),
        f("mkv", "MKV Video", "video", &["mkv"], "video/x-matroska", "Matroska video (needs FFmpeg)"),
        f("avi", "AVI Video", "video", &["avi"], "video/x-msvideo", "AVI video (needs FFmpeg)"),
        f("webm", "WebM Video", "video", &["webm"], "video/webm", "Web video (needs FFmpeg)"),
        f("mov", "QuickTime", "video", &["mov"], "video/quicktime", "QuickTime (needs FFmpeg)"),
        f("srt", "SubRip Subtitles", "subtitle", &["srt"], "application/x-subrip", "Subtitles (native)"),
        f("vtt", "WebVTT Subtitles", "subtitle", &["vtt"], "text/vtt", "Web subtitles (native)"),
        f("m3u", "M3U Playlist", "playlist", &["m3u", "m3u8"], "audio/x-mpegurl", "Playlist (native)"),
        f("obj", "Wavefront OBJ", "3d", &["obj"], "model/obj", "3D model (needs Blender)"),
        f("fbx", "FBX Model", "3d", &["fbx"], "application/octet-stream", "Filmbox 3D (needs Blender)"),
        f("gltf", "glTF Model", "3d", &["gltf", "glb"], "model/gltf+json", "GL transmit (needs Blender)"),
        f("stl", "STL Model", "3d", &["stl"], "model/stl", "Stereolithography (needs Blender)"),
        f("ttf", "TrueType Font", "font", &["ttf"], "font/ttf", "Font (needs FontForge)"),
        f("otf", "OpenType Font", "font", &["otf"], "font/otf", "Font (needs FontForge)"),
        f("woff", "WOFF Font", "font", &["woff"], "font/woff", "Web font (needs FontForge)"),
        f("woff2", "WOFF2 Font", "font", &["woff2"], "font/woff2", "Web font v2 (needs FontForge)"),
        f("b64", "Base64 Text", "data", &["b64", "base64"], "text/plain", "Base64-encoded blob (native)"),
        f("bin", "Raw Binary", "binary", &["bin", "dat"], "application/octet-stream", "Opaque bytes (copy-only)"),
    ]
}

pub fn find_by_extension(ext: &str) -> Option<Format> {
    let e = ext.to_lowercase();
    all_formats()
        .into_iter()
        .find(|x| x.extensions.iter().any(|x| x == &e) || x.id == e)
}

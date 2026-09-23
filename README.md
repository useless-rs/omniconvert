# OmniConvert — The last file converter you'll ever need. EVERYTHING → EVERYTHING.

![OmniConvert logo](brand/logo.svg)

Drop any file. Get any file. Audio, video, images, documents, spreadsheets, presentations, archives, 3D models, fonts, ebooks, data, subtitles, playlists, CAD/GIS, code, binaries — if it exists, it converts.

## Why

Converters today are islands: one for video, one for docs, one for images. OmniConvert is a **universal conversion graph**: every format is a node, every adapter is an edge. Add a pair without touching the core.

## Features

- **Universal graph** — 50+ formats registered day one, designed to grow to all of them
- **Content detection** — magic bytes first (`infer` + image sniff + text heuristics), extension only as fallback
- **Plugin adapters** — `Adapter` trait; native Rust fast-paths + external best-in-class tools
- **External tools** — FFmpeg, ImageMagick, Pandoc, LibreOffice, 7-Zip, Ghostscript, Inkscape, Calibre, Blender, FontForge; auto-detected, graceful degrade with install hints
- **Queue** — parallel conversions, progress, cancel, retry
- **Three modes** — GUI (Tauri v2), CLI (`omni`), watch-folder daemon
- **Presets** — builtin + custom (localStorage + JSON file)
- **Cross-platform** — Windows, macOS, Linux

```
Frontend (Vite+TS) ⇄ Tauri commands ⇄ OmniEngine ⇄ Registry + Adapters + Queue + Deps
                                          ├─ native_text (txt/md/html/b64/srt/vtt + json/yaml/toml/xml/csv)
                                          ├─ native_image (png/jpg/bmp/ico via `image`, rayon-parallel)
                                          ├─ native_archive (zip bundle/list via `zip`)
                                          └─ external (ffmpeg/magick/pandoc/libreoffice/7z/gs/inkscape/calibre/blender)
```

## Works out of the box (no external tools)

| From | To | How |
|---|---|---|
| txt, md, html, srt, vtt, m3u | each other | native_text |
| json, yaml, toml, xml, csv, tsv | each other + txt/md | native_data |
| png, jpg, bmp, ico (+ decode gif/webp/tiff) | png/jpg/bmp/ico | native_image |
| anything | zip | native_archive (bundle) |
| same → same | same | copy |

Everything else routes to external tools when installed, else a clear error: `external tool 'ffmpeg' not found. Install: brew install ffmpeg | ...`.

## Quickstart

Prereqs: Rust 1.85+, Node 20+.

```bash
npm install
npm run build        # frontend → dist/
cargo check -p omni-core
cargo run -p omniconvert --bin omni -- detect ./file.bin
cargo run -p omniconvert --bin omni -- convert in.csv out.json
cargo run -p omniconvert --bin omni -- tools
cargo run -p omniconvert --bin omni -- batch --to json a.csv b.yaml --out-dir ./out
cargo run -p omniconvert --bin omni -- watch ./inbox --to pdf --out-dir ./out
npm run tauri dev    # GUI (needs Tauri system deps)
```

External tools (optional, unlock more pairs):

| Tool | macOS | Debian/Ubuntu | Windows |
|---|---|---|---|
| ffmpeg | `brew install ffmpeg` | `apt install ffmpeg` | `choco install ffmpeg` |
| magick | `brew install imagemagick` | `apt install imagemagick` | `choco install imagemagick` |
| pandoc | `brew install pandoc` | `apt install pandoc` | `choco install pandoc` |
| libreoffice | `brew install --cask libreoffice` | `apt install libreoffice` | `choco install libreoffice` |
| 7z | `brew install p7zip` | `apt install p7zip-full` | `choco install 7zip` |

`scripts/check-deps.sh` reports what's present; `scripts/install-deps.sh` prints all install lines.

## Add a new format pair (no core changes)

```rust
// crates/omni-core/src/adapters/my_format.rs
pub async fn convert(from: &str, to: &str, input: &Path, output: &Path) -> Result<()> { /* ... */ }
// registry.rs: self.add("dwg", "obj", "my_tool", Some("my_tool"));
// deps.rs: register binary + install hint. Done — engine, CLI, GUI pick it up.
```

## Project layout

```
omniconvert/
  Cargo.toml                 # workspace
  package.json / vite.config.ts / index.html / src/   # frontend (vanilla TS, snappy)
  crates/omni-core/src/      # engine: detect, formats, registry, adapters/*, queue, deps, presets
  src-tauri/                 # Tauri v2 shell + CLI (omni) + watcher
  scripts/                   # check-deps.sh, install-deps.sh
```

## Roadmap to universality

- [x] Native text/data/image/archive graph + magic-bytes detect + queue + CLI + watch
- [ ] Streaming progress for ffmpeg (parse stderr), real % bars
- [ ] More native Rust: `calamine`/`rust-xlsxwriter`, `docx-rs`, `printpdf`, `lofty` (audio tags)
- [ ] WASM preview pane, OCR (`ocrs`), GIS (`gdal`), CAD kernels

## Contributing / License

PRs welcome — especially new adapters. MIT (see LICENSE).

---

## Brand (MVP brand sprint, cycle 1)

**Positioning:** OmniConvert is the universal file converter for power users
that routes every format through one conversion graph — unlike single-purpose
converters that each cover one island. Precise, fast, honest; a peer, not a pitch.

**Mark:** twin exchange arrows (⇄) forming an "O" for Omni. No globes, hexagons,
or gradient spheres. Assets in `brand/`: `logo.svg` (lockup), `mark.svg`
(icon-only), `logo-mono.svg` (single-color), `favicon.svg` (16px-optimized),
`tokens.css` (dark default + light mapping — the single source of truth).

**Palette:** one accent `#7C5CFF` (violet), gradient partner `#22D3EE` (cyan),
on dark `#0B0E17/#12172A`, text `#E8ECFF`, muted `#93A0C4`. Functional:
`#34D399/#F87171/#FBBF24`. Type: Inter/system-ui + ui-monospace for code.

**Voice:** explain, don't sell. Errors state what + why + exact fix.
Banned: revolutionary, seamless, next-generation, disrupt.

**Don'ts:** no stretch/recolor/shadows/rotation; clear space = half mark height;
gradient only in mark, hero headline, progress bars.

## Backlog (persistent — score = impact × (6−effort) − risk)

- [16] #1 MVP brand system — DONE (cycle 1)
- [16] #12 Round-trip property tests (csv→json→csv, png→jpg→png)
- [12] #4 OG card 1200×630 + PWA icon ladder from SVG
- [12] #5 Refine `icons/icon.png` from official mark (placeholder now)
- [12] #10 Light-theme QA pass (contrast ≥ 4.5:1)
- [12] #11 `omni doctor` command (deps + PATH + versions)
- [11] #3 Streaming ffmpeg progress (% bars from stderr)
- [11] #6 Native xlsx (calamine + rust-xlsxwriter)
- [8] #7 Watch-mode tray + notifications
- [8] #9 Subtitle shift/resync preset options
- [5] #8 OCR input (`ocrs`)
- Parking: GDAL/GIS, CAD kernels, ebook covers, waveform preview, WASM plugins.

## Changelog

- **Unreleased (cycle 1):** MVP brand system (`brand/` + this section).
- **0.1.0 — 2026-09-23:** universal engine (detect/graph/native adapters/queue/
  deps/presets), Tauri v2 shell + `omni` CLI + watch mode, TS frontend.

## Improvement log

- **Cycle 1 — 2026-09-23 (branding):** web research → dev-tool brands win on
  credibility (substance, product-surface voice); logos must be favicon-first,
  mono-capable, SVG-mastered; colors as tokens with dark-mode parity. Delivered
  `brand/` set above. Next: E2E proof commit, then #3 or #12.

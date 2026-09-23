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

## Brand (Anthropic styling, cycle 8)

**Positioning:** OmniConvert is the universal file converter for power users
that routes every format through one conversion graph — unlike single-purpose
converters that each cover one island. Precise, fast, honest; a peer, not a pitch.

**Mark:** twin exchange arrows (⇄) forming an "O" for Omni. No globes, hexagons,
or gradient spheres. Assets in `brand/`: `logo.svg` (lockup), `mark.svg`
(icon-only), `logo-mono.svg` (single-color), `favicon.svg` (16px-optimized),
`tokens.css` (dark default + light mapping — the single source of truth).

**Palette (Anthropic):** Dark `#141413`, Light `#FAF9F5`, Mid Gray `#B0AEA5`,
Light Gray `#E8E6DC`. Accents: Orange `#D97757` (primary), Blue `#6A9BCC`,
Green `#788C5D`. CTA: solid orange, dark text (5.90:1). Borders Mid Gray
(7.15:1 dark) / `#8F8D84` light (3.33:1). Light muted `#6E6C64` (4.99:1).
Type: Poppins (Arial fallback) display, Lora (Georgia fallback) body,
ui-monospace for code. Full audit: cycle 5 + 8 logs below.

**Voice:** explain, don't sell. Errors state what + why + exact fix.
Banned: revolutionary, seamless, next-generation, disrupt.

**Don'ts:** no stretch/recolor/shadows/rotation; clear space = half mark height;
gradient only in mark, hero headline, progress bars.

## Backlog (persistent — score = impact × (6−effort) − risk)

- [16] #1 MVP brand system — DONE (cycle 1)
- [16] #12 Round-trip property tests (csv→json→csv, png→jpg→png) — DONE (cycle 2)
- [12] #11 `omni doctor` command (deps + PATH + versions) — DONE (cycle 3)
- [12] #5 Refine `icons/icon.png` from official mark (placeholder now) — DONE (cycle 4)
- [12] #10 Light-theme QA pass (contrast ≥ 4.5:1) — DONE (cycle 5)
- [12] #4 OG card 1200×630 + PWA icon ladder from SVG — card DONE (cycle 6)
- [11] #3 Streaming ffmpeg progress (% bars from stderr) — DONE (cycle 7)
- [11] #6 Native xlsx (calamine + rust-xlsxwriter) — read DONE (cycle 9)
- Native archives (zip/tar/gz/7z, zero external tools) — DONE (cycle 10)
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

- **Unreleased (cycle 10):** native archives — tar/gzip/7z join zip
  (`tar` + `flate2` + `sevenz-rust2`). 7-Zip tool now needed only for RAR.
- **Unreleased (cycle 9):** native spreadsheet reading (calamine):
  xlsx/xls/ods → csv/json/yaml/txt/md, no LibreOffice; container-aware
  detection (OOXML ≠ zip).
- **Unreleased (cycle 8):** Anthropic rebrand (palette, Poppins/Lora stacks,
  recolored logo set, icons, OG card, CTA now solid orange).
- **Unreleased (cycle 7):** live ffmpeg progress (`omni: 42% (00:01:12/…)`
- **Unreleased (cycle 6):** OG social card (`public/og-card.png`, meta tags).
- **Unreleased (cycle 5):** contrast audit fixes (action violet, visible
  input borders, light muted darkened — all pairs now pass).
- **Unreleased (cycle 4):** real icon set from the brand mark (32/128/256/512
  RGBA PNGs via ImageMagick, `bundle.icon` wired, browser favicon).
- **Unreleased (cycle 3):** `omni doctor` (tool versions, PATH sanity,
  writability, engine self-test, exit 1 on issues); wired dead `watcher`
  code (`watch_start` Tauri command, CLI shares `convert_dropped`).
- **Unreleased (cycle 2):** 8 round-trip integration tests
  (`crates/omni-core/tests/roundtrip.rs`, isolated tempdirs, no external tools);
  untracked generated `src-tauri/gen/` schemas (gitignored).
- **Unreleased (cycle 1):** MVP brand system (`brand/` + this section).
- **0.1.0 — 2026-09-23:** universal engine (detect/graph/native adapters/queue/
  deps/presets), Tauri v2 shell + `omni` CLI + watch mode, TS frontend.

## Improvement log

- **Cycle 10 — 2026-09-23 (native archives):** per the native-first directive,
  replaced the 7-Zip dependency with pure-Rust crates (`tar`, `flate2`,
  `sevenz-rust2` maintained fork). `native_archive` rewritten around
  extract→stage→pack: bundle/list/repack across zip/tar/gz/7z, gzip single-
  stream semantics, zip-slip-safe entries. Registry: native clique, RAR stays
  external-only (proprietary write — stated honestly). 19 tests green
  (4 new archive tests). Next: audio→wav via symphonia, or PDF via
  printpdf/pdf-extract.
- **Cycle 9 — 2026-09-23 (native sheets):** research → calamine is the pure-
  Rust standard (xls/xlsx/xlsm/xlsb/ods). New `native_sheet` adapter (first
  sheet → csv/json/yaml/txt/md via shared `convert_value`), registry edges,
  engine route. Fixed a real detection bug en route: OOXML files are ZIPs, so
  `infer` reported every xlsx as `zip` — now zip magic + known extension +
  container probe (`[Content_Types].xml`) resolves xlsx/docx/pptx/odt/epub.
  Fixture `tests/fixtures/sample.xlsx` built from stdlib python (hermetic).
  15 tests green. Next: xlsx *writing*, #7 tray, or #8 OCR.
- **Cycle 8 — 2026-09-23 (Anthropic rebrand):** applied the brand-guidelines
  skill across every surface: warm neutrals, Coral/Blue/Green accents, Poppins
  display + Lora body (Liberation fallbacks where the fonts aren't installed).
  Recolored logo set, icons, OG card; CTA is now solid orange with dark text
  (white-on-orange fails at 3.12:1); all token pairs re-audited and passing.
  Frontend rebuilt. Uncursed.
- **Cycle 7 — 2026-09-23 (live progress):** ffmpeg jobs ran silent. Added
  `adapters/ffmpeg_progress.rs` (pure `-progress`/`Duration:` parser, 4 unit
  tests) + streaming runner (`-progress pipe:1`, 3-field percent lines on
  stderr, tail of stderr on failure). Verified live: mp4→mkv printed
  `omni: 98% (00:00:05/00:00:06)` → `100%` → OK. All 12 tests green.
  Next: rebrand per Anthropic guidelines (user request).
- **Cycle 6 — 2026-09-23 (social preview):** research → 1200×630, split
  layout, ≤60-char headline, 80px safe margins, PNG sRGB <1MB, width/height/
  alt + `summary_large_image` tags. Built with ImageMagick from brand assets;
  caught real defects by visual inspection (density-scaled 300px type, opaque
  SVG raster, overflowing copy) and fixed all three. Verified in `dist/`.
  Note: `og:image` needs an absolute URL on deploy. Next: #3 ffmpeg progress
  or #6 native xlsx.
- **Cycle 5 — 2026-09-23 (contrast QA):** computed WCAG ratios for all 17
  token pairs (4.5:1 text, 3:1 large/UI). Found 4 real failures: input borders
  1.23–1.41:1 (must identify components per 1.4.11), CTA white-on-violet 4.35,
  light muted 4.45 (unrounded → fails). Fixed: `--oc-line` → `#5A6694` /
  `#6B87A3`, new `--oc-action #6547F0` for text-bearing surfaces, light muted
  → `#55677E`. Ratios now recorded as comments in `tokens.css` so the values
  can't regress silently. `ltxt/pri` 4.11 documented as a forbidden non-pair.
  Frontend rebuilt. Next: #3 ffmpeg progress or #6 native xlsx.
- **Cycle 4 — 2026-09-23 (app icon):** research → match `tauri icon` default
  output (32/128/128@2x/icon.png, square RGBA 32bpp); `.icns/.ico` need
  platform tooling (still open). Rendered from `brand/favicon.svg`
  (tile-based, favicon-first per cycle-1 principles); caught + fixed a lost
  alpha channel on the 32px pass via `identify` verification. Browser tab
  covered (`public/favicon.png` + link). Verified: `cargo check` (codegen
  accepts set), `npm run build` (favicon in `dist/`). Left: `.icns/.ico`,
  OG card (#4 remains).
- **Cycle 3 — 2026-09-23 (diagnosability):** research → doctor commands earn
  trust via per-check status + versions + actionable fixes + honest summary
  (brew/flutter pattern). `omni doctor` checks 10 tools (3s-timeout version
  probes), PATH dangling entries, temp writability, live csv→json self-test;
  exits 1 with issue count. Also fixed 2 dead-code warnings by wiring
  `watcher` into both shells. Verified: `cargo check --all-targets` clean,
  doctor output exact on this machine (4 ok, 6 miss as expected). Next: #3
  streaming ffmpeg progress or #4 OG/PWA assets.
- **Cycle 2 — 2026-09-23 (correctness net):** research → example-based
  integration tests in `tests/` with `tempfile` isolation beat proptest here
  (deterministic, fast, no new harness; proptest deferred to backlog). 8 tests:
  csv/json/yaml/toml/image/zip/subtitle round-trips, wrong-extension detection,
  actionable-error quality gate. Found nothing broken (TOML root-table bug was
  caught in cycle-0 E2E). `cargo test -p omni-core`: 8 passed. Next: #3
  streaming ffmpeg progress or #11 `omni doctor`.
- **Cycle 1 — 2026-09-23 (branding):** web research → dev-tool brands win on
  credibility (substance, product-surface voice); logos must be favicon-first,
  mono-capable, SVG-mastered; colors as tokens with dark-mode parity. Delivered
  `brand/` set above. Next: E2E proof commit, then #3 or #12.

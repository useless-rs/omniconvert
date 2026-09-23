//! Round-trip integration tests: the engine's core promise is that data
//! survives EVERYTHING -> EVERYTHING conversion. Each test converts forward
//! and back through the public API and asserts the invariant that matters.
//!
//! Isolation: every test gets its own `tempfile::TempDir` (parallel-safe,
//! no shared state). No external tools required — native adapters only.

use std::path::{Path, PathBuf};

use omni_core::{convert_file, detect};
use tempfile::TempDir;

fn dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.path().join(name);
    std::fs::write(&p, bytes).expect("write fixture");
    p
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).expect("read output")
}

#[tokio::test]
async fn csv_to_json_to_csv_preserves_rows() {
    let d = dir();
    let csv = write(&d, "in.csv", b"name,age\nada,36\ngrace,85\n");
    let json = d.path().join("mid.json");
    let back = d.path().join("out.csv");
    convert_file(&csv, &json).await.expect("csv->json");
    convert_file(&json, &back).await.expect("json->csv");
    let mut rdr = csv::Reader::from_path(&back).expect("parse back");
    let headers = rdr.headers().expect("headers").clone();
    let rows: Vec<_> = rdr.records().map(|r| r.expect("record")).collect();
    assert_eq!(rows.len(), 2, "both rows survive the round trip");
    let name_col = headers.iter().position(|h| h == "name").expect("name col");
    let age_col = headers.iter().position(|h| h == "age").expect("age col");
    assert_eq!(&rows[0][name_col], "ada");
    assert_eq!(&rows[1][age_col], "85");
}

#[tokio::test]
async fn json_to_yaml_to_json_preserves_value() {
    let d = dir();
    let src = write(&d, "in.json", br#"{"name":"ada","tags":["a","b"],"n":36}"#);
    let yaml = d.path().join("mid.yaml");
    let back = d.path().join("out.json");
    convert_file(&src, &yaml).await.expect("json->yaml");    convert_file(&yaml, &back).await.expect("yaml->json");
    let v_src: serde_json::Value = serde_json::from_str(&read(&src)).unwrap();
    let v_back: serde_json::Value = serde_json::from_str(&read(&back)).unwrap();
    assert_eq!(v_src, v_back, "value identical after yaml round trip");
}

#[tokio::test]
async fn json_object_to_toml_to_json_preserves_value() {
    let d = dir();
    let src = write(&d, "in.json", br#"{"title":"doc","n":7}"#);
    let toml = d.path().join("mid.toml");
    let back = d.path().join("out.json");
    convert_file(&src, &toml).await.expect("json->toml");
    convert_file(&toml, &back).await.expect("toml->json");
    let v_src: serde_json::Value = serde_json::from_str(&read(&src)).unwrap();
    let v_back: serde_json::Value = serde_json::from_str(&read(&back)).unwrap();
    assert_eq!(v_src, v_back, "object survives toml (tables map 1:1)");
}

#[tokio::test]
async fn png_to_jpg_to_png_preserves_dimensions() {
    let d = dir();
    // 16x12 solid image written with the same `image` crate the adapter uses.
    let img = image::RgbImage::from_pixel(16, 12, image::Rgb([200u8, 100, 50]));
    let src = d.path().join("in.png");
    img.save(&src).expect("write png");
    let jpg = d.path().join("mid.jpg");
    let back = d.path().join("out.png");
    convert_file(&src, &jpg).await.expect("png->jpg");
    assert!(jpg.exists() && std::fs::metadata(&jpg).unwrap().len() > 0);
    convert_file(&jpg, &back).await.expect("jpg->png");
    let rt = image::open(&back).expect("open round-tripped png");
    assert_eq!((rt.width(), rt.height()), (16, 12));
}

#[tokio::test]
async fn txt_to_zip_lists_original_name() {
    let d = dir();
    let src = write(&d, "hello.txt", b"hello omniconvert\n");
    let zip = d.path().join("out.zip");
    convert_file(&src, &zip).await.expect("txt->zip");
    let listing = d.path().join("listing.txt");
    convert_file(&zip, &listing).await.expect("zip->txt listing");
    let text = read(&listing);
    assert!(text.contains("hello.txt"), "listing names the bundled file: {text}");
}

#[tokio::test]
async fn srt_vtt_round_trip_keeps_cues() {
    let d = dir();
    let src = write(&d, "in.srt", b"1\n00:00:00,000 --> 00:00:01,000\nHi\n");
    let vtt = d.path().join("mid.vtt");
    let back = d.path().join("out.srt");
    convert_file(&src, &vtt).await.expect("srt->vtt");
    assert!(read(&vtt).starts_with("WEBVTT"));
    convert_file(&vtt, &back).await.expect("vtt->srt");
    assert!(read(&back).contains("Hi"), "cue text survives");
}

#[tokio::test]
async fn detect_ignores_wrong_extension_reads_content() {
    let d = dir();
    // JSON content wearing a .bin disguise must still be detected as json.
    let p = write(&d, "mystery.bin", br#"{"real":"json"}"#);
    let det = detect(&p).expect("detect");
    assert_eq!(det.format_id, "json", "content beats extension");
    // ...and it must actually convert.
    let out = d.path().join("out.yaml");
    convert_file(&p, &out).await.expect("bin-disguised json converts");
    assert!(read(&out).contains("real"));
}

#[tokio::test]
async fn impossible_pair_fails_with_actionable_error() {
    let d = dir();
    let src = write(&d, "in.png", b"not a real png, but extension routes it");
    let err = convert_file(&src, &d.path().join("out.mp3"))
        .await
        .expect_err("png->mp3 must fail without ffmpeg graph edge");
    let msg = err.to_string();
    assert!(msg.len() > 20, "error explains itself: {msg}");
    assert!(
        msg.contains("ffmpeg") || msg.contains("not supported"),
        "error names the missing tool or the gap: {msg}"
    );
}

fn fixture_xlsx(dir: &TempDir) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.xlsx");
    let dst = dir.path().join("sample.xlsx");
    std::fs::copy(&src, &dst).expect("copy fixture");
    dst
}

#[tokio::test]
async fn xlsx_detected_by_container_not_as_zip() {
    let d = dir();
    let xlsx = fixture_xlsx(&d);
    let det = detect(&xlsx).expect("detect");
    assert_eq!(det.format_id, "xlsx", "zip container resolved via extension+probe, got {}", det.format_id);
}

#[tokio::test]
async fn xlsx_to_csv_reads_first_sheet() {
    let d = dir();
    let xlsx = fixture_xlsx(&d);
    let out = d.path().join("out.csv");
    convert_file(&xlsx, &out).await.expect("xlsx->csv");
    let text = read(&out);
    assert!(text.contains("ada") && text.contains("grace"), "rows present: {text}");
    assert!(text.contains("name"), "header present: {text}");
}

#[tokio::test]
async fn xlsx_to_json_maps_header_to_values() {
    let d = dir();
    let xlsx = fixture_xlsx(&d);
    let out = d.path().join("out.json");
    convert_file(&xlsx, &out).await.expect("xlsx->json");
    let v: serde_json::Value = serde_json::from_str(&read(&out)).expect("valid json");
    let rows = v.as_array().expect("array of objects");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["name"], serde_json::Value::String("ada".into()));
}

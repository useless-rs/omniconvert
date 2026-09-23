export interface PresetDef { name: string; from: string; to: string; options: Record<string, unknown>; }

export const BUILTIN: PresetDef[] = [
  { name: "Audio MP3 320k", from: "wav", to: "mp3", options: {} },
  { name: "Video H264 1080p", from: "mkv", to: "mp4", options: {} },
  { name: "Image → PNG", from: "jpg", to: "png", options: {} },
  { name: "Doc → PDF", from: "docx", to: "pdf", options: {} },
  { name: "CSV → JSON", from: "csv", to: "json", options: {} },
  { name: "Bundle ZIP", from: "txt", to: "zip", options: {} },
];

const KEY = "omniconvert.presets.v1";

export function loadPresets(): PresetDef[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return [...BUILTIN];
    return [...BUILTIN, ...(JSON.parse(raw) as PresetDef[])];
  } catch {
    return [...BUILTIN];
  }
}

export function saveCustom(p: PresetDef): void {
  try {
    const raw = localStorage.getItem(KEY);
    const arr: PresetDef[] = raw ? JSON.parse(raw) : [];
    arr.push(p);
    localStorage.setItem(KEY, JSON.stringify(arr));
  } catch { /* ignore */ }
}

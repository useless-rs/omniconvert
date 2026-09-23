export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
declare global {
  interface Window { __TAURI__?: unknown; }
}

async function invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  if (!isTauri()) throw new Error("not-in-tauri");
  const mod = await import("@tauri-apps/api/core");
  return (mod.invoke as InvokeFn)(cmd, args);
}

export interface Detected { format_id: string; mime: string; confidence: number; method: string; }
export interface ToolStatus { binary: string; label: string; installed: boolean; path?: string; install_hint: string; }
export interface Preset { name: string; from: string; to: string; options: unknown; }

export const api = {
  async detectFile(path: string): Promise<Detected> {
    return (await invoke("detect_file", { path })) as Detected;
  },
  async listTargets(from: string): Promise<string[]> {
    return (await invoke("list_targets", { from })) as string[];
  },
  async convertSingle(input: string, output: string): Promise<string> {
    return (await invoke("convert_single", { input, output })) as string;
  },
  async toolStatus(): Promise<ToolStatus[]> {
    return (await invoke("get_tool_status")) as ToolStatus[];
  },
  async presets(): Promise<Preset[]> {
    return (await invoke("get_presets")) as Preset[];
  },
};

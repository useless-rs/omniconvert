import { api, isTauri } from "./tauri.js";
import { loadPresets } from "./presets.js";
import { makeJob, type Job } from "./queue.js";

const jobs: Job[] = [];
let logEl: HTMLElement;

function log(msg: string): void {
  logEl.textContent += msg + "\n";
  logEl.scrollTop = logEl.scrollHeight;
}

function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1).toLowerCase() : "bin";
}

async function refreshTargets(sel: HTMLSelectElement, fromFile: string): Promise<void> {
  const from = extOf(fromFile);
  sel.innerHTML = "";
  try {
    const targets = isTauri() ? await api.listTargets(from) : ["md", "txt", "html", "json", "yaml", "png", "jpg", "zip"];
    for (const t of targets) {
      const o = document.createElement("option");
      o.value = t; o.textContent = t;
      sel.appendChild(o);
    }
  } catch (e) {
    log(`targets error: ${String(e)}`);
  }
}

function renderJobs(tbody: HTMLElement): void {
  tbody.innerHTML = "";
  for (const j of jobs) {
    const tr = document.createElement("tr");
    const fname = (j.file as File).name ?? (j.file as { name: string }).name;
    tr.innerHTML = `<td>${fname}</td><td>${j.target}</td>
      <td><div class="prog"><div style="width:${Math.round(j.progress * 100)}%"></div></div></td>
      <td><span class="pill ${j.state}">${j.state}</span></td>
      <td>${j.message}</td>`;
    const act = document.createElement("td");
    if (j.state === "queued" || j.state === "running") {
      const b = document.createElement("button");
      b.textContent = "Cancel";
      b.onclick = () => { j.cancelled = true; j.state = "cancelled"; j.message = "cancelled by user"; renderJobs(tbody); };
      act.appendChild(b);
    } else if (j.state === "failed" || j.state === "cancelled") {
      const b = document.createElement("button");
      b.textContent = "Retry";
      b.onclick = () => { j.state = "queued"; j.message = ""; j.cancelled = false; j.progress = 0; renderJobs(tbody); void runQueue(tbody); };
      act.appendChild(b);
    }
    tr.appendChild(act);
    tbody.appendChild(tr);
  }
}

async function runQueue(tbody: HTMLElement): Promise<void> {
  const PARALLEL = 4;
  const pending = jobs.filter((j) => j.state === "queued");
  const workers = Array.from({ length: Math.min(PARALLEL, pending.length) }, async () => {
    while (true) {
      const job = pending.find((j) => j.state === "queued");
      if (!job) return;
      job.state = "running";
      renderJobs(tbody);
      try {
        if (isTauri()) {
          // In Tauri, backend resolves real paths; browser File objects need save-dialog flow.
          const path = (job.file as { path?: string }).path ?? (job.file as File).name;
          const out = path.replace(/\.[^.]+$/, "") + "." + job.target;
          job.progress = 0.5; renderJobs(tbody);
          await api.convertSingle(path, out);
          job.progress = 1; job.state = "done"; job.message = `→ ${out}`;
          log(`OK ${path} → ${out}`);
        } else {
          // Demo mode: fake progress, real FileReader sniff
          for (let p = 0; p <= 10; p++) {
            if (job.cancelled) break;
            job.progress = p / 10;
            renderJobs(tbody);
            await new Promise((r) => setTimeout(r, 60));
          }
          if (job.cancelled) { job.state = "cancelled"; job.message = "cancelled"; }
          else { job.state = "done"; job.message = "demo mode — run in Tauri/CLI for real conversion"; }
          log(`demo: ${(job.file as File).name ?? "file"} → *.${job.target}`);
        }
      } catch (e) {
        job.state = "failed";
        job.message = String(e);
        log(`FAILED: ${job.message}`);
      }
      renderJobs(tbody);
    }
  });
  await Promise.all(workers);
}

export function mount(root: HTMLElement): void {
  root.innerHTML = `
    <div class="hero">
      <h1>OmniConvert <span class="grad">EVERYTHING → EVERYTHING</span></h1>
      <p>The last file converter you'll ever need. Drop files, pick a target, convert.</p>
      <div class="chips"><span class="pill">Audio</span><span class="pill">Video</span><span class="pill">Image</span><span class="pill">Docs</span><span class="pill">Archives</span><span class="pill">3D</span><span class="pill">Fonts</span><span class="pill">Ebooks</span><span class="pill">Data</span></div>
    </div>
    <div class="bar">
      <select id="preset"></select>
      <select id="target"></select>
      <button id="add" class="primary">Convert queue</button>
      <button id="pick">Browse…</button>
    </div>
    <div class="drop" id="drop">Drag &amp; drop files here (batch supported)<br/><small>Detected by content (magic bytes), not just extension</small></div>
    <table><thead><tr><th>File</th><th>Target</th><th>Progress</th><th>State</th><th>Info</th><th></th></tr></thead><tbody id="rows"></tbody></table>
    <div class="log" id="log"></div>
    <h3>External tools</h3>
    <div class="tools" id="tools"></div>`;
  logEl = root.querySelector("#log")!;
  const drop = root.querySelector("#drop") as HTMLElement;
  const rows = root.querySelector("#rows") as HTMLElement;
  const target = root.querySelector("#target") as HTMLSelectElement;
  const presetSel = root.querySelector("#preset") as HTMLSelectElement;

  for (const p of loadPresets()) {
    const o = document.createElement("option");
    o.value = p.to; o.textContent = `${p.name} (→${p.to})`;
    presetSel.appendChild(o);
  }
  presetSel.onchange = () => { target.value = presetSel.value; };
  void refreshTargets(target, "file.txt");

  const addFiles = (files: FileList | File[]) => {
    for (const f of Array.from(files)) {
      jobs.push(makeJob(f, target.value));
      log(`queued: ${f.name} → *.${target.value}`);
    }
    renderJobs(rows);
    void refreshTargets(target, (jobs[0]?.file as File)?.name ?? "file.txt");
  };
  drop.ondragover = (e) => { e.preventDefault(); drop.classList.add("over"); };
  drop.ondragleave = () => drop.classList.remove("over");
  drop.ondrop = (e) => { e.preventDefault(); drop.classList.remove("over"); if (e.dataTransfer?.files) addFiles(e.dataTransfer.files); };
  (root.querySelector("#pick") as HTMLButtonElement).onclick = async () => {
    if (!isTauri()) { log("browser demo mode: use drag & drop; real paths need Tauri or CLI."); return; }
    const { open } = await import("@tauri-apps/plugin-dialog");
    const sel = await open({ multiple: true });
    if (sel) {
      for (const p of Array.isArray(sel) ? sel : [sel]) {
        jobs.push(makeJob({ name: String(p).split("/").pop() ?? String(p), path: String(p) }, target.value));
      }
      renderJobs(rows);
    }
  };
  (root.querySelector("#add") as HTMLButtonElement).onclick = () => void runQueue(rows);

  (async () => {
    const box = root.querySelector("#tools") as HTMLElement;
    try {
      const tools = isTauri() ? await api.toolStatus() : [];
      box.innerHTML = tools.length
        ? tools.map((t) => `<div class="tool"><span class="dot ${t.installed ? "ok" : "miss"}"></span><b>${t.binary}</b> — ${t.label}<br/><small>${t.installed ? t.path : "missing: " + t.install_hint}</small></div>`).join("")
        : `<div class="tool">Demo mode: run <code>npm run tauri dev</code> or <code>omni tools</code> for live tool detection.</div>`;
    } catch (e) {
      box.textContent = String(e);
    }
  })();
  renderJobs(rows);
  log(isTauri() ? "Tauri backend connected." : "Browser demo mode. Backend features (real convert, tool scan) activate inside Tauri or via CLI `omni`.");
}

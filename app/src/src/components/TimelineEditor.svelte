<script lang="ts">
  import { t } from "../lib/i18n";
  import { createUndo } from "../lib/undo";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";

  // ---------------------------------------------------------------------------
  // Types (clean-room, pola umum timeline editor)
  // ---------------------------------------------------------------------------
  type TrackKind = "video" | "audio" | "text";

  interface Track { id: string; kind: TrackKind; name: string; }

  interface TClip {
    id: string;
    trackId: string;
    src: string;       // path file (video/audio) atau "" untuk teks
    name: string;
    start: number;     // posisi di timeline (detik)
    duration: number;  // durasi di timeline (detik)
    in: number;        // titik trim-in di source (detik)
    volume: number;    // 0-100, untuk audio
    text: string;      // untuk klip teks
  }

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------
  const PX_PER_SEC = 12;
  const LS_KEY = "minime.timeline";

  let tracks = $state<Track[]>([
    { id: "v1", kind: "video", name: "V1" },
    { id: "a1", kind: "audio", name: "A1" },
    { id: "t1", kind: "text", name: "T1" },
  ]);
  let clips = $state<TClip[]>([]);
  // Undo history (maks 20) — via lib/undo.ts, snapshot clips
  const tUndo = createUndo<string>(20);
  let canTUndo = $state(false);
  function snapClips(): string { return JSON.stringify(clips); }
  function pushTHist() {
    tUndo.push(snapClips());
    canTUndo = tUndo.canUndo;
  }
  function undoClips() {
    const prev = tUndo.undo();
    if (!prev) return;
    clips = JSON.parse(prev);
    selectedId = null;
    canTUndo = tUndo.canUndo;
    save();
  }
  $effect(() => { if (tUndo.size === 0) { tUndo.push(snapClips()); } });
  let selectedId = $state<string | null>(null);
  let playhead = $state(0);          // detik
  let playing = $state(false);
  let zoom = $state(1);
  let exporting = $state(false);
  let exportError = $state("");
  let exportPath = $state("");
  let addPath = $state("");
  let addKind = $state<TrackKind>("video");
  let addText = $state("");

  let videoEl: HTMLVideoElement | null = $state(null);
  let rafId = 0;

  const uid = () => `c${Date.now()}${Math.floor(Math.random() * 1000)}`;
  const totalDuration = $derived(
    clips.reduce((m, c) => Math.max(m, c.start + c.duration), 10)
  );
  const selected = $derived(clips.find((c) => c.id === selectedId) ?? null);

  function fmt(s: number) {
    const m = Math.floor(s / 60);
    const sec = Math.floor(s % 60);
    return `${m}:${sec.toString().padStart(2, "0")}`;
  }

  // --- persistence ---
  function save() {
    try { localStorage.setItem(LS_KEY, JSON.stringify({ tracks, clips })); } catch {}
  }
  function load() {
    try {
      const raw = localStorage.getItem(LS_KEY);
      if (raw) {
        const d = JSON.parse(raw);
        if (Array.isArray(d.tracks)) tracks = d.tracks;
        if (Array.isArray(d.clips)) clips = d.clips;
      }
    } catch {}
  }
  load();

  // --- operasi klip ---
  function addClip() {
    const track = tracks.find((t) => t.kind === addKind);
    if (!track) return;
    if (addKind === "text") {
      const t = addText.trim();
      if (!t) return;
      clips.push({
        id: uid(), trackId: track.id, src: "", name: t.slice(0, 20),
        start: playhead, duration: 3, in: 0, volume: 100, text: t,
      });
      addText = "";
    } else {
      const p = addPath.trim();
      if (!p) return;
      const name = p.split("/").pop() ?? "klip";
      // durasi default 5 dtk, user bisa trim; durasi asli dibaca saat export
      clips.push({
        id: uid(), trackId: track.id, src: p, name,
        start: playhead, duration: 5, in: 0, volume: 100, text: "",
      });
      addPath = "";
    }
    pushTHist(); save();
  }

  function deleteSelected() {
    if (!selectedId) return;
    clips = clips.filter((c) => c.id !== selectedId);
    selectedId = null;
    pushTHist(); save();
  }

  function splitAtPlayhead() {
    const c = selected;
    if (!c) return;
    const rel = playhead - c.start;
    if (rel <= 0.1 || rel >= c.duration - 0.1) return;
    const right: TClip = {
      ...c, id: uid(),
      start: c.start + rel,
      duration: c.duration - rel,
      in: c.in + rel,
      name: c.name,
    };
    c.duration = rel;
    clips.push(right);
    pushTHist(); save();
  }

  function moveClip(c: TClip, deltaSec: number) {
    c.start = Math.max(0, c.start + deltaSec);
    pushTHist(); save();
  }

  function trimClip(c: TClip, edge: "left" | "right", deltaSec: number) {
    if (edge === "left") {
      const d = Math.min(deltaSec, c.duration - 0.2);
      if (c.start + d < 0 || c.in + d < 0) return;
      c.start += d; c.in += d; c.duration -= d;
    } else {
      c.duration = Math.max(0.2, c.duration + deltaSec);
    }
    pushTHist(); save();
  }

  // --- drag ---
  let dragState: { id: string; mode: "move" | "trim-l" | "trim-r"; x0: number; orig: TClip } | null = null;

  function onClipPointerDown(e: PointerEvent, c: TClip, mode: "move" | "trim-l" | "trim-r") {
    e.stopPropagation();
    selectedId = c.id;
    dragState = { id: c.id, mode, x0: e.clientX, orig: { ...c } };
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function onTimelinePointerMove(e: PointerEvent) {
    if (!dragState) return;
    const c = clips.find((x) => x.id === dragState!.id);
    if (!c) return;
    const dSec = (e.clientX - dragState.x0) / (PX_PER_SEC * zoom);
    const o = dragState.orig;
    if (dragState.mode === "move") {
      c.start = Math.max(0, o.start + dSec);
    } else if (dragState.mode === "trim-l") {
      const d = Math.max(-o.start, Math.min(dSec, o.duration - 0.2));
      if (o.in + d >= 0) { c.start = o.start + d; c.in = o.in + d; c.duration = o.duration - d; }
    } else {
      c.duration = Math.max(0.2, o.duration + dSec);
    }
  }

  function onTimelinePointerUp() {
    if (dragState) { dragState = null; pushTHist(); save(); }
  }

  // --- playhead ---
  function seekTo(e: MouseEvent, el: HTMLElement) {
    const r = el.getBoundingClientRect();
    playhead = Math.max(0, (e.clientX - r.left) / (PX_PER_SEC * zoom));
    syncPreview();
  }

  function syncPreview() {
    if (!videoEl) return;
    // cari klip video aktif di playhead
    const vc = clips.find((c) => {
      const t = tracks.find((tr) => tr.id === c.trackId);
      return t?.kind === "video" && playhead >= c.start && playhead < c.start + c.duration;
    });
    if (vc && vc.src) {
      const wantSrc = convertFileSrc(vc.src);
      if (videoEl.getAttribute("src") !== wantSrc) videoEl.src = wantSrc;
      const t = vc.in + (playhead - vc.start);
      if (Math.abs(videoEl.currentTime - t) > 0.3) videoEl.currentTime = t;
    }
  }

  function togglePlay() {
    playing = !playing;
    if (playing) {
      videoEl?.play().catch(() => {});
      const tick = () => {
        playhead += 1 / 30;
        if (playhead >= totalDuration) { playing = false; return; }
        syncPreview();
        rafId = requestAnimationFrame(tick);
      };
      rafId = requestAnimationFrame(tick);
    } else {
      cancelAnimationFrame(rafId);
      videoEl?.pause();
    }
  }

  // --- export ---
  async function doExport() {
    if (clips.length === 0 || exporting) return;
    exporting = true; exportError = ""; exportPath = "";
    try {
      exportPath = await invoke<string>("export_timeline", {
        timelineJson: JSON.stringify({ tracks, clips }),
        outputName: `timeline_${Date.now()}`,
      });
    } catch (e) { exportError = String(e); }
    finally { exporting = false; }
  }

  function trackClips(trackId: string) {
    return clips.filter((c) => c.trackId === trackId).sort((a, b) => a.start - b.start);
  }

  function clipColor(kind: TrackKind) {
    return kind === "video" ? "#4FA6F0" : kind === "audio" ? "#B5E8C0" : "#D3BFF0";
  }
</script>

<div class="tl-editor">
  <!-- Preview -->
  <div class="preview-bar">
    <div class="preview-box">
      <video bind:this={videoEl} muted playsinline></video>
      {#if !clips.some((c) => tracks.find((t) => t.id === c.trackId)?.kind === "video")}
        <div class="preview-empty">Tambahkan klip video untuk preview</div>
      {/if}
    </div>
    <div class="transport">
      <button class="btn-ghost sm" onclick={togglePlay}>{playing ? "⏸" : "▶"}</button>
      <span class="time">{fmt(playhead)} / {fmt(totalDuration)}</span>
      <span class="spacer"></span>
      <button class="btn-ghost sm" onclick={() => (zoom = Math.max(0.25, zoom / 1.25))}>−</button>
      <span class="hint-sm">Zoom</span>
      <button class="btn-ghost sm" onclick={() => (zoom = Math.min(4, zoom * 1.25))}>+</button>
    </div>
  </div>

  <!-- Toolbar -->
  <div class="toolbar">
    <select bind:value={addKind} aria-label="Jenis klip">
      <option value="video">Video</option>
      <option value="audio">Audio</option>
      <option value="text">Teks</option>
    </select>
    {#if addKind === "text"}
      <input type="text" bind:value={addText} placeholder="Teks overlay..." style="flex:1" />
    {:else}
      <input type="text" bind:value={addPath} placeholder="/path/ke/file.mp4" style="flex:1" />
    {/if}
    <button class="btn-primary sm" onclick={addClip}>+ Klip</button>
    <span class="sep"></span>
    <button class="btn-ghost sm" onclick={splitAtPlayhead} disabled={!selected} title="Pisah di playhead">✂ Pisah</button>
    <button class="btn-ghost sm" onclick={deleteSelected} disabled={!selected} title={$t("common.delete.clip")}>🗑 {$t("common.delete")}</button>
    <button class="btn-ghost sm" onclick={undoClips} disabled={!canTUndo} title={$t("common.undo.last")}>{$t("common.undo")}</button>
    <span class="spacer"></span>
    <button class="btn-primary sm" onclick={doExport} disabled={clips.length === 0 || exporting}>
      {exporting ? "Mengekspor..." : "⬇ Export MP4"}
    </button>
  </div>
  {#if exportError}<p class="err">{exportError}</p>{/if}
  {#if exportPath}<p class="ok">Tersimpan: <code>{exportPath}</code></p>{/if}

  <!-- Timeline -->
  <div class="timeline-wrap">
    <div class="ruler" onclick={(e) => seekTo(e, e.currentTarget)}>
      {#each Array(Math.ceil(totalDuration / 5) + 1) as _, i}
        <span class="tick" style:left={`${i * 5 * PX_PER_SEC * zoom}px`}>{fmt(i * 5)}</span>
      {/each}
      <div class="playhead" style:left={`${playhead * PX_PER_SEC * zoom}px`}></div>
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tracks"
      onpointermove={onTimelinePointerMove}
      onpointerup={onTimelinePointerUp}
      onpointercancel={onTimelinePointerUp}
    >
      {#each tracks as tr}
        <div class="track-row">
          <div class="track-label">{tr.name}</div>
          <div
            class="track-lane"
            style:width={`${totalDuration * PX_PER_SEC * zoom}px`}
            onclick={(e) => { if (e.target === e.currentTarget) { selectedId = null; seekTo(e, e.currentTarget); } }}
          >
            {#each trackClips(tr.id) as c}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="clip {selectedId === c.id ? 'active' : ''}"
                style:left={`${c.start * PX_PER_SEC * zoom}px`}
                style:width={`${c.duration * PX_PER_SEC * zoom}px`}
                style:--cc={clipColor(tr.kind)}
                onpointerdown={(e) => onClipPointerDown(e, c, "move")}
              >
                <div class="trim trim-l" onpointerdown={(e) => onClipPointerDown(e, c, "trim-l")}></div>
                <span class="clip-name">{c.name}</span>
                <span class="clip-dur">{c.duration.toFixed(1)}s</span>
                <div class="trim trim-r" onpointerdown={(e) => onClipPointerDown(e, c, "trim-r")}></div>
              </div>
            {/each}
            <div class="playhead-line" style:left={`${playhead * PX_PER_SEC * zoom}px`}></div>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Properties -->
  {#if selected}
    {@const tr = tracks.find((t) => t.id === selected.trackId)}
    <div class="props">
      <h4>Properti Klip</h4>
      <div class="prop-grid">
        <label>Nama <input type="text" bind:value={selected.name} oninput={save} /></label>
        <label>Mulai (d) <input type="number" step="0.1" min="0" bind:value={selected.start} oninput={save} /></label>
        <label>Durasi (d) <input type="number" step="0.1" min="0.2" bind:value={selected.duration} oninput={save} /></label>
        {#if tr?.kind !== "text"}
          <label>Trim-in (d) <input type="number" step="0.1" min="0" bind:value={selected.in} oninput={save} /></label>
        {/if}
        {#if tr?.kind === "audio"}
          <label>Volume <input type="number" min="0" max="100" bind:value={selected.volume} oninput={save} /></label>
        {/if}
        {#if tr?.kind === "text"}
          <label class="wide">Teks <input type="text" bind:value={selected.text} oninput={save} /></label>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .tl-editor { display: flex; flex-direction: column; gap: 10px; height: 100%; }
  .preview-bar { display: flex; gap: 10px; align-items: stretch; }
  .preview-box { position: relative; width: 280px; height: 158px; background: #000; border-radius: 8px; overflow: hidden; flex-shrink: 0; }
  .preview-box video { width: 100%; height: 100%; object-fit: contain; }
  .preview-empty { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; color: #888; font-size: 13px; text-align: center; padding: 12px; }
  .transport { display: flex; align-items: center; gap: 8px; flex: 1; }
  .time { font-variant-numeric: tabular-nums; font-size: 14px; }
  .spacer { flex: 1; }
  .hint-sm { font-size: 12px; color: var(--dim); }
  .toolbar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .toolbar select, .toolbar input { padding: 7px 10px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); font-size: 13px; }
  .sep { width: 1px; height: 24px; background: var(--border); }
  .err { color: #ef4444; font-size: 13px; margin: 0; }
  .ok { color: #4FA6F0; font-size: 13px; margin: 0; }
  .ok code, .err code { font-size: 12px; }
  .timeline-wrap { border: 1px solid var(--border); border-radius: 10px; overflow-x: auto; background: var(--card); }
  .ruler { position: relative; height: 26px; border-bottom: 1px solid var(--border); cursor: pointer; user-select: none; }
  .tick { position: absolute; top: 4px; font-size: 11px; color: var(--dim); transform: translateX(4px); }
  .playhead { position: absolute; top: 0; bottom: 0; width: 2px; background: #4FA6F0; z-index: 5; pointer-events: none; }
  .playhead::before { content: ""; position: absolute; top: 0; left: -5px; border: 6px solid transparent; border-top: 8px solid #4FA6F0; }
  .tracks { user-select: none; }
  .track-row { display: flex; border-bottom: 1px solid var(--border); }
  .track-row:last-child { border-bottom: none; }
  .track-label { width: 48px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700; color: var(--dim); border-right: 1px solid var(--border); background: var(--bg); }
  .track-lane { position: relative; height: 64px; }
  .clip { position: absolute; top: 8px; bottom: 8px; background: color-mix(in srgb, var(--cc) 25%, transparent); border: 1px solid var(--cc); border-radius: 6px; cursor: grab; overflow: hidden; display: flex; align-items: center; gap: 6px; padding: 0 8px; min-width: 24px; }
  .clip.active { outline: 2px solid #4FA6F0; outline-offset: 1px; }
  .clip-name { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .clip-dur { font-size: 10px; color: var(--dim); white-space: nowrap; }
  .trim { position: absolute; top: 0; bottom: 0; width: 10px; cursor: ew-resize; z-index: 2; }
  .trim-l { left: 0; border-left: 3px solid var(--cc); }
  .trim-r { right: 0; border-right: 3px solid var(--cc); }
  .playhead-line { position: absolute; top: 0; bottom: 0; width: 2px; background: #4FA6F0; z-index: 4; pointer-events: none; }
  .props { border: 1px solid var(--border); border-radius: 10px; padding: 12px; background: var(--card); }
  .props h4 { margin: 0 0 8px; font-size: 14px; }
  .prop-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 8px; }
  .prop-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--dim); }
  .prop-grid label.wide { grid-column: 1 / -1; }
  .prop-grid input { padding: 7px 10px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); font-size: 13px; }
  .btn-ghost.sm, .btn-primary.sm { padding: 7px 12px; border-radius: 8px; font-size: 13px; cursor: pointer; }
  .btn-ghost.sm { border: 1px solid var(--border); background: transparent; color: var(--text); }
  .btn-ghost.sm:disabled { opacity: 0.4; cursor: default; }
  .btn-primary.sm { background: #4FA6F0; color: #fff; border: none; font-weight: 600; }
  .btn-primary.sm:disabled { opacity: 0.5; cursor: default; }
</style>

<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";

  // ---------------------------------------------------------------------------
  // Types
  // ---------------------------------------------------------------------------
  type Engine = "voicestudio" | "yovoice" | "none";

  interface Voice { id: string; name: string; lang: string; }

  interface Clip {
    id: number;
    text: string;
    voice: string;          // voice_id; "default" = engine default
    speed: number;          // 0.5 - 2.0
    volume: number;         // 0 - 100
    trimStart: number;      // seconds into the generated audio
    trimLen: number | null; // seconds to use; null = full duration
    audioPath: string;      // absolute path of the generated wav
    duration: number;       // generated audio duration (seconds)
    engine: string;         // "voicestudio" | "yovoice"
    refAudio: string;       // optional clone reference path
    masterPath: string;     // mastered wav path (via audio_master)
    generating: boolean;
    mastering: boolean;
    error: string;
  }

  interface Track { id: number; name: string; clips: Clip[]; }

  type PlayResult = "done" | "stopped" | "error";

  // ---------------------------------------------------------------------------
  // State + persistence (localStorage key: minime.voice-editor)
  // ---------------------------------------------------------------------------
  const LS_KEY = "minime.voice-editor";
  const uid = () => Date.now() * 1000 + Math.floor(Math.random() * 1000);
  const clampNum = (v: number, lo: number, hi: number) =>
    Math.min(hi, Math.max(lo, Number.isFinite(v) ? v : lo));

  function newClip(): Clip {
    return {
      id: uid(), text: "", voice: "default", speed: 1, volume: 100,
      trimStart: 0, trimLen: null, audioPath: "", duration: 0, engine: "",
      refAudio: "", masterPath: "", generating: false, mastering: false, error: "",
    };
  }

  function newTrack(n: number): Track { return { id: uid(), name: `Track ${n}`, clips: [] }; }

  function sanitizeClip(c: any): Clip {
    const base = newClip();
    return {
      ...base,
      id: typeof c.id === "number" ? c.id : uid(),
      text: String(c.text ?? ""),
      voice: String(c.voice ?? "default"),
      speed: clampNum(Number(c.speed) || 1, 0.5, 2),
      volume: clampNum(c.volume == null ? 100 : Number(c.volume), 0, 100),
      trimStart: Math.max(0, Number(c.trimStart) || 0),
      trimLen: c.trimLen == null ? null : Math.max(0.1, Number(c.trimLen) || 0.1),
      audioPath: String(c.audioPath ?? ""),
      duration: Math.max(0, Number(c.duration) || 0),
      engine: String(c.engine ?? ""),
      refAudio: String(c.refAudio ?? ""),
      masterPath: String(c.masterPath ?? ""),
    };
  }

  function loadState(): Track[] {
    try {
      const raw = localStorage.getItem(LS_KEY);
      if (!raw) return [newTrack(1), newTrack(2)];
      const j = JSON.parse(raw);
      if (!Array.isArray(j.tracks) || j.tracks.length === 0) return [newTrack(1), newTrack(2)];
      return j.tracks.map((t: any, ti: number) => ({
        id: typeof t.id === "number" ? t.id : uid() + ti,
        name: String(t.name || `Track ${ti + 1}`),
        clips: Array.isArray(t.clips) ? t.clips.map(sanitizeClip) : [],
      }));
    } catch {
      return [newTrack(1), newTrack(2)];
    }
  }

  let tracks = $state<Track[]>(loadState());
  // Undo history (maks 20) — snapshot tracks
  let vHist = $state<string[]>([]);
  let vHIdx = $state(-1);
  let canVUndo = $state(false);
  function snapTracks(): string { return JSON.stringify(tracks); }
  function pushVHist() {
    vHist = vHist.slice(0, vHIdx + 1);
    vHist.push(snapTracks());
    if (vHist.length > 20) vHist.shift();
    vHIdx = vHist.length - 1;
    canVUndo = vHIdx > 0;
  }
  function undoTracks() {
    if (vHIdx <= 0) return;
    vHIdx--;
    tracks = JSON.parse(vHist[vHIdx]);
    canVUndo = vHIdx > 0;
    persist();
  }
  function initVHist() { vHist = [snapTracks()]; vHIdx = 0; canVUndo = false; }
  $effect(() => { if (vHIdx === -1) initVHist(); });

  function persist() {
    try {
      localStorage.setItem(LS_KEY, JSON.stringify({
        tracks: tracks.map((t) => ({
          id: t.id, name: t.name,
          clips: t.clips.map((c) => ({
            id: c.id, text: c.text, voice: c.voice, speed: c.speed,
            volume: c.volume, trimStart: c.trimStart, trimLen: c.trimLen,
            audioPath: c.audioPath, duration: c.duration, engine: c.engine,
            refAudio: c.refAudio, masterPath: c.masterPath,
          })),
        })),
      }));
    } catch { /* storage full / unavailable: keep working in memory */ }
  }

  function findClip(id: number): Clip | undefined {
    for (const t of tracks) {
      const c = t.clips.find((x) => x.id === id);
      if (c) return c;
    }
    return undefined;
  }

  // ---------------------------------------------------------------------------
  // Engine status (auto-refresh every 15s)
  // ---------------------------------------------------------------------------
  let engine = $state<Engine>("none");
  let engineVersion = $state<string | null>(null);
  let checking = $state(false);
  let openingVs = $state(false);
  let vsMsg = $state("");
  let voices = $state<Voice[]>([{ id: "default", name: "Default", lang: "" }]);
  let yvModels = $state<Array<{ id: string; name: string }>>([]);
  let yvModel = $state("");

  async function checkStatus() {
    checking = true;
    try {
      let next: Engine = "none";
      let ver: string | null = null;
      try {
        // Contract: {"reachable": bool, "version": string|null,
        //           "engine": "voicestudio"|"yovoice"|"none"}
        const j = JSON.parse(await invoke<string>("voicestudio_status"));
        if (j && j.reachable === true) {
          if (j.engine === "voicestudio") {
            next = "voicestudio";
            ver = typeof j.version === "string" ? j.version : null;
          } else if (j.engine === "yovoice") {
            next = "yovoice";
          }
        }
      } catch { /* voicestudio_* commands not registered yet — fall through */ }
      if (next === "none") {
        try {
          const j = JSON.parse(await invoke<string>("yovoice_status"));
          if (j && j.ready === true) next = "yovoice";
        } catch { /* yovoice unavailable */ }
      }
      if (next !== engine) {
        engine = next;
        engineVersion = ver;
        await loadVoices();
      } else {
        engineVersion = ver;
      }
    } finally {
      checking = false;
    }
  }

  async function loadVoices() {
    try {
      if (engine === "voicestudio") {
        const arr = JSON.parse(await invoke<string>("voicestudio_voices"));
        voices = [
          { id: "default", name: "Default", lang: "" },
          ...(Array.isArray(arr) ? arr : []).map((v: any) => ({
            id: String(v.voice_id ?? v.id ?? "default"),
            name: String(v.name ?? v.voice_id ?? "Voice"),
            lang: String(v.language ?? v.lang ?? ""),
          })),
        ];
      } else if (engine === "yovoice") {
        const [modelsArr, voicesArr] = await Promise.all([
          JSON.parse(await invoke<string>("yovoice_models")).catch(() => []),
          JSON.parse(await invoke<string>("yovoice_voices")).catch(() => []),
        ]);
        yvModels = (Array.isArray(modelsArr) ? modelsArr : []).map((m: any) => ({
          id: String(m.id), name: String(m.name || m.id),
        }));
        if (!yvModels.some((m) => m.id === yvModel)) {
          yvModel = yvModels[0]?.id ?? "";
        }
        voices = [
          { id: "default", name: "Default", lang: "" },
          ...(Array.isArray(voicesArr) ? voicesArr : []).map((v: any) => ({
            id: String(v.id),
            name: String(v.name || v.id) + (v.kind === "saved" ? " (klon)" : ""),
            lang: "",
          })),
        ];
      } else {
        voices = [{ id: "default", name: "Default", lang: "" }];
      }
      // Keep clip voice selections valid after a voice list change.
      for (const t of tracks)
        for (const c of t.clips)
          if (!voices.some((v) => v.id === c.voice)) c.voice = "default";
    } catch { /* keep previous list */ }
  }

  async function openVoiceStudio() {
    openingVs = true;
    vsMsg = "";
    try {
      await invoke("voicestudio_open");
      await checkStatus();
      vsMsg = engine === "none"
        ? "VoiceStudio belum terdeteksi — pastikan aplikasinya sudah berjalan."
        : "Status diperbarui.";
    } catch (e) {
      vsMsg = e instanceof Error ? e.message : String(e);
    } finally {
      openingVs = false;
    }
  }

  $effect(() => {
    void checkStatus();
    const id = setInterval(() => { void checkStatus(); }, 15000);
    return () => { clearInterval(id); stopAll(); };
  });

  // ---------------------------------------------------------------------------
  // Track / clip management
  // ---------------------------------------------------------------------------
  function nextTrackName(): string {
    let n = 0;
    for (const t of tracks) {
      const m = /^Track (\d+)$/.exec(t.name);
      if (m) n = Math.max(n, Number(m[1]));
    }
    return `Track ${n + 1}`;
  }

  function addTrack() { tracks.push({ id: uid(), name: nextTrackName(), clips: [] }); pushVHist(); persist(); }

  function removeTrack(trackId: number) {
    if (tracks.length <= 1) return;
    const t = tracks.find((x) => x.id === trackId);
    if (!t) return;
    if (t.clips.length > 0 && !confirm(`Hapus ${t.name} beserta ${t.clips.length} clip di dalamnya?`)) return;
    tracks = tracks.filter((x) => x.id !== trackId);
    persist();
  }

  function addClip(trackId: number) {
    const t = tracks.find((x) => x.id === trackId);
    if (!t) return;
    t.clips.push(newClip());
    persist();
  }

  function removeClip(trackId: number, clipId: number) {
    const t = tracks.find((x) => x.id === trackId);
    if (!t) return;
    t.clips = t.clips.filter((c) => c.id !== clipId);
    pushVHist(); persist();
  }

  function moveClip(trackId: number, clipId: number, dir: -1 | 1) {
    const t = tracks.find((x) => x.id === trackId);
    if (!t) return;
    const i = t.clips.findIndex((c) => c.id === clipId);
    const j = i + dir;
    if (i < 0 || j < 0 || j >= t.clips.length) return;
    [t.clips[i], t.clips[j]] = [t.clips[j], t.clips[i]];
    pushVHist(); persist();
  }

  function moveClipTrack(trackId: number, clipId: number, dir: -1 | 1) {
    const ti = tracks.findIndex((x) => x.id === trackId);
    if (ti < 0) return;
    const tj = ti + dir;
    if (tj < 0 || tj >= tracks.length) return;
    const ci = tracks[ti].clips.findIndex((c) => c.id === clipId);
    if (ci < 0) return;
    const [c] = tracks[ti].clips.splice(ci, 1);
    tracks[tj].clips.push(c);
    pushVHist(); persist();
  }

  // ---------------------------------------------------------------------------
  // Audio helpers
  // ---------------------------------------------------------------------------
  function parsePathJson(raw: string): { path: string; engine: string } {
    const s = String(raw ?? "");
    try {
      const j = JSON.parse(s);
      if (j && typeof j.path === "string")
        return { path: j.path, engine: typeof j.engine === "string" ? j.engine : "" };
    } catch { /* not JSON — treat as a plain path */ }
    return { path: s, engine: "" };
  }

  async function decodeFile(path: string): Promise<AudioBuffer> {
    const res = await fetch(convertFileSrc(path));
    if (!res.ok) throw new Error(`Gagal membaca file audio (HTTP ${res.status}).`);
    const raw = await res.arrayBuffer();
    const AC: typeof AudioContext =
      window.AudioContext || (window as any).webkitAudioContext;
    const actx = new AC();
    try {
      if (actx.state === "suspended") await actx.resume().catch(() => {});
      return await actx.decodeAudioData(raw);
    } finally {
      actx.close().catch(() => {});
    }
  }

  async function probeDuration(path: string): Promise<number> {
    try {
      const a = new Audio();
      a.preload = "metadata";
      a.src = convertFileSrc(path);
      await new Promise<void>((resolve, reject) => {
        a.onloadedmetadata = () => resolve();
        a.onerror = () => reject(new Error("metadata"));
      });
      return Number.isFinite(a.duration) ? a.duration : 0;
    } catch {
      return 0;
    }
  }

  /** Effective output seconds of a clip (trim applied; yovoice speed via playbackRate). */
  function outDur(c: Clip): number {
    const base = Math.max(0, (c.trimLen ?? c.duration) || 0);
    const rate = c.engine === "yovoice" ? c.speed : 1;
    return base / Math.max(0.1, rate);
  }

  function fmtDur(s: number): string {
    const v = Math.max(0, s || 0);
    const m = Math.floor(v / 60);
    const r = v - m * 60;
    return `${m}:${r < 10 ? "0" : ""}${r.toFixed(1)}`;
  }

  const genClips = $derived(
    tracks.flatMap((t) => t.clips.filter((c) => c.audioPath))
  );
  const totalDur = $derived(genClips.reduce((s, c) => s + outDur(c), 0));

  // ---------------------------------------------------------------------------
  // TTS generation per clip
  // ---------------------------------------------------------------------------
  async function generateClip(clip: Clip) {
    const text = clip.text.trim();
    if (!text || clip.generating) return;
    if (engine === "none") {
      clip.error = "Tidak ada engine suara yang tersedia — generate nonaktif.";
      persist();
      return;
    }
    clip.generating = true;
    clip.error = "";
    persist();
    try {
      const outName = `ve_${clip.id}_${Date.now()}`;
      if (engine === "voicestudio") {
        const ref = clip.refAudio.trim();
        const raw = ref
          ? await invoke<string>("voicestudio_clone_tts", {
              text, refAudioPath: ref, language: "id", outputName: outName,
            })
          : await invoke<string>("voicestudio_tts", {
              text, voice: clip.voice, speed: clip.speed, outputName: outName,
            });
        const j = parsePathJson(raw);
        if (!j.path) throw new Error("Backend tidak mengembalikan path file.");
        clip.audioPath = j.path;
        clip.engine = j.engine || "voicestudio";
      } else {
        // yovoice fallback: no speed parameter — speed is applied at
        // playback/mixdown via playbackRate instead.
        const path = await invoke<string>("yovoice_tts", {
          text,
          voice: clip.voice === "default" ? "" : clip.voice,
          reference: clip.refAudio.trim(),
          outputName: outName,
          model: yvModel,
        });
        if (!path) throw new Error("Backend tidak mengembalikan path file.");
        clip.audioPath = path;
        clip.engine = "yovoice";
      }
      clip.masterPath = "";
      clip.duration = await probeDuration(clip.audioPath);
      clip.trimStart = 0;
      clip.trimLen = null;
      persist();
      drawWave(clip.id);
    } catch (e) {
      clip.error = e instanceof Error ? e.message : String(e);
      persist();
    } finally {
      clip.generating = false;
      persist();
    }
  }

  // ---------------------------------------------------------------------------
  // Per-clip mastering (real backend: audio_master on the clip's file)
  // ---------------------------------------------------------------------------
  async function masterClip(clip: Clip) {
    if (!clip.audioPath || clip.mastering) return;
    clip.mastering = true;
    clip.error = "";
    persist();
    try {
      clip.masterPath = await invoke<string>("audio_master", {
        inputPath: clip.audioPath,
        outputName: `ve_master_${clip.id}_${Date.now()}`,
      });
      persist();
    } catch (e) {
      clip.error = e instanceof Error ? e.message : String(e);
      persist();
    } finally {
      clip.mastering = false;
      persist();
    }
  }

  // ---------------------------------------------------------------------------
  // Playback (per clip, mastered preview, play-all)
  // ---------------------------------------------------------------------------
  let playingId = $state<number | null>(null);
  let playAllActive = $state(false);
  let playToken = 0;
  let currentAudio: HTMLAudioElement | null = null;
  let previewAudio: HTMLAudioElement | null = null;
  let transportMsg = $state("");

  function stopAll() {
    playToken++;
    playAllActive = false;
    playingId = null;
    if (currentAudio) { try { currentAudio.pause(); } catch {} currentAudio = null; }
    if (previewAudio) { try { previewAudio.pause(); } catch {} previewAudio = null; }
  }

  function playClipOnce(clip: Clip): Promise<PlayResult> {
    const token = playToken;
    return new Promise((resolve) => {
      let done = false;
      const a = new Audio();
      currentAudio = a;
      const rate = clip.engine === "yovoice" ? clip.speed : 1;
      const cleanup = () => {
        a.onended = null;
        a.onerror = null;
        a.onloadedmetadata = null;
        a.removeEventListener("timeupdate", onTime);
        a.removeEventListener("pause", onPause);
        if (currentAudio === a) currentAudio = null;
      };
      const finish = (r: PlayResult) => {
        if (done) return;
        done = true;
        cleanup();
        resolve(r);
      };
      const onTime = () => {
        const endAt = clip.trimLen == null
          ? (clip.duration || Infinity)
          : Math.min(clip.trimStart + clip.trimLen, clip.duration || Infinity);
        if (a.currentTime >= endAt) finish("done");
      };
      const onPause = () => { if (token !== playToken) finish("stopped"); };
      a.onended = () => finish("done");
      a.onerror = () => finish("error");
      a.addEventListener("timeupdate", onTime);
      a.addEventListener("pause", onPause);
      a.preload = "auto";
      a.src = convertFileSrc(clip.audioPath);
      a.volume = clampNum(clip.volume / 100, 0, 1);
      a.playbackRate = rate;
      const begin = () => {
        try {
          a.currentTime = Math.max(0, Math.min(clip.trimStart, a.duration || 0));
        } catch { /* play from 0 */ }
        a.play().catch(() => finish("error"));
      };
      if (a.readyState >= 1) begin();
      else a.onloadedmetadata = begin;
    });
  }

  async function toggleClipPlay(clip: Clip) {
    if (playingId === clip.id) { stopAll(); return; }
    stopAll();
    playingId = clip.id;
    transportMsg = "";
    const r = await playClipOnce(clip);
    if (playingId === clip.id) playingId = null;
    if (r === "error") {
      clip.error = "Gagal memutar audio — file mungkin sudah tidak ada.";
      persist();
    }
  }

  function playPreview(path: string, volume: number) {
    stopAll();
    try {
      const a = new Audio(convertFileSrc(path));
      previewAudio = a;
      a.volume = clampNum(volume / 100, 0, 1);
      a.play().catch(() => {});
    } catch { /* ignore */ }
  }

  async function playAll() {
    const list = genClips;
    if (!list.length || playAllActive) return;
    stopAll();
    playAllActive = true;
    transportMsg = "";
    for (let i = 0; i < list.length; i++) {
      if (!playAllActive) break;
      playingId = list[i].id;
      transportMsg = `Memutar clip ${i + 1}/${list.length}…`;
      const r = await playClipOnce(list[i]);
      if (r === "error") {
        transportMsg = `Clip ${i + 1} gagal diputar — berhenti.`;
        break;
      }
      if (r === "stopped") break;
    }
    playingId = null;
    playAllActive = false;
    if (transportMsg.startsWith("Memutar")) transportMsg = "";
  }

  // ---------------------------------------------------------------------------
  // Waveform (canvas + decodeAudioData, no extra libs)
  // ---------------------------------------------------------------------------
  const canvasMap = new Map<number, HTMLCanvasElement>();

  function regCanvas(node: HTMLCanvasElement, id: number) {
    canvasMap.set(id, node);
    drawWave(id);
    return { destroy() { canvasMap.delete(id); } };
  }

  function drawWave(id: number) {
    const clip = findClip(id);
    const canvas = canvasMap.get(id);
    if (!clip || !canvas || !clip.audioPath) return;
    const g = canvas.getContext("2d");
    if (!g) return;
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(60, canvas.clientWidth || 320);
    const h = 56;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.fillStyle = "rgba(127,127,127,.12)";
    g.fillRect(0, 0, w, h);
    g.fillStyle = "#98a1ac";
    g.font = "12px sans-serif";
    g.fillText("Memuat waveform…", 10, h / 2 + 4);
    decodeFile(clip.audioPath).then((ab) => {
      if (!canvasMap.get(id)) return; // unmounted meanwhile
      if (!clip.duration) { clip.duration = ab.duration; persist(); }
      renderPeaks(g, w, h, ab, clip);
    }).catch(() => {
      // Fallback: duration bar (keeps the clip usable without waveform).
      g.fillStyle = "rgba(127,127,127,.12)";
      g.fillRect(0, 0, w, h);
      g.fillStyle = "#4FA6F0";
      g.fillRect(0, h / 2 - 4, w, 8);
      g.fillStyle = "#98a1ac";
      g.font = "12px sans-serif";
      g.fillText(`Durasi ${fmtDur(clip.duration)} — waveform tidak tersedia`, 10, 16);
    });
  }

  function renderPeaks(
    g: CanvasRenderingContext2D, w: number, h: number,
    ab: AudioBuffer, clip: Clip,
  ) {
    const data = ab.getChannelData(0);
    const bars = Math.max(40, Math.floor(w / 3));
    const step = Math.max(1, Math.floor(data.length / bars));
    g.fillStyle = "rgba(127,127,127,.12)";
    g.fillRect(0, 0, w, h);
    const mid = h / 2;
    g.fillStyle = "#4FA6F0";
    for (let i = 0; i < bars; i++) {
      const s = i * step;
      const end = Math.min(s + step, data.length);
      let peak = 0;
      for (let j = s; j < end; j += 4) {
        const v = Math.abs(data[j]);
        if (v > peak) peak = v;
      }
      const bh = Math.max(1, peak * (h - 8));
      g.fillRect(i * (w / bars), mid - bh / 2, Math.max(1, w / bars - 1), bh);
    }
    // Shade the trimmed-away regions so the used part stands out.
    const total = ab.duration;
    if (total > 0) {
      const x0 = (Math.min(clip.trimStart, total) / total) * w;
      const usedEnd = clip.trimLen == null ? total : clip.trimStart + clip.trimLen;
      const x1 = Math.min(w, (Math.min(usedEnd, total) / total) * w);
      g.fillStyle = "rgba(0,0,0,.45)";
      if (x0 > 0) g.fillRect(0, 0, x0, h);
      if (x1 < w) g.fillRect(x1, 0, w - x1, h);
    }
  }

  function clampTrim(clip: Clip) {
    clip.trimStart = Math.max(0, clip.trimStart || 0);
    if (clip.duration > 0) {
      clip.trimStart = Math.min(clip.trimStart, Math.max(0, clip.duration - 0.1));
      if (clip.trimLen != null)
        clip.trimLen = Math.min(Math.max(0.1, clip.trimLen), Math.max(0.1, clip.duration - clip.trimStart));
    }
    persist();
    drawWave(clip.id);
  }

  // ---------------------------------------------------------------------------
  // Mixdown & export (OfflineAudioContext -> WAV encoder -> download)
  // ---------------------------------------------------------------------------
  let exporting = $state(false);
  let mixMsg = $state("");
  let mixError = $state("");

  function encodeWav(buffer: AudioBuffer): Blob {
    const numCh = 2;
    const sr = buffer.sampleRate;
    const ch0 = buffer.getChannelData(0);
    const ch1 = buffer.numberOfChannels > 1 ? buffer.getChannelData(1) : ch0;
    const n = buffer.length;
    const dataSize = n * numCh * 2;
    const ab = new ArrayBuffer(44 + dataSize);
    const v = new DataView(ab);
    const wstr = (o: number, s: string) => {
      for (let i = 0; i < s.length; i++) v.setUint8(o + i, s.charCodeAt(i));
    };
    wstr(0, "RIFF");
    v.setUint32(4, 36 + dataSize, true);
    wstr(8, "WAVE");
    wstr(12, "fmt ");
    v.setUint32(16, 16, true);
    v.setUint16(20, 1, true); // PCM
    v.setUint16(22, numCh, true);
    v.setUint32(24, sr, true);
    v.setUint32(28, sr * numCh * 2, true);
    v.setUint16(32, numCh * 2, true);
    v.setUint16(34, 16, true);
    wstr(36, "data");
    v.setUint32(40, dataSize, true);
    let o = 44;
    for (let i = 0; i < n; i++) {
      for (let ch = 0; ch < numCh; ch++) {
        const s = Math.max(-1, Math.min(1, (ch === 0 ? ch0 : ch1)[i]));
        v.setInt16(o, s < 0 ? s * 0x8000 : s * 0x7fff, true);
        o += 2;
      }
    }
    return new Blob([ab], { type: "audio/wav" });
  }

  async function exportMixdown() {
    const list = genClips;
    if (!list.length || exporting) return;
    exporting = true;
    mixMsg = "Merender audio…";
    mixError = "";
    try {
      const sr = 44100;
      const parts: { ab: AudioBuffer; off: number; len: number; rate: number; vol: number }[] = [];
      let totalOut = 0;
      for (let i = 0; i < list.length; i++) {
        const c = list[i];
        mixMsg = `Memuat clip ${i + 1}/${list.length}…`;
        let ab: AudioBuffer;
        try {
          ab = await decodeFile(c.audioPath);
        } catch {
          throw new Error(`Clip ${i + 1}: file audio tidak bisa dibaca.`);
        }
        const startSamp = Math.floor(Math.min(c.trimStart, ab.duration) * ab.sampleRate);
        const maxLen = ab.length - startSamp;
        const wantLen = c.trimLen == null ? maxLen : Math.floor(c.trimLen * ab.sampleRate);
        const len = Math.max(0, Math.min(maxLen, wantLen));
        if (len <= 0) continue;
        const rate = c.engine === "yovoice" ? c.speed : 1;
        parts.push({ ab, off: startSamp, len, rate, vol: clampNum(c.volume / 100, 0, 1) });
        totalOut += len / ab.sampleRate / rate;
      }
      if (!parts.length) throw new Error("Tidak ada audio yang bisa digabung.");
      mixMsg = "Menggabung…";
      const octx = new OfflineAudioContext(2, Math.max(1, Math.ceil(totalOut * sr)), sr);
      let t = 0;
      for (const p of parts) {
        const src = octx.createBufferSource();
        src.buffer = p.ab;
        src.playbackRate.value = p.rate;
        const g = octx.createGain();
        g.gain.value = p.vol;
        src.connect(g);
        g.connect(octx.destination);
        src.start(t, p.off / p.ab.sampleRate, p.len / p.ab.sampleRate);
        t += p.len / p.ab.sampleRate / p.rate;
      }
      const rendered = await octx.startRendering();
      const blob = encodeWav(rendered);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `minime_voice_${new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-")}.wav`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 8000);
      mixMsg = `Mixdown selesai (${parts.length} clip, ${fmtDur(totalOut)}). File WAV diunduh.`;
    } catch (e) {
      mixError = e instanceof Error ? e.message : String(e);
      mixMsg = "";
    } finally {
      exporting = false;
    }
  }
</script>

<div class="ve">
  <!-- Header + engine status -->
  <div class="ve-head">
    <div>
      <h3>Editor Suara</h3>
      <p class="hint">Susun segmen voiceover di timeline multi-track, generate suaranya, lalu gabungkan jadi satu file WAV.</p>
      <button class="btn-ghost sm" onclick={undoTracks} disabled={!canVUndo} title="Urungkan perubahan terakhir">↩ Urungkan</button>
    </div>
    <div class="engine" data-engine={engine}>
      <span class="dot" aria-hidden="true"></span>
      <div class="engine-info">
        {#if engine === "voicestudio"}
          <strong>🟢 VoiceStudio terhubung</strong>
          {#if engineVersion}<span class="meta">v{engineVersion}</span>{/if}
        {:else if engine === "yovoice"}
          <strong>🟡 Fallback yovoice</strong>
          <span class="meta">VoiceStudio tidak terjangkau</span>
          {#if yvModels.length > 0}
            <label class="meta" for="yv-model">Model:
              <select id="yv-model" bind:value={yvModel} onchange={persist}>
                {#each yvModels as m}<option value={m.id}>{m.name}</option>{/each}
              </select>
            </label>
          {/if}
        {:else}
          <strong>🔴 Tidak ada engine</strong>
          <span class="meta">Generate suara nonaktif</span>
        {/if}
      </div>
      <button class="btn-ghost sm" onclick={checkStatus} disabled={checking}>
        {checking ? "Mengecek…" : "Cek ulang"}
      </button>
    </div>
  </div>

  {#if engine === "none"}
    <div class="notice">
      <p><strong>Tidak ada engine suara yang terjangkau.</strong> Buka aplikasi VoiceStudio, lalu cek ulang status.</p>
      <div class="row">
        <button class="btn-primary sm" onclick={openVoiceStudio} disabled={openingVs}>
          {openingVs ? "Membuka…" : "Buka VoiceStudio"}
        </button>
        <button class="btn-ghost sm" onclick={checkStatus}>Cek ulang status</button>
      </div>
      {#if vsMsg}<p class="hint">{vsMsg}</p>{/if}
    </div>
  {/if}

  <!-- Transport -->
  <div class="transport">
    <button class="btn-ghost" onclick={playAll} disabled={!genClips.length || playAllActive}>
      {playAllActive ? "⏸ Memutar…" : "▶ Play Semua"}
    </button>
    <button class="btn-ghost" onclick={stopAll} disabled={!playAllActive && playingId === null}>⏹ Stop</button>
    <button class="btn-primary" onclick={exportMixdown} disabled={!genClips.length || exporting}>
      {exporting ? "Merender…" : "⤓ Mixdown & Unduh WAV"}
    </button>
    <span class="meta">{genClips.length} clip siap · total {fmtDur(totalDur)}</span>
  </div>
  {#if transportMsg}<p class="hint">{transportMsg}</p>{/if}
  {#if mixMsg}<p class="ok">{mixMsg}</p>{/if}
  {#if mixError}<p class="err">{mixError}</p>{/if}
  <p class="hint micro">Urutan putar &amp; mixdown: Track 1 → Track 2 → …; di dalam track dari clip pertama ke terakhir.</p>

  <!-- Tracks -->
  {#each tracks as track, ti (track.id)}
    <section class="track">
      <div class="track-head">
        <strong>{track.name}</strong>
        <span class="meta">{track.clips.length} clip</span>
        <span class="spacer"></span>
        <button class="btn-ghost sm" onclick={() => addClip(track.id)}>+ Tambah Clip</button>
        {#if tracks.length > 1}
          <button class="btn-ghost sm danger" onclick={() => removeTrack(track.id)}>Hapus Track</button>
        {/if}
      </div>

      {#if track.clips.length === 0}
        <p class="hint">Belum ada clip — klik “Tambah Clip”.</p>
      {/if}

      {#each track.clips as clip, ci (clip.id)}
        <article class="clip" class:playing={playingId === clip.id}>
          <div class="clip-top">
            <strong>Clip {ci + 1}</strong>
            {#if clip.engine}
              <span class="badge">{clip.engine === "voicestudio" ? "VoiceStudio" : "yovoice"}</span>
            {/if}
            {#if clip.audioPath}
              <span class="meta">{fmtDur(outDur(clip))}</span>
            {/if}
            <span class="spacer"></span>
            <div class="mv">
              <button title="Pindah ke atas" aria-label="Pindah ke atas" onclick={() => moveClip(track.id, clip.id, -1)} disabled={ci === 0}>↑</button>
              <button title="Pindah ke bawah" aria-label="Pindah ke bawah" onclick={() => moveClip(track.id, clip.id, 1)} disabled={ci === track.clips.length - 1}>↓</button>
              <button title="Pindah ke track atas" aria-label="Pindah ke track atas" onclick={() => moveClipTrack(track.id, clip.id, -1)} disabled={ti === 0}>⇠</button>
              <button title="Pindah ke track bawah" aria-label="Pindah ke track bawah" onclick={() => moveClipTrack(track.id, clip.id, 1)} disabled={ti === tracks.length - 1}>⇢</button>
              <button class="danger" title="Hapus clip" aria-label="Hapus clip" onclick={() => removeClip(track.id, clip.id)}>×</button>
            </div>
          </div>

          <div class="field">
            <label for="vet-{clip.id}">Teks</label>
            <textarea id="vet-{clip.id}" rows="2" bind:value={clip.text}
              oninput={persist} placeholder="Tulis naskah segmen ini…"></textarea>
          </div>

          <div class="grid2">
            <div class="field">
              <label for="vev-{clip.id}">Voice</label>
              <select id="vev-{clip.id}" bind:value={clip.voice} onchange={persist}>
                {#each voices as v}
                  <option value={v.id}>{v.name}{v.lang ? ` (${v.lang})` : ""}</option>
                {/each}
              </select>
            </div>
            <div class="field">
              <label for="ves-{clip.id}">Kecepatan: {clip.speed.toFixed(1)}×</label>
              <input id="ves-{clip.id}" type="range" min="0.5" max="2" step="0.1"
                bind:value={clip.speed} onchange={persist} />
              <p class="micro">{engine === "yovoice" ? "Diterapkan saat putar / mixdown." : "Diterapkan saat Generate."}</p>
            </div>
          </div>

          <div class="field">
            <label for="ver-{clip.id}">File referensi clone (opsional)</label>
            <input id="ver-{clip.id}" type="text" bind:value={clip.refAudio}
              oninput={persist} placeholder="/path/ke/suara-referensi.wav (5–15 detik)" />
          </div>

          <button class="btn-primary sm" onclick={() => generateClip(clip)}
            disabled={!clip.text.trim() || clip.generating || engine === "none"}>
            {clip.generating ? "Membuat suara…" : "✦ Generate"}
          </button>
          {#if clip.error}<p class="err">{clip.error}</p>{/if}

          {#if clip.audioPath}
            <div class="wave-wrap">
              <canvas class="wave" use:regCanvas={clip.id} aria-label="Waveform clip {ci + 1}"></canvas>
            </div>
            <div class="clip-controls">
              <button class="btn-ghost sm" onclick={() => toggleClipPlay(clip)}>
                {playingId === clip.id ? "⏸ Stop" : "▶ Play"}
              </button>
              <button class="btn-ghost sm" onclick={() => masterClip(clip)} disabled={clip.mastering}>
                {clip.mastering ? "Memproses…" : "✨ Master"}
              </button>
              {#if clip.masterPath}
                <button class="btn-ghost sm" onclick={() => playPreview(clip.masterPath, clip.volume)}>
                  ▶ Putar hasil master
                </button>
              {/if}
              <div class="field inline">
                <label for="vevol-{clip.id}">Volume: {Math.round(clip.volume)}</label>
                <input id="vevol-{clip.id}" type="range" min="0" max="100" step="1"
                  bind:value={clip.volume} onchange={persist} />
              </div>
              <div class="field inline">
                <label for="vets-{clip.id}">Mulai (dtk)</label>
                <input id="vets-{clip.id}" type="number" min="0" step="0.1"
                  value={clip.trimStart}
                  onchange={(e) => { clip.trimStart = Math.max(0, Number((e.target as HTMLInputElement).value) || 0); clampTrim(clip); }} />
              </div>
              <div class="field inline">
                <label for="vetl-{clip.id}">Durasi pakai (dtk)</label>
                <input id="vetl-{clip.id}" type="number" min="0" step="0.1"
                  value={clip.trimLen ?? ""} placeholder="penuh"
                  onchange={(e) => {
                    const v = (e.target as HTMLInputElement).value;
                    clip.trimLen = v === "" ? null : Math.max(0.1, Number(v) || 0.1);
                    clampTrim(clip);
                  }} />
              </div>
            </div>
            {#if clip.masterPath}<p class="ok micro">✓ Mastered: <code>{clip.masterPath}</code></p>{/if}
          {/if}
        </article>
      {/each}
    </section>
  {/each}

  <button class="btn-ghost" onclick={addTrack}>+ Tambah Track</button>
  <p class="hint micro">Tips: mastering per clip dulu (tombol ✨ Master), lalu Mixdown — hasil unduhan sudah di-master. Semua tersimpan otomatis di perangkat ini.</p>
</div>

<style>
  .ve { display: flex; flex-direction: column; gap: 16px; }
  .ve-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; flex-wrap: wrap; }
  .ve-head h3 { margin: 0 0 6px; font-size: 20px; }
  .hint { font-size: 14px; color: var(--dim); margin: 0 0 8px; }
  .micro { font-size: 12px; margin: 4px 0 0; }
  .meta { font-size: 13px; color: var(--dim); }
  .spacer { flex: 1; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  @media (max-width: 720px) { .grid2 { grid-template-columns: 1fr; } }

  .engine {
    display: flex; align-items: center; gap: 10px;
    border: 1px solid var(--border); border-radius: 10px;
    padding: 10px 14px; background: var(--bg);
  }
  .engine .dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .engine[data-engine="voicestudio"] .dot { background: #22c55e; }
  .engine[data-engine="yovoice"] .dot { background: #f59e0b; }
  .engine[data-engine="none"] .dot { background: #ef4444; }
  .engine-info { display: flex; flex-direction: column; font-size: 14px; }

  .notice {
    border: 1px solid rgba(239, 68, 68, .4); border-radius: 10px;
    padding: 14px 16px; background: rgba(239, 68, 68, .07);
  }
  .notice p { font-size: 14px; margin: 0 0 10px; }

  .transport {
    display: flex; align-items: center; gap: 10px; flex-wrap: wrap;
    border: 1px solid var(--border); border-radius: 10px;
    padding: 12px 14px; background: var(--bg);
  }

  .track {
    border: 1px solid var(--border); border-radius: 12px;
    padding: 14px; background: var(--card);
  }
  .track-head { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
  .track-head strong { font-size: 16px; }

  .clip {
    border: 1px solid var(--border); border-radius: 10px;
    padding: 14px; margin-bottom: 12px; background: var(--bg);
  }
  .clip:last-child { margin-bottom: 0; }
  .clip.playing { border-color: #4FA6F0; }
  .clip-top { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; }
  .clip-top strong { font-size: 15px; }
  .badge { font-size: 11px; padding: 3px 10px; border-radius: 999px; font-weight: 600;
    background: rgba(79, 166, 240, .15); color: #4FA6F0; }
  .mv { display: flex; gap: 4px; }
  .mv button {
    background: none; border: 1px solid var(--border); border-radius: 6px;
    min-width: 30px; height: 30px; cursor: pointer; color: var(--text); font-size: 14px;
  }
  .mv button:hover:not(:disabled) { background: var(--hover); }
  .mv button:disabled { opacity: .35; cursor: default; }
  .mv button.danger:hover { background: #ef4444; color: #fff; border-color: #ef4444; }

  .field { margin-bottom: 12px; }
  .field label { display: block; font-weight: 600; margin-bottom: 6px; font-size: 14px; }
  .field textarea, .field select, .field input[type="text"], .field input[type="number"] {
    width: 100%; padding: 10px 12px; border: 1px solid var(--border);
    border-radius: 8px; background: var(--bg); color: var(--text); font-size: 14px;
    font-family: inherit;
  }
  .field textarea { resize: vertical; }
  .field input[type="range"] { width: 100%; accent-color: #4FA6F0; height: 32px; }
  .field.inline { margin-bottom: 0; min-width: 130px; flex: 1; }
  .field.inline label { font-size: 13px; font-weight: 600; }

  .wave-wrap { margin: 12px 0 4px; }
  .wave { width: 100%; height: 56px; display: block; border-radius: 8px; }

  .clip-controls { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; margin-top: 8px; }

  .btn-primary {
    background: #3b82f6; color: #fff; border: none; border-radius: 8px;
    font-weight: 600; cursor: pointer; font-size: 15px;
    padding: 10px 22px; min-height: 44px;
  }
  .btn-primary:hover:not(:disabled) { background: #2563eb; }
  .btn-primary:disabled { opacity: .5; cursor: default; }
  .btn-primary.sm { padding: 8px 18px; font-size: 14px; min-height: 40px; }
  .btn-ghost {
    border: 1px solid var(--border); border-radius: 8px; background: transparent;
    color: var(--text); cursor: pointer; font-size: 14px; padding: 8px 16px; min-height: 40px;
  }
  .btn-ghost:hover:not(:disabled) { background: var(--hover); }
  .btn-ghost:disabled { opacity: .5; cursor: default; }
  .btn-ghost.sm { padding: 6px 14px; font-size: 13px; min-height: 34px; }
  .btn-ghost.danger:hover { background: #ef4444; color: #fff; border-color: #ef4444; }

  .ok { color: #22c55e; font-size: 14px; margin: 8px 0 0; }
  .err { color: #ef4444; font-size: 14px; margin: 8px 0 0; }
  code { font-size: 12px; word-break: break-all; }
</style>

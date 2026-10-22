<script lang="ts">
  import { t } from "../lib/i18n";
  import { brandKit, storyboard, scheduleQueue } from "../lib/store";
  import type { Scene } from "../lib/store";

  let prompt = $state("");
  let size = $state("square");
  let generating = $state(false);
  let imageUrl = $state<string | null>(null);
  let error = $state("");
  let dalleKey = $state("");
  let keySaved = $state(false);
  let savingKey = $state(false);

  const SIZES = [
    { id: "square", label: "1:1 (1080×1080)" },
    { id: "portrait", label: "9:16 (1080×1920)" },
    { id: "landscape", label: "16:9 (1920×1080)" },
  ];

  async function saveKey() {
    const k = dalleKey.trim();
    if (!k) return;
    savingKey = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_dalle_key", { apiKey: k });
      keySaved = true;
      dalleKey = "";
    } catch (e) {
      error = String(e);
    } finally {
      savingKey = false;
    }
  }

  async function generate() {
    const p = prompt.trim();
    if (!p || generating) return;
    generating = true;
    error = "";
    imageUrl = null;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const url = await invoke<string>("generate_image", { prompt: p, size });
      imageUrl = url;
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }

  let newVisual = $state("");
  let newNarration = $state("");
  let newDuration = $state(5);

  function addScene() {
    const v = newVisual.trim();
    if (!v) return;
    const id = Date.now();
    storyboard.update((scenes) => [
      ...scenes,
      { id, duration: newDuration, visual: v, narration: newNarration.trim() },
    ]);
    newVisual = "";
    newNarration = "";
    newDuration = 5;
  }

  function removeScene(id: number) {
    storyboard.update((scenes) => scenes.filter((s) => s.id !== id));
  }

  function moveScene(id: number, dir: -1 | 1) {
    storyboard.update((scenes) => {
      const i = scenes.findIndex((s) => s.id === id);
      const j = i + dir;
      if (i < 0 || j < 0 || j >= scenes.length) return scenes;
      const copy = [...scenes];
      [copy[i], copy[j]] = [copy[j], copy[i]];
      return copy;
    });
  }

  let schedTitle = $state("");
  let schedPlatform = $state("instagram");
  let schedAt = $state("");

  function addSchedule() {
    const t = schedTitle.trim();
    if (!t || !schedAt) return;
    scheduleQueue.update((q) => [
      ...q,
      { id: Date.now(), title: t, platform: schedPlatform, scheduledAt: schedAt, status: "scheduled" },
    ]);
    schedTitle = "";
    schedAt = "";
  }

  function removeSchedule(id: number) {
    scheduleQueue.update((q) => q.filter((i) => i.id !== id));
  }

  const STAGES = [
    { id: "goal", label: "Goal & Data" },
    { id: "script", label: "Script" },
    { id: "storyboard", label: "Storyboard" },
    { id: "assets", label: "Aset Visual" },
    { id: "edit", label: "Editing" },
    { id: "schedule", label: "Jadwal" },
  ];
  let stageDone = $state<Record<string, boolean>>({});
  try {
    stageDone = JSON.parse(localStorage.getItem("minime.pipeline") || "{}");
  } catch {}

  function toggleStage(id: string) {
    stageDone[id] = !stageDone[id];
    try {
      localStorage.setItem("minime.pipeline", JSON.stringify(stageDone));
    } catch {}
  }

  function resetPipeline() {
    stageDone = {};
    try {
      localStorage.removeItem("minime.pipeline");
    } catch {}
  }

  function downloadImage() {
    if (!imageUrl) return;
    const a = document.createElement("a");
    a.href = imageUrl;
    a.download = `minime-studio-${Date.now()}.png`;
    a.target = "_blank";
    a.click();
  }
</script>

<div class="studio">
  <header class="studio-head">
    <h2>AI Media Studio</h2>
    <p class="lede">Pipeline: goal → script → storyboard → aset → edit → jadwal.</p>
  </header>

  <section class="card">
    <div class="pipe-head">
      <h3>Pipeline</h3>
      <button class="btn-ghost sm" onclick={resetPipeline}>Reset</button>
    </div>
    <div class="pipeline">
      {#each STAGES as st, i}
        <button
          class="stage {stageDone[st.id] ? 'done' : ''}"
          onclick={() => toggleStage(st.id)}
          aria-pressed={!!stageDone[st.id]}
        >
          <span class="stage-num">{stageDone[st.id] ? "✓" : i + 1}</span>
          <span class="stage-label">{st.label}</span>
        </button>
        {#if i < STAGES.length - 1}<span class="pipe-arrow">→</span>{/if}
      {/each}
    </div>
  </section>

  <section class="card">
    <h3>Brand Kit</h3>
    <p class="hint">Dipakai untuk konsistensi visual di semua konten.</p>
    <div class="brand-grid">
      <div class="field">
        <label for="brand-primary">Warna primer</label>
        <input id="brand-primary" type="color" bind:value={$brandKit.primary} />
      </div>
      <div class="field">
        <label for="brand-secondary">Warna sekunder</label>
        <input id="brand-secondary" type="color" bind:value={$brandKit.secondary} />
      </div>
    </div>
    <div class="field">
      <label for="brand-font">Font</label>
      <select id="brand-font" bind:value={$brandKit.font}>
        <option>Inter</option>
        <option>Poppins</option>
        <option>Montserrat</option>
        <option>Playfair Display</option>
        <option>Roboto</option>
      </select>
    </div>
    <div class="field">
      <label for="brand-tone">Tone of voice</label>
      <input id="brand-tone" type="text" bind:value={$brandKit.tone}
        placeholder="Contoh: profesional dan hangat" />
    </div>
  </section>

  <section class="card">
    <h3>Storyboard</h3>
    <p class="hint">Susun scene untuk video. Urutan bisa diubah dengan tombol ↑ ↓.</p>
    {#if $storyboard.length === 0}
      <p class="hint">Belum ada scene. Tambahkan di bawah.</p>
    {:else}
      <ol class="scene-list">
        {#each $storyboard as sc, i}
          <li class="scene">
            <div class="scene-head">
              <strong>Scene {i + 1}</strong>
              <span class="dur">{sc.duration}s</span>
              <span class="spacer"></span>
              <button class="icon-btn" onclick={() => moveScene(sc.id, -1)} aria-label="Naik">↑</button>
              <button class="icon-btn" onclick={() => moveScene(sc.id, 1)} aria-label="Turun">↓</button>
              <button class="icon-btn danger" onclick={() => removeScene(sc.id)} aria-label="Hapus">×</button>
            </div>
            <p class="scene-visual">{sc.visual}</p>
            {#if sc.narration}<p class="scene-narr">🎙️ {sc.narration}</p>{/if}
          </li>
        {/each}
      </ol>
    {/if}
    <div class="field">
      <label for="sc-visual">Deskripsi visual</label>
      <input id="sc-visual" type="text" bind:value={newVisual}
        placeholder="Contoh: close-up tangan menyerahkan donasi" />
    </div>
    <div class="field">
      <label for="sc-narr">Narasi / VO (opsional)</label>
      <input id="sc-narr" type="text" bind:value={newNarration}
        placeholder="Contoh: Setiap rupiahmu berarti..." />
    </div>
    <div class="row">
      <div class="field" style="flex:1">
        <label for="sc-dur">Durasi (detik)</label>
        <input id="sc-dur" type="number" bind:value={newDuration} min="1" max="60" />
      </div>
      <button class="btn-primary sm" onclick={addScene} disabled={!newVisual.trim()}>
        + Tambah Scene
      </button>
    </div>
  </section>

  <section class="card">
    <h3>API Key DALL-E</h3>
    <p class="hint">Disimpan di OS Keychain, tidak pernah di-hardcode.</p>
    <div class="row">
      <input type="password" bind:value={dalleKey} placeholder="sk-..." aria-label="DALL-E API Key" />
      <button class="btn-primary sm" onclick={saveKey} disabled={!dalleKey.trim() || savingKey}>
        {savingKey ? "..." : "Simpan"}
      </button>
    </div>
    {#if keySaved}<p class="ok">API key tersimpan.</p>{/if}
  </section>

  <section class="card">
    <h3>Generate Gambar</h3>
    <div class="field">
      <label for="studio-prompt">Prompt</label>
      <textarea id="studio-prompt" bind:value={prompt} rows="3"
        placeholder="Contoh: poster minimalis untuk kampanye donasi Ramadhan, warna biru dan emas"></textarea>
    </div>
    <div class="field">
      <label for="studio-size">Ukuran</label>
      <select id="studio-size" bind:value={size}>
        {#each SIZES as s}<option value={s.id}>{s.label}</option>{/each}
      </select>
    </div>
    <button class="btn-primary" onclick={generate} disabled={!prompt.trim() || generating}>
      {generating ? "Membuat gambar..." : "Generate"}
    </button>
    {#if error}<p class="err">{error}</p>{/if}
  </section>

  <section class="card">
    <h3>Jadwal Konten</h3>
    <p class="hint">Dijadwalkan sebagai <strong>draft</strong> — tidak ada auto-posting (aturan keras).</p>
    {#if $scheduleQueue.length === 0}
      <p class="hint">Belum ada jadwal.</p>
    {:else}
      <ul class="sched-list">
        {#each $scheduleQueue as item}
          <li class="sched-item">
            <div>
              <strong>{item.title}</strong>
              <span class="meta">{item.platform} · {item.scheduledAt}</span>
            </div>
            <span class="badge warn">draft</span>
            <button class="icon-btn danger" onclick={() => removeSchedule(item.id)} aria-label="Hapus">×</button>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="field">
      <label for="sched-title">Judul konten</label>
      <input id="sched-title" type="text" bind:value={schedTitle} placeholder="Contoh: Poster donasi Ramadhan" />
    </div>
    <div class="row">
      <div class="field" style="flex:1">
        <label for="sched-platform">Platform</label>
        <select id="sched-platform" bind:value={schedPlatform}>
          <option value="instagram">Instagram</option>
          <option value="threads">Threads</option>
          <option value="facebook">Facebook</option>
        </select>
      </div>
      <div class="field" style="flex:2">
        <label for="sched-at">Jadwal</label>
        <input id="sched-at" type="datetime-local" bind:value={schedAt} />
      </div>
    </div>
    <button class="btn-primary sm" onclick={addSchedule} disabled={!schedTitle.trim() || !schedAt}>
      + Jadwalkan Draft
    </button>
  </section>

  {#if imageUrl}
    <section class="card">
      <h3>Hasil</h3>
      <img src={imageUrl} alt="Hasil generate" class="result-img" />
      <div class="row">
        <button class="btn-ghost sm" onclick={downloadImage}>Unduh</button>
        <a href={imageUrl} target="_blank" rel="noopener" class="btn-ghost sm link">Buka penuh</a>
      </div>
    </section>
  {/if}
</div>

<style>
  .studio { padding: 24px; max-width: 720px; }
  .studio-head h2 { margin: 0 0 4px; font-size: 22px; }
  .lede { color: var(--dim); margin: 0 0 20px; }
  .card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 18px; margin-bottom: 16px; }
  .card h3 { margin: 0 0 8px; font-size: 16px; }
  .hint { font-size: 13px; color: var(--dim); margin: 0 0 12px; }
  .row { display: flex; gap: 8px; align-items: center; }
  .row input { flex: 1; }
  .field { margin-bottom: 12px; }
  .field label { display: block; font-weight: 600; margin-bottom: 6px; font-size: 14px; }
  .field textarea, .field select, .row input {
    width: 100%; padding: 10px 12px; border: 1px solid var(--border);
    border-radius: 8px; background: var(--bg); color: var(--text); font-size: 14px;
  }
  .field textarea { resize: vertical; }
  .btn-primary { background: #3b82f6; color: white; border: none; padding: 10px 20px; border-radius: 8px; font-weight: 600; cursor: pointer; font-size: 14px; }
  .btn-primary:hover:not(:disabled) { background: #2563eb; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }
  .btn-primary.sm { padding: 8px 16px; font-size: 13px; }
  .btn-ghost.sm { padding: 8px 16px; font-size: 13px; border-radius: 8px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; text-decoration: none; display: inline-block; }
  .btn-ghost.sm:hover { background: var(--hover); }
  .ok { color: #22c55e; font-size: 13px; margin-top: 8px; }
  .err { color: #ef4444; font-size: 13px; margin-top: 8px; }
  .result-img { width: 100%; border-radius: 8px; margin-bottom: 12px; border: 1px solid var(--border); }
  .brand-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  input[type="color"] { width: 100%; height: 40px; padding: 4px; cursor: pointer; }
  .scene-list { list-style: none; padding: 0; margin: 0 0 16px; }
  .scene { border: 1px solid var(--border); border-radius: 8px; padding: 12px; margin-bottom: 8px; }
  .scene-head { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
  .scene-head .dur { font-size: 12px; color: var(--dim); background: var(--hover); padding: 2px 8px; border-radius: 999px; }
  .scene-head .spacer { flex: 1; }
  .icon-btn { background: none; border: 1px solid var(--border); border-radius: 6px; width: 28px; height: 28px; cursor: pointer; color: var(--text); }
  .icon-btn:hover { background: var(--hover); }
  .icon-btn.danger:hover { background: #ef4444; color: white; border-color: #ef4444; }
  .scene-visual { margin: 0 0 4px; font-size: 14px; }
  .scene-narr { margin: 0; font-size: 13px; color: var(--dim); font-style: italic; }
  .sched-list { list-style: none; padding: 0; margin: 0 0 16px; }
  .sched-item { display: flex; align-items: center; gap: 8px; border: 1px solid var(--border); border-radius: 8px; padding: 10px 12px; margin-bottom: 8px; }
  .sched-item > div { flex: 1; }
  .sched-item .meta { display: block; font-size: 12px; color: var(--dim); }
  .pipe-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .pipe-head h3 { margin: 0; }
  .pipeline { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .stage { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px solid var(--border); border-radius: 999px; background: transparent; color: var(--text); cursor: pointer; font-size: 13px; }
  .stage:hover { background: var(--hover); }
  .stage.done { background: rgba(34, 197, 94, 0.15); border-color: #22c55e; }
  .stage-num { width: 22px; height: 22px; border-radius: 50%; background: var(--hover); display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700; }
  .stage.done .stage-num { background: #22c55e; color: white; }
  .pipe-arrow { color: var(--dim); }
</style>

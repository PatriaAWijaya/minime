<script lang="ts">
  import { t } from "../lib/i18n";
  import { brandKit, storyboard, scheduleQueue } from "../lib/store";
  import type { Scene } from "../lib/store";
  import ImageEditor from "./ImageEditor.svelte";
  import VideoEditor from "./VideoEditor.svelte";

  let prompt = $state("");
  let size = $state("square");
  let generating = $state(false);
  let imageUrl = $state<string | null>(null);
  let error = $state("");

  const SIZES = [
    { id: "square", label: "1:1 (1080×1080)" },
    { id: "portrait", label: "9:16 (1080×1920)" },
    { id: "landscape", label: "16:9 (1920×1080)" },
  ];

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

  let editingImage = $state<string | null>(null);
  let editingVideo = $state(false);
  let draftApproved = $state(false);

  let postizUrl = $state("");
  let postizKey = $state("");
  let metaPageId = $state("");
  let metaToken = $state("");
  let socialSaved = $state("");
  let savingSocial = $state(false);
  let draftContent = $state("");
  let draftPlatform = $state("postiz");
  let creatingDraft = $state(false);
  let draftResult = $state("");

  async function savePostiz() {
    if (!postizUrl.trim() || !postizKey.trim()) return;
    savingSocial = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_postiz_config", { apiUrl: postizUrl.trim(), apiKey: postizKey.trim() });
      socialSaved = "Postiz tersimpan.";
      postizUrl = ""; postizKey = "";
    } catch (e) { socialSaved = String(e); }
    finally { savingSocial = false; }
  }

  async function saveMeta() {
    if (!metaPageId.trim() || !metaToken.trim()) return;
    savingSocial = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_meta_token", { pageId: metaPageId.trim(), token: metaToken.trim() });
      socialSaved = "Meta tersimpan.";
      metaPageId = ""; metaToken = "";
    } catch (e) { socialSaved = String(e); }
    finally { savingSocial = false; }
  }

  async function createDraft() {
    const c = draftContent.trim();
    if (!c || creatingDraft) return;
    creatingDraft = true;
    draftResult = "";
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      if (draftPlatform === "postiz") {
        const r = await invoke("create_postiz_draft", {
          content: c, platforms: ["instagram"], scheduledAt: null, mediaUrls: [],
        });
        draftResult = "Draft Postiz dibuat: " + r.draft_id;
      } else {
        const r = await invoke("create_meta_draft", {
          pageId: "", message: c, imageUrl: null,
        });
        draftResult = "Draft Meta dibuat: " + r.draft_id;
      }
      draftContent = "";
    } catch (e) { draftResult = String(e); }
    finally { creatingDraft = false; }
  }

  let toolStatus = $state<Array<{id:string;name:string;connected:boolean;detail:string}>>([]);
  let pixabayKey = $state("");
  let savingPx = $state(false);

  async function loadToolStatus() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      toolStatus = await invoke("get_media_tool_status");
    } catch (e) { console.error(e); }
  }

  async function savePixabay() {
    const k = pixabayKey.trim();
    if (!k) return;
    savingPx = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_pixabay_key", { apiKey: k });
      pixabayKey = "";
      await loadToolStatus();
    } catch (e) {}
    finally { savingPx = false; }
  }

  $effect(() => { loadToolStatus(); });

  // Collapsible sections
  let openSections = $state<Record<string, boolean>>({ pipeline: true });
  function toggleSection(id: string) {
    openSections[id] = !openSections[id];
  }

  function approveDraft() {
    draftApproved = true;
    try {
      localStorage.setItem("minime.draft-approved", JSON.stringify({ at: Date.now() }));
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
    <h2>Media Studio</h2>
    <p class="lede">Pipeline: goal → script → storyboard → aset → edit → jadwal.</p>
  </header>

  <div class="duo-row">
  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("pipeline") }>
      <h3>Pipeline</h3>
      <span class="chevron">{openSections["pipeline"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["pipeline"]}
    <div class="card-body">
    <div class="row" style="margin-bottom:12px">
      <span class="hint" style="margin:0">Klik tahap untuk menandai selesai.</span>
      <span style="flex:1"></span>
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
      </div>
    {/if}
  </section>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("koneksi") }>
      <h3>Koneksi Tools</h3>
      <span class="chevron">{openSections["koneksi"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["koneksi"]}
    <div class="card-body">
    <p class="hint">Status koneksi 9 tools Media Studio.</p>
    <div class="conn-list">
      {#each toolStatus as t}
        <div class="conn-row">
          <span class="dot {t.connected ? 'on' : 'off'}"></span>
          <div class="conn-info">
            <strong>{t.name}</strong>
            <span class="conn-detail">{t.detail}</span>
          </div>
          <span class="badge {t.connected ? 'ok' : 'warn'}">{t.connected ? "Terhubung" : "Belum"}</span>
        </div>
      {/each}
    </div>
    <div class="field" style="margin-top:12px">
      <label for="px-key">Pixabay API Key (gratis di pixabay.com/api/docs)</label>
      <div class="row">
        <input id="px-key" type="password" bind:value={pixabayKey} placeholder="API key Pixabay" />
        <button class="btn-primary sm" onclick={savePixabay} disabled={!pixabayKey.trim() || savingPx}>
          {savingPx ? "..." : "Simpan"}
        </button>
      </div>
    </div>
    <button class="btn-ghost sm" onclick={loadToolStatus}>Refresh Status</button>
    </div>
    {/if}
  </section>
  </div>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("brandkit") }>
      <h3>Brand Kit</h3>
      <span class="chevron">{openSections["brandkit"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["brandkit"]}
    <div class="card-body">
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
    <div class="field">
      <label for="brand-photo">Foto / Logo brand</label>
      <div class="photo-row">
        {#if $brandKit.photo}
          <img src={$brandKit.photo} alt="Brand" class="brand-photo" />
          <button class="btn-ghost sm" onclick={() => ($brandKit.photo = "")}>Hapus</button>
        {:else}
          <input id="brand-photo" type="file" accept="image/*"
            onchange={(e) => {
              const f = (e.currentTarget as HTMLInputElement).files?.[0];
              if (!f) return;
              if (f.size > 500 * 1024) { alert("Maksimal 500KB"); return; }
              const r = new FileReader();
              r.onload = () => { $brandKit.photo = r.result as string; };
              r.readAsDataURL(f);
            }} />
        {/if}
      </div>
    </div>
    <div class="field">
      <label for="brand-visual">Grafik / Elemen visual</label>
      <input id="brand-visual" type="text" bind:value={$brandKit.visualElement}
        placeholder="Contoh: pola batik halus, gradien biru, shape organik" />
    </div>
    <div class="field">
      <label for="brand-icon">Gaya ikon</label>
      <select id="brand-icon" bind:value={$brandKit.iconStyle}>
        <option value="minimal">Minimal</option>
        <option value="rounded">Rounded</option>
        <option value="filled">Filled</option>
        <option value="outline">Outline</option>
        <option value="3d">3D</option>
      </select>
    </div>
    <div class="field">
      <label for="brand-layout">Layout</label>
      <select id="brand-layout" bind:value={$brandKit.layout}>
        <option value="center">Tengah</option>
        <option value="left">Kiri</option>
        <option value="grid">Grid</option>
        <option value="split">Split</option>
        <option value="full">Full-bleed</option>
      </select>
    </div>
    <div class="field">
      <label for="brand-template">Template</label>
      <select id="brand-template" bind:value={$brandKit.template}>
        <option value="carousel">Carousel</option>
        <option value="reels">Reels</option>
        <option value="story">Story</option>
        <option value="poster">Poster</option>
        <option value="thumbnail">Thumbnail</option>
      </select>
    </div>
      </div>
    {/if}
  </section>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("storyboard") }>
      <h3>Storyboard</h3>
      <span class="chevron">{openSections["storyboard"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["storyboard"]}
    <div class="card-body">
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
      </div>
    {/if}
  </section>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("draft") }>
      <h3>Review Draft</h3>
      <span class="chevron">{openSections["draft"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["draft"]}
    <div class="card-body">
    {#if $storyboard.length === 0 && !imageUrl}
      <p class="hint">Belum ada draft. Buat storyboard atau generate gambar dulu.</p>
    {:else}
      <p class="hint">
        Storyboard: {$storyboard.length} scene ·
        Gambar: {imageUrl ? "1" : "0"}
      </p>
      <div class="row">
        {#if $storyboard.length > 0}
          <button class="btn-ghost sm" onclick={() => (editingVideo = true)}>Edit Video</button>
        {/if}
        {#if !draftApproved}
          <button class="btn-primary sm" onclick={approveDraft}>Approve → Jadwal</button>
        {:else}
          <span class="ok">✓ Draft disetujui. Lanjut ke penjadwalan di bawah.</span>
        {/if}
      </div>
    {/if}
      </div>
    {/if}
  </section>


  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("genimage") }>
      <h3>Generate Gambar</h3>
      <span class="chevron">{openSections["genimage"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["genimage"]}
    <div class="card-body">
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
      </div>
    {/if}
  </section>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("social") }>
      <h3>Integrasi Postiz / Meta</h3>
      <span class="chevron">{openSections["social"] ? "\u25be" : "\u25b8"}</span>
    </button>
    {#if openSections["social"]}
    <div class="card-body">
    <p class="hint">Hanya membuat <strong>draft</strong> — tidak ada auto-posting.</p>
    <h4>Postiz</h4>
    <div class="field">
      <label for="pz-url">API URL</label>
      <input id="pz-url" type="text" bind:value={postizUrl} placeholder="https://postiz.example.com" />
    </div>
    <div class="field">
      <label for="pz-key">API Key</label>
      <input id="pz-key" type="password" bind:value={postizKey} placeholder="API key Postiz" />
    </div>
    <button class="btn-primary sm" onclick={savePostiz} disabled={!postizUrl.trim() || !postizKey.trim() || savingSocial}>
      Simpan Postiz
    </button>
    <h4 style="margin-top:16px">Meta Business Suite</h4>
    <div class="field">
      <label for="mt-page">Page ID</label>
      <input id="mt-page" type="text" bind:value={metaPageId} placeholder="Facebook Page ID" />
    </div>
    <div class="field">
      <label for="mt-token">Page Access Token</label>
      <input id="mt-token" type="password" bind:value={metaToken} placeholder="Token" />
    </div>
    <button class="btn-primary sm" onclick={saveMeta} disabled={!metaPageId.trim() || !metaToken.trim() || savingSocial}>
      Simpan Meta
    </button>
    {#if socialSaved}<p class="ok">{socialSaved}</p>{/if}
    <h4 style="margin-top:16px">Buat Draft</h4>
    <div class="field">
      <label for="dc-content">Konten</label>
      <textarea id="dc-content" bind:value={draftContent} rows="3" placeholder="Tulis konten..."></textarea>
    </div>
    <div class="field">
      <label for="dc-platform">Platform</label>
      <select id="dc-platform" bind:value={draftPlatform}>
        <option value="postiz">Postiz</option>
        <option value="meta">Meta</option>
      </select>
    </div>
    <button class="btn-primary sm" onclick={createDraft} disabled={!draftContent.trim() || creatingDraft}>
      {creatingDraft ? "Membuat..." : "Buat Draft"}
    </button>
    {#if draftResult}<p class="ok">{draftResult}</p>{/if}
    </div>
    {/if}
  </section>

  <section class="card collapsible">
    <button class="card-toggle" onclick={() => toggleSection("schedule") }>
      <h3>Jadwal Konten</h3>
      <span class="chevron">{openSections["schedule"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["schedule"]}
    <div class="card-body">
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
      </div>
    {/if}
  </section>

  {#if imageUrl}
    <section class="card">
      <h3>Hasil</h3>
      <div class="card-body">
      <img src={imageUrl} alt="Hasil generate" class="result-img" />
      <div class="row">
        <button class="btn-ghost sm" onclick={downloadImage}>Unduh</button>
        <button class="btn-ghost sm" onclick={() => (editingImage = imageUrl)}>Edit</button>
        <a href={imageUrl} target="_blank" rel="noopener" class="btn-ghost sm link">Buka penuh</a>
      </div>
      </div>
  </section>
  {/if}
{#if editingImage}
  <ImageEditor src={editingImage} onClose={() => (editingImage = null)} />
{/if}
{#if editingVideo}
  <VideoEditor scenes={$storyboard} onClose={() => (editingVideo = false)} />
{/if}
</div>

<style>
  .studio { padding: 24px; width: 100%; height: 100%; overflow-y: auto; box-sizing: border-box; }
  .card.collapsible { padding: 0; overflow: hidden; }
  .card-toggle {
    width: 100%; display: flex; align-items: center; justify-content: space-between;
    background: none; border: none; cursor: pointer; padding: 18px; color: var(--text);
  }
  .card-toggle:hover { background: var(--hover); }
  .card-toggle h3 { margin: 0; font-size: 16px; }
  .card-toggle .chevron { font-size: 14px; color: var(--dim); }
  .card-body { padding: 0 18px 18px; }
  .card.collapsible { margin-bottom: 12px; }
  .conn-list { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
  .conn-row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 8px; }
  .conn-row .dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .conn-row .dot.on { background: #22c55e; }
  .conn-row .dot.off { background: #ef4444; }
  .conn-info { flex: 1; display: flex; flex-direction: column; }
  .conn-info strong { font-size: 14px; }
  .conn-detail { font-size: 12px; color: var(--dim); }
  .badge { font-size: 11px; padding: 3px 10px; border-radius: 999px; font-weight: 600; }
  .badge.ok { background: rgba(34,197,94,.15); color: #22c55e; }
  .badge.warn { background: rgba(239,68,68,.15); color: #ef4444; }
  .studio-head h2 { margin: 0 0 4px; font-size: 22px; }
  .lede { color: var(--dim); margin: 0 0 20px; }
  .card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 18px; margin-bottom: 16px; }
  .card h3 { margin: 0 0 8px; font-size: 16px; }
  .hint { font-size: 13px; color: var(--dim); margin: 0 0 12px; }
  .row { display: flex; gap: 8px; align-items: center; }
  .row input { flex: 1; }
  .duo-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: start; margin-bottom: 12px; }
  .duo-row .card.collapsible { margin-bottom: 0; }
  @media (max-width: 860px) { .duo-row { grid-template-columns: 1fr; } }
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
  .photo-row { display: flex; gap: 10px; align-items: center; }
  .brand-photo { width: 64px; height: 64px; object-fit: cover; border-radius: 8px; border: 1px solid var(--border); }
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

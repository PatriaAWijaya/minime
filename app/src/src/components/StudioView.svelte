<script lang="ts">
  import { t } from "../lib/i18n";
  import { brandKit, brandFonts, addBrandFont, removeBrandFont, storyboard, scheduleQueue, userInputs, prodScripts } from "../lib/store";
  import type { Scene } from "../lib/store";
  import ImageEditor from "./ImageEditor.svelte";
  import VideoEditor from "./VideoEditor.svelte";
  import TimelineEditor from "./TimelineEditor.svelte";
  import VoiceEditorView from "./VoiceEditorView.svelte";

  let prompt = $state("");
  let size = $state("square");
  let generating = $state(false);
  let imageUrl = $state<string | null>(null);
  let error = $state("");

  const SIZES = [
    { id: "square", label: "1:1 (1080×1080)" },
    { id: "portrait45", label: "4:5 (1080×1350)" },
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
  let newDialogue = $state("");
  let newAction = $state("");
  let newNotes = $state("");
  let newShotType = $state("medium");
  let editingScene = $state<number | null>(null);

  function addScene() {
    const v = newVisual.trim();
    if (!v) return;
    const id = Date.now();
    storyboard.update((scenes) => [
      ...scenes,
      {
        id, duration: newDuration, visual: v, narration: newNarration.trim(),
        dialogue: newDialogue.trim(), action: newAction.trim(),
        notes: newNotes.trim(), shotType: newShotType,
      },
    ]);
    newVisual = "";
    newNarration = "";
    newDialogue = "";
    newAction = "";
    newNotes = "";
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

  let newInputKind = $state("riset");
  let newInputTitle = $state("");
  let newInputContent = $state("");

  function addInput() {
    const t = newInputTitle.trim();
    const c = newInputContent.trim();
    if (!t || !c) return;
    userInputs.update((r) => [
      ...r,
      { id: Date.now(), kind: newInputKind, title: t, content: c, createdAt: new Date().toISOString() },
    ]);
    newInputTitle = "";
    newInputContent = "";
  }

  function removeInput(id: number) {
    userInputs.update((r) => r.filter((x) => x.id !== id));
  }

  function scriptToStoryboard(script: { content: string }) {
    // Pecah script menjadi scene per baris/paragraph
    const lines = script.content.split(/\n+/).map((l) => l.trim()).filter(Boolean);
    const scenes = lines.map((line, i) => ({
      id: Date.now() + i,
      duration: 5,
      visual: line.slice(0, 120),
      narration: line,
      dialogue: "", action: "", notes: "", shotType: "medium",
    }));
    storyboard.set(scenes);
    prodScripts.update((ps) =>
      ps.map((p) => p.content === script.content ? { ...p, status: "in_production" as const } : p)
    );
  }

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
  let openSections = $state<Record<string, boolean>>({ pipeline: true, planner: true });
  let activeTab = $state("research");
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

  <div class="studio-layout">
    <aside class="sidebar">
  <section class="card collapsible side-card">
    <button class="card-toggle" onclick={() => toggleSection("pipeline") }>
      <h3>Pipeline</h3>
      <span class="chevron">{openSections["pipeline"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["pipeline"]}
    <div class="card-body side-body">
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
  <section class="card collapsible side-card">
    <button class="card-toggle" onclick={() => toggleSection("koneksi") }>
      <h3>Koneksi Tools</h3>
      <span class="chevron">{openSections["koneksi"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["koneksi"]}
    <div class="card-body side-body">
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
  <section class="card collapsible side-card">
    <button class="card-toggle" onclick={() => toggleSection("brandkit") }>
      <h3>Brand Kit</h3>
      <span class="chevron">{openSections["brandkit"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["brandkit"]}
    <div class="card-body side-body">
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
        {#each $brandFonts as f}
          <option value={f}>{f}</option>
        {/each}
      </select>
      <div class="font-add">
        <input id="brand-font-new" type="text" placeholder="Nama font baru..."
          onkeydown={(e) => {
            if (e.key === "Enter") {
              addBrandFont((e.currentTarget as HTMLInputElement).value);
              (e.currentTarget as HTMLInputElement).value = "";
            }
          }} />
        <button class="btn-ghost sm" onclick={() => {
          const el = document.getElementById("brand-font-new") as HTMLInputElement;
          addBrandFont(el.value);
          el.value = "";
        }}>＋ Tambah</button>
      </div>
      {#if !["Inter","Poppins","Montserrat","Playfair Display","Roboto"].includes($brandKit.font)}
        <button class="btn-ghost sm" onclick={() => { removeBrandFont($brandKit.font); $brandKit.font = "Inter"; }}>
          Hapus "{$brandKit.font}"
        </button>
      {/if}
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
  <section class="card collapsible side-card">
    <button class="card-toggle" onclick={() => toggleSection("planner") }>
      <h3>📅 Planner</h3>
      <span class="chevron">{openSections["planner"] ? "▾" : "▸"}</span>
    </button>
    {#if openSections["planner"]}
    <div class="card-body side-body">
    <h4>Integrasi Postiz / Meta</h4>
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
    <hr />
    <h4>Jadwal Konten</h4>
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
    </aside>
    <main class="workspace">
<nav class="tab-nav">
  <button class="tab-btn {activeTab === 'research' ? 'active' : ''}" onclick={() => (activeTab = 'research')}>📥 Input User</button>
  <button class="tab-btn {activeTab === 'scripts' ? 'active' : ''}" onclick={() => (activeTab = 'scripts')}>📝 Script</button>
  <button class="tab-btn {activeTab === 'storyboard' ? 'active' : ''}" onclick={() => (activeTab = 'storyboard')}>🎬 Storyboard</button>
  <button class="tab-btn {activeTab === 'draft' ? 'active' : ''}" onclick={() => (activeTab = 'draft')}>👁 Review Draft</button>
  <button class="tab-btn {activeTab === 'genimage' ? 'active' : ''}" onclick={() => (activeTab = 'genimage')}>🖼 Studio Gambar</button>
  <button class="tab-btn {activeTab === 'suara' ? 'active' : ''}" onclick={() => (activeTab = 'suara')}>🎙 Studio Suara</button>
  <button class="tab-btn {activeTab === 'timeline' ? 'active' : ''}" onclick={() => (activeTab = 'timeline')}>🎞 Studio Video</button>
</nav>      <div class="tab-content">
{#if activeTab === 'research'}
      <div class="tab-pane">
      <p class="hint">Tambahkan apa saja yang diinginkan — selebihnya OTOMATIS ditangani agen Kreatif.</p>
      {#each $userInputs as r}
        <div class="script-item">
          <div class="script-head">
            <strong>{r.title}</strong>
            <span class="tag">{r.kind}</span>
            <span class="spacer"></span>
            <button class="icon-btn danger" onclick={() => removeInput(r.id)} aria-label="Hapus">×</button>
          </div>
          <p class="script-content">{r.content.slice(0, 200)}{r.content.length > 200 ? "..." : ""}</p>
        </div>
      {/each}
      <div class="row">
        <div class="field" style="flex:1">
          <label for="inp-kind">Jenis</label>
          <select id="inp-kind" bind:value={newInputKind}>
            <option value="riset">Riset</option>
            <option value="goal">Goal / Tujuan</option>
            <option value="preferensi">Preferensi</option>
            <option value="materi">Materi mentah</option>
            <option value="lain">Lainnya</option>
          </select>
        </div>
        <div class="field" style="flex:2">
          <label for="inp-title">Judul</label>
          <input id="inp-title" type="text" bind:value={newInputTitle} placeholder="Contoh: Kampanye donasi Ramadhan" />
        </div>
      </div>
      <div class="field">
        <label for="inp-content">Isi</label>
        <textarea id="inp-content" bind:value={newInputContent} rows="4"
          placeholder="Tulis atau tempel di sini..."></textarea>
      </div>
      <button class="btn-primary sm" onclick={addInput} disabled={!newInputTitle.trim() || !newInputContent.trim()}>
        + Tambah Input
      </button>
      </div>
      {/if}
{#if activeTab === 'scripts'}
      <div class="tab-pane">
      <p class="hint">Dihasilkan OTOMATIS oleh agen Kreatif dari input di atas. Setiap skrip otomatis menjadi sumber produksi media.</p>
      {#if $prodScripts.length === 0}
        <p class="hint">Belum ada skrip. Minta agen Kreatif di chat: "kembangkan riset menjadi skrip".</p>
      {/if}
      {#each $prodScripts as sc}
        <div class="script-item">
          <div class="script-head">
            <strong>{sc.title}</strong>
            <span class="tag">{sc.kind}</span>
            <span class="status-{sc.status}">{sc.status === "draft" ? "Draft" : sc.status === "in_production" ? "Produksi" : "Selesai"}</span>
            <span class="spacer"></span>
            <button class="btn-ghost sm" onclick={() => scriptToStoryboard(sc)}>
              → Storyboard
            </button>
          </div>
          <p class="script-content">{sc.content.slice(0, 250)}{sc.content.length > 250 ? "..." : ""}</p>
        </div>
      {/each}
      </div>
      {/if}
{#if activeTab === 'storyboard'}
      <div class="tab-pane">
      <p class="hint">Susun shot per scene ala storyboard film. Klik scene untuk edit detail.</p>

    {#if $storyboard.length > 0}
      <!-- Timeline strip ala Storyboarder -->
      <div class="timeline">
        {#each $storyboard as sc, i}
          <button class="tl-shot" style:flex-grow={sc.duration}
            class:active={editingScene === sc.id}
            onclick={() => (editingScene = editingScene === sc.id ? null : sc.id)}
            title="Scene {i + 1}: {sc.visual}">
            <span class="tl-num">{i + 1}</span>
            <span class="tl-dur">{sc.duration}s</span>
          </button>
        {/each}
      </div>
      <p class="hint">Total durasi: {$storyboard.reduce((a, sc) => a + sc.duration, 0)}s</p>
    {/if}

    {#if $storyboard.length === 0}
      <p class="hint">Belum ada scene. Tambahkan di bawah.</p>
    {:else}
      <ol class="scene-list">
        {#each $storyboard as sc, i}
          <li class="scene">
            <div class="scene-head">
              <strong>Shot {i + 1}</strong>
              <span class="shot-type">{sc.shotType}</span>
              <span class="dur">{sc.duration}s</span>
              <span class="spacer"></span>
              <button class="icon-btn" onclick={() => moveScene(sc.id, -1)} aria-label="Naik">↑</button>
              <button class="icon-btn" onclick={() => moveScene(sc.id, 1)} aria-label="Turun">↓</button>
              <button class="icon-btn" onclick={() => (editingScene = editingScene === sc.id ? null : sc.id)} aria-label="Edit">✎</button>
              <button class="icon-btn danger" onclick={() => removeScene(sc.id)} aria-label="Hapus">×</button>
            </div>
            <p class="scene-visual">🎬 {sc.visual}</p>
            {#if sc.dialogue}<p class="scene-field"><b>Dialog:</b> "{sc.dialogue}"</p>{/if}
            {#if sc.action}<p class="scene-field"><b>Aksi:</b> {sc.action}</p>{/if}
            {#if sc.narration}<p class="scene-narr">🎙️ {sc.narration}</p>{/if}
            {#if editingScene === sc.id}
              <div class="board-editor">
                <div class="field">
                  <label>Dialog</label>
                  <input type="text" bind:value={sc.dialogue} placeholder="Dialog karakter..." />
                </div>
                <div class="field">
                  <label>Aksi</label>
                  <input type="text" bind:value={sc.action} placeholder="Apa yang terjadi..." />
                </div>
                <div class="field">
                  <label>Catatan</label>
                  <input type="text" bind:value={sc.notes} placeholder="Catatan sutradara..." />
                </div>
                <div class="row">
                  <div class="field" style="flex:1">
                    <label>Shot type</label>
                    <select bind:value={sc.shotType}>
                      <option value="wide">Wide</option>
                      <option value="medium">Medium</option>
                      <option value="closeup">Close-up</option>
                      <option value="extreme">Extreme close-up</option>
                    </select>
                  </div>
                  <div class="field" style="flex:1">
                    <label>Durasi (detik)</label>
                    <input type="number" bind:value={sc.duration} min="1" max="60" />
                  </div>
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}

    <div class="field">
      <label for="sc-visual">Deskripsi visual</label>
      <input id="sc-visual" type="text" bind:value={newVisual}
        placeholder="Contoh: close-up tangan menyerahkan donasi" />
    </div>
    <div class="row">
      <div class="field" style="flex:1">
        <label for="sc-shot">Shot type</label>
        <select id="sc-shot" bind:value={newShotType}>
          <option value="wide">Wide</option>
          <option value="medium">Medium</option>
          <option value="closeup">Close-up</option>
          <option value="extreme">Extreme close-up</option>
        </select>
      </div>
      <div class="field" style="flex:1">
        <label for="sc-dur">Durasi (detik)</label>
        <input id="sc-dur" type="number" bind:value={newDuration} min="1" max="60" />
      </div>
    </div>
    <div class="field">
      <label for="sc-dialog">Dialog (opsional)</label>
      <input id="sc-dialog" type="text" bind:value={newDialogue} placeholder="Dialog karakter..." />
    </div>
    <div class="field">
      <label for="sc-action">Aksi (opsional)</label>
      <input id="sc-action" type="text" bind:value={newAction} placeholder="Apa yang terjadi..." />
    </div>
    <div class="field">
      <label for="sc-narr">Narasi / VO (opsional)</label>
      <input id="sc-narr" type="text" bind:value={newNarration}
        placeholder="Contoh: Setiap rupiahmu berarti..." />
    </div>
    <div class="field">
      <label for="sc-notes">Catatan (opsional)</label>
      <input id="sc-notes" type="text" bind:value={newNotes} placeholder="Catatan sutradara..." />
    </div>
    <div class="row">
      <button class="btn-primary sm" onclick={addScene} disabled={!newVisual.trim()}>
        + Tambah Shot
      </button>
    </div>
      </div>
      {/if}
{#if activeTab === 'draft'}
      <div class="tab-pane">
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
{#if activeTab === 'genimage'}
      <div class="tab-pane">
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
{#if activeTab === 'suara'}
      <div class="tab-pane">
      <p class="hint">Voiceover AI lokal + clone suara + mastering otomatis + timeline multi-track.</p>
      <VoiceEditorView />
      </div>
      {/if}
{#if activeTab === 'timeline'}
      <div class="tab-pane">
      <p class="hint">Editor video multi-track ala CapCut: tambah klip video/audio/teks, drag untuk pindah, tarik tepi untuk trim, ✂ untuk pisah, lalu export MP4.</p>
      <TimelineEditor />
      </div>
      {/if}
      </div>
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
    </main>
  </div>
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
  .studio-layout { display: grid; grid-template-columns: 300px 1fr; gap: 16px; align-items: start; }
  .sidebar { display: flex; flex-direction: column; gap: 12px; position: sticky; top: 0; max-height: calc(100vh - 120px); overflow-y: auto; }
  .side-card { margin-bottom: 0 !important; }
  .side-card .card-toggle h3 { font-size: 14px; }
  .side-body { font-size: 13px; }
  .side-body h4 { margin: 12px 0 8px; font-size: 14px; }
  .side-body h4:first-child { margin-top: 0; }
  .side-body hr { border: none; border-top: 1px solid var(--border); margin: 16px 0; }
  .workspace { min-width: 0; }
  .tab-nav { display: flex; gap: 6px; flex-wrap: wrap; margin-bottom: 12px; }
  .tab-btn { padding: 8px 14px; border: 1px solid var(--border); border-radius: 8px; background: var(--card); color: var(--text); font-size: 13px; font-weight: 600; cursor: pointer; }
  .tab-btn:hover { background: var(--hover); }
  .tab-btn.active { background: #4FA6F0; border-color: #4FA6F0; color: #fff; }
  .tab-content { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 18px; }
  .tab-pane { animation: fadeIn 0.2s ease; }
  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @media (max-width: 900px) {
    .studio-layout { grid-template-columns: 1fr; }
    .sidebar { position: static; max-height: none; }
  }
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
  .font-add { display: flex; gap: 8px; margin-top: 8px; }
  .font-add input { flex: 1; padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); font-size: 13px; }
  .photo-row { display: flex; gap: 10px; align-items: center; }
  .brand-photo { width: 64px; height: 64px; object-fit: cover; border-radius: 8px; border: 1px solid var(--border); }
  .brand-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  input[type="color"] { width: 100%; height: 40px; padding: 4px; cursor: pointer; }
  .timeline { display: flex; gap: 4px; margin: 12px 0; }
  .tl-shot {
    min-width: 48px; padding: 8px 4px; border: 1px solid var(--border); border-radius: 6px;
    background: var(--bg); color: var(--text); cursor: pointer;
    display: flex; flex-direction: column; align-items: center; gap: 2px;
  }
  .tl-shot:hover { border-color: #3b82f6; }
  .tl-shot.active { border-color: #3b82f6; background: rgba(59,130,246,.1); }
  .tl-num { font-weight: 700; font-size: 14px; }
  .tl-dur { font-size: 10px; color: var(--dim); font-family: monospace; }
  .shot-type { font-size: 10px; padding: 2px 8px; border-radius: 999px; background: rgba(59,130,246,.15); color: #3b82f6; font-family: monospace; }
  .scene-field { font-size: 13px; margin: 4px 0; color: var(--text); }
  .board-editor { margin-top: 10px; padding: 12px; border: 1px dashed var(--border); border-radius: 8px; background: rgba(0,0,0,.1); }
  .script-item { border: 1px solid var(--border); border-radius: 8px; padding: 12px; margin-bottom: 10px; }
  .script-head { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
  .script-content { font-size: 13px; color: var(--dim); margin: 0; white-space: pre-wrap; }
  .tag { font-size: 10px; padding: 2px 8px; border-radius: 999px; background: rgba(59,130,246,.15); color: #3b82f6; font-family: monospace; }
  .status-draft { font-size: 10px; color: var(--dim); font-family: monospace; }
  .status-in_production { font-size: 10px; color: #f59e0b; font-family: monospace; }
  .status-done { font-size: 10px; color: #22c55e; font-family: monospace; }
  textarea { width: 100%; padding: 9px 12px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); font-size: 13px; font-family: inherit; resize: vertical; }
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

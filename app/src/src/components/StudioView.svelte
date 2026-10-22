<script lang="ts">
  import { t } from "../lib/i18n";

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
    <p class="lede">Generate gambar untuk poster, carousel, dan konten visual.</p>
  </header>

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
</style>

<script lang="ts">
  let { scenes, onClose }: { scenes: Array<{ visual: string; narration: string; duration: number }>; onClose: () => void } = $props();

  let overlayText = $state("");
  let selectedScene = $state(0);
  let totalDuration = $derived(scenes.reduce((a, s) => a + s.duration, 0));

  function fmt(sec: number) {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function exportJson() {
    const data = {
      scenes: scenes.map((s, i) => ({
        ...s,
        overlay: i === selectedScene ? overlayText : undefined,
      })),
      totalDuration,
      exportedAt: new Date().toISOString(),
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `minime-video-${Date.now()}.json`;
    a.click();
  }
</script>

<div class="editor-overlay" onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
  <div class="editor" role="dialog" aria-modal="true" aria-label="Video Editor">
    <header>
      <h3>Edit Video</h3>
      <button class="icon-btn" onclick={onClose} aria-label="Tutup">×</button>
    </header>

    <p class="hint">Total durasi: {fmt(totalDuration)} · {scenes.length} scene</p>

    <div class="timeline">
      {#each scenes as sc, i}
        <button
          class="clip {i === selectedScene ? 'active' : ''}"
          style:flex-grow={sc.duration}
          onclick={() => (selectedScene = i)}
          aria-label="Scene {i + 1}"
        >
          <span class="clip-num">{i + 1}</span>
          <span class="clip-dur">{sc.duration}s</span>
        </button>
      {/each}
    </div>

    {#if scenes[selectedScene]}
      <div class="scene-detail">
        <h4>Scene {selectedScene + 1}</h4>
        <p><strong>Visual:</strong> {scenes[selectedScene].visual}</p>
        {#if scenes[selectedScene].narration}
          <p><strong>Narasi:</strong> {scenes[selectedScene].narration}</p>
        {/if}
        <p><strong>Durasi:</strong> {scenes[selectedScene].duration} detik</p>
      </div>
    {/if}

    <div class="field">
      <label for="ve-overlay">Teks overlay (untuk scene terpilih)</label>
      <input id="ve-overlay" type="text" bind:value={overlayText} placeholder="Contoh: Promo Spesial!" />
    </div>

    <div class="row">
      <button class="btn-primary sm" onclick={exportJson}>Export JSON</button>
      <button class="btn-ghost sm" onclick={onClose}>Tutup</button>
    </div>
    <p class="hint">JSON berisi timeline + overlay, siap untuk Pireel/render.</p>
  </div>
</div>

<style>
  .editor-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .editor { background: var(--card); border: 1px solid var(--border); border-radius: 12px; max-width: 640px; width: 90%; max-height: 90vh; overflow-y: auto; padding: 18px; }
  .editor header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .editor header h3 { margin: 0; }
  .icon-btn { background: none; border: 1px solid var(--border); border-radius: 6px; width: 30px; height: 30px; cursor: pointer; color: var(--text); font-size: 16px; }
  .icon-btn:hover { background: var(--hover); }
  .hint { font-size: 13px; color: var(--dim); margin: 0 0 12px; }
  .timeline { display: flex; gap: 4px; margin-bottom: 16px; min-height: 60px; }
  .clip { border: 1px solid var(--border); border-radius: 8px; background: var(--hover); cursor: pointer; padding: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; min-width: 60px; color: var(--text); }
  .clip:hover { border-color: #3b82f6; }
  .clip.active { border-color: #3b82f6; background: rgba(59, 130, 246, 0.15); }
  .clip-num { font-weight: 700; font-size: 16px; }
  .clip-dur { font-size: 11px; color: var(--dim); }
  .scene-detail { border: 1px solid var(--border); border-radius: 8px; padding: 12px; margin-bottom: 12px; }
  .scene-detail h4 { margin: 0 0 8px; }
  .scene-detail p { margin: 4px 0; font-size: 14px; }
  .field { margin-bottom: 12px; }
  .field label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 4px; }
  .field input { width: 100%; padding: 8px 10px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); }
  .row { display: flex; gap: 8px; }
  .btn-primary.sm { background: #3b82f6; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-weight: 600; cursor: pointer; }
  .btn-ghost.sm { padding: 8px 16px; border: 1px solid var(--border); border-radius: 8px; background: transparent; color: var(--text); cursor: pointer; }
</style>

<script lang="ts">
  let { src, onClose }: { src: string; onClose: () => void } = $props();

  let canvas: HTMLCanvasElement;
  let img: HTMLImageElement;
  let text = $state("");
  let textY = $state(50);
  let filter = $state("none");
  let loaded = $state(false);

  const FILTERS = [
    { id: "none", label: "Normal" },
    { id: "grayscale(100%)", label: "Hitam-Putih" },
    { id: "sepia(60%)", label: "Sepia" },
    { id: "contrast(120%) brightness(110%)", label: "Vivid" },
    { id: "blur(1px) brightness(90%)", label: "Soft" },
  ];

  function loadImage() {
    img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      loaded = true;
      draw();
    };
    img.src = src;
  }

  function draw() {
    if (!canvas || !img) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;
    ctx.filter = filter;
    ctx.drawImage(img, 0, 0);
    ctx.filter = "none";
    if (text.trim()) {
      ctx.font = `bold ${Math.floor(canvas.width / 12)}px Inter, sans-serif`;
      ctx.textAlign = "center";
      ctx.fillStyle = "white";
      ctx.strokeStyle = "black";
      ctx.lineWidth = 4;
      const y = (textY / 100) * canvas.height;
      ctx.strokeText(text, canvas.width / 2, y);
      ctx.fillText(text, canvas.width / 2, y);
    }
  }

  $effect(() => {
    text; textY; filter;
    if (loaded) draw();
  });

  function download() {
    if (!canvas) return;
    const a = document.createElement("a");
    a.href = canvas.toDataURL("image/png");
    a.download = `minime-edit-${Date.now()}.png`;
    a.click();
  }

  $effect(() => { loadImage(); });
</script>

<div class="editor-overlay" onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
  <div class="editor" role="dialog" aria-modal="true" aria-label="Image Editor">
    <header>
      <h3>Edit Gambar</h3>
      <button class="icon-btn" onclick={onClose} aria-label="Tutup">×</button>
    </header>
    <div class="canvas-wrap">
      <canvas bind:this={canvas}></canvas>
      {#if !loaded}<p class="hint">Memuat...</p>{/if}
    </div>
    <div class="controls">
      <div class="field">
        <label for="ie-text">Teks overlay</label>
        <input id="ie-text" type="text" bind:value={text} placeholder="Tulis teks..." />
      </div>
      <div class="field">
        <label for="ie-y">Posisi teks: {textY}%</label>
        <input id="ie-y" type="range" min="5" max="95" bind:value={textY} />
      </div>
      <div class="field">
        <label for="ie-filter">Filter</label>
        <select id="ie-filter" bind:value={filter}>
          {#each FILTERS as f}<option value={f.id}>{f.label}</option>{/each}
        </select>
      </div>
      <div class="row">
        <button class="btn-primary sm" onclick={download}>Unduh Hasil</button>
        <button class="btn-ghost sm" onclick={onClose}>Tutup</button>
      </div>
    </div>
  </div>
</div>

<style>
  .editor-overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.7);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .editor {
    background: var(--card); border: 1px solid var(--border); border-radius: 12px;
    max-width: 640px; width: 90%; max-height: 90vh; overflow-y: auto; padding: 18px;
  }
  .editor header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .editor header h3 { margin: 0; }
  .icon-btn { background: none; border: 1px solid var(--border); border-radius: 6px; width: 30px; height: 30px; cursor: pointer; color: var(--text); font-size: 16px; }
  .icon-btn:hover { background: var(--hover); }
  .canvas-wrap { display: flex; justify-content: center; margin-bottom: 12px; background: #000; border-radius: 8px; overflow: hidden; }
  .canvas-wrap canvas { max-width: 100%; max-height: 50vh; }
  .hint { color: var(--dim); padding: 20px; }
  .controls .field { margin-bottom: 10px; }
  .controls label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 4px; }
  .controls input[type="text"], .controls select { width: 100%; padding: 8px 10px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); }
  .controls input[type="range"] { width: 100%; }
  .row { display: flex; gap: 8px; margin-top: 12px; }
  .btn-primary.sm { background: #3b82f6; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-weight: 600; cursor: pointer; }
  .btn-ghost.sm { padding: 8px 16px; border: 1px solid var(--border); border-radius: 8px; background: transparent; color: var(--text); cursor: pointer; }
</style>

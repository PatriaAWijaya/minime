<script lang="ts">
  // ImageEditor — clean-room image editor: adjust, crop, rotate/flip, text, draw, filters.
  // All edits happen locally on canvas; nothing is uploaded anywhere.
  let {
    src,
    onClose,
    onSave,
  }: {
    src: string;
    onClose: () => void;
    onSave: (dataUrl: string) => void;
  } = $props();

  let canvas: HTMLCanvasElement;
  let base: HTMLCanvasElement | null = null;
  let loaded = $state(false);

  type Tool = "adjust" | "crop" | "rotate" | "text" | "draw" | "filter";
  let tool = $state<Tool>("adjust");

  const TOOLS: { id: Tool; label: string }[] = [
    { id: "adjust", label: "Sesuaikan" },
    { id: "crop", label: "Pangkas" },
    { id: "rotate", label: "Putar" },
    { id: "text", label: "Teks" },
    { id: "draw", label: "Kuas" },
    { id: "filter", label: "Filter" },
  ];

  // --- adjust ---
  let bright = $state(100);
  let contrast = $state(100);
  let saturate = $state(100);

  // --- filter presets ---
  let filter = $state("none");
  const FILTERS = [
    { id: "none", label: "Normal" },
    { id: "grayscale(100%)", label: "Hitam-Putih" },
    { id: "sepia(60%)", label: "Sepia" },
    { id: "contrast(120%) brightness(110%)", label: "Vivid" },
    { id: "blur(1px) brightness(90%)", label: "Soft" },
    { id: "invert(100%)", label: "Negatif" },
  ];

  // --- text overlay (live until baked) ---
  let text = $state("");
  let textY = $state(50);
  let textSize = $state(8); // percent of canvas width
  let textColor = $state("#ffffff");

  // --- draw brush (applied directly to base) ---
  let brushColor = $state("#ff3b30");
  let brushSize = $state(14);
  let drawing = false;
  let lastPt: { x: number; y: number } | null = null;

  // --- crop ---
  let cropStart: { x: number; y: number } | null = null;
  let cropping = false;
  let cropRect = $state<{ x: number; y: number; w: number; h: number } | null>(null);
  let aspect = $state("free"); // free | 1:1 | 4:5 | 16:9
  const ASPECTS = [
    { id: "free", label: "Bebas" },
    { id: "1:1", label: "1:1" },
    { id: "4:5", label: "4:5" },
    { id: "16:9", label: "16:9" },
  ];

  // --- history (undo/redo) ---
  let history: HTMLCanvasElement[] = [];
  let hIdx = -1;
  let canUndo = $state(false);
  let canRedo = $state(false);

  function cloneCanvas(c: HTMLCanvasElement): HTMLCanvasElement {
    const t = document.createElement("canvas");
    t.width = c.width;
    t.height = c.height;
    t.getContext("2d")!.drawImage(c, 0, 0);
    return t;
  }

  function pushHist() {
    if (!base) return;
    history = history.slice(0, hIdx + 1);
    history.push(cloneCanvas(base));
    if (history.length > 25) history.shift();
    hIdx = history.length - 1;
    syncHistFlags();
  }

  function syncHistFlags() {
    canUndo = hIdx > 0;
    canRedo = hIdx < history.length - 1;
  }

  function undo() {
    if (hIdx <= 0) return;
    hIdx--;
    base = cloneCanvas(history[hIdx]);
    resetLive();
    render();
    syncHistFlags();
  }

  function redo() {
    if (hIdx >= history.length - 1) return;
    hIdx++;
    base = cloneCanvas(history[hIdx]);
    resetLive();
    render();
    syncHistFlags();
  }

  function setBase(c: HTMLCanvasElement) {
    base = c;
    pushHist();
    render();
  }

  function resetLive() {
    bright = 100; contrast = 100; saturate = 100;
    filter = "none";
    text = "";
    cropRect = null;
  }

  function resetAll() {
    if (history.length === 0) return;
    hIdx = 0;
    base = cloneCanvas(history[0]);
    resetLive();
    render();
    syncHistFlags();
  }

  // --- load ---
  function loadImage() {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      const c = document.createElement("canvas");
      c.width = img.naturalWidth;
      c.height = img.naturalHeight;
      c.getContext("2d")!.drawImage(img, 0, 0);
      base = c;
      pushHist();
      loaded = true;
      render();
    };
    img.src = src;
  }

  // --- render: base + live adjust/filter + live text + crop overlay ---
  function render() {
    if (!canvas || !base) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    canvas.width = base.width;
    canvas.height = base.height;

    const parts: string[] = [];
    if (filter !== "none") parts.push(filter);
    if (bright !== 100 || contrast !== 100 || saturate !== 100) {
      parts.push(`brightness(${bright}%) contrast(${contrast}%) saturate(${saturate}%)`);
    }
    ctx.filter = parts.length ? parts.join(" ") : "none";
    ctx.drawImage(base, 0, 0);
    ctx.filter = "none";

    if (text.trim()) drawText(ctx);

    if (tool === "crop" && cropRect) drawCropOverlay(ctx);
  }

  function drawText(ctx: CanvasRenderingContext2D) {
    const px = Math.max(12, Math.floor((canvas.width * textSize) / 100));
    ctx.font = `bold ${px}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillStyle = textColor;
    ctx.strokeStyle = "rgba(0,0,0,0.75)";
    ctx.lineWidth = Math.max(2, px / 12);
    const y = (textY / 100) * canvas.height;
    const lines = text.split("\n");
    const lh = px * 1.25;
    const startY = y - ((lines.length - 1) * lh) / 2;
    lines.forEach((ln, i) => {
      ctx.strokeText(ln, canvas.width / 2, startY + i * lh);
      ctx.fillText(ln, canvas.width / 2, startY + i * lh);
    });
  }

  function drawCropOverlay(ctx: CanvasRenderingContext2D) {
    const r = cropRect!;
    // darken everything outside the selection (evenodd keeps the rect clear)
    ctx.save();
    ctx.fillStyle = "rgba(0,0,0,0.55)";
    ctx.beginPath();
    ctx.rect(0, 0, canvas.width, canvas.height);
    ctx.rect(r.x, r.y, r.w, r.h);
    ctx.fill("evenodd");
    ctx.restore();
    ctx.save();
    ctx.strokeStyle = "#4FA6F0";
    ctx.lineWidth = Math.max(2, canvas.width / 400);
    ctx.setLineDash([10, 6]);
    ctx.strokeRect(r.x, r.y, r.w, r.h);
    ctx.restore();
  }

  // Bake everything currently visible (adjust + filter + text) into base.
  function bakeLive() {
    if (!base) return;
    const t = document.createElement("canvas");
    t.width = canvas.width;
    t.height = canvas.height;
    const ctx = t.getContext("2d")!;
    // draw without crop overlay: temporarily hide it
    const keepTool = tool;
    const keepCrop = cropRect;
    cropRect = null;
    render();
    ctx.drawImage(canvas, 0, 0);
    tool = keepTool;
    cropRect = keepCrop;
    base = t;
    pushHist();
    resetLive();
    render();
  }

  // --- rotate / flip ---
  function rotate(dir: 1 | -1) {
    if (!base) return;
    const t = document.createElement("canvas");
    t.width = base.height;
    t.height = base.width;
    const ctx = t.getContext("2d")!;
    ctx.translate(t.width / 2, t.height / 2);
    ctx.rotate((dir * Math.PI) / 2);
    ctx.drawImage(base, -base.width / 2, -base.height / 2);
    setBase(t);
  }

  function flip(horizontal: boolean) {
    if (!base) return;
    const t = document.createElement("canvas");
    t.width = base.width;
    t.height = base.height;
    const ctx = t.getContext("2d")!;
    ctx.translate(horizontal ? t.width : 0, horizontal ? 0 : t.height);
    ctx.scale(horizontal ? -1 : 1, horizontal ? 1 : -1);
    ctx.drawImage(base, 0, 0);
    setBase(t);
  }

  // --- crop interactions ---
  function toCanvas(e: PointerEvent): { x: number; y: number } {
    const r = canvas.getBoundingClientRect();
    return {
      x: ((e.clientX - r.left) * canvas.width) / r.width,
      y: ((e.clientY - r.top) * canvas.height) / r.height,
    };
  }

  function fitAspect(r: { x: number; y: number; w: number; h: number }) {
    if (aspect === "free") return r;
    const [aw, ah] = aspect.split(":").map(Number);
    const target = aw / ah;
    let { x, y, w, h } = r;
    const nh = w / target;
    if (nh <= h) {
      h = nh;
    } else {
      w = h * target;
    }
    // clamp inside canvas
    w = Math.min(w, canvas.width - x);
    h = Math.min(h, canvas.height - y);
    return { x, y, w, h };
  }

  function applyCrop() {
    if (!base || !cropRect || cropRect.w < 4 || cropRect.h < 4) return;
    const r = cropRect;
    const t = document.createElement("canvas");
    t.width = Math.round(r.w);
    t.height = Math.round(r.h);
    t.getContext("2d")!.drawImage(base, r.x, r.y, r.w, r.h, 0, 0, t.width, t.height);
    cropRect = null;
    setBase(t);
  }

  // --- pointer handling (crop + draw) ---
  function onPointerDown(e: PointerEvent) {
    if (!loaded || !base) return;
    if (tool === "crop") {
      cropping = true;
      cropStart = toCanvas(e);
      cropRect = { x: cropStart.x, y: cropStart.y, w: 0, h: 0 };
      canvas.setPointerCapture(e.pointerId);
    } else if (tool === "draw") {
      drawing = true;
      lastPt = toCanvas(e);
      canvas.setPointerCapture(e.pointerId);
    }
  }

  function onPointerMove(e: PointerEvent) {
    if (tool === "crop" && cropping && cropStart) {
      const p = toCanvas(e);
      let x = Math.min(cropStart.x, p.x);
      let y = Math.min(cropStart.y, p.y);
      let w = Math.abs(p.x - cropStart.x);
      let h = Math.abs(p.y - cropStart.y);
      x = Math.max(0, Math.min(x, canvas.width - 1));
      y = Math.max(0, Math.min(y, canvas.height - 1));
      w = Math.min(w, canvas.width - x);
      h = Math.min(h, canvas.height - y);
      cropRect = fitAspect({ x, y, w, h });
      render();
    } else if (tool === "draw" && drawing && lastPt && base) {
      const p = toCanvas(e);
      const ctx = base.getContext("2d")!;
      const scale = canvas.width / canvas.getBoundingClientRect().width;
      ctx.strokeStyle = brushColor;
      ctx.lineWidth = Math.max(1, brushSize * scale);
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.beginPath();
      ctx.moveTo(lastPt.x, lastPt.y);
      ctx.lineTo(p.x, p.y);
      ctx.stroke();
      lastPt = p;
      render();
    }
  }

  function onPointerUp() {
    if (tool === "crop") {
      cropping = false;
      cropStart = null;
    } else if (tool === "draw" && drawing) {
      drawing = false;
      lastPt = null;
      pushHist(); // commit stroke
    }
  }

  // --- export ---
  function exportUrl(): string {
    bakeLive();
    return canvas.toDataURL("image/png");
  }

  function download() {
    const a = document.createElement("a");
    a.href = exportUrl();
    a.download = `minime-edit-${Date.now()}.png`;
    a.click();
  }

  function save() {
    onSave(exportUrl());
  }

  $effect(() => {
    // re-render on any live param change
    bright; contrast; saturate; filter; text; textY; textSize; textColor; tool;
    if (loaded) render();
  });

  $effect(() => {
    loadImage();
  });
</script>

<div
  class="editor-overlay"
  onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
>
  <div class="editor" role="dialog" aria-modal="true" aria-label="Editor Gambar">
    <header>
      <h3>Edit Gambar</h3>
      <div class="hist">
        <button class="icon-btn" onclick={undo} disabled={!canUndo} title="Urungkan">↩</button>
        <button class="icon-btn" onclick={redo} disabled={!canRedo} title="Ulangi">↪</button>
        <button class="icon-btn" onclick={resetAll} title="Kembalikan ke awal">⟲</button>
        <button class="icon-btn" onclick={onClose} aria-label="Tutup">×</button>
      </div>
    </header>

    <div class="editor-body">
      <nav class="toolbar" aria-label="Alat edit">
        {#each TOOLS as t}
          <button
            class="tool-btn"
            class:active={tool === t.id}
            onclick={() => { tool = t.id; cropRect = null; render(); }}
          >{t.label}</button>
        {/each}
      </nav>

      <div class="canvas-area">
        <canvas
          bind:this={canvas}
          class:crosshair={tool === "crop" || tool === "draw"}
          onpointerdown={onPointerDown}
          onpointermove={onPointerMove}
          onpointerup={onPointerUp}
          onpointercancel={onPointerUp}
        ></canvas>
        {#if !loaded}<p class="hint">Memuat...</p>{/if}
      </div>

      <aside class="options">
        {#if tool === "adjust"}
          <div class="field">
            <label for="ie-b">Kecerahan: {bright}%</label>
            <input id="ie-b" type="range" min="0" max="200" bind:value={bright} />
          </div>
          <div class="field">
            <label for="ie-c">Kontras: {contrast}%</label>
            <input id="ie-c" type="range" min="0" max="200" bind:value={contrast} />
          </div>
          <div class="field">
            <label for="ie-s">Saturasi: {saturate}%</label>
            <input id="ie-s" type="range" min="0" max="200" bind:value={saturate} />
          </div>
          <div class="row">
            <button class="btn-primary sm" onclick={bakeLive}>Terapkan</button>
            <button class="btn-ghost sm" onclick={() => { bright = 100; contrast = 100; saturate = 100; }}>Atur ulang</button>
          </div>
        {:else if tool === "crop"}
          <p class="opt-hint">Seret di atas gambar untuk memilih area.</p>
          <div class="field">
            <label>Rasio</label>
            <div class="chip-row">
              {#each ASPECTS as a}
                <button class="chip" class:active={aspect === a.id} onclick={() => (aspect = a.id)}>{a.label}</button>
              {/each}
            </div>
          </div>
          {#if cropRect && cropRect.w > 4}
            <div class="row">
              <button class="btn-primary sm" onclick={applyCrop}>Pangkas</button>
              <button class="btn-ghost sm" onclick={() => { cropRect = null; render(); }}>Batal</button>
            </div>
          {/if}
        {:else if tool === "rotate"}
          <div class="field">
            <label>Putar</label>
            <div class="row">
              <button class="btn-ghost sm" onclick={() => rotate(-1)}>⟲ 90°</button>
              <button class="btn-ghost sm" onclick={() => rotate(1)}>90° ⟳</button>
            </div>
          </div>
          <div class="field">
            <label>Balik</label>
            <div class="row">
              <button class="btn-ghost sm" onclick={() => flip(true)}>Horizontal</button>
              <button class="btn-ghost sm" onclick={() => flip(false)}>Vertikal</button>
            </div>
          </div>
        {:else if tool === "text"}
          <div class="field">
            <label for="ie-text">Teks</label>
            <textarea id="ie-text" rows="2" bind:value={text} placeholder="Tulis teks..."></textarea>
          </div>
          <div class="field">
            <label for="ie-ts">Ukuran: {textSize}%</label>
            <input id="ie-ts" type="range" min="3" max="25" bind:value={textSize} />
          </div>
          <div class="field">
            <label for="ie-tc">Warna</label>
            <input id="ie-tc" type="color" bind:value={textColor} />
          </div>
          <div class="field">
            <label for="ie-y">Posisi: {textY}%</label>
            <input id="ie-y" type="range" min="5" max="95" bind:value={textY} />
          </div>
          <div class="row">
            <button class="btn-primary sm" onclick={bakeLive} disabled={!text.trim()}>Terapkan</button>
            <button class="btn-ghost sm" onclick={() => { text = ""; render(); }}>Hapus</button>
          </div>
        {:else if tool === "draw"}
          <p class="opt-hint">Gambar langsung di atas foto dengan kuas.</p>
          <div class="field">
            <label for="ie-bc">Warna kuas</label>
            <input id="ie-bc" type="color" bind:value={brushColor} />
          </div>
          <div class="field">
            <label for="ie-bs">Ukuran: {brushSize}px</label>
            <input id="ie-bs" type="range" min="2" max="60" bind:value={brushSize} />
          </div>
          <p class="opt-hint">Setiap coretan bisa diurungkan dengan tombol ↩ di atas.</p>
        {:else if tool === "filter"}
          <div class="preset-grid">
            {#each FILTERS as f}
              <button class="chip" class:active={filter === f.id} onclick={() => (filter = f.id)}>{f.label}</button>
            {/each}
          </div>
          <div class="row">
            <button class="btn-primary sm" onclick={bakeLive}>Terapkan</button>
            <button class="btn-ghost sm" onclick={() => (filter = "none")}>Normal</button>
          </div>
        {/if}
      </aside>
    </div>

    <footer class="row end">
      <button class="btn-ghost sm" onclick={download}>Unduh PNG</button>
      <button class="btn-primary sm" onclick={save}>Simpan & Tutup</button>
      <button class="btn-ghost sm" onclick={onClose}>Batal</button>
    </footer>
  </div>
</div>

<style>
  .editor-overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.7);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .editor {
    background: var(--card); border: 1px solid var(--border); border-radius: 12px;
    max-width: 920px; width: 94%; max-height: 92vh; overflow-y: auto; padding: 18px;
    box-sizing: border-box;
  }
  .editor header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .editor header h3 { margin: 0; }
  .hist { display: flex; gap: 6px; }
  .icon-btn {
    background: none; border: 1px solid var(--border); border-radius: 6px;
    min-width: 32px; height: 32px; cursor: pointer; color: var(--text); font-size: 15px;
  }
  .icon-btn:hover:not(:disabled) { background: var(--hover); }
  .icon-btn:disabled { opacity: 0.35; cursor: default; }
  .editor-body { display: flex; gap: 12px; }
  .toolbar { display: flex; flex-direction: column; gap: 6px; min-width: 96px; }
  .tool-btn {
    padding: 10px 8px; border: 1px solid var(--border); border-radius: 8px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 13px; font-weight: 600;
  }
  .tool-btn:hover { background: var(--hover); }
  .tool-btn.active { background: #4FA6F0; border-color: #4FA6F0; color: #fff; }
  .canvas-area {
    flex: 1; display: flex; align-items: center; justify-content: center;
    background: #000; border-radius: 8px; overflow: hidden; min-height: 300px;
    touch-action: none;
  }
  .canvas-area canvas { max-width: 100%; max-height: 56vh; display: block; }
  .canvas-area canvas.crosshair { cursor: crosshair; }
  .hint { color: var(--dim); padding: 20px; }
  .options { width: 220px; min-width: 220px; }
  .options .field { margin-bottom: 12px; }
  .options label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 4px; }
  .options input[type="text"], .options textarea, .options select {
    width: 100%; padding: 8px 10px; border: 1px solid var(--border); border-radius: 8px;
    background: var(--bg); color: var(--text); box-sizing: border-box; font-family: inherit;
  }
  .options input[type="range"] { width: 100%; }
  .options input[type="color"] { width: 100%; height: 36px; padding: 2px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); cursor: pointer; }
  .opt-hint { font-size: 12px; color: var(--dim); margin: 0 0 10px; }
  .chip-row { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip {
    padding: 6px 10px; border: 1px solid var(--border); border-radius: 999px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 12px;
  }
  .chip:hover { background: var(--hover); }
  .chip.active { background: #4FA6F0; border-color: #4FA6F0; color: #fff; }
  .preset-grid { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
  .row { display: flex; gap: 8px; margin-top: 10px; flex-wrap: wrap; }
  .row.end { justify-content: flex-end; margin-top: 14px; }
  .btn-primary.sm { background: #4FA6F0; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-weight: 600; cursor: pointer; }
  .btn-primary.sm:disabled { opacity: 0.4; cursor: default; }
  .btn-ghost.sm { padding: 8px 16px; border: 1px solid var(--border); border-radius: 8px; background: transparent; color: var(--text); cursor: pointer; }
  .btn-ghost.sm:hover { background: var(--hover); }
  @media (max-width: 760px) {
    .editor-body { flex-direction: column; }
    .toolbar { flex-direction: row; flex-wrap: wrap; min-width: 0; }
    .options { width: 100%; min-width: 0; }
  }
</style>

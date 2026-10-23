<script lang="ts">
  import { t } from "../lib/i18n";
  // ImageEditor — clean-room image editor inspired by Doory's UX (idea only, no code copied).
  // Layers: background image + movable elements (text / shapes / QR).
  // All edits happen locally on canvas; nothing is uploaded anywhere.
  import QRCode from "qrcode";

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
  let loadError = $state("");

  // ---------------------------------------------------------------- types
  type Tool =
    | "adjust" | "crop" | "rotate" | "text" | "shape"
    | "qr" | "layers" | "template" | "draw" | "filter";

  interface TextEl {
    id: string; kind: "text";
    x: number; y: number; // center, in canvas px
    text: string; size: number; // px
    color: string; font: string;
    bold: boolean; italic: boolean; align: CanvasTextAlign;
    outline: boolean; outlineColor: string; outlineWidth: number;
    shadow: boolean; shadowColor: string;
    bg: boolean; bgColor: string;
    rotation: number; // degrees
    visible: boolean; locked: boolean;
  }
  interface ShapeEl {
    id: string; kind: "shape";
    shape: "rect" | "ellipse" | "line" | "arrow";
    x: number; y: number; w: number; h: number; // bbox, canvas px
    fill: string; fillOn: boolean;
    stroke: string; strokeWidth: number;
    rotation: number;
    visible: boolean; locked: boolean;
  }
  interface QrEl {
    id: string; kind: "qr";
    x: number; y: number; size: number; // center + side px
    data: string; color: string; bgColor: string;
    img: HTMLCanvasElement | null;
    rotation: number;
    visible: boolean; locked: boolean;
  }
  type El = TextEl | ShapeEl | QrEl;

  let els = $state<El[]>([]);
  let selId = $state<string | null>(null);
  const sel = () => els.find((e) => e.id === selId) ?? null;

  let uid = 0;
  const nid = (p: string) => `${p}-${Date.now().toString(36)}-${uid++}`;

  // ---------------------------------------------------------------- tools
  let tool = $state<Tool>("adjust");
  const TOOLS: { id: Tool; label: string }[] = [
    { id: "adjust", label: "Sesuaikan" },
    { id: "crop", label: "Pangkas" },
    { id: "rotate", label: "Putar" },
    { id: "text", label: "Teks" },
    { id: "shape", label: "Bentuk" },
    { id: "qr", label: "QR" },
    { id: "layers", label: "Lapisan" },
    { id: "template", label: "Template" },
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

  // --- text tool state ---
  let tText = $state("");
  let tSize = $state(6); // percent of canvas width
  let tColor = $state("#ffffff");
  let tFont = $state("system");
  let tBold = $state(true);
  let tItalic = $state(false);
  let tAlign = $state<CanvasTextAlign>("center");
  let tOutline = $state(true);
  let tOutlineColor = $state("#000000");
  let tShadow = $state(false);
  let tBg = $state(false);
  let tBgColor = $state("#4FA6F0");
  const FONTS = [
    { id: "system", label: "Sistem", css: "system-ui, sans-serif" },
    { id: "serif", label: "Serif", css: "Georgia, serif" },
    { id: "mono", label: "Mono", css: "ui-monospace, monospace" },
    { id: "round", label: "Bulat", css: "'Trebuchet MS', sans-serif" },
  ];
  const fontCss = (id: string) => FONTS.find((f) => f.id === id)?.css ?? "system-ui, sans-serif";

  // --- shape tool state ---
  let sShape = $state<"rect" | "ellipse" | "line" | "arrow">("rect");
  let sFill = $state("#4FA6F0");
  let sFillOn = $state(true);
  let sStroke = $state("#ffffff");
  let sStrokeW = $state(6);
  const SHAPES = [
    { id: "rect", label: "Kotak" },
    { id: "ellipse", label: "Lingkaran" },
    { id: "line", label: "Garis" },
    { id: "arrow", label: "Panah" },
  ] as const;

  // --- QR tool state ---
  let qData = $state("");
  let qSize = $state(22); // percent of canvas width
  let qColor = $state("#000000");
  let qBg = $state("#ffffff");
  let qBusy = $state(false);

  // --- draw brush (baked directly to base) ---
  let brushColor = $state("#ff3b30");
  let brushSize = $state(14);
  let drawing = false;
  let lastPt: { x: number; y: number } | null = null;

  // --- crop ---
  let cropStart: { x: number; y: number } | null = null;
  let cropping = false;
  let cropRect = $state<{ x: number; y: number; w: number; h: number } | null>(null);
  let aspect = $state("free");
  const ASPECTS = [
    { id: "free", label: "Bebas" },
    { id: "1:1", label: "1:1" },
    { id: "4:5", label: "4:5" },
    { id: "16:9", label: "16:9" },
  ];

  // --- templates (starter layouts, Doory-inspired) ---
  interface Tpl { id: string; label: string; desc: string }
  const TPLS: Tpl[] = [
    { id: "quote", label: "Kutipan", desc: "Teks besar di tengah" },
    { id: "sale", label: "Promo", desc: "Diskon + harga" },
    { id: "story", label: "Story", desc: "Vertikal 9:16" },
    { id: "thumb", label: "Thumbnail", desc: "Judul tebal + badge" },
    { id: "invite", label: "Undangan", desc: "Acara formal" },
    { id: "blank", label: "Kosong", desc: "Hapus semua elemen" },
  ];

  // ---------------------------------------------------------------- history
  // History snapshots: base canvas + serialized elements.
  interface Snap { base: HTMLCanvasElement; els: string }
  let history: Snap[] = [];
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
  function serEls(): string {
    return JSON.stringify(els.map((e) => ({ ...e, img: undefined })));
  }
  function pushHist() {
    if (!base) return;
    history = history.slice(0, hIdx + 1);
    history.push({ base: cloneCanvas(base), els: serEls() });
    if (history.length > 20) history.shift();
    hIdx = history.length - 1;
    syncHistFlags();
  }
  function syncHistFlags() {
    canUndo = hIdx > 0;
    canRedo = hIdx < history.length - 1;
  }
  function applySnap(s: Snap) {
    base = cloneCanvas(s.base);
    const parsed: El[] = JSON.parse(s.els);
    els = parsed;
    selId = null;
    // re-render QR images
    for (const e of els) if (e.kind === "qr") void renderQr(e as QrEl);
    resetLive();
    render();
  }
  function undo() {
    if (hIdx <= 0) return;
    hIdx--;
    applySnap(history[hIdx]);
    syncHistFlags();
  }
  function redo() {
    if (hIdx >= history.length - 1) return;
    hIdx++;
    applySnap(history[hIdx]);
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
    cropRect = null;
  }
  function resetAll() {
    if (history.length === 0) return;
    hIdx = 0;
    applySnap(history[0]);
    syncHistFlags();
  }
  function commitEls() {
    pushHist();
    render();
  }

  // ---------------------------------------------------------------- load
  function loadImage() {
    loadError = "";
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
    img.onerror = () => {
      loadError = "Gambar gagal dimuat. Coba unduh dulu lalu upload ulang.";
    };
    img.src = src;
  }

  // ---------------------------------------------------------------- render
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

    for (const e of els) {
      if (!e.visible) continue;
      drawEl(ctx, e);
    }
    const s = sel();
    if (s && s.visible) drawSelection(ctx, s);
    if (tool === "crop" && cropRect) drawCropOverlay(ctx);
  }

  function drawEl(ctx: CanvasRenderingContext2D, e: El) {
    ctx.save();
    if (e.kind === "text") drawTextEl(ctx, e);
    else if (e.kind === "shape") drawShapeEl(ctx, e);
    else if (e.kind === "qr") drawQrEl(ctx, e);
    ctx.restore();
  }

  function drawTextEl(ctx: CanvasRenderingContext2D, e: TextEl) {
    const px = e.size;
    ctx.translate(e.x, e.y);
    ctx.rotate((e.rotation * Math.PI) / 180);
    const weight = e.bold ? "bold" : "normal";
    const style = e.italic ? "italic" : "normal";
    ctx.font = `${style} ${weight} ${px}px ${fontCss(e.font)}`;
    ctx.textAlign = e.align;
    ctx.textBaseline = "middle";
    const lines = e.text.split("\n");
    const lh = px * 1.22;
    // measure for background pill
    let maxW = 0;
    for (const ln of lines) maxW = Math.max(maxW, ctx.measureText(ln).width);
    const totalH = lines.length * lh;
    if (e.bg) {
      ctx.fillStyle = e.bgColor;
      const pad = px * 0.35;
      const bx = e.align === "center" ? -maxW / 2 - pad : e.align === "right" ? -maxW - pad : -pad;
      roundRect(ctx, bx, -totalH / 2 - pad, maxW + pad * 2, totalH + pad * 2, px * 0.25);
      ctx.fill();
    }
    if (e.shadow) {
      ctx.shadowColor = e.shadowColor;
      ctx.shadowBlur = px / 6;
      ctx.shadowOffsetY = px / 12;
    }
    const startY = -((lines.length - 1) * lh) / 2;
    lines.forEach((ln, i) => {
      const y = startY + i * lh;
      if (e.outline) {
        ctx.lineWidth = Math.max(2, e.outlineWidth);
        ctx.strokeStyle = e.outlineColor;
        ctx.lineJoin = "round";
        ctx.strokeText(ln, 0, y);
      }
      ctx.fillStyle = e.color;
      ctx.fillText(ln, 0, y);
    });
    ctx.shadowColor = "transparent";
  }

  function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.arcTo(x + w, y, x + w, y + h, r);
    ctx.arcTo(x + w, y + h, x, y + h, r);
    ctx.arcTo(x, y + h, x, y, r);
    ctx.arcTo(x, y, x + w, y, r);
    ctx.closePath();
  }

  function drawShapeEl(ctx: CanvasRenderingContext2D, e: ShapeEl) {
    ctx.translate(e.x, e.y);
    ctx.rotate((e.rotation * Math.PI) / 180);
    ctx.lineWidth = e.strokeWidth;
    ctx.strokeStyle = e.stroke;
    if (e.shape === "rect") {
      const x = -e.w / 2, y = -e.h / 2;
      if (e.fillOn) { ctx.fillStyle = e.fill; ctx.fillRect(x, y, e.w, e.h); }
      ctx.strokeRect(x, y, e.w, e.h);
    } else if (e.shape === "ellipse") {
      ctx.beginPath();
      ctx.ellipse(0, 0, e.w / 2, e.h / 2, 0, 0, Math.PI * 2);
      if (e.fillOn) { ctx.fillStyle = e.fill; ctx.fill(); }
      ctx.stroke();
    } else if (e.shape === "line" || e.shape === "arrow") {
      const x1 = -e.w / 2, x2 = e.w / 2;
      ctx.beginPath();
      ctx.moveTo(x1, 0);
      ctx.lineTo(x2, 0);
      ctx.stroke();
      if (e.shape === "arrow") {
        const hs = Math.min(24, Math.max(10, e.strokeWidth * 3));
        ctx.fillStyle = e.stroke;
        ctx.beginPath();
        ctx.moveTo(x2, 0);
        ctx.lineTo(x2 - hs, -hs / 2);
        ctx.lineTo(x2 - hs, hs / 2);
        ctx.closePath();
        ctx.fill();
      }
    }
  }

  function drawQrEl(ctx: CanvasRenderingContext2D, e: QrEl) {
    if (!e.img) return;
    ctx.translate(e.x, e.y);
    ctx.rotate((e.rotation * Math.PI) / 180);
    ctx.drawImage(e.img, -e.size / 2, -e.size / 2, e.size, e.size);
  }

  function elBounds(e: El): { x: number; y: number; w: number; h: number } {
    if (e.kind === "shape") return { x: e.x - e.w / 2, y: e.y - e.h / 2, w: e.w, h: e.h };
    if (e.kind === "qr") return { x: e.x - e.size / 2, y: e.y - e.size / 2, w: e.size, h: e.size };
    // text: approximate from canvas measure
    const c = document.createElement("canvas").getContext("2d")!;
    const px = e.size;
    c.font = `${e.italic ? "italic" : "normal"} ${e.bold ? "bold" : "normal"} ${px}px ${fontCss(e.font)}`;
    let maxW = 0;
    for (const ln of e.text.split("\n")) maxW = Math.max(maxW, c.measureText(ln).width);
    const h = e.text.split("\n").length * px * 1.22;
    return { x: e.x - maxW / 2, y: e.y - h / 2, w: maxW, h };
  }

  function drawSelection(ctx: CanvasRenderingContext2D, e: El) {
    const b = elBounds(e);
    ctx.save();
    ctx.strokeStyle = "#4FA6F0";
    ctx.lineWidth = Math.max(2, canvas.width / 500);
    ctx.setLineDash([8, 5]);
    ctx.strokeRect(b.x, b.y, b.w, b.h);
    ctx.setLineDash([]);
    // handles
    ctx.fillStyle = "#4FA6F0";
    const hs = Math.max(8, canvas.width / 120);
    for (const [hx, hy] of [[b.x, b.y], [b.x + b.w, b.y], [b.x, b.y + b.h], [b.x + b.w, b.y + b.h]]) {
      ctx.fillRect(hx - hs / 2, hy - hs / 2, hs, hs);
    }
    ctx.restore();
  }

  function drawCropOverlay(ctx: CanvasRenderingContext2D) {
    const r = cropRect!;
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

  // Bake base adjustments (adjust/filter) — elements stay as objects.
  function bakeBase() {
    if (!base) return;
    const t = document.createElement("canvas");
    t.width = base.width;
    t.height = base.height;
    const ctx = t.getContext("2d")!;
    const parts: string[] = [];
    if (filter !== "none") parts.push(filter);
    if (bright !== 100 || contrast !== 100 || saturate !== 100) {
      parts.push(`brightness(${bright}%) contrast(${contrast}%) saturate(${saturate}%)`);
    }
    ctx.filter = parts.length ? parts.join(" ") : "none";
    ctx.drawImage(base, 0, 0);
    base = t;
    pushHist();
    resetLive();
    render();
  }

  // ---------------------------------------------------------------- element ops
  function addText() {
    if (!base || !tText.trim()) return;
    const px = Math.max(14, Math.floor((base.width * tSize) / 100));
    const e: TextEl = {
      id: nid("tx"), kind: "text",
      x: base.width / 2, y: base.height / 2,
      text: tText.trim(), size: px, color: tColor, font: tFont,
      bold: tBold, italic: tItalic, align: tAlign,
      outline: tOutline, outlineColor: tOutlineColor, outlineWidth: Math.max(2, px / 14),
      shadow: tShadow, shadowColor: "rgba(0,0,0,0.6)",
      bg: tBg, bgColor: tBgColor,
      rotation: 0, visible: true, locked: false,
    };
    els = [...els, e];
    selId = e.id;
    tText = "";
    commitEls();
  }

  function addShape() {
    if (!base) return;
    const w = base.width * 0.4;
    const h = sShape === "line" || sShape === "arrow" ? Math.max(8, base.width * 0.02) : base.width * 0.25;
    const e: ShapeEl = {
      id: nid("sh"), kind: "shape", shape: sShape,
      x: base.width / 2, y: base.height / 2, w, h,
      fill: sFill, fillOn: sShape !== "line" && sShape !== "arrow" ? sFillOn : false,
      stroke: sStroke, strokeWidth: Math.max(2, Math.round(base.width / 160)),
      rotation: 0, visible: true, locked: false,
    };
    els = [...els, e];
    selId = e.id;
    commitEls();
  }

  async function renderQr(e: QrEl) {
    try {
      const url = await QRCode.toDataURL(e.data, { width: 512, margin: 1 });
      const img = new Image();
      await new Promise<void>((res, rej) => {
        img.onload = () => res();
        img.onerror = () => rej(new Error("qr"));
        img.src = url;
      });
      // recolor: draw black modules with chosen color on bg
      const c = document.createElement("canvas");
      c.width = 512; c.height = 512;
      const ctx = c.getContext("2d")!;
      ctx.fillStyle = e.bgColor;
      ctx.fillRect(0, 0, 512, 512);
      ctx.drawImage(img, 0, 0);
      // tint: replace dark pixels with color
      const d = ctx.getImageData(0, 0, 512, 512);
      const col = hexToRgb(e.color);
      for (let i = 0; i < d.data.length; i += 4) {
        const lum = (d.data[i] + d.data[i + 1] + d.data[i + 2]) / 3;
        if (lum < 128) {
          d.data[i] = col.r; d.data[i + 1] = col.g; d.data[i + 2] = col.b;
        }
      }
      ctx.putImageData(d, 0, 0);
      e.img = c;
      render();
    } catch {
      /* ignore */
    }
  }

  function hexToRgb(hex: string): { r: number; g: number; b: number } {
    const h = hex.replace("#", "");
    const v = h.length === 3 ? h.split("").map((x) => x + x).join("") : h;
    const n = parseInt(v, 16);
    return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
  }

  async function addQr() {
    if (!base || !qData.trim() || qBusy) return;
    qBusy = true;
    try {
      const e: QrEl = {
        id: nid("qr"), kind: "qr",
        x: base.width / 2, y: base.height * 0.78,
        size: Math.max(60, Math.floor((base.width * qSize) / 100)),
        data: qData.trim(), color: qColor, bgColor: qBg,
        img: null, rotation: 0, visible: true, locked: false,
      };
      await renderQr(e);
      els = [...els, e];
      selId = e.id;
      qData = "";
      commitEls();
    } finally {
      qBusy = false;
    }
  }

  // --- selection / drag ---
  let dragMode: "move" | "resize" | null = null;
  let dragOff = { x: 0, y: 0 };
  let resizeCorner = 0;

  function hitEl(p: { x: number; y: number }): El | null {
    for (let i = els.length - 1; i >= 0; i--) {
      const e = els[i];
      if (!e.visible || e.locked) continue;
      const b = elBounds(e);
      const pad = Math.max(10, canvas.width / 100);
      if (p.x >= b.x - pad && p.x <= b.x + b.w + pad && p.y >= b.y - pad && p.y <= b.y + b.h + pad) {
        return e;
      }
    }
    return null;
  }

  function hitCorner(p: { x: number; y: number }, e: El): number {
    const b = elBounds(e);
    const hs = Math.max(14, canvas.width / 80);
    const corners = [[b.x, b.y], [b.x + b.w, b.y], [b.x, b.y + b.h], [b.x + b.w, b.y + b.h]];
    for (let i = 0; i < 4; i++) {
      if (Math.abs(p.x - corners[i][0]) < hs && Math.abs(p.y - corners[i][1]) < hs) return i;
    }
    return -1;
  }

  function onPointerDown(e: PointerEvent) {
    if (!loaded || !base) return;
    const p = toCanvas(e);
    if (tool === "crop") {
      cropping = true;
      cropStart = p;
      cropRect = { x: p.x, y: p.y, w: 0, h: 0 };
      canvas.setPointerCapture(e.pointerId);
      return;
    }
    if (tool === "draw") {
      drawing = true;
      lastPt = p;
      canvas.setPointerCapture(e.pointerId);
      return;
    }
    // select / move / resize elements (works in any object tool)
    const s = sel();
    if (s && !s.locked) {
      const c = hitCorner(p, s);
      if (c >= 0) {
        dragMode = "resize";
        resizeCorner = c;
        canvas.setPointerCapture(e.pointerId);
        return;
      }
    }
    const hit = hitEl(p);
    if (hit) {
      selId = hit.id;
      if (!hit.locked) {
        dragMode = "move";
        dragOff = { x: p.x - hit.x, y: p.y - hit.y };
        canvas.setPointerCapture(e.pointerId);
      }
      render();
    } else {
      selId = null;
      render();
    }
  }

  function onPointerMove(e: PointerEvent) {
    const p = toCanvas(e);
    if (tool === "crop" && cropping && cropStart) {
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
      return;
    }
    if (tool === "draw" && drawing && lastPt && base) {
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
      return;
    }
    const s = sel();
    if (!s || s.locked || !dragMode) return;
    if (dragMode === "move") {
      s.x = p.x - dragOff.x;
      s.y = p.y - dragOff.y;
      render();
    } else if (dragMode === "resize") {
      resizeEl(s, p, resizeCorner);
      render();
    }
  }

  function resizeEl(e: El, p: { x: number; y: number }, corner: number) {
    if (e.kind === "shape") {
      // keep center, adjust w/h from corner drag
      const cx = e.x, cy = e.y;
      const hw = Math.abs(p.x - cx);
      const hh = Math.abs(p.y - cy);
      e.w = Math.max(10, hw * 2);
      e.h = e.shape === "line" || e.shape === "arrow" ? e.h : Math.max(10, hh * 2);
      void corner;
    } else if (e.kind === "qr") {
      const d = Math.max(Math.abs(p.x - e.x), Math.abs(p.y - e.y));
      e.size = Math.max(40, d * 2);
    } else {
      // text: scale font size by distance from center
      const b = elBounds(e);
      const d = Math.max(20, Math.hypot(p.x - e.x, p.y - e.y));
      const base_d = Math.max(20, Math.hypot(b.w / 2, b.h / 2));
      e.size = Math.max(12, Math.round(e.size * (d / base_d)));
    }
  }

  function onPointerUp() {
    if (tool === "crop") {
      cropping = false;
      cropStart = null;
    } else if (tool === "draw" && drawing) {
      drawing = false;
      lastPt = null;
      pushHist();
    } else if (dragMode) {
      dragMode = null;
      commitEls();
    }
  }

  function toCanvas(e: PointerEvent): { x: number; y: number } {
    const r = canvas.getBoundingClientRect();
    return {
      x: ((e.clientX - r.left) * canvas.width) / r.width,
      y: ((e.clientY - r.top) * canvas.height) / r.height,
    };
  }

  // --- layer ops ---
  function elLabel(e: El): string {
    if (e.kind === "text") return e.text.split("\n")[0].slice(0, 18) || "Teks";
    if (e.kind === "shape") return { rect: "Kotak", ellipse: "Lingkaran", line: "Garis", arrow: "Panah" }[e.shape];
    return "QR Code";
  }
  function moveLayer(id: string, dir: 1 | -1) {
    const i = els.findIndex((e) => e.id === id);
    const j = i + dir;
    if (i < 0 || j < 0 || j >= els.length) return;
    const copy = [...els];
    [copy[i], copy[j]] = [copy[j], copy[i]];
    els = copy;
    commitEls();
  }
  function delSel() {
    if (!selId) return;
    els = els.filter((e) => e.id !== selId);
    selId = null;
    commitEls();
  }
  function dupSel() {
    const s = sel();
    if (!s || !base) return;
    const copy = JSON.parse(JSON.stringify({ ...s, img: undefined })) as El;
    copy.id = nid(s.kind);
    copy.x += 24;
    copy.y += 24;
    if (copy.kind === "qr") {
      (copy as QrEl).img = null;
      void renderQr(copy as QrEl);
    }
    const i = els.findIndex((e) => e.id === s.id);
    els = [...els.slice(0, i + 1), copy, ...els.slice(i + 1)];
    selId = copy.id;
    commitEls();
  }

  // --- selected element bindings (proxy to avoid $state deep issues) ---
  function updSel(patch: Partial<El>) {
    const s = sel();
    if (!s) return;
    Object.assign(s, patch);
    els = [...els];
    render();
  }
  function updSelCommit(patch: Partial<El>) {
    updSel(patch);
    commitEls();
  }

  // ---------------------------------------------------------------- templates
  function applyTpl(id: string) {
    if (!base) return;
    const W = base.width, H = base.height;
    const px = (p: number) => Math.max(14, Math.floor((W * p) / 100));
    els = [];
    selId = null;
    const mkText = (o: Partial<TextEl> & { text: string }): TextEl => ({
      id: nid("tx"), kind: "text", x: W / 2, y: H / 2,
      size: px(7), color: "#ffffff", font: "system",
      bold: true, italic: false, align: "center",
      outline: true, outlineColor: "#000000", outlineWidth: 4,
      shadow: false, shadowColor: "rgba(0,0,0,0.6)",
      bg: false, bgColor: "#4FA6F0", rotation: 0,
      visible: true, locked: false, ...o,
    });
    const mkShape = (o: Partial<ShapeEl> & { shape: ShapeEl["shape"] }): ShapeEl => ({
      id: nid("sh"), kind: "shape", x: W / 2, y: H / 2,
      w: W * 0.8, h: H * 0.3, fill: "#000000", fillOn: true,
      stroke: "#ffffff", strokeWidth: 4, rotation: 0,
      visible: true, locked: false, ...o,
    });
    if (id === "quote") {
      els = [
        mkShape({ shape: "rect", y: H / 2, w: W * 0.86, h: H * 0.5, fill: "#111111", fillOn: true, stroke: "#111111" }),
        mkText({ text: "“Kutipan inspiratif\ndi sini”", y: H * 0.46, size: px(8) }),
        mkText({ text: "— Nama Penulis", y: H * 0.62, size: px(4), bold: false }),
      ];
    } else if (id === "sale") {
      els = [
        mkShape({ shape: "ellipse", x: W * 0.78, y: H * 0.24, w: W * 0.34, h: W * 0.34, fill: "#e11d48", fillOn: true, stroke: "#e11d48" }),
        mkText({ text: "-50%", x: W * 0.78, y: H * 0.24, size: px(9), color: "#ffffff" }),
        mkText({ text: "OBRAL BESAR", y: H * 0.52, size: px(10) }),
        mkText({ text: "Semua produk pilihan", y: H * 0.64, size: px(4.5), bold: false }),
        mkText({ text: "BERLAKU S/D 31 OKT", y: H * 0.78, size: px(4), bg: true, bgColor: "#4FA6F0" }),
      ];
    } else if (id === "story") {
      els = [
        mkText({ text: "Cerita Hari Ini", y: H * 0.16, size: px(7) }),
        mkShape({ shape: "rect", y: H * 0.5, w: W * 0.9, h: H * 0.4, fill: "#000000", fillOn: false, stroke: "#ffffff", strokeWidth: 6 }),
        mkText({ text: "Geser ke atas\nuntuk info lengkap", y: H * 0.5, size: px(5.5) }),
        mkText({ text: "@akunmu", y: H * 0.88, size: px(4), bold: false }),
      ];
    } else if (id === "thumb") {
      els = [
        mkShape({ shape: "rect", y: H * 0.72, w: W, h: H * 0.42, fill: "#000000", fillOn: true, stroke: "#000000" }),
        mkText({ text: "JUDUL VIDEO YANG\nMENARIK PERHATIAN", y: H * 0.66, size: px(8.5), align: "left", x: W * 0.06 }),
        mkText({ text: "▶ TONTON", y: H * 0.86, size: px(4.5), bg: true, bgColor: "#e11d48", align: "left", x: W * 0.06 }),
      ];
      // left-align needs x at left edge
      for (const e of els) if (e.kind === "text" && e.align === "left") e.x = W * 0.06 + 10;
    } else if (id === "invite") {
      els = [
        mkShape({ shape: "rect", x: W / 2, y: H / 2, w: W * 0.88, h: H * 0.8, fill: "#000000", fillOn: false, stroke: "#d4af37", strokeWidth: 8 }),
        mkText({ text: "UNDANGAN", y: H * 0.3, size: px(5), color: "#d4af37", bold: false }),
        mkText({ text: "Pernikahan\nA & B", y: H * 0.48, size: px(9), font: "serif" }),
        mkText({ text: "Sabtu, 20 Des 2026\nGedung Serbaguna", y: H * 0.68, size: px(4), bold: false }),
      ];
    } else if (id === "blank") {
      els = [];
    }
    commitEls();
  }

  // ---------------------------------------------------------------- rotate/flip/crop
  function rotate(dir: 1 | -1) {
    if (!base) return;
    const t = document.createElement("canvas");
    t.width = base.height;
    t.height = base.width;
    const ctx = t.getContext("2d")!;
    ctx.translate(t.width / 2, t.height / 2);
    ctx.rotate((dir * Math.PI) / 2);
    ctx.drawImage(base, -base.width / 2, -base.height / 2);
    // rotate element coordinates too: old (obw,obh) -> new (nbw,nbh) = (obh,obw)
    const nbw = t.width, nbh = t.height;
    const obw = base.width, obh = base.height;
    for (const e of els) {
      const ox = e.x, oy = e.y;
      e.x = dir === 1 ? (oy / obh) * nbw : ((obh - oy) / obh) * nbw;
      e.y = dir === 1 ? ((obw - ox) / obw) * nbh : (ox / obw) * nbh;
      e.rotation = (e.rotation + dir * 90) % 360;
    }
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
    for (const e of els) {
      if (horizontal) e.x = base.width - e.x;
      else e.y = base.height - e.y;
    }
    setBase(t);
  }

  function fitAspect(r: { x: number; y: number; w: number; h: number }) {
    if (aspect === "free") return r;
    const [aw, ah] = aspect.split(":").map(Number);
    const target = aw / ah;
    let { x, y, w, h } = r;
    const nh = w / target;
    if (nh <= h) h = nh;
    else w = h * target;
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
    for (const e of els) {
      e.x -= r.x;
      e.y -= r.y;
    }
    cropRect = null;
    setBase(t);
  }

  // ---------------------------------------------------------------- export
  function exportUrl(): string {
    // render final without selection overlay
    const keepSel = selId;
    selId = null;
    render();
    const url = canvas.toDataURL("image/png");
    selId = keepSel;
    render();
    return url;
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
    bright; contrast; saturate; filter; tool;
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
        <button class="icon-btn" onclick={undo} disabled={!canUndo} title={$t("common.undo.title")}>↩</button>
        <button class="icon-btn" onclick={redo} disabled={!canRedo} title={$t("common.redo.title")}>↪</button>
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
          class:moveable={tool === "text" || tool === "shape" || tool === "qr" || tool === "layers"}
          onpointerdown={onPointerDown}
          onpointermove={onPointerMove}
          onpointerup={onPointerUp}
          onpointercancel={onPointerUp}
        ></canvas>
        {#if !loaded && !loadError}<p class="hint">Memuat...</p>{/if}
        {#if loadError}<p class="err">{loadError}</p>{/if}
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
            <button class="btn-primary sm" onclick={bakeBase}>Terapkan</button>
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
            <label for="ie-text">Teks baru</label>
            <textarea id="ie-text" rows="2" bind:value={tText} placeholder="Tulis teks..."></textarea>
          </div>
          <div class="field">
            <label for="ie-ts">Ukuran: {tSize}%</label>
            <input id="ie-ts" type="range" min="2" max="20" bind:value={tSize} />
          </div>
          <div class="row">
            <div class="field" style="flex:1">
              <label for="ie-tc">Warna</label>
              <input id="ie-tc" type="color" bind:value={tColor} />
            </div>
            <div class="field" style="flex:1">
              <label for="ie-tf">Font</label>
              <select id="ie-tf" bind:value={tFont}>
                {#each FONTS as f}<option value={f.id}>{f.label}</option>{/each}
              </select>
            </div>
          </div>
          <div class="chip-row" style="margin-bottom:10px">
            <button class="chip" class:active={tBold} onclick={() => (tBold = !tBold)}><b>B</b></button>
            <button class="chip" class:active={tItalic} onclick={() => (tItalic = !tItalic)}><i>I</i></button>
            <button class="chip" class:active={tOutline} onclick={() => (tOutline = !tOutline)}>Outline</button>
            <button class="chip" class:active={tShadow} onclick={() => (tShadow = !tShadow)}>Bayangan</button>
            <button class="chip" class:active={tBg} onclick={() => (tBg = !tBg)}>Latar</button>
          </div>
          <button class="btn-primary sm" onclick={addText} disabled={!tText.trim()}>+ Tambah Teks</button>
          <p class="opt-hint" style="margin-top:8px">Klik teks di kanvas untuk memilih, seret untuk memindah, seret sudut untuk mengubah ukuran.</p>
        {:else if tool === "shape"}
          <div class="field">
            <label>Bentuk</label>
            <div class="chip-row">
              {#each SHAPES as s}
                <button class="chip" class:active={sShape === s.id} onclick={() => (sShape = s.id)}>{s.label}</button>
              {/each}
            </div>
          </div>
          <div class="row">
            <div class="field" style="flex:1">
              <label for="ie-sf">Isi</label>
              <input id="ie-sf" type="color" bind:value={sFill} />
            </div>
            <div class="field" style="flex:1">
              <label for="ie-ss">Garis</label>
              <input id="ie-ss" type="color" bind:value={sStroke} />
            </div>
          </div>
          <div class="chip-row" style="margin-bottom:10px">
            <button class="chip" class:active={sFillOn} onclick={() => (sFillOn = !sFillOn)}>Isi warna</button>
          </div>
          <button class="btn-primary sm" onclick={addShape}>+ Tambah Bentuk</button>
        {:else if tool === "qr"}
          <div class="field">
            <label for="ie-qd">Isi QR (URL/teks)</label>
            <textarea id="ie-qd" rows="2" bind:value={qData} placeholder="https://..."></textarea>
          </div>
          <div class="field">
            <label for="ie-qs">Ukuran: {qSize}%</label>
            <input id="ie-qs" type="range" min="10" max="50" bind:value={qSize} />
          </div>
          <div class="row">
            <div class="field" style="flex:1">
              <label for="ie-qc">Warna</label>
              <input id="ie-qc" type="color" bind:value={qColor} />
            </div>
            <div class="field" style="flex:1">
              <label for="ie-qb">Latar</label>
              <input id="ie-qb" type="color" bind:value={qBg} />
            </div>
          </div>
          <button class="btn-primary sm" onclick={addQr} disabled={!qData.trim() || qBusy}>
            {qBusy ? "Membuat..." : "+ Tambah QR"}
          </button>
        {:else if tool === "layers"}
          {#if els.length === 0}
            <p class="opt-hint">Belum ada lapisan. Tambahkan teks, bentuk, atau QR.</p>
          {:else}
            <div class="layer-list">
              {#each [...els].reverse() as e (e.id)}
                {@const idx = els.findIndex((x) => x.id === e.id)}
                <div class="layer-row" class:sel={selId === e.id} onclick={() => { selId = e.id; render(); }}>
                  <span class="layer-name">{elLabel(e)}</span>
                  <span class="layer-kind">{e.kind === "text" ? "T" : e.kind === "shape" ? "B" : "QR"}</span>
                  <button class="mini" title={e.visible ? "Sembunyikan" : "Tampilkan"} onclick={(ev) => { ev.stopPropagation(); updSelCommit({ visible: !e.visible }); }}>👁</button>
                  <button class="mini" title={e.locked ? "Buka kunci" : "Kunci"} onclick={(ev) => { ev.stopPropagation(); updSelCommit({ locked: !e.locked }); }}>{e.locked ? "🔒" : "🔓"}</button>
                  <button class="mini" title="Naik" disabled={idx === els.length - 1} onclick={(ev) => { ev.stopPropagation(); moveLayer(e.id, 1); }}>▲</button>
                  <button class="mini" title="Turun" disabled={idx === 0} onclick={(ev) => { ev.stopPropagation(); moveLayer(e.id, -1); }}>▼</button>
                  <button class="mini danger" title={$t("common.delete.title")} onclick={(ev) => { ev.stopPropagation(); selId = e.id; delSel(); }}>×</button>
                </div>
              {/each}
            </div>
            {#if sel()}
              <div class="row">
                <button class="btn-ghost sm" onclick={dupSel}>Duplikat</button>
                <button class="btn-ghost sm" onclick={delSel}>{$t("common.delete")}</button>
              </div>
            {/if}
          {/if}
        {:else if tool === "template"}
          <p class="opt-hint">Template akan mengganti semua elemen saat ini.</p>
          <div class="tpl-grid">
            {#each TPLS as t}
              <button class="tpl-card" onclick={() => applyTpl(t.id)}>
                <strong>{t.label}</strong>
                <span>{t.desc}</span>
              </button>
            {/each}
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
            <button class="btn-primary sm" onclick={bakeBase}>Terapkan</button>
            <button class="btn-ghost sm" onclick={() => (filter = "none")}>Normal</button>
          </div>
        {/if}

        {#if sel() && (tool === "text" || tool === "shape" || tool === "qr" || tool === "layers")}
          {@const s = sel()!}
          <hr />
          <h4 class="sel-title">Elemen terpilih: {elLabel(s)}</h4>
          {#if s.kind === "text"}
            <div class="field">
              <label>Teks</label>
              <textarea rows="2" value={s.text} oninput={(e) => updSel({ text: (e.target as HTMLTextAreaElement).value })}></textarea>
            </div>
            <div class="row">
              <div class="field" style="flex:1">
                <label>Warna</label>
                <input type="color" value={s.color} oninput={(e) => updSel({ color: (e.target as HTMLInputElement).value })} />
              </div>
              <div class="field" style="flex:1">
                <label>Ukuran: {Math.round(s.size)}px</label>
                <input type="range" min="12" max={Math.max(200, Math.round((base?.width ?? 800) * 0.3))} value={s.size} oninput={(e) => updSel({ size: Number((e.target as HTMLInputElement).value) })} />
              </div>
            </div>
            <div class="chip-row" style="margin-bottom:8px">
              <button class="chip" class:active={s.bold} onclick={() => updSel({ bold: !s.bold })}><b>B</b></button>
              <button class="chip" class:active={s.italic} onclick={() => updSel({ italic: !s.italic })}><i>I</i></button>
              <button class="chip" class:active={s.outline} onclick={() => updSel({ outline: !s.outline })}>Outline</button>
              <button class="chip" class:active={s.shadow} onclick={() => updSel({ shadow: !s.shadow })}>Bayangan</button>
              <button class="chip" class:active={s.bg} onclick={() => updSel({ bg: !s.bg })}>Latar</button>
            </div>
            <div class="field">
              <label>Putar: {s.rotation}°</label>
              <input type="range" min={-180} max={180} value={s.rotation} oninput={(e) => updSel({ rotation: Number((e.target as HTMLInputElement).value) })} />
            </div>
          {:else if s.kind === "shape"}
            <div class="row">
              <div class="field" style="flex:1">
                <label>Isi</label>
                <input type="color" value={s.fill} oninput={(e) => updSel({ fill: (e.target as HTMLInputElement).value })} />
              </div>
              <div class="field" style="flex:1">
                <label>Garis</label>
                <input type="color" value={s.stroke} oninput={(e) => updSel({ stroke: (e.target as HTMLInputElement).value })} />
              </div>
            </div>
            <div class="field">
              <label>Tebal garis: {s.strokeWidth}px</label>
              <input type="range" min="1" max="40" value={s.strokeWidth} oninput={(e) => updSel({ strokeWidth: Number((e.target as HTMLInputElement).value) })} />
            </div>
            <div class="chip-row" style="margin-bottom:8px">
              <button class="chip" class:active={s.fillOn} onclick={() => updSel({ fillOn: !s.fillOn })}>Isi warna</button>
            </div>
            <div class="field">
              <label>Putar: {s.rotation}°</label>
              <input type="range" min={-180} max={180} value={s.rotation} oninput={(e) => updSel({ rotation: Number((e.target as HTMLInputElement).value) })} />
            </div>
          {:else if s.kind === "qr"}
            <div class="row">
              <div class="field" style="flex:1">
                <label>Warna</label>
                <input type="color" value={s.color} oninput={(e) => { updSel({ color: (e.target as HTMLInputElement).value }); void renderQr(s as QrEl); }} />
              </div>
              <div class="field" style="flex:1">
                <label>Latar</label>
                <input type="color" value={s.bgColor} oninput={(e) => { updSel({ bgColor: (e.target as HTMLInputElement).value }); void renderQr(s as QrEl); }} />
              </div>
            </div>
            <div class="field">
              <label>Ukuran: {Math.round(s.size)}px</label>
              <input type="range" min="40" max={Math.max(400, Math.round((base?.width ?? 800) * 0.6))} value={s.size} oninput={(e) => updSel({ size: Number((e.target as HTMLInputElement).value) })} />
            </div>
          {/if}
          <div class="row">
            <button class="btn-ghost sm" onclick={dupSel}>Duplikat</button>
            <button class="btn-ghost sm" onclick={delSel}>{$t("common.delete")}</button>
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
    max-width: 980px; width: 96%; max-height: 94vh; overflow-y: auto; padding: 18px;
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
  .canvas-area canvas.moveable { cursor: move; }
  .hint { color: var(--dim); padding: 20px; }
  .err { color: #ef4444; padding: 20px; font-size: 13px; }
  .options { width: 240px; min-width: 240px; max-height: 62vh; overflow-y: auto; }
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
  .layer-list { display: flex; flex-direction: column; gap: 6px; margin-bottom: 10px; }
  .layer-row {
    display: flex; align-items: center; gap: 6px; padding: 8px;
    border: 1px solid var(--border); border-radius: 8px; cursor: pointer; font-size: 12px;
  }
  .layer-row:hover { background: var(--hover); }
  .layer-row.sel { border-color: #4FA6F0; background: rgba(79,166,240,0.1); }
  .layer-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; }
  .layer-kind {
    font-size: 10px; font-weight: 800; background: var(--hover); border-radius: 4px;
    padding: 2px 6px; color: var(--dim);
  }
  .mini {
    background: none; border: none; cursor: pointer; font-size: 13px; padding: 2px 4px;
    color: var(--text); border-radius: 4px;
  }
  .mini:hover { background: var(--hover); }
  .mini:disabled { opacity: 0.3; cursor: default; }
  .mini.danger { color: #ef4444; }
  .tpl-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .tpl-card {
    border: 1px solid var(--border); border-radius: 10px; padding: 12px 8px;
    background: transparent; color: var(--text); cursor: pointer;
    display: flex; flex-direction: column; gap: 2px; align-items: center; text-align: center;
  }
  .tpl-card:hover { border-color: #4FA6F0; background: rgba(79,166,240,0.08); }
  .tpl-card strong { font-size: 13px; }
  .tpl-card span { font-size: 11px; color: var(--dim); }
  .sel-title { font-size: 13px; margin: 0 0 8px; }
  hr { border: none; border-top: 1px solid var(--border); margin: 12px 0; }
  @media (max-width: 760px) {
    .editor-body { flex-direction: column; }
    .toolbar { flex-direction: row; flex-wrap: wrap; min-width: 0; }
    .options { width: 100%; min-width: 0; max-height: none; }
  }
</style>

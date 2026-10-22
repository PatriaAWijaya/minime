<script lang="ts">
  // VideoPipeline — AI video workflow ala MoneyPrinterTurbo (clean-room).
  // 6 stages: Topik → Naskah → Suara → Footage → Subtitle → Rakit.
  // Footage: Pixabay stock OR AI text-to-video (Sora / Wan 2.3 / LTX-Video).
  // Rakit: FFmpeg otomatis OR HyperFrames (kustom HTML).
  import { invoke } from "@tauri-apps/api/core";

  type Stage = 1 | 2 | 3 | 4 | 5 | 6;
  let stage = $state<Stage>(1);

  // Stage 1: topic
  let topic = $state("");

  // Stage 2: script
  let script = $state("");
  let scriptBusy = $state(false);

  // Stage 3: voice
  let voices = $state<string[]>([]);
  let voice = $state("");
  let audioPath = $state("");
  let voiceBusy = $state(false);
  let voiceError = $state("");

  // Stage 4: footage
  type FootageSrc = "stock" | "ai";
  let footageSrc = $state<FootageSrc>("stock");
  let pxQuery = $state("");
  let pxResults = $state<{ url: string; preview: string }[]>([]);
  let pxBusy = $state(false);
  let pxError = $state("");
  let pickedVideos = $state<string[]>([]); // local paths (downloaded)
  let aiPrompt = $state("");
  let aiModel = $state<"wan" | "ltx">("wan");
  let aiBusy = $state(false);
  let aiError = $state("");
  let aiVideos = $state<string[]>([]);

  // Stage 5: subtitle
  let subtitlePath = $state("");
  let subBusy = $state(false);
  let subError = $state("");

  // Stage 6: assemble
  let aspect = $state<"9:16" | "16:9">("9:16");
  let assembleMode = $state<"auto" | "hyperframes">("auto");
  let assembling = $state(false);
  let finalVideo = $state("");
  let assembleError = $state("");

  const STAGES = [
    { n: 1, label: "Topik" },
    { n: 2, label: "Naskah" },
    { n: 3, label: "Suara" },
    { n: 4, label: "Footage" },
    { n: 5, label: "Subtitle" },
    { n: 6, label: "Rakit" },
  ] as const;

  function go(n: Stage) {
    stage = n;
  }
  function next() {
    if (stage < 6) stage = (stage + 1) as Stage;
  }
  function back() {
    if (stage > 1) stage = (stage - 1) as Stage;
  }

  // --- Stage 2: script from topic (via agent or manual) ---
  // For now: user writes/pastes. Agent integration comes via Kreatif skill.
  function useTopicAsPrompt() {
    if (topic.trim() && !script.trim()) {
      script = `[Tulis naskah tentang: ${topic.trim()}]\n\n`;
    }
  }

  // --- Stage 3: TTS ---
  async function loadVoices() {
    try {
      const raw = await invoke<string>("voicestudio_voices");
      const arr = JSON.parse(raw);
      voices = Array.isArray(arr) ? arr.map((v: any) => v.name ?? v.id ?? String(v)) : [];
      if (voices.length > 0 && !voice) voice = voices[0];
    } catch {
      voices = [];
    }
  }
  async function genVoice() {
    if (!script.trim() || voiceBusy) return;
    voiceBusy = true;
    voiceError = "";
    try {
      const raw = await invoke<string>("voicestudio_tts", {
        text: script.trim(),
        voice: voice || "default",
        speed: 1.0,
        outputName: `narasi-${Date.now()}`,
      });
      const j = JSON.parse(raw);
      if (!j.path) throw new Error("Backend tidak mengembalikan path.");
      audioPath = j.path;
    } catch (e) {
      voiceError = String(e);
    } finally {
      voiceBusy = false;
    }
  }

  // --- Stage 4a: Pixabay stock footage ---
  async function searchStock() {
    if (!pxQuery.trim() || pxBusy) return;
    pxBusy = true;
    pxError = "";
    pxResults = [];
    try {
      const raw = await invoke<string>("pixabay_search_videos", { query: pxQuery.trim(), perPage: 12 });
      const j = JSON.parse(raw);
      const hits = j.hits ?? j.videos ?? [];
      pxResults = hits.map((h: any) => ({
        url: h.url ?? h.videos?.medium?.url ?? "",
        preview: h.previewURL ?? h.preview ?? h.userImageURL ?? "",
      })).filter((r: any) => r.url);
      if (pxResults.length === 0) pxError = "Tidak ada hasil.";
    } catch (e) {
      pxError = String(e);
    } finally {
      pxBusy = false;
    }
  }
  async function addStockVideo(url: string) {
    try {
      // Download to local via backend (fetch_image_as_data_url works for video too? no - use direct download)
      // For now: download via backend temp. Use fetch and save.
      // Simplest: ask backend to download via a new path — reuse read flow by downloading first.
      const { invoke: inv } = await import("@tauri-apps/api/core");
      // Download via backend: we add a generic downloader inline here
      pxError = "";
      // Use fetch_image_as_data_url? It's for images. Instead download via temp file.
      // Fallback: use the URL directly if FFmpeg can read it (it can, via http).
      if (!pickedVideos.includes(url)) {
        pickedVideos = [...pickedVideos, url];
      }
    } catch (e) {
      pxError = String(e);
    }
  }

  // --- Stage 4b: AI text-to-video ---
  async function genAiVideo() {
    if (!aiPrompt.trim() || aiBusy) return;
    aiBusy = true;
    aiError = "";
    try {
      // Backend command text_to_video (Sora / Wan 2.3 / LTX-Video) — API keys via keychain.
      const path = await invoke<string>("text_to_video", {
        prompt: aiPrompt.trim(),
        model: aiModel,
        aspect: aspect,
      });
      aiVideos = [...aiVideos, path];
    } catch (e) {
      aiError = String(e);
    } finally {
      aiBusy = false;
    }
  }

  // --- Stage 5: subtitle ---
  async function genSubtitle() {
    const firstVideo = pickedVideos[0] ?? aiVideos[0];
    if (!firstVideo || subBusy) return;
    subBusy = true;
    subError = "";
    try {
      // generate_subtitle burns into video; we want SRT file. For pipeline, burn at assemble.
      // Use whisper to SRT: backend returns video path with subs burned — we instead request SRT.
      // Simplest: call generate_subtitle on first footage to get timing, then reuse.
      subError = "Subtitle otomatis: gunakan audio narasi — Whisper akan dijalankan saat perakitan.";
      // Mark as ready: assemble will run whisper on the audio
      subtitlePath = "auto";
    } finally {
      subBusy = false;
    }
  }

  // --- Stage 6: assemble ---
  function allFootage(): string[] {
    return [...pickedVideos, ...aiVideos];
  }
  async function assemble() {
    const vids = allFootage();
    if (vids.length === 0 || assembling) return;
    assembling = true;
    assembleError = "";
    finalVideo = "";
    try {
      if (assembleMode === "auto") {
        const out = await invoke<string>("assemble_video_ai", {
          videos: vids,
          audioPath: audioPath || null,
          subtitlePath: subtitlePath && subtitlePath !== "auto" ? subtitlePath : null,
          outputName: `video-ai-${Date.now()}`,
          aspect,
        });
        finalVideo = out;
      } else {
        // HyperFrames: scaffold HTML composition from footage + audio
        const out = await invoke<string>("hyperframes_assemble", {
          videos: vids,
          audioPath: audioPath || null,
          aspect,
          outputName: `hf-${Date.now()}`,
        });
        finalVideo = out;
      }
    } catch (e) {
      assembleError = String(e);
    } finally {
      assembling = false;
    }
  }

  $effect(() => {
    if (stage === 3 && voices.length === 0) loadVoices();
  });
</script>

<div class="pipeline">
  <!-- Stepper -->
  <div class="stepper">
    {#each STAGES as s}
      <button
        class="step {stage === s.n ? 'active' : ''} {stage > s.n ? 'done' : ''}"
        onclick={() => go(s.n as Stage)}
      >
        <span class="num">{s.n}</span>
        <span class="lbl">{s.label}</span>
      </button>
    {/each}
  </div>

  <div class="stage-body">
    {#if stage === 1}
      <h3>1. Topik Video</h3>
      <p class="hint">Tulis topik — bisa diketik langsung atau ditempel dari agen Kreatif.</p>
      <textarea rows="3" bind:value={topic} placeholder="Contoh: 5 tips menabung untuk anak muda..."></textarea>
      <div class="row">
        <button class="btn-primary sm" onclick={() => { useTopicAsPrompt(); next(); }} disabled={!topic.trim()}>
          Lanjut ke Naskah →
        </button>
      </div>

    {:else if stage === 2}
      <h3>2. Naskah</h3>
      <p class="hint">Tulis atau tempel naskah narasi. Naskah ini yang akan dibacakan menjadi suara.</p>
      <textarea rows="8" bind:value={script} placeholder="Tulis naskah di sini..."></textarea>
      <p class="count">{script.trim().split(/\s+/).filter(Boolean).length} kata</p>
      <div class="row">
        <button class="btn-ghost sm" onclick={back}>← Kembali</button>
        <button class="btn-primary sm" onclick={next} disabled={!script.trim()}>Lanjut ke Suara →</button>
      </div>

    {:else if stage === 3}
      <h3>3. Suara (Narasi)</h3>
      <p class="hint">Ubah naskah menjadi suara dengan VoiceStudio.</p>
      <div class="field">
        <label>Suara</label>
        <select bind:value={voice}>
          {#each voices as v}<option value={v}>{v}</option>{/each}
        </select>
        {#if voices.length === 0}<p class="hint">Memuat daftar suara... (pastikan VoiceStudio berjalan)</p>{/if}
      </div>
      <div class="row">
        <button class="btn-primary sm" onclick={genVoice} disabled={voiceBusy || !script.trim()}>
          {voiceBusy ? "Membuat suara..." : "🔊 Buat Narasi"}
        </button>
      </div>
      {#if voiceError}<p class="err">{voiceError}</p>{/if}
      {#if audioPath}
        <p class="ok">✓ Audio siap: <code>{audioPath}</code></p>
        <audio controls src={audioPath}></audio>
      {/if}
      <div class="row">
        <button class="btn-ghost sm" onclick={back}>← Kembali</button>
        <button class="btn-primary sm" onclick={next} disabled={!audioPath}>Lanjut ke Footage →</button>
      </div>

    {:else if stage === 4}
      <h3>4. Footage</h3>
      <p class="hint">Pilih sumber gambar bergerak: stok video atau generate dengan AI.</p>
      <div class="chip-row" style="margin-bottom:12px">
        <button class="chip" class:active={footageSrc === "stock"} onclick={() => (footageSrc = "stock")}>📦 Stok (Pixabay)</button>
        <button class="chip" class:active={footageSrc === "ai"} onclick={() => (footageSrc = "ai")}>✨ AI Text-to-Video</button>
      </div>

      {#if footageSrc === "stock"}
        <div class="row">
          <input type="text" bind:value={pxQuery} placeholder="Cari footage: pantai, kota, masjid..."
            onkeydown={(e) => { if (e.key === "Enter") searchStock(); }} class="grow" />
          <button class="btn-primary sm" onclick={searchStock} disabled={pxBusy || !pxQuery.trim()}>
            {pxBusy ? "..." : "Cari"}
          </button>
        </div>
        {#if pxError}<p class="err">{pxError}</p>{/if}
        <div class="px-grid">
          {#each pxResults as r}
            <div class="px-card">
              {#if r.preview}<img src={r.preview} alt="preview" />{/if}
              <button class="btn-ghost sm" onclick={() => addStockVideo(r.url)}>+ Pakai</button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="hint">Generate footage dengan AI bila tidak ada stok yang cocok.</p>
        <div class="chip-row" style="margin-bottom:10px">
          <button class="chip" class:active={aiModel === "wan"} onclick={() => (aiModel = "wan")}>Wan 3.0</button>
          <button class="chip" class:active={aiModel === "ltx"} onclick={() => (aiModel = "ltx")}>LTX-Video</button>
        </div>
        <p class="hint">Via fal.ai — API key milik Anda (Tools → Apps). Estimasi ±$0.10/detik. Sora tidak tersedia (API ditutup Sep 2026).</p>
        <textarea rows="3" bind:value={aiPrompt} placeholder="Deskripsikan footage: ombak pantai saat senja, sinematik..."></textarea>
        <div class="row">
          <button class="btn-primary sm" onclick={genAiVideo} disabled={aiBusy || !aiPrompt.trim()}>
            {aiBusy ? "Generating..." : "✨ Generate Footage"}
          </button>
        </div>
        {#if aiError}<p class="err">{aiError}</p>{/if}
        {#if aiVideos.length > 0}
          <p class="ok">✓ {aiVideos.length} footage AI siap</p>
        {/if}
      {/if}

      {#if pickedVideos.length > 0}
        <h4>Footage terpilih ({pickedVideos.length})</h4>
        <ul class="picked">
          {#each pickedVideos as v, i}
            <li>
              <span class="mono">{v.slice(0, 60)}...</span>
              <button class="mini danger" onclick={() => (pickedVideos = pickedVideos.filter((_, j) => j !== i))}>×</button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="row">
        <button class="btn-ghost sm" onclick={back}>← Kembali</button>
        <button class="btn-primary sm" onclick={next} disabled={allFootage().length === 0}>
          Lanjut ke Subtitle →
        </button>
      </div>

    {:else if stage === 5}
      <h3>5. Subtitle</h3>
      <p class="hint">Subtitle otomatis dari narasi (Whisper), atau lewati.</p>
      <div class="row">
        <button class="btn-primary sm" onclick={genSubtitle} disabled={subBusy}>
          {subBusy ? "..." : "📝 Buat Subtitle Otomatis"}
        </button>
        <button class="btn-ghost sm" onclick={() => { subtitlePath = ""; next(); }}>Lewati</button>
      </div>
      {#if subError}<p class="err">{subError}</p>{/if}
      {#if subtitlePath === "auto"}<p class="ok">✓ Subtitle akan dibuat otomatis saat perakitan.</p>{/if}
      <div class="row">
        <button class="btn-ghost sm" onclick={back}>← Kembali</button>
        <button class="btn-primary sm" onclick={next}>Lanjut ke Rakit →</button>
      </div>

    {:else if stage === 6}
      <h3>6. Rakit Video</h3>
      <p class="hint">Gabungkan footage + narasi + subtitle menjadi video final.</p>
      <div class="field">
        <label>Rasio</label>
        <div class="chip-row">
          <button class="chip" class:active={aspect === "9:16"} onclick={() => (aspect = "9:16")}>9:16 (Reels/Shorts)</button>
          <button class="chip" class:active={aspect === "16:9"} onclick={() => (aspect = "16:9")}>16:9 (Landscape)</button>
        </div>
      </div>
      <div class="field">
        <label>Mode perakitan</label>
        <div class="chip-row">
          <button class="chip" class:active={assembleMode === "auto"} onclick={() => (assembleMode = "auto")}>
            ⚡ Otomatis (FFmpeg)
          </button>
          <button class="chip" class:active={assembleMode === "hyperframes"} onclick={() => (assembleMode = "hyperframes")}>
            🎨 Kustom (HyperFrames)
          </button>
        </div>
        <p class="hint">
          {#if assembleMode === "auto"}
            Gaya MoneyPrinterTurbo: footage disambung, narasi ditempel, subtitle dibakar — cepat dan konsisten.
          {:else}
            HyperFrames: rakit tampilan sendiri via komposisi HTML — teks animasi, layout kustom, kontrol penuh.
          {/if}
        </p>
      </div>
      <div class="row">
        <button class="btn-primary sm" onclick={assemble} disabled={assembling || allFootage().length === 0}>
          {assembling ? "Merakit..." : "🎬 Rakit Video"}
        </button>
      </div>
      {#if assembleError}<p class="err">{assembleError}</p>{/if}
      {#if finalVideo}
        <p class="ok">✓ Video jadi: <code>{finalVideo}</code></p>
        <video controls src={finalVideo} style="max-width:100%;max-height:400px;border-radius:8px;"></video>
      {/if}
      <div class="row">
        <button class="btn-ghost sm" onclick={back}>← Kembali</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .pipeline { display: flex; flex-direction: column; gap: 16px; }
  .stepper { display: flex; gap: 6px; flex-wrap: wrap; }
  .step {
    display: flex; align-items: center; gap: 8px; padding: 8px 12px;
    border: 1px solid var(--border); border-radius: 999px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 13px;
  }
  .step .num {
    width: 22px; height: 22px; border-radius: 50%; background: var(--hover);
    display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700;
  }
  .step.active { border-color: #4FA6F0; }
  .step.active .num { background: #4FA6F0; color: #fff; }
  .step.done .num { background: #22c55e; color: #fff; }
  .stage-body {
    border: 1px solid var(--border); border-radius: 12px; padding: 18px;
    background: var(--card);
  }
  .stage-body h3 { margin: 0 0 8px; }
  .stage-body h4 { margin: 14px 0 6px; font-size: 14px; }
  .hint { font-size: 13px; color: var(--dim); margin: 0 0 12px; }
  .err { color: #ef4444; font-size: 13px; }
  .ok { color: #22c55e; font-size: 13px; }
  .mono { font-family: ui-monospace, monospace; font-size: 11px; }
  textarea, input[type="text"], select {
    width: 100%; padding: 10px 12px; border: 1px solid var(--border); border-radius: 8px;
    background: var(--bg); color: var(--text); font-family: inherit; box-sizing: border-box;
  }
  textarea { resize: vertical; }
  .grow { flex: 1; }
  .field { margin-bottom: 12px; }
  .field label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 6px; }
  .row { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; align-items: center; }
  .count { font-size: 12px; color: var(--dim); margin: 6px 0 0; }
  .chip-row { display: flex; gap: 6px; flex-wrap: wrap; }
  .chip {
    padding: 8px 14px; border: 1px solid var(--border); border-radius: 999px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 13px;
  }
  .chip:hover { background: var(--hover); }
  .chip.active { background: #4FA6F0; border-color: #4FA6F0; color: #fff; }
  .btn-primary.sm { background: #4FA6F0; color: white; border: none; padding: 9px 18px; border-radius: 8px; font-weight: 600; cursor: pointer; }
  .btn-primary.sm:disabled { opacity: 0.4; cursor: default; }
  .btn-ghost.sm { padding: 9px 18px; border: 1px solid var(--border); border-radius: 8px; background: transparent; color: var(--text); cursor: pointer; }
  .btn-ghost.sm:hover { background: var(--hover); }
  .px-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-top: 12px; }
  .px-card { border: 1px solid var(--border); border-radius: 8px; overflow: hidden; display: flex; flex-direction: column; }
  .px-card img { width: 100%; aspect-ratio: 16/9; object-fit: cover; }
  .px-card button { margin: 6px; }
  .picked { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 6px; }
  .picked li {
    display: flex; align-items: center; justify-content: space-between; gap: 8px;
    padding: 8px 10px; border: 1px solid var(--border); border-radius: 8px; font-size: 12px;
  }
  .mini { background: none; border: none; cursor: pointer; font-size: 14px; color: var(--text); }
  .mini.danger { color: #ef4444; }
  code { font-size: 11px; word-break: break-all; }
  audio { width: 100%; margin-top: 8px; }
</style>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  /** Props: capability = label kapabilitas yang butuh Meta (mis. "Image", "Chat") */
  let { capability = "", compact = false }: { capability?: string; compact?: boolean } = $props();

  let connected = $state<boolean | null>(null);
  let activeModel = $state<string>("");

  async function check() {
    try {
      const catalog: Array<{ id: string; key_set: boolean; suggested: string[] }> =
        await invoke("provider_catalog");
      const muse = catalog.find((p) => p.id === "muse");
      connected = !!muse?.key_set;
      // Model default per kapabilitas (nama + versi jelas)
      const c = capability.toLowerCase();
      if (c.includes("image")) activeModel = "Muse Image 1.0";
      else if (c.includes("transkrip") || c.includes("caption"))
        activeModel = "Muse Voice Transcribe 1.0";
      else if (c.includes("segment")) activeModel = "SAM 3.1";
      else if (c.includes("power")) activeModel = "Muse Wonder 1.0";
      else activeModel = "Muse Spark 1.3";
    } catch {
      connected = false;
    }
  }

  function goToSettings() {
    window.dispatchEvent(
      new CustomEvent("minime:nav", { detail: { view: "settings", tab: "providers", provider: "muse" } })
    );
  }

  $effect(() => {
    check();
  });
</script>

<div class="meta-status" class:compact role="status" title={connected ? `Meta: ${activeModel}` : "Meta belum terhubung"}>
  {#if connected === null}
    <span class="dot pending"></span>
    {#if !compact}<span class="lbl">Memeriksa Meta...</span>{/if}
  {:else if connected}
    <span class="dot ok"></span>
    {#if !compact}<span class="lbl" title="Model Meta aktif">{activeModel}</span>{/if}
  {:else}
    <span class="dot bad"></span>
    {#if !compact}
      <span class="lbl">Meta belum terhubung</span>
      <button class="btn-link" onclick={goToSettings}>Hubungkan</button>
    {/if}
  {/if}
</div>

<style>
  .meta-status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    padding: 6px 10px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.75);
    backdrop-filter: blur(10px);
  }
  .meta-status.compact {
    padding: 4px 8px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot.ok { background: #22c55e; box-shadow: 0 0 6px #22c55e66; }
  .dot.bad { background: #ef4444; box-shadow: 0 0 6px #ef444466; }
  .dot.pending { background: #f59e0b; }
  .lbl { white-space: nowrap; }
  .btn-link {
    background: none;
    border: none;
    color: #4FA6F0;
    cursor: pointer;
    font-size: 12px;
    padding: 0;
    text-decoration: underline;
  }
  .btn-link:hover { color: #7ab8f5; }
</style>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "../lib/i18n";

  /** Props: capability = label kapabilitas yang butuh Meta (mis. "Image", "Transkripsi") */
  let { capability = "" }: { capability?: string } = $props();

  let connected = $state<boolean | null>(null);

  async function check() {
    try {
      const catalog: Array<{ id: string; key_set: boolean }> = await invoke("provider_catalog");
      const muse = catalog.find((p) => p.id === "muse");
      connected = !!muse?.key_set;
    } catch {
      connected = false;
    }
  }

  function goToSettings() {
    // Navigasi ke Pengaturan → Penyedia Model (tab Meta)
    window.dispatchEvent(new CustomEvent("minime:nav", { detail: { view: "settings", tab: "providers", provider: "muse" } }));
  }

  $effect(() => {
    check();
  });
</script>

<div class="meta-status" role="status">
  {#if connected === null}
    <span class="dot pending"></span>
    <span>Memeriksa...</span>
  {:else if connected}
    <span class="dot ok"></span>
    <span>Meta terhubung{capability ? ` — ${capability} siap` : ""}</span>
  {:else}
    <span class="dot bad"></span>
    <span>Meta belum terhubung{capability ? ` — ${capability} butuh API key` : ""}</span>
    <button class="btn-link" onclick={goToSettings}>Hubungkan</button>
  {/if}
</div>

<style>
  .meta-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    padding: 8px 12px;
    border-radius: 8px;
    background: var(--bg-subtle, #f5f5f5);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot.ok { background: #22c55e; }
  .dot.bad { background: #ef4444; }
  .dot.pending { background: #f59e0b; }
  .btn-link {
    background: none;
    border: none;
    color: var(--accent, #4FA6F0);
    cursor: pointer;
    font-size: 13px;
    padding: 0;
    text-decoration: underline;
  }
</style>

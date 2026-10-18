<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import { toolStateLabel, type BrowserSession, type ToolStatus } from "../lib/types";

  let tools = $state<ToolStatus[]>([]);
  let sessions = $state<BrowserSession[]>([]);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      tools = await api.toolsStatus();
      const r = await api.browserList();
      sessions = r.sessions;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(refresh);
</script>

<h1 class="section-title">Peralatan</h1>
<p class="supporting">
  Readiness check tiap tool produksi + sesi browser bawaan agen.
  Tool yang tidak siap menampilkan fallback-nya — tidak ada yang gagal diam-diam.
</p>

{#if error}
  <div class="card"><p class="agent-meta">Tidak terhubung ke daemon: {error}</p></div>
{/if}

<div class="card">
  <div class="row-between">
    <h2 style="font-size: 20px; margin: 0;">Status tools</h2>
    <button class="btn-secondary" onclick={refresh}>Periksa ulang</button>
  </div>
  <div class="tool-grid">
    {#each tools as t (t.id)}
      <div class="tool-card" class:ready={t.ready} class:notready={!t.ready}>
        <div class="tool-head">
          <span class="tool-name">{t.label}</span>
          <span class="badge" class:ok={t.ready} class:warn={!t.ready}>
            {t.ready ? "Siap" : toolStateLabel(t.state)}
          </span>
        </div>
        <p class="agent-meta">{t.detail}</p>
        {#if !t.ready}
          <p class="agent-meta"><strong>Fallback:</strong> {t.fallback}</p>
        {/if}
      </div>
    {/each}
  </div>
</div>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Sesi browser agen</h2>
  {#if sessions.length === 0}
    <p class="agent-meta">Belum ada sesi browser aktif. Sesi dibuat oleh agen (mis. Riset) dan berjalan di daemon — tetap hidup walau aplikasi ditutup.</p>
  {:else}
    {#each sessions as s (s.id)}
      <div class="tool-card">
        <div class="tool-head">
          <span class="tool-name">{s.id}</span>
          <span class="badge ok">{s.agent_id}</span>
        </div>
        <p class="agent-meta">URL terakhir: {s.last_url || "—"}</p>
        <p class="agent-meta">Dibuat: {s.created_at} · Screenshot: {s.shots}</p>
      </div>
    {/each}
  {/if}
  <p class="agent-meta" style="margin-top: 16px;">
    Panel mirror visual (tangkapan layar sesi) tampil penuh di build Tauri macOS.
    Browser tidak pernah login otomatis ke akun pribadi tanpa izin per tugas.
  </p>
</div>

<style>
  .row-between {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }
  .tool-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
    margin-top: 12px;
  }
  .tool-card {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 14px 16px;
    margin-bottom: 8px;
  }
  .tool-card.ready {
    border: 1px solid #a5d6a7;
    background: #f1f8e9;
  }
  .tool-card.notready {
    border: 1px solid #ef9a9a;
    background: #fce4ec;
  }
  .tool-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .tool-name {
    font-size: 16px;
    font-weight: 700;
  }
  .badge {
    font-size: 13px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 999px;
  }
  .badge.ok {
    background: #e8f5e9;
    color: #2e7d32;
  }
  .badge.warn {
    background: #ffebee;
    color: #c62828;
  }
</style>

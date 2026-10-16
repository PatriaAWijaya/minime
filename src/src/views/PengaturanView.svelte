<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "../lib/daemon";
  import type { DaemonStatus } from "../lib/types";

  let status = $state<DaemonStatus | null>(null);

  onMount(async () => {
    try {
      status = await api.daemonStatus();
    } catch {
      status = null;
    }
  });
</script>

<h1 class="section-title">Pengaturan</h1>
<p class="supporting">Pengaturan aplikasi, provider model, dan status daemon.</p>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Daemon</h2>
  {#if status}
    <p class="agent-meta">Versi: {status.version}</p>
    <p class="agent-meta">Uptime: {status.uptime_secs} detik</p>
    <p class="agent-meta">Agen: {status.agents} · Antre: {status.queued_tasks} · Berjalan: {status.running_tasks}</p>
  {:else}
    <p class="agent-meta">Tidak terhubung ke daemon.</p>
  {/if}
  {#if !isTauri}
    <p class="agent-meta" style="margin-top: 16px;">
      Providers, lisensi, dan akun tersedia di build Tauri penuh (Fase 2/5/6).
    </p>
  {/if}
</div>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Providers</h2>
  <p class="agent-meta">Kelola API key dan model per agen — tersedia di Fase 2.</p>
  <button class="btn-secondary" disabled>Tambah provider</button>
</div>

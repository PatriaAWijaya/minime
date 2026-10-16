<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { Approval } from "../lib/types";

  let approvals = $state<Approval[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      approvals = await api.approvalsList("pending");
    } finally {
      loading = false;
    }
  });
</script>

<h1 class="section-title">Persetujuan</h1>
<p class="supporting">Output yang menunggu keputusan Patria. Tidak ada aksi eksternal tanpa persetujuan.</p>

<div class="card">
  {#if loading}
    <div class="empty">Memuat…</div>
  {:else if approvals.length === 0}
    <div class="empty">Tidak ada yang menunggu persetujuan.</div>
  {:else}
    {#each approvals as a (a.id)}
      <div style="margin-bottom: 24px; padding-bottom: 24px; border-bottom: 1px solid var(--border);">
        <p style="font-size: 16px; font-weight: 600; margin: 0 0 8px 0;">Stage: {a.stage}</p>
        <p class="agent-meta">{a.payload_summary}</p>
        <div style="display: flex; gap: 12px; margin-top: 16px;">
          <button class="btn-primary" disabled title="Tersedia di Fase 2">Setujui</button>
          <button class="btn-secondary" disabled title="Tersedia di Fase 2">Tolak</button>
        </div>
      </div>
    {/each}
  {/if}
</div>

<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { Task } from "../lib/types";

  let tasks = $state<Task[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      tasks = await api.tasksList();
    } finally {
      loading = false;
    }
  });

  function statusID(s: string): string {
    const map: Record<string, string> = {
      queued: "Antre",
      running: "Berjalan",
      paused: "Jeda",
      done: "Selesai",
      failed: "Gagal",
      cancelled: "Batal",
    };
    return map[s] ?? s;
  }
</script>

<h1 class="section-title">Antrean</h1>
<p class="supporting">Semua task di antrean FIFO tiap agen.</p>

<div class="card">
  {#if loading}
    <div class="empty">Memuat antrean…</div>
  {:else if tasks.length === 0}
    <div class="empty">Belum ada task. Buat dari menu ⋯ pada kartu agen (New task).</div>
  {:else}
    <table class="data">
      <thead>
        <tr><th>Judul</th><th>Agen</th><th>Status</th><th>Progres</th></tr>
      </thead>
      <tbody>
        {#each tasks as t (t.id)}
          <tr>
            <td>{t.title}</td>
            <td>{t.agent_id}</td>
            <td>{statusID(t.status)}</td>
            <td>
              <div class="progress"><div style="width: {t.progress}%"></div></div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

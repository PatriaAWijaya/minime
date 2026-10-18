<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { Task } from "../lib/types";

  let tasks = $state<Task[]>([]);
  let loading = $state(true);

  async function load() {
    tasks = (await api.tasksList()).filter((t) => t.status === "running");
    loading = false;
  }

  onMount(async () => {
    await load();
    const timer = setInterval(load, 2000);
    return () => clearInterval(timer);
  });
</script>

<h1 class="section-title">Berjalan</h1>
<p class="supporting">Task yang sedang dieksekusi daemon — diperbarui otomatis tiap 2 detik.</p>

<div class="card">
  {#if loading}
    <div class="empty">Memuat…</div>
  {:else if tasks.length === 0}
    <div class="empty">Tidak ada task berjalan saat ini.</div>
  {:else}
    {#each tasks as t (t.id)}
      <div style="margin-bottom: 24px;">
        <p style="font-size: 16px; font-weight: 600; margin: 0 0 8px 0;">{t.title}</p>
        <p class="agent-meta" style="margin: 0 0 8px 0;">{t.agent_id}</p>
        <div class="progress"><div style="width: {t.progress}%"></div></div>
      </div>
    {/each}
  {/if}
</div>

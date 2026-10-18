<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { Task } from "../lib/types";

  let tasks = $state<Task[]>([]);
  let loading = $state(true);

  async function refresh() {
    try {
      // Ambil task yang berjalan + antrean
      const running = await api.tasksList({ status: "running" });
      const queued = await api.tasksList({ status: "queued" });
      tasks = [...running, ...queued];
    } catch {
      tasks = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    refresh();
    const t = setInterval(refresh, 5000);
    return () => clearInterval(t);
  });

  function statusLabel(s: string): string {
    const map: Record<string, string> = {
      running: "Berjalan",
      queued: "Antre",
      paused: "Jeda",
      failed: "Gagal",
    };
    return map[s] ?? s;
  }
</script>

<aside class="task-monitor">
  <div class="tm-header">
    <span class="tm-title">Tugas Berjalan</span>
    <button class="tm-refresh" onclick={refresh} title="Segarkan">↻</button>
  </div>
  <div class="tm-list">
    {#if loading}
      <div class="tm-empty">Memuat…</div>
    {:else if tasks.length === 0}
      <div class="tm-empty">Tidak ada tugas berjalan.</div>
    {:else}
      {#each tasks as task (task.id)}
        <div class="tm-item">
          <div class="tm-top">
            <span class="tm-badge" data-status={task.status}>{statusLabel(task.status)}</span>
            <span class="tm-agent">{task.agent_id}</span>
          </div>
          <div class="tm-title">{task.title ?? task.id}</div>
          {#if task.progress != null}
            <div class="tm-progress">
              <div class="tm-bar" style="width: {task.progress}%"></div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .task-monitor {
    width: 260px;
    min-width: 260px;
    background: #fafafa;
    border-left: 1px solid #e5e5e5;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .tm-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
  }
  .tm-title {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #666;
  }
  .tm-refresh {
    border: none;
    background: transparent;
    font-size: 16px;
    cursor: pointer;
    color: #888;
    padding: 4px 8px;
    border-radius: 6px;
  }
  .tm-refresh:hover {
    background: #eee;
  }
  .tm-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 12px 12px;
  }
  .tm-empty {
    padding: 16px;
    font-size: 14px;
    color: #999;
    text-align: center;
  }
  .tm-item {
    background: #fff;
    border: 1px solid #e5e5e5;
    border-radius: 8px;
    padding: 10px 12px;
    margin-bottom: 8px;
  }
  .tm-top {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .tm-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    text-transform: uppercase;
  }
  .tm-badge[data-status="running"] {
    background: #fff3cd;
    color: #856404;
  }
  .tm-badge[data-status="queued"] {
    background: #e2e3e5;
    color: #383d41;
  }
  .tm-badge[data-status="paused"] {
    background: #f8d7da;
    color: #721c24;
  }
  .tm-agent {
    font-size: 12px;
    color: #888;
  }
  .tm-title {
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 6px;
  }
  .tm-progress {
    height: 4px;
    background: #eee;
    border-radius: 2px;
    overflow: hidden;
  }
  .tm-bar {
    height: 100%;
    background: #2196f3;
    transition: width 0.3s;
  }
</style>

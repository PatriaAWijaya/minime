<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "../lib/daemon";
  import type { Agent } from "../lib/types";

  let agents = $state<Agent[]>([]);
  let selected = $state<string>("mimi");
  let loading = $state(true);

  onMount(async () => {
    try {
      agents = await api.agentsList();
      if (agents.length > 0 && !agents.find((a) => a.id === selected)) {
        selected = agents[0].id;
      }
    } finally {
      loading = false;
    }
  });

  function statusColor(status: string): string {
    if (status === "working") return "#d4a017";
    if (status === "attention") return "#c0392b";
    return "#27ae60";
  }

  function statusLabel(status: string): string {
    if (status === "working") return "Aktif";
    if (status === "attention") return "Perhatian";
    return "Idle";
  }
</script>

<aside class="agent-list">
  <div class="al-header">
    <span class="al-title">Agen</span>
    <span class="al-count">{agents.length}</span>
  </div>
  <nav class="al-items">
    {#if loading}
      <div class="al-empty">Memuat…</div>
    {:else}
      {#each agents as agent (agent.id)}
        <button
          class="al-item"
          class:active={selected === agent.id}
          onclick={() => (selected = agent.id)}
        >
          <span class="al-dot" style="background: {statusColor(agent.status)}"></span>
          <span class="al-text">
            <span class="al-name">{agent.name}</span>
            <span class="al-role">{agent.role} · {statusLabel(agent.status)}</span>
          </span>
        </button>
      {/each}
    {/if}
  </nav>
</aside>

<style>
  .agent-list {
    width: 220px;
    min-width: 220px;
    background: #fff;
    border-right: 1px solid #e5e5e5;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .al-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
  }
  .al-title {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #666;
  }
  .al-count {
    font-size: 12px;
    background: #f0f0f0;
    border-radius: 10px;
    padding: 2px 8px;
    color: #666;
  }
  .al-items {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
  }
  .al-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    margin-bottom: 2px;
  }
  .al-item:hover {
    background: #f5f5f5;
  }
  .al-item.active {
    background: #e3f2fd;
  }
  .al-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .al-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .al-name {
    font-size: 14px;
    font-weight: 500;
  }
  .al-role {
    font-size: 12px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .al-empty {
    padding: 16px;
    font-size: 14px;
    color: #999;
    text-align: center;
  }
</style>

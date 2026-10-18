<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "../lib/daemon";
  import type { Agent } from "../lib/types";
  import AgentCard from "../components/AgentCard.svelte";

  let agents = $state<Agent[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      agents = await api.agentsList();
    } finally {
      loading = false;
    }
  });
</script>

<h1 class="section-title">Agen</h1>
<p class="supporting">Lima agen senior Minime. Klik ⋯ pada kartu untuk menu konteks.</p>

{#if !isTauri}
  <div class="demo-badge">
    Mode demo — data contoh. Hubungkan via aplikasi Tauri untuk status realtime dari daemon.
  </div>
{/if}

{#if loading}
  <div class="empty">Memuat daftar agen…</div>
{:else}
  <div class="agent-grid">
    {#each agents as agent (agent.id)}
      <AgentCard {agent} />
    {/each}
  </div>
{/if}

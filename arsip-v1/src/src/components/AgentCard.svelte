<script lang="ts">
  import { avatarUrl, statusLabel, type Agent } from "../lib/types";
  import { api } from "../lib/daemon";

  interface Props {
    agent: Agent;
  }
  let { agent }: Props = $props();

  let menuOpen = $state(false);
  let showComputer = $state(true);

  const skills: string[] = $derived(JSON.parse(agent.skills || "[]"));

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  function closeMenu() {
    menuOpen = false;
  }

  async function newTask() {
    const title = prompt(`Task baru untuk ${agent.name}:`);
    if (!title) {
      closeMenu();
      return;
    }
    try {
      await api.tasksEnqueue(agent.id, title);
      alert("Task masuk antrean.");
    } catch (e) {
      alert(`Gagal: ${e instanceof Error ? e.message : e}`);
    }
    closeMenu();
  }

  function stub(fitur: string) {
    alert(`${fitur} — tersedia penuh di Fase 2.`);
    closeMenu();
  }
</script>

<div class="agent-card">
  <button class="menu-btn" onclick={toggleMenu} aria-label="Menu agen">⋯</button>
  {#if menuOpen}
    <div class="context-menu" role="menu">
      <div class="menu-section">Tasks</div>
      <button onclick={() => stub("Daftar task")}>Lihat tasks</button>
      <button onclick={newTask}>+ New task</button>
      <div class="menu-section">Agen</div>
      <button onclick={() => stub("Pengaturan agen")}>Pengaturan agen</button>
      <button
        onclick={() => {
          showComputer = !showComputer;
          closeMenu();
        }}
      >
        {showComputer ? "Sembunyikan komputer" : "Tampilkan komputer"}
      </button>
    </div>
  {/if}

  <div class="agent-head">
    <img src={avatarUrl(agent.avatar_path)} alt={`Avatar ${agent.name}`} />
    <div>
      <p class="agent-name">{agent.name}</p>
      <p class="agent-role">{agent.role}</p>
    </div>
  </div>

  <div class="status-row">
    <span class="status-dot status-{agent.status}"></span>
    <span>{statusLabel(agent.status)}</span>
  </div>

  <p class="agent-meta">
    Model: {agent.model} · Effort: {agent.effort}
  </p>
  {#if skills.length > 0}
    <p class="agent-meta">Skills: {skills.length} terpasang</p>
  {/if}
  {#if showComputer}
    <p class="agent-meta" style="margin-top: 8px; font-style: italic;">
      Panel komputer: stub Fase 1 (browser mirror di Fase 4)
    </p>
  {/if}
</div>

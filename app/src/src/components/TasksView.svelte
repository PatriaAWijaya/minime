<script lang="ts">
  import { agents, getAgent } from "../lib/agents";
  import {
    activeAgentId,
    activeTaskByAgent,
    agentStatus,
    newTaskSignal,
  } from "../lib/store";
  import { t } from "../lib/i18n";
  import ChatPanel from "./ChatPanel.svelte";

  const agent = $derived(getAgent($activeAgentId));
  const task = $derived(activeTaskByAgent[$activeAgentId]);

  function focusComposer() {
    newTaskSignal.update((n) => n + 1);
  }
</script>

<div class="split">
  <div class="list-col">
    <h2>{$t("tasks.title")}</h2>
    {#each agents as a}
      {@const status = $agentStatus[a.id] ?? "idle"}
      <button
        class="agent-row"
        class:active={$activeAgentId === a.id}
        onclick={() => activeAgentId.set(a.id)}
      >
        <span class="badge" style:background={a.color}>{a.badge}</span>
        <span class="meta">
          <span class="name">{a.name}</span>
          <span class="role">{$t(a.roleKey)}</span>
        </span>
        <span class="dot {status}" aria-hidden="true"></span>
      </button>
    {/each}
    <button class="btn-ghost" onclick={focusComposer}>{$t("nav.newTask")}</button>
  </div>

  <div class="chat-col">
    {#if task}
      <div class="active-banner">
        <span class="dot working" aria-hidden="true"></span>
        <span><strong>{$t("tasks.active")}:</strong> {$t(task.titleKey)}</span>
        <span class="progress" aria-hidden="true"><i style:width={`${task.progress}%`}></i></span>
        <span>{$t("tasks.progress", { n: String(task.progress) })}</span>
      </div>
    {:else}
      <div class="active-banner">
        <span class="dot idle" aria-hidden="true"></span>
        <span>{$t("tasks.none")}</span>
      </div>
    {/if}
    <div class="chat-wrap">
      <ChatPanel />
    </div>
  </div>
</div>

<style>
  .chat-col { display: flex; flex-direction: column; min-height: 0; min-width: 0; }
  .chat-wrap { flex: 1; min-height: 0; display: flex; }
  .chat-wrap > :global(.chat) { flex: 1; }
  .list-col .btn-ghost { margin-top: 4px; }
</style>

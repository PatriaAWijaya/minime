<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";
  import { agents, agentDisplayName, agentDisplayRole } from "../lib/agents";
  import { activeFlowProjectId, agentNames, agentStatus, agentTitles, customAgents, kanbanCards } from "../lib/store";
  import { t } from "../lib/i18n";

  interface Props {
    data: {
      agentId: string;
      nodeId: string;
      onAddChild: (nodeId: string, agentId: string) => void;
      onDisconnect: (nodeId: string) => void;
      onDelete: (nodeId: string) => void;
    };
    selected?: boolean;
  }
  let { data, selected = false }: Props = $props();

  const allAgents = $derived([...agents, ...$customAgents]);
  const agent = $derived(
    allAgents.find((a) => a.id === data.agentId) ?? allAgents[0],
  );
  const status = $derived($agentStatus[agent.id] ?? "idle");
  /** Open tasks for this agent (Kanban columns except "Selesai") in the active project. */
  const taskCount = $derived(
    $kanbanCards.filter(
      (c) =>
        c.agentId === data.agentId &&
        c.col !== "done" &&
        c.projectId === $activeFlowProjectId,
    ).length,
  );

  let childMenuOpen = $state(false);
  let menuEl: HTMLDivElement | null = $state(null);

  function pickChild(agentId: string) {
    data.onAddChild(data.nodeId, agentId);
    childMenuOpen = false;
  }

  $effect(() => {
    if (!childMenuOpen) return;
    const onDoc = (e: MouseEvent) => {
      if (menuEl && !menuEl.contains(e.target as HTMLElement))
        childMenuOpen = false;
    };
    document.addEventListener("click", onDoc);
    return () => document.removeEventListener("click", onDoc);
  });
</script>

<div class="flow-node" class:selected aria-label={agent.name}>
  <Handle type="target" position={Position.Left} aria-label={$t("tasks.flow.connectIn")} />
  <div class="node-actions nodrag" role="toolbar" aria-label={$t("tasks.flow.nodeActions")}>
    <button
      class="node-act"
      onclick={(e) => {
        e.stopPropagation();
        data.onDisconnect(data.nodeId);
      }}
      aria-label={$t("tasks.flow.disconnect")}
      title={$t("tasks.flow.disconnect")}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M9 15 4 20l-1-1 5-5" />
        <path d="M15 9l5-5 1 1-5 5" />
        <path d="M8.5 8.5l7 7" />
      </svg>
    </button>
    <button
      class="node-act danger"
      onclick={(e) => {
        e.stopPropagation();
        data.onDelete(data.nodeId);
      }}
      aria-label={$t("tasks.flow.deleteNode")}
      title={$t("tasks.flow.deleteNode")}
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M4 7h16" />
        <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
        <path d="M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
      </svg>
    </button>
  </div>
  <span class="badge" style:background={agent.color}>{agent.badge}</span>
  <span class="meta">
    <span class="name">{agentDisplayName(agent, $agentNames, $t)}</span>
    <span class="role">{agentDisplayRole(agent, $agentTitles, $t)}</span>
  </span>
  <span class="dot {status}" aria-hidden="true"></span>
  {#if taskCount > 0}
    <span
      class="task-count"
      title={$t("tasks.flow.taskCount", { n: String(taskCount) })}
      aria-label={$t("tasks.flow.taskCount", { n: String(taskCount) })}
    >{taskCount}</span>
  {/if}
  <button
    class="add-child nodrag"
    onclick={(e) => {
      e.stopPropagation();
      childMenuOpen = !childMenuOpen;
    }}
    aria-label={$t("tasks.flow.addChild")}
    aria-expanded={childMenuOpen}
    title={$t("tasks.flow.addChild")}
  >
    +
  </button>
  {#if childMenuOpen}
    <div class="child-menu nodrag" role="menu" aria-label={$t("tasks.flow.pickAgent")} bind:this={menuEl}>
      {#each allAgents as a (a.id)}
        <button role="menuitem" onclick={() => pickChild(a.id)}>
          <span class="badge sm" style:background={a.color}>{a.badge}</span>
          <span class="name">{agentDisplayName(a, $agentNames, $t)}</span>
        </button>
      {/each}
    </div>
  {/if}
  <Handle type="source" position={Position.Right} aria-label={$t("tasks.flow.connectOut")} />
</div>

<style>
  .flow-node {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 200px;
    max-width: 240px;
    padding: 12px 14px;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    color: var(--text, #e8eaed);
    box-shadow: 0 4px 14px rgb(0 0 0 / 0.35);
  }
  .flow-node.selected {
    border-color: var(--accent, #4FA6F0);
    box-shadow: 0 0 0 2px rgb(79 166 240 / 0.35);
  }
  /* Per-card actions: visible on hover, or always when selected / touch. */
  .node-actions {
    position: absolute;
    top: -20px;
    right: -8px;
    z-index: 15;
    display: flex;
    gap: 6px;
    padding: 4px;
    border-radius: 10px;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    box-shadow: 0 4px 12px rgb(0 0 0 / 0.4);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease;
  }
  .flow-node:hover .node-actions,
  .flow-node.selected .node-actions {
    opacity: 1;
    pointer-events: auto;
  }
  @media (hover: none) {
    .flow-node .node-actions {
      opacity: 1;
      pointer-events: auto;
    }
  }
  .node-act {
    width: 36px;
    height: 36px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border, #2a2e36);
    border-radius: 8px;
    background: var(--bg, #0b0d10);
    color: var(--muted, #9aa0aa);
    cursor: pointer;
    padding: 0;
  }
  .node-act:hover {
    color: var(--text, #e8eaed);
    border-color: var(--accent, #4FA6F0);
  }
  .node-act.danger:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
  .node-act svg {
    width: 18px;
    height: 18px;
  }
  .badge {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 16px;
    color: #0a1f33;
    flex: none;
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }
  .name {
    font-weight: 600;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .role {
    font-size: 12px;
    color: var(--muted, #9aa0aa);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex: none;
  }
  .dot.idle { background: var(--ok); }
  .dot.working { background: var(--warn); }
  .dot.attention { background: var(--danger); }
  .task-count {
    flex: none;
    min-width: 24px;
    height: 24px;
    padding: 0 7px;
    border-radius: 12px;
    background: var(--accent, #4FA6F0);
    color: #0a1f33;
    font-size: 12px;
    font-weight: 800;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .add-child {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 50%;
    border: 2px solid #0b0d10;
    background: var(--accent, #4FA6F0);
    color: #0a1f33;
    font-size: 18px;
    font-weight: 700;
    line-height: 1;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }
  .add-child:hover {
    filter: brightness(1.1);
  }
  .child-menu {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 20;
    min-width: 210px;
    max-height: 264px;
    overflow-y: auto;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }
  .child-menu button {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 6px 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text, #e8eaed);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }
  .child-menu button:hover {
    background: var(--border, #262a32);
  }
  .child-menu .badge.sm {
    width: 28px;
    height: 28px;
    font-size: 13px;
    border-radius: 8px;
  }
  .child-menu .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .flow-node :global(.svelte-flow__handle) {
    width: 12px;
    height: 12px;
    background: var(--accent, #4FA6F0);
    border: 2px solid #0b0d10;
  }
</style>

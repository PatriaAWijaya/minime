<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";
  import { agents } from "../lib/agents";
  import { agentStatus, customAgents } from "../lib/store";
  import { t } from "../lib/i18n";

  interface Props {
    data: {
      agentId: string;
      nodeId: string;
      onAddChild: (nodeId: string, agentId: string) => void;
    };
    selected?: boolean;
  }
  let { data, selected = false }: Props = $props();

  const allAgents = $derived([...agents, ...$customAgents]);
  const agent = $derived(
    allAgents.find((a) => a.id === data.agentId) ?? allAgents[0],
  );
  const status = $derived($agentStatus[agent.id] ?? "idle");

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
  <span class="badge" style:background={agent.color}>{agent.badge}</span>
  <span class="meta">
    <span class="name">{agent.name}</span>
    <span class="role">{$t(agent.roleKey)}</span>
  </span>
  <span class="dot {status}" aria-hidden="true"></span>
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
          <span class="name">{a.name}</span>
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
    border-color: var(--accent, #10b981);
    box-shadow: 0 0 0 2px rgb(16 185 129 / 0.35);
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
    color: #fff;
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
  .dot.idle { background: #22c55e; }
  .dot.working { background: #eab308; }
  .dot.attention { background: #ef4444; }
  .add-child {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 50%;
    border: 2px solid #0b0d10;
    background: var(--accent, #10b981);
    color: #06281e;
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
    background: var(--accent, #10b981);
    border: 2px solid #0b0d10;
  }
</style>

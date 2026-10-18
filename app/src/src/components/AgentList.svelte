<script lang="ts">
  import { agents } from "../lib/agents";
  import { activeAgentId, activeView, agentStatus } from "../lib/store";
  import { t } from "../lib/i18n";

  function select(id: string) {
    activeAgentId.set(id);
    activeView.set("chat");
  }
</script>

<section class="agent-list" aria-label={$t("agents.title")}>
  <h2 class="heading">{$t("agents.title")}</h2>
  <div class="cards">
    {#each agents as agent}
      {@const status = $agentStatus[agent.id] ?? "idle"}
      <button
        class="card"
        class:active={$activeAgentId === agent.id}
        onclick={() => select(agent.id)}
      >
        <span class="badge" style:background={agent.color}>{agent.badge}</span>
        <span class="meta">
          <span class="name">{agent.name}</span>
          <span class="role">{$t(agent.roleKey)}</span>
          <span class="model">{agent.model}</span>
        </span>
        <span class="status">
          <span class="dot {status}"></span>
          <span class="status-text">{$t(`status.${status}`)}</span>
        </span>
      </button>
    {/each}
  </div>
</section>

<style>
  .agent-list {
    background: var(--panel);
    border-right: 1px solid var(--border);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }
  .heading { font-size: 18px; padding: 0 4px; }
  .cards { display: flex; flex-direction: column; gap: 8px; overflow-y: auto; }
  .card {
    display: flex; align-items: center; gap: 10px;
    padding: 10px;
    border: 1px solid transparent;
    border-radius: var(--radius);
    background: transparent;
    text-align: left;
    cursor: pointer;
  }
  .card:hover { background: var(--panel-2); }
  .card.active { background: var(--panel-2); border-color: var(--accent); }
  .badge {
    width: 44px; height: 44px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    color: #fff; font-size: 20px; font-weight: 800; flex: none;
  }
  .meta { display: flex; flex-direction: column; min-width: 0; flex: 1; }
  .name { font-size: 15px; font-weight: 700; }
  .role { font-size: 13px; color: var(--dim); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .model { font-size: 12px; color: var(--dim); opacity: 0.8; }
  .status { display: flex; align-items: center; gap: 6px; flex: none; }
  .status-text { font-size: 12px; color: var(--dim); }
</style>

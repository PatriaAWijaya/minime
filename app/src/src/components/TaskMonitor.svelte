<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tasks } from "../lib/store";
  import { getAgent } from "../lib/agents";
  import { t } from "../lib/i18n";

  let interval: ReturnType<typeof setInterval> | null = null;

  // Demo animation: mock tasks inch forward until done.
  onMount(() => {
    interval = setInterval(() => {
      tasks.update((list) =>
        list.map((task) =>
          task.done
            ? task
            : { ...task, progress: Math.min(100, task.progress + 5), done: task.progress + 5 >= 100 }
        )
      );
    }, 2000);
  });
  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<aside class="monitor" aria-label={$t("monitor.title")}>
  <h2 class="heading">{$t("monitor.title")}</h2>
  <div class="list">
    {#if $tasks.length === 0}
      <p class="empty">{$t("monitor.empty")}</p>
    {:else}
      {#each $tasks as task (task.id)}
        {@const agent = getAgent(task.agentId)}
        <div class="task">
          <div class="row">
            <span class="badge" style:background={agent.color}>{agent.badge}</span>
            <span class="title">{$t(task.titleKey)}</span>
            <span class="pct">{task.progress}%</span>
          </div>
          <div class="bar" role="progressbar" aria-valuenow={task.progress} aria-valuemin="0" aria-valuemax="100">
            <div class="fill" style:width={`${task.progress}%`}></div>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .monitor {
    background: var(--panel);
    border-left: 1px solid var(--border);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }
  .heading { font-size: 18px; padding: 0 4px; }
  .list { display: flex; flex-direction: column; gap: 10px; overflow-y: auto; }
  .empty { font-size: 14px; color: var(--dim); padding: 0 4px; }
  .task {
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .row { display: flex; align-items: center; gap: 8px; }
  .badge {
    width: 28px; height: 28px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    color: #fff; font-size: 14px; font-weight: 800; flex: none;
  }
  .title { flex: 1; font-size: 14px; font-weight: 600; min-width: 0; }
  .pct { font-size: 13px; color: var(--dim); font-variant-numeric: tabular-nums; }
  .bar { height: 6px; border-radius: 3px; background: var(--border); overflow: hidden; }
  .fill { height: 100%; background: var(--accent); border-radius: 3px; transition: width 0.6s; }
</style>

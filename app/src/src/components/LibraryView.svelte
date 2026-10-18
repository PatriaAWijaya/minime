<script lang="ts">
  import { agents, getAgent } from "../lib/agents";
  import { libraryItems } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { LibraryType } from "../lib/types";

  let typeFilter = $state<string>("all");
  let agentFilter = $state<string>("all");
  let taskFilter = $state<string>("all");
  let timeFilter = $state<string>("30");

  const types: LibraryType[] = ["doc", "table", "deck", "image", "video", "app"];
  const ranges = ["7", "30", "90", "365"];

  // Task options come from the mock items themselves.
  const taskOptions = $derived(
    Array.from(new Set(libraryItems.map((i) => i.taskKey))),
  );

  const visible = $derived(
    libraryItems.filter((i) => {
      if (typeFilter !== "all" && i.type !== typeFilter) return false;
      if (agentFilter !== "all" && i.agentId !== agentFilter) return false;
      if (taskFilter !== "all" && i.taskKey !== taskFilter) return false;
      if (i.daysAgo > Number(timeFilter)) return false;
      return true;
    }),
  );
</script>

<div class="page">
  <div class="page-head">
    <h2>{$t("library.title")}</h2>
  </div>

  <div class="filters">
    <div class="field">
      <label for="f-type">{$t("library.type")}</label>
      <select id="f-type" bind:value={typeFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each types as ty}
          <option value={ty}>{$t(`library.type.${ty}`)}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-agent">{$t("library.agent")}</label>
      <select id="f-agent" bind:value={agentFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each agents as a}
          <option value={a.id}>{a.name}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-task">{$t("library.task")}</label>
      <select id="f-task" bind:value={taskFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each taskOptions as tk}
          <option value={tk}>{$t(tk)}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-time">{$t("library.time")}</label>
      <select id="f-time" bind:value={timeFilter}>
        {#each ranges as r}
          <option value={r}>{$t(`library.time.${r}`)}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if visible.length === 0}
    <p class="empty-note">{$t("library.empty")}</p>
  {:else}
    <div class="grid">
      {#each visible as item (item.id)}
        <div class="card">
          <span class="chip">{$t(`library.type.${item.type}`)}</span>
          <h4>{$t(item.titleKey)}</h4>
          <p>{$t("library.by", { agent: getAgent(item.agentId).name })} · {$t(item.dateKey)}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>

<script lang="ts">
  import { agents, getAgent } from "../lib/agents";
  import { costRows } from "../lib/store";
  import { t } from "../lib/i18n";

  let agentFilter = $state<string>("all");
  let metric = $state<"cost" | "tokens">("cost");
  let range = $state<string>("30");

  const ranges = ["7", "30", "90", "365"];

  const visible = $derived(
    costRows.filter((r) => {
      if (agentFilter !== "all" && r.agentId !== agentFilter) return false;
      if (r.daysAgo > Number(range)) return false;
      return true;
    }),
  );

  const totalTokens = $derived(visible.reduce((s, r) => s + r.tokens, 0));
  const totalCost = $derived(visible.reduce((s, r) => s + r.costUsd, 0));

  function fmt(n: number): string {
    return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  }
</script>

<div class="page">
  <div class="page-head">
    <h2>{$t("cost.title")}</h2>
  </div>

  <div class="filters">
    <div class="field">
      <label for="c-agent">{$t("cost.agent")}</label>
      <select id="c-agent" bind:value={agentFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each agents as a}
          <option value={a.id}>{a.name}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="c-metric">{$t("cost.metric")}</label>
      <select id="c-metric" bind:value={metric}>
        <option value="cost">{$t("cost.metric.cost")}</option>
        <option value="tokens">{$t("cost.metric.tokens")}</option>
      </select>
    </div>
    <div class="field">
      <label for="c-range">{$t("cost.range")}</label>
      <select id="c-range" bind:value={range}>
        {#each ranges as r}
          <option value={r}>{$t(`library.time.${r}`)}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="stat">
    <div class="big">
      {metric === "cost" ? `$${totalCost.toFixed(2)}` : fmt(totalTokens)}
    </div>
    <div class="cap">
      {$t("cost.total")} · {metric === "cost" ? $t("cost.metric.cost") : $t("cost.tokens")} · {$t(`library.time.${range}`)}
    </div>
  </div>

  {#if visible.length === 0}
    <p class="empty-note">{$t("cost.empty")}</p>
  {:else}
    <div class="table-wrap">
      <table class="data">
        <thead>
          <tr>
            <th>{$t("cost.col.agent")}</th>
            <th class="num">{$t("cost.col.tokens")}</th>
            <th class="num">{$t("cost.col.cost")} (USD)</th>
            <th>{$t("cost.col.date")}</th>
          </tr>
        </thead>
        <tbody>
          {#each visible as row (row.id)}
            <tr>
              <td>{getAgent(row.agentId).name}</td>
              <td class="num">{fmt(row.tokens)}</td>
              <td class="num">${row.costUsd.toFixed(2)}</td>
              <td>{$t(row.dateKey)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

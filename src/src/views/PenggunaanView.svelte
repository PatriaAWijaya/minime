<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { UsageRow } from "../lib/types";

  let rows = $state<UsageRow[]>([]);
  let loading = $state(true);

  const fmtUsd = (v: number) => `$${v.toFixed(2)}`;
  const fmtNum = (v: number) => v.toLocaleString("id-ID");

  let totalToday = $derived(rows.reduce((s, r) => s + r.spent_today_usd, 0));
  let totalAll = $derived(rows.reduce((s, r) => s + r.spent_total_usd, 0));
  let totalTokens = $derived(rows.reduce((s, r) => s + r.tokens_total, 0));

  function pct(r: UsageRow): number {
    if (r.daily_budget_usd <= 0) return 0;
    return Math.min(100, (r.spent_today_usd / r.daily_budget_usd) * 100);
  }

  onMount(async () => {
    try {
      rows = await api.usageSummary();
    } catch {
      rows = [];
    } finally {
      loading = false;
    }
  });
</script>

<h1 class="section-title">Penggunaan</h1>
<p class="supporting">Metering token & estimasi biaya per agen — budget harian dijaga daemon (cap tercapai → task di-pause).</p>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Ringkasan</h2>
  {#if loading}
    <p class="agent-meta">Memuat…</p>
  {:else}
    <div class="summary-grid">
      <div class="summary-item">
        <span class="summary-label">Belanja hari ini</span>
        <span class="summary-value">{fmtUsd(totalToday)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Total belanja</span>
        <span class="summary-value">{fmtUsd(totalAll)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Total token</span>
        <span class="summary-value">{fmtNum(totalTokens)}</span>
      </div>
    </div>
  {/if}
</div>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Per agen</h2>
  {#if loading}
    <p class="agent-meta">Memuat…</p>
  {:else if rows.length === 0}
    <p class="agent-meta">Belum ada data penggunaan.</p>
  {:else}
    <table class="usage-table">
      <thead>
        <tr>
          <th>Agen</th>
          <th class="num">Hari ini</th>
          <th class="num">Budget harian</th>
          <th class="num">Total</th>
          <th class="num">Token</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as r}
          <tr>
            <td>
              <strong>{r.agent_name}</strong>
              <div class="bar"><div class="bar-fill" style="width: {pct(r)}%"></div></div>
            </td>
            <td class="num">{fmtUsd(r.spent_today_usd)}</td>
            <td class="num">{r.daily_budget_usd > 0 ? fmtUsd(r.daily_budget_usd) : "—"}</td>
            <td class="num">{fmtUsd(r.spent_total_usd)}</td>
            <td class="num">{fmtNum(r.tokens_total)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .summary-grid {
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
  }
  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .summary-label {
    font-size: 14px;
    color: #666;
  }
  .summary-value {
    font-size: 28px;
    font-weight: 700;
  }
  .usage-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 15px;
  }
  .usage-table th,
  .usage-table td {
    text-align: left;
    padding: 12px 8px;
    border-bottom: 1px solid var(--border, #e5e5e5);
  }
  .usage-table th.num,
  .usage-table td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .bar {
    height: 6px;
    background: #eee;
    border-radius: 3px;
    margin-top: 6px;
    max-width: 160px;
  }
  .bar-fill {
    height: 100%;
    background: #4caf50;
    border-radius: 3px;
  }
</style>

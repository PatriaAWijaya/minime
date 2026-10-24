<script lang="ts">
  import { agents, getAgent, agentDisplayName } from "../lib/agents";
  import { agentNames, libraryItems } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { LibraryItem, LibraryType } from "../lib/types";

  let typeFilter = $state<string>("all");
  let agentFilter = $state<string>("all");
  let taskFilter = $state<string>("all");
  let timeFilter = $state<string>("30");

  const types: LibraryType[] = ["doc", "table", "deck", "image", "video", "app"];
  const ranges = ["7", "30", "90", "365"];

  // Task options come from the items themselves (mock seeds + real results).
  const taskOptions = $derived(
    Array.from(new Set($libraryItems.map((i) => i.taskKey))),
  );

  const visible = $derived(
    $libraryItems.filter((i) => {
      if (typeFilter !== "all" && i.type !== typeFilter) return false;
      if (agentFilter !== "all" && i.agentId !== agentFilter) return false;
      if (taskFilter !== "all" && i.taskKey !== taskFilter) return false;
      if (i.daysAgo > Number(timeFilter)) return false;
      return true;
    }),
  );
  let viewItem: LibraryItem | null = $state(null);

  function downloadItem(item: LibraryItem) {
    const blob = new Blob([item.content ?? item.summary ?? ""], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (item.title ?? "output").replace(/[^a-zA-Z0-9-_ ]/g, "").trim() + ".txt";
    a.click();
    URL.revokeObjectURL(url);
  }

  function reuseItem(item: LibraryItem) {
    // Salin konten ke clipboard untuk dipakai di tugas berikutnya
    if (item.content) {
      navigator.clipboard.writeText(item.content).catch(() => {});
    }
  }
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
          <option value={a.id}>{agentDisplayName(a, $agentNames, $t)}</option>
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
          <h4>{item.title ?? $t(item.titleKey)}</h4>
          {#if item.summary}
            <p class="summary">{item.summary}</p>
          {/if}
          <p>{$t("library.by", { agent: getAgent(item.agentId).name })} · {item.dateLabel ?? $t(item.dateKey)}</p>
          {#if item.content}
            <div class="actions">
              <button class="btn-sm" onclick={() => (viewItem = item)}>Lihat</button>
              <button class="btn-sm" onclick={() => downloadItem(item)}>Unduh</button>
              <button class="btn-sm" onclick={() => reuseItem(item)}>Pakai lagi</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if viewItem}
    <div class="dialog-backdrop" onclick={() => (viewItem = null)}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>{viewItem.title}</h3>
        <pre class="content">{viewItem.content}</pre>
        <div class="actions">
          <button class="btn-sm" onclick={() => downloadItem(viewItem!)}>Unduh</button>
          <button class="btn-sm" onclick={() => { reuseItem(viewItem!); viewItem = null; }}>Salin & Tutup</button>
          <button class="btn-sm" onclick={() => (viewItem = null)}>Tutup</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .btn-sm {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
  }
  .btn-sm:hover {
    background: var(--hover);
  }
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    max-width: 700px;
    max-height: 80vh;
    width: 90%;
    overflow: auto;
  }
  .dialog .content {
    white-space: pre-wrap;
    font-size: 13px;
    line-height: 1.6;
    max-height: 50vh;
    overflow: auto;
  }
  .summary {
    font-size: 13px;
    color: var(--muted);
    margin: 6px 0;
    line-height: 1.45;
  }
</style>

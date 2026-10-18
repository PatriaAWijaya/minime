<script lang="ts">
  import { t } from "../lib/i18n";

  const tools = [
    { nameKey: "tools.meta", descKey: "tools.metaDesc" },
    { nameKey: "tools.postiz", descKey: "tools.postizDesc" },
    { nameKey: "tools.pireel", descKey: "tools.pireelDesc" },
    { nameKey: "tools.drive", descKey: "tools.driveDesc" },
  ];
</script>

<section class="view">
  <h2 class="title">{$t("tools.title")}</h2>
  <div class="list">
    {#each tools as tool}
      <div class="tool">
        <div class="info">
          <div class="name">{$t(tool.nameKey)}</div>
          <div class="desc">{$t(tool.descKey)}</div>
        </div>
        <span class="status"><span class="dot attention"></span>{$t("tools.notConnected")}</span>
        <button class="btn-ghost" disabled title="Tahap 3">{$t("tools.connect")}</button>
      </div>
    {/each}
  </div>
  <p class="note">{$t("tools.note")}</p>
</section>

<style>
  .view { padding: 24px; max-width: 720px; overflow-y: auto; }
  .title { font-size: 22px; margin-bottom: 20px; }
  .list { display: flex; flex-direction: column; gap: 10px; margin-bottom: 16px; }
  .tool {
    display: flex; align-items: center; gap: 14px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px 16px;
  }
  .info { flex: 1; min-width: 0; }
  .name { font-size: 15px; font-weight: 700; }
  .desc { font-size: 13px; color: var(--dim); }
  .status { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--dim); white-space: nowrap; }
  .note { font-size: 14px; color: var(--dim); }
</style>

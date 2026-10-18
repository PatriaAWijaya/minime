<script lang="ts">
  import { t } from "../lib/i18n";
</script>

<section class="view">
  <h2 class="title">{$t("usage.title")}</h2>
  <div class="stats">
    <div class="stat">
      <div class="value">0</div>
      <div class="label">{$t("usage.messages")}</div>
    </div>
    <div class="stat">
      <div class="value">0</div>
      <div class="label">{$t("usage.tasks")}</div>
    </div>
  </div>
  <p class="note">{$t("usage.note")}</p>
</section>

<style>
  .view { padding: 24px; max-width: 640px; overflow-y: auto; }
  .title { font-size: 22px; margin-bottom: 20px; }
  .stats { display: flex; gap: 12px; margin-bottom: 16px; }
  .stat {
    flex: 1;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 18px;
  }
  .value { font-size: 32px; font-weight: 800; font-variant-numeric: tabular-nums; }
  .label { font-size: 14px; color: var(--dim); margin-top: 4px; }
  .note { font-size: 14px; color: var(--dim); }
</style>

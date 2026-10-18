<script lang="ts">
  import { addScheduleItem, scheduleItems } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { ScheduleRange } from "../lib/types";

  let range = $state<ScheduleRange>("day");

  const ranges: ScheduleRange[] = ["day", "week", "month"];

  const visible = $derived($scheduleItems.filter((i) => i.range === range));

  function toggle(id: number, value: boolean) {
    scheduleItems.update((items) =>
      items.map((i) => (i.id === id ? { ...i, active: value } : i)),
    );
  }
</script>

<div class="page">
  <div class="page-head">
    <h2>{$t("schedule.title")}</h2>
    <button class="btn-primary" onclick={addScheduleItem}>{$t("nav.newTask")}</button>
  </div>

  <div class="seg" role="group" aria-label={$t("schedule.title")}>
    {#each ranges as r}
      <button
        class:active={range === r}
        onclick={() => (range = r)}
      >{$t(`schedule.${r}`)}</button>
    {/each}
  </div>

  <div class="section">
    {#if visible.length === 0}
      <p class="empty-note">{$t("schedule.empty")}</p>
    {:else}
      {#each visible as item (item.id)}
        <div class="row">
          <div class="meta">
            <span class="label">{$t(item.nameKey)}</span>
            <span class="value">{$t(item.whenKey)}</span>
          </div>
          <span class="value state">{item.active ? $t("schedule.active") : $t("schedule.inactive")}</span>
          <label class="switch">
            <input
              type="checkbox"
              checked={item.active}
              onchange={(e) => toggle(item.id, e.currentTarget.checked)}
              aria-label={$t(item.nameKey)}
            />
            <span class="track" aria-hidden="true"></span>
          </label>
          <span class="row-actions">
            <button class="btn-ghost sm">{$t("schedule.view")}</button>
            <button class="btn-ghost sm">{$t("schedule.edit")}</button>
          </span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .seg { margin-top: 12px; }
  .meta { display: flex; flex-direction: column; min-width: 0; flex: 1; }
  .state { min-width: 90px; }
  .row-actions { display: flex; gap: 8px; }
  .btn-ghost.sm { min-height: 40px; padding: 0 14px; font-size: 14px; }
</style>

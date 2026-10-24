<script lang="ts">
  import { libraryItems } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { LibraryItem, LibraryType } from "../lib/types";

  interface Props {
    open: boolean;
    title?: string;
    allowTypes?: LibraryType[] | null;
    onPick: (item: LibraryItem) => void;
    onClose: () => void;
  }
  let { open, title = "", allowTypes = null, onPick, onClose }: Props = $props();

  let query = $state("");
  let typeFilter = $state<string>("all");

  const shownTypes = $derived(
    allowTypes ?? ["doc", "table", "deck", "image", "video", "app"],
  );

  const visible = $derived(
    $libraryItems.filter((i) => {
      if (allowTypes && !allowTypes.includes(i.type)) return false;
      if (typeFilter !== "all" && i.type !== typeFilter) return false;
      if (query.trim()) {
        const q = query.toLowerCase();
        const hay = `${i.title ?? ""} ${i.summary ?? ""}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    }),
  );

  function itemTitle(i: LibraryItem): string {
    return i.title || (i.titleKey ? $t(i.titleKey) : "Output");
  }
</script>

{#if open}
  <div class="dialog-backdrop" onclick={onClose}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <h3>{title || $t("library.pickerTitle")}</h3>
      <div class="row">
        <input
          type="text"
          bind:value={query}
          placeholder={$t("library.searchLibrary")}
          class="grow"
        />
        <select bind:value={typeFilter} aria-label={$t("library.type")}>
          <option value="all">{$t("library.allTypes")}</option>
          {#each shownTypes as ty}
            <option value={ty}>{$t(`library.type.${ty}`)}</option>
          {/each}
        </select>
      </div>
      {#if visible.length === 0}
        <p class="empty-note">{$t("library.noMatch")}</p>
      {:else}
        <div class="pick-list">
          {#each visible as item (item.id)}
            <div class="pick-row">
              <div class="pick-main">
                <strong>{itemTitle(item)}</strong>
                <span class="chip">{$t(`library.type.${item.type}`)}</span>
                {#if item.summary}
                  <p class="summary">{item.summary}</p>
                {/if}
              </div>
              <button class="btn-sm" onclick={() => onPick(item)}>{$t("library.pick")}</button>
            </div>
          {/each}
        </div>
      {/if}
      <div class="actions">
        <button class="btn-sm" onclick={onClose}>Tutup</button>
      </div>
    </div>
  </div>
{/if}

<style>
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
  .row {
    display: flex;
    gap: 8px;
    margin: 12px 0;
  }
  .row .grow {
    flex: 1;
  }
  .pick-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 12px 0;
  }
  .pick-row {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px;
  }
  .pick-main {
    flex: 1;
    min-width: 0;
  }
  .pick-main .summary {
    font-size: 13px;
    color: var(--muted);
    margin: 4px 0 0;
    line-height: 1.45;
  }
  .chip {
    font-size: 11px;
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 1px 8px;
    margin-left: 6px;
    color: var(--muted);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }
  .btn-sm {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
    white-space: nowrap;
  }
  .btn-sm:hover {
    background: var(--hover);
  }
  .empty-note {
    color: var(--muted);
    font-size: 13px;
  }
</style>

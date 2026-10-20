<script lang="ts">
  import { activeView, settingsPrefs } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { View } from "../lib/store";

  interface Tab {
    id: View;
    labelKey: string;
    descKey: string;
    icon: string;
  }

  // Feather-style line icons (stroke only, no fills).
  const tabs: Tab[] = [
    {
      id: "agents",
      labelKey: "nav.agents",
      descKey: "nav.agents.desc",
      icon: '<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>',
    },
    {
      id: "tasks",
      labelKey: "nav.tasks",
      descKey: "nav.tasks.desc",
      icon: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
    },
    {
      id: "schedule",
      labelKey: "nav.schedule",
      descKey: "nav.schedule.desc",
      icon: '<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
    },
    {
      id: "library",
      labelKey: "nav.library",
      descKey: "nav.library.desc",
      icon: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
    },
    {
      id: "cost",
      labelKey: "nav.cost",
      descKey: "nav.cost.desc",
      icon: '<path d="M18 20V10M12 20V4M6 20v-6"/>',
    },
    {
      id: "tools",
      labelKey: "nav.tools",
      descKey: "nav.tools.desc",
      icon: '<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>',
    },
  ];

  const settingsTab: Tab = {
    id: "settings",
    labelKey: "nav.settings",
    descKey: "nav.settings.desc",
    icon: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
  };
</script>

<aside class="rail" aria-label="Minime">
  <div class="logo" style:background={$settingsPrefs.logoColor} aria-hidden="true">M</div>
  <nav class="tabs" aria-label="Minime">
    {#each tabs as tab}
      <button
        class="icon-btn"
        class:active={$activeView === tab.id}
        onclick={() => activeView.set(tab.id)}
        aria-label={$t(tab.labelKey)}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{@html tab.icon}</svg>
        <span class="tip" aria-hidden="true">
          <strong>{$t(tab.labelKey)}</strong>
          <span>{$t(tab.descKey)}</span>
        </span>
      </button>
    {/each}
  </nav>
  <div class="spacer"></div>
  <button
    class="icon-btn"
    class:active={$activeView === settingsTab.id}
    onclick={() => activeView.set(settingsTab.id)}
    aria-label={$t(settingsTab.labelKey)}
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{@html settingsTab.icon}</svg>
    <span class="tip" aria-hidden="true">
      <strong>{$t(settingsTab.labelKey)}</strong>
      <span>{$t(settingsTab.descKey)}</span>
    </span>
  </button>
</aside>

<style>
  .tabs { display: flex; flex-direction: column; gap: 6px; }
</style>

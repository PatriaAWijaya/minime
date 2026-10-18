<script lang="ts">
  import { activeView, activeAgentId, newTaskSignal } from "../lib/store";
  import { t } from "../lib/i18n";
  import type { View } from "../lib/store";

  const items: { id: View; labelKey: string; icon: string }[] = [
    { id: "chat", labelKey: "nav.chat", icon: "M4 5h16v10H8l-4 4V5z" },
    { id: "settings", labelKey: "nav.settings", icon: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM4 12h3M17 12h3M12 4v3M12 17v3" },
    { id: "usage", labelKey: "nav.usage", icon: "M4 20V10M10 20V4M16 20v-8M22 20H2" },
    { id: "tools", labelKey: "nav.tools", icon: "M14 7a4 4 0 0 1 5.6 5.1L14 17.7 11.5 15l5.6-5.6A4 4 0 0 1 14 7zM9 13l-5.4 5.4 2.5 2.5L11.5 15" },
  ];

  function go(view: View) {
    activeView.set(view);
  }

  function newTask() {
    activeAgentId.set("mimi");
    activeView.set("chat");
    newTaskSignal.update((n) => n + 1);
  }
</script>

<aside class="sidebar">
  <div class="brand">
    <div class="logo">M</div>
    <div>
      <div class="name">Minime</div>
      <div class="tagline">{$t("app.tagline")}</div>
    </div>
  </div>

  <!-- One primary action: start something in a single click -->
  <button class="btn-primary new-task" onclick={newTask}>{$t("nav.newTask")}</button>

  <nav class="menu">
    {#each items as item}
      <button
        class="menu-item"
        class:active={$activeView === item.id}
        onclick={() => go(item.id)}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d={item.icon} />
        </svg>
        <span>{$t(item.labelKey)}</span>
      </button>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    background: var(--panel);
    border-right: 1px solid var(--border);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
  }
  .brand { display: flex; align-items: center; gap: 10px; padding: 0 4px; }
  .logo {
    width: 40px; height: 40px; border-radius: 12px;
    background: var(--accent); color: var(--accent-ink);
    display: flex; align-items: center; justify-content: center;
    font-size: 22px; font-weight: 800;
  }
  .name { font-size: 17px; font-weight: 800; }
  .tagline { font-size: 12px; color: var(--dim); }
  .new-task { width: 100%; }
  .menu { display: flex; flex-direction: column; gap: 4px; }
  .menu-item {
    display: flex; align-items: center; gap: 10px;
    min-height: 44px; padding: 0 12px;
    border: 0; border-radius: var(--radius);
    background: transparent; color: var(--dim);
    font-size: 15px; font-weight: 600; text-align: left;
    cursor: pointer;
  }
  .menu-item svg { width: 20px; height: 20px; flex: none; }
  .menu-item:hover { background: var(--panel-2); color: var(--text); }
  .menu-item.active { background: var(--panel-2); color: var(--accent); }
</style>

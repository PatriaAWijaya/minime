<script lang="ts">
  import { onMount } from "svelte";
  import IconSidebar from "./components/IconSidebar.svelte";
  import AgentsView from "./components/AgentsView.svelte";
  import TasksView from "./components/TasksView.svelte";  import ScheduleView from "./components/ScheduleView.svelte";
  import LibraryView from "./components/LibraryView.svelte";
  import CostView from "./components/CostView.svelte";
  import ToolsView from "./components/ToolsView.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import { activeView, refreshProviders, theme, type Theme } from "./lib/store";

  /** Apply the effective theme to <html> as data-theme="dark"|"light". */
  function applyTheme(t: Theme) {
    const dark =
      t === "dark" ||
      (t === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  // Re-apply whenever the theme setting changes.
  $effect(() => applyTheme($theme));

  onMount(() => {
    // Pindai provider nyata sekali saat aplikasi dibuka (untuk daftar
    // model Ollama dinamis + status Penyedia Model).
    refreshProviders();
    // Live response when the OS theme changes and setting = "system".
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      if ($theme === "system") applyTheme("system");
    };
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  });
</script>

<div class="layout">
  <IconSidebar />
  <main class="content">
    {#if $activeView === "agents"}
      <AgentsView />
    {:else if $activeView === "tasks"}
      <TasksView />
    {:else if $activeView === "schedule"}
      <ScheduleView />
    {:else if $activeView === "library"}
      <LibraryView />
    {:else if $activeView === "cost"}
      <CostView />
    {:else if $activeView === "tools"}
      <ToolsView />
    {:else}
      <SettingsView />
    {/if}
  </main>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 46px minmax(0, 1fr);
    height: 100%;
  }
  .content {
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .content > :global(*) { flex: 1; min-height: 0; }
</style>

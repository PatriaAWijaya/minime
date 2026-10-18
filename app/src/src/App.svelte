<script lang="ts">
  import IconSidebar from "./components/IconSidebar.svelte";
  import AgentsView from "./components/AgentsView.svelte";
  import TasksView from "./components/TasksView.svelte";
  import ScheduleView from "./components/ScheduleView.svelte";
  import LibraryView from "./components/LibraryView.svelte";
  import CostView from "./components/CostView.svelte";
  import ToolsView from "./components/ToolsView.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import { activeView } from "./lib/store";
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
    grid-template-columns: 56px minmax(0, 1fr);
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

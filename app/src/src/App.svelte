<script lang="ts">
  import Sidebar from "./components/Sidebar.svelte";
  import AgentList from "./components/AgentList.svelte";
  import ChatPanel from "./components/ChatPanel.svelte";
  import TaskMonitor from "./components/TaskMonitor.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import UsageView from "./components/UsageView.svelte";
  import ToolsView from "./components/ToolsView.svelte";
  import { activeView } from "./lib/store";
</script>

<div class="layout">
  <Sidebar />
  <AgentList />
  <main class="content">
    {#if $activeView === "chat"}
      <ChatPanel />
    {:else if $activeView === "settings"}
      <SettingsView />
    {:else if $activeView === "usage"}
      <UsageView />
    {:else}
      <ToolsView />
    {/if}
  </main>
  <TaskMonitor />
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 208px 248px minmax(0, 1fr) 264px;
    height: 100%;
    min-width: 1100px;
  }
  .content {
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .content > :global(*) { flex: 1; min-height: 0; }
</style>

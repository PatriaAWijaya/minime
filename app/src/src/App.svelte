<script lang="ts">
  import { onMount } from "svelte";
  import IconSidebar from "./components/IconSidebar.svelte";
  import AgentsView from "./components/AgentsView.svelte";
  import TasksView from "./components/TasksView.svelte";  import ScheduleView from "./components/ScheduleView.svelte";
  import LibraryView from "./components/LibraryView.svelte";
  import CostView from "./components/CostView.svelte";
  import ToolsView from "./components/ToolsView.svelte";
  import StudioView from "./components/StudioView.svelte";
  import SettingsView from "./components/SettingsView.svelte";
  import IslandView from "./components/IslandView.svelte";
  import {
    activeView,
    agentStatus,
    checkForUpdate,
    isTauri,
    kanbanCards,
    profileStore,
    refreshProviders,
    sendNotification,
    settingsPrefs,
    syncIsland,
    theme,
    updateInfo,
    type IslandPayload,
    type Theme,
  } from "./lib/store";
  import { get } from "svelte/store";
  import { t } from "./lib/i18n";

  /** The Dynamic Island window loads the same bundle with ?view=island. */
  const isIsland =
    typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).get("view") === "island";

  let islandAttention = $state(0);
  /** Island payload for the inline title-bar island (also synced to backend). */
  let islandPayload = $state<IslandPayload | null>(null);

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

  // Dynamic Island: compose the payload from the live stores and push it
  // to the backend whenever agents, cards, prefs or the server change.
  $effect(() => {
    if (isIsland) return;
    const prefs = $settingsPrefs;
    const working = Object.values($agentStatus).filter(
      (s) => s === "working",
    ).length;
    const attentionAgents = Object.values($agentStatus).filter(
      (s) => s === "attention",
    ).length;
    const approvalCards = $kanbanCards.filter(
      (c) => c.col === "approval",
    ).length;
    const attention = attentionAgents + approvalCards;
    islandAttention = attention;
    const hasStatus = working > 0 || attention > 0;
    const mode = !prefs.islandNotch
      ? "off"
      : hasStatus
        ? "status"
        : prefs.islandIdle
          ? "idle"
          : "off";
    const name = $profileStore.name.trim();
    const firstName = name ? name.split(" ")[0] : "";

    const payload: IslandPayload = {
      notch: prefs.islandNotch,
      idle: prefs.islandIdle,
      extraDisplays: prefs.islandExtra,
      haptics: prefs.islandHaptics,
      position: prefs.islandPosition,
      widthPct: prefs.islandWidth,
      heightPct: prefs.islandHeight,
      mode,
      title:
        mode === "status"
          ? attention > 0
            ? `${attention} ${$t("island.attention")}`
            : `${working} ${$t("island.working")}`
          : `${$t("island.hello")}${firstName ? `, ${firstName}` : ""}`,
      sub: mode === "status" ? $t("server.local") : $t("island.idle.sub"),
      attention: attention > 0,
      logoColor: prefs.logoColor,
    };
    islandPayload = payload;
    // The island now lives in the title bar (inline) — keep the floating
    // window hidden by syncing mode "off" to the backend.
    void syncIsland({ ...payload, mode: "off" });
  });

  onMount(() => {
    // Pindai provider nyata sekali saat aplikasi dibuka (untuk daftar
    // model Ollama dinamis + status Penyedia Model).
    refreshProviders();
    // Automatic update check at launch (Settings > Updates toggle).
    if (!isIsland && $settingsPrefs.autoUpdate) {
      void checkForUpdate().then(() => {
        const info = get(updateInfo);
        if (info?.update_available) {
          void sendNotification(
            $t("upd.notify.title"),
            `${$t("upd.notify.body")} v${info.latest}`,
          );
        }
      });
    }
    // Live response when the OS theme changes and setting = "system".
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      if ($theme === "system") applyTheme("system");
    };
    mq.addEventListener("change", onChange);
    // Clicking the Dynamic Island brings this window forward: to Tasks
    // when something needs approval, otherwise to Agents.
    let unlisten: (() => void) | undefined;
    if (!isIsland) {
      void (async () => {
        if (!(await isTauri())) return;
        const { listen } = await import("@tauri-apps/api/event");
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        unlisten = await listen("island-activated", async () => {
          activeView.set(islandAttention > 0 ? "tasks" : "agents");
          const win = getCurrentWindow();
          await win.show().catch(() => {});
          await win.unminimize().catch(() => {});
          await win.setFocus().catch(() => {});
        });
      })();
    }
    return () => {
      mq.removeEventListener("change", onChange);
      unlisten?.();
    };
  });
</script>

{#if isIsland}
  <IslandView />
{:else}
<div class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <div class="titlebar-island" style:max-width={islandPayload ? `${Math.round(4.2 * islandPayload.widthPct)}px` : undefined}>
      {#if islandPayload}
        <IslandView payload={islandPayload} />
      {/if}
    </div>
  </header>
  <div class="layout">
  <IconSidebar />
  <main class="content">
    {#if $activeView === "agents"}
      <AgentsView />
    {:else if $activeView === "tasks"}
      <TasksView />
    {:else if $activeView === "studio"}
      <StudioView />
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
</div>
{/if}

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  /* Custom title bar (titleBarStyle: Overlay) — the Dynamic Island lives here. */
  .titlebar {
    height: 40px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    /* Leave room for macOS traffic lights on the left */
    padding-left: 78px;
    padding-right: 12px;
    -webkit-user-select: none;
    user-select: none;
  }
  .titlebar-island {
    max-width: 420px;
    width: auto;
  }
  .titlebar-island :global(.pill) {
    height: auto;
    min-height: 30px;
    padding: 4px 12px 4px 6px;
  }
  .titlebar-island :global(.logo-tile) {
    width: 22px;
    height: 22px;
    border-radius: 7px;
  }
  .titlebar-island :global(.title) {
    font-size: 12px;
  }
  .titlebar-island :global(.sub) {
    font-size: 10.5px;
  }
  .layout {
    display: grid;
    grid-template-columns: 46px minmax(0, 1fr);
    flex: 1;
    min-height: 0;
  }
  .content {
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .content > :global(*) { flex: 1; min-height: 0; }
</style>

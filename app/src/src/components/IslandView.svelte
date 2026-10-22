<script lang="ts">
  import { onMount } from "svelte";
  import {
    fetchIslandState,
    isTauri,
    type IslandPayload,
  } from "../lib/store";
  import appIcon from "../assets/app-icon.png";

  // The Dynamic Island: a floating pill at the desktop top bar (macOS menu
  // bar / Windows top) that mirrors agent status. State arrives from the
  // backend `island-state` event (pushed by the main window via island_sync);
  // on mount we also fetch the last payload so a fresh window never starts blank.
  let state = $state<IslandPayload | null>(null);

  async function activate() {
    if (!(await isTauri())) return;
    const [{ emit }, { invoke }] = await Promise.all([
      import("@tauri-apps/api/event"),
      import("@tauri-apps/api/core"),
    ]);
    if (state?.haptics) {
      await invoke("island_haptic").catch(() => {});
    }
    await emit("island-activated").catch(() => {});
  }

  onMount(async () => {
    if (!(await isTauri())) return;
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<IslandPayload>("island-state", (e) => {
      state = e.payload;
    });
    const last = await fetchIslandState();
    if (last && !state) state = last;
    return unlisten;
  });
</script>

{#if state && state.mode !== "off"}
  <button class="pill" onclick={activate} aria-label="Minime Dynamic Island">
    <img class="logo-tile" src={appIcon} alt="Minime" draggable="false" />
    <span class="texts">
      <span class="title">{state.title}</span>
      <span class="sub">{state.sub}</span>
    </span>
    {#if state.attention}
      <span class="alert-dot" aria-hidden="true"></span>
    {:else if state.mode === "idle"}
      <span class="wave" aria-hidden="true">👋</span>
    {:else}
      <span class="work-dot" aria-hidden="true"></span>
    {/if}
  </button>
{/if}

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    margin: 0;
    overflow: hidden;
  }
  .pill {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 100vh;
    box-sizing: border-box;
    padding: 6px 14px 6px 8px;
    border: 0;
    border-radius: 999px;
    background: rgba(8, 8, 11, 0.94);
    color: #fff;
    cursor: pointer;
    font: inherit;
    text-align: left;
  }
  .logo-tile {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    flex: none;
    object-fit: cover;
  }
  .texts {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.15;
  }
  .title {
    font-size: 13.5px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.62);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .alert-dot {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #ff5147;
    flex: none;
    animation: pulse 1.4s ease-in-out infinite;
  }
  .work-dot {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #ffd60a;
    flex: none;
    animation: pulse 1.4s ease-in-out infinite;
  }
  .wave {
    font-size: 16px;
    flex: none;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.35;
    }
  }
</style>

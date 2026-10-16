<script lang="ts">
  import AgenView from "./views/AgenView.svelte";
  import AntreanView from "./views/AntreanView.svelte";
  import BerjalanView from "./views/BerjalanView.svelte";
  import PersetujuanView from "./views/PersetujuanView.svelte";
  import DeliverableView from "./views/DeliverableView.svelte";
  import PengaturanView from "./views/PengaturanView.svelte";
  import PeralatanView from "./views/PeralatanView.svelte";

  type ViewId = "agen" | "antrean" | "berjalan" | "persetujuan" | "deliverable" | "peralatan" | "pengaturan";

  let view = $state<ViewId>("agen");

  const nav: { id: ViewId; label: string }[] = [
    { id: "agen", label: "Agen" },
    { id: "antrean", label: "Antrean" },
    { id: "berjalan", label: "Berjalan" },
    { id: "persetujuan", label: "Persetujuan" },
    { id: "deliverable", label: "Deliverable" },
    { id: "peralatan", label: "Peralatan" },
    { id: "pengaturan", label: "Pengaturan" },
  ];
</script>

<div class="layout">
  <aside class="sidebar">
    <div class="brand">
      <img src="/avatars/mimi-orkestrator.webp" alt="Logo Minime" />
      <span class="brand-name">Minime</span>
    </div>
    {#each nav as item (item.id)}
      <button
        class="nav-item"
        class:active={view === item.id}
        onclick={() => (view = item.id)}
      >
        {item.label}
      </button>
    {/each}
  </aside>

  <main class="main">
    {#if view === "agen"}
      <AgenView />
    {:else if view === "antrean"}
      <AntreanView />
    {:else if view === "berjalan"}
      <BerjalanView />
    {:else if view === "persetujuan"}
      <PersetujuanView />
    {:else if view === "deliverable"}
      <DeliverableView />
    {:else if view === "peralatan"}
      <PeralatanView />
    {:else}
      <PengaturanView />
    {/if}
  </main>
</div>

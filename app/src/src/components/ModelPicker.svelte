<script lang="ts">
  // Xiryn-style model picker: searchable popover, grouped by provider,
  // spec format "provider/model", live model fetch on open.
  import type { CatalogEntry } from "./ProviderRow.svelte";

  let {
    value = "",
    onChange,
    providers,
  }: {
    value: string;
    onChange: (spec: string) => void;
    providers: CatalogEntry[];
  } = $props();

  let open = $state(false);
  let query = $state("");
  let listings = $state<Record<string, { state: string; models: string[]; error?: string }>>({});
  let pickerEl = $state<HTMLDivElement | null>(null);

  const configured = $derived(providers.filter((p) => p.local || p.key_set || p.base_url_set));

  function parseSpec(spec: string): { provider: string; model: string } | null {
    const i = spec.indexOf("/");
    if (i <= 0 || i === spec.length - 1) return null;
    return { provider: spec.slice(0, i), model: spec.slice(i + 1) };
  }

  async function fetchAll(refresh = false) {
    for (const p of configured) {
      if (!refresh && listings[p.id]?.state === "ready") continue;
      listings[p.id] = { state: "loading", models: listings[p.id]?.models ?? [] };
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const models = await invoke<string[]>("fetch_provider_models", { providerId: p.id });
        listings[p.id] = { state: "ready", models };
      } catch (e) {
        listings[p.id] = { state: "error", models: listings[p.id]?.models ?? [], error: String(e) };
      }
      listings = { ...listings };
    }
  }

  function pick(spec: string) {
    onChange(spec);
    open = false;
    query = "";
  }

  function toggle() {
    open = !open;
    if (open) fetchAll();
  }

  function onDocClick(e: MouseEvent) {
    if (pickerEl && !pickerEl.contains(e.target as Node)) open = false;
  }
  $effect(() => {
    document.addEventListener("click", onDocClick);
    return () => document.removeEventListener("click", onDocClick);
  });

  const typed = $derived(query.trim());
  const typedSpec = $derived(parseSpec(typed));
  const q = $derived(typed.toLowerCase());

  function filteredModels(p: CatalogEntry): string[] {
    const live = listings[p.id]?.models ?? [];
    const all = [...new Set([...p.suggested, ...live])];
    if (!q) return all;
    return all.filter((m) => m.toLowerCase().includes(q) || p.name.toLowerCase().includes(q));
  }

  const specParts = $derived(parseSpec(value));
</script>

<div class="picker" bind:this={pickerEl}>
  <button class="trigger" onclick={toggle} aria-expanded={open} type="button">
    {#if value && specParts}
      <span class="spec"><span class="prov">{specParts.provider}/</span><span class="model">{specParts.model}</span></span>
    {:else if value}
      <span class="spec"><span class="model">{value}</span></span>
    {:else}
      <span class="placeholder">Pilih model...</span>
    {/if}
    <span class="chev">▾</span>
  </button>

  {#if open}
    <div class="pop">
      <input
        class="search"
        type="text"
        bind:value={query}
        placeholder="Cari, atau ketik provider/model"
        spellcheck={false}
      />
      <div class="list">
        {#if typedSpec}
          <div class="group-label">Kustom</div>
          <button class="item" onclick={() => pick(typed)}>
            <span class="spec"><span class="prov">{typedSpec.provider}/</span><span class="model">{typedSpec.model}</span></span>
            <span class="use">pakai ↵</span>
          </button>
        {/if}
        {#each configured as p}
          {@const models = filteredModels(p)}
          {#if models.length > 0}
            <div class="group-label">
              {p.name}
              {#if listings[p.id]?.state === "loading"}<span class="spin">…</span>{/if}
              {#if listings[p.id]?.state === "error"}<span class="err-lbl">gagal memuat</span>{/if}
            </div>
            {#each models as m}
              {@const spec = `${p.id}/${m}`}
              <button class="item" onclick={() => pick(spec)}>
                <span class="model mono">{m}</span>
                {#if value === spec}<span class="check">✓</span>{/if}
              </button>
            {/each}
          {/if}
        {/each}
        {#if configured.length === 0}
          <p class="empty">Belum ada provider terhubung — tambahkan di Pengaturan.</p>
        {/if}
      </div>
      <div class="foot">
        <span class="dim">{configured.length} provider terhubung</span>
        <button class="refresh" onclick={() => fetchAll(true)} type="button">↻ Refresh</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .picker { position: relative; width: 100%; }
  .trigger {
    width: 100%; display: flex; align-items: center; gap: 8px;
    padding: 9px 12px; border: 1px solid var(--border); border-radius: 8px;
    background: var(--bg); color: var(--text); cursor: pointer; font-size: 13px; text-align: left;
  }
  .trigger:hover { border-color: #3b82f6; }
  .spec { font-family: var(--font-mono); font-size: 12px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spec .prov { color: var(--dim); }
  .placeholder { color: var(--dim); flex: 1; }
  .chev { color: var(--dim); font-size: 11px; }
  .pop {
    position: absolute; top: calc(100% + 4px); left: 0; right: 0; z-index: 50;
    background: #1a1a1e; border: 1px solid var(--border); border-radius: 10px;
    box-shadow: 0 8px 30px rgba(0,0,0,.6); overflow: hidden;
  }
  .search {
    width: 100%; padding: 10px 12px; border: none; border-bottom: 1px solid var(--border);
    background: transparent; color: var(--text); font-size: 13px; outline: none;
  }
  .list { max-height: 320px; overflow-y: auto; padding: 4px; }
  .group-label {
    font-size: 10px; text-transform: uppercase; letter-spacing: .06em; color: var(--dim);
    padding: 8px 10px 4px; display: flex; align-items: center; gap: 6px;
  }
  .spin { animation: pulse 1s infinite; }
  @keyframes pulse { 50% { opacity: .3; } }
  .err-lbl { color: #ef4444; text-transform: none; letter-spacing: 0; }
  .item {
    width: 100%; display: flex; align-items: center; gap: 8px; padding: 8px 10px;
    background: none; border: none; border-radius: 6px; cursor: pointer;
    color: var(--text); font-size: 13px; text-align: left;
  }
  .item:hover { background: var(--hover); }
  .mono { font-family: var(--font-mono); font-size: 12px; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .check { color: #22c55e; font-weight: 700; }
  .use { color: var(--dim); font-size: 11px; }
  .empty { padding: 16px 12px; color: var(--dim); font-size: 13px; margin: 0; }
  .foot {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-top: 1px solid var(--border); font-size: 11px;
  }
  .foot .dim { color: var(--dim); }
  .refresh { background: none; border: none; color: var(--text); cursor: pointer; font-size: 11px; padding: 4px 8px; border-radius: 4px; }
  .refresh:hover { background: var(--hover); }
</style>

<script lang="ts">
  // Xiryn-style provider row: collapsible, status dot, test connection.
  export interface CatalogEntry {
    id: string;
    name: string;
    local: boolean;
    requires_key: boolean;
    requires_base_url: boolean;
    key_env: string | null;
    base_url_env: string | null;
    default_base_url: string | null;
    suggested: string[];
    keys_url: string | null;
    key_set: boolean;
    base_url_set: boolean;
  }

  let { entry, onChanged }: { entry: CatalogEntry; onChanged: () => void } = $props();

  let open = $state(false);
  let key = $state("");
  let baseUrl = $state("");
  let testing = $state(false);
  let saving = $state(false);
  let testResult = $state<{ ok: boolean; msg: string } | null>(null);

  const configured = $derived(
    entry.local ? true : entry.requires_base_url ? entry.base_url_set : entry.key_set
  );

  async function save() {
    const k = key.trim();
    const u = baseUrl.trim();
    if (!k && !u) return;
    saving = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      if (entry.id === "routerku" && u) {
        await invoke("routerku_set_base_url", { baseUrl: u });
      }
      if (k) {
        await invoke("provider_connect", { providerId: entry.id, apiKey: k });
      }
      key = "";
      baseUrl = "";
      testResult = { ok: true, msg: "Tersimpan." };
      // Optimistic: langsung anggap terkonfigurasi agar tombol Tes aktif
      // (onChanged akan refresh status resmi dari backend)
      entry = { ...entry, key_set: true };
      onChanged();
    } catch (e) {
      testResult = { ok: false, msg: String(e) };
    } finally {
      saving = false;
    }
  }

  async function test() {
    testing = true;
    testResult = null;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const models = await invoke<string[]>("fetch_provider_models", { providerId: entry.id });
      testResult = { ok: true, msg: `${models.length} model tersedia` };
    } catch (e) {
      testResult = { ok: false, msg: String(e) };
    } finally {
      testing = false;
    }
  }
</script>

<div class="prov" class:open>
  <button class="prov-head" onclick={() => (open = !open)} aria-expanded={open}>
    <span class="dot {configured ? 'on' : 'off'}"></span>
    <span class="prov-name">{entry.name}</span>
    {#if entry.local}
      <span class="tag">lokal</span>
    {/if}
    <span class="spacer"></span>
    {#if entry.key_set}
      <span class="src-badge">tersimpan</span>
    {:else if !entry.requires_key}
      <span class="src-badge dim">default</span>
    {/if}
    <span class="chev">{open ? "▾" : "▸"}</span>
  </button>
  {#if open}
    <div class="prov-body">
      {#if entry.requires_key}
        <div class="field">
          <label for="key-{entry.id}">
            API key
            {#if entry.key_env}<span class="env">{entry.key_env}</span>{/if}
          </label>
          <input
            id="key-{entry.id}"
            type="password"
            bind:value={key}
            placeholder={entry.key_set ? "•••••••• (kosongkan untuk tetap)" : "Tempel key"}
            autocomplete="off"
            spellcheck={false}
          />
        </div>
      {/if}
      <div class="field">
        <label for="url-{entry.id}">
          Endpoint{entry.requires_base_url || entry.local ? "" : " (opsional)"}
          {#if entry.base_url_env}<span class="env">{entry.base_url_env}</span>{/if}
        </label>
        <input
          id="url-{entry.id}"
          type="text"
          bind:value={baseUrl}
          placeholder={entry.default_base_url ?? "https://endpoint-anda/v1"}
          spellcheck={false}
        />
      </div>
      <div class="row">
        <button class="btn-primary sm" onclick={save} disabled={saving || (!key.trim() && !baseUrl.trim())}>
          {saving ? "..." : "Simpan"}
        </button>
        <button class="btn-test sm" onclick={test} disabled={testing || !configured}>
          {testing ? "..." : "Tes koneksi"}
        </button>
        {#if entry.keys_url}
          <a href={entry.keys_url} target="_blank" rel="noopener" class="btn-ghost sm link">Dapatkan key ↗</a>
        {/if}
      </div>
      {#if testResult}
        <div class="notice {testResult.ok ? 'ok' : 'err'}" role="status">
          <span class="notice-icon">{testResult.ok ? "✓" : "✕"}</span>
          <span>{testResult.msg}</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .prov { border: 1px solid var(--border); border-radius: 10px; margin-bottom: 8px; overflow: hidden; }
  .prov-head {
    width: 100%; display: flex; align-items: center; gap: 10px;
    padding: 12px 14px; background: none; border: none; cursor: pointer;
    color: var(--text); text-align: left; font-size: 14px;
  }
  .prov-head:hover { background: var(--hover); }
  .dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .dot.on { background: #22c55e; }
  .dot.off { background: var(--dim); opacity: 0.5; }
  .prov-name { font-weight: 600; }
  .tag {
    font-size: 10px; padding: 2px 8px; border-radius: 999px;
    border: 1px solid var(--border); color: var(--dim); font-family: var(--font-mono);
  }
  .spacer { flex: 1; }
  .src-badge {
    font-size: 10px; padding: 2px 8px; border-radius: 4px; font-family: var(--font-mono);
    border: 1px solid rgba(34,197,94,.4); color: #22c55e;
  }
  .src-badge.dim { border-color: var(--border); color: var(--dim); }
  .chev { color: var(--dim); font-size: 12px; }
  .prov-body { padding: 0 14px 14px; border-top: 1px solid var(--border); background: rgba(0,0,0,.15); }
  .field { margin: 12px 0; }
  .field label { display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--dim); text-transform: uppercase; letter-spacing: .04em; }
  .field .env { font-family: var(--font-mono); font-weight: 400; text-transform: none; }
  .field input {
    width: 100%; padding: 9px 12px; border: 1px solid var(--border); border-radius: 8px;
    background: var(--bg); color: var(--text); font-size: 13px; font-family: var(--font-mono);
  }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .btn-primary.sm { background: #3b82f6; color: #fff; border: none; padding: 8px 16px; border-radius: 8px; font-weight: 600; cursor: pointer; font-size: 13px; }
  .btn-primary.sm:disabled { opacity: .5; }
  .btn-ghost.sm {
    padding: 8px 14px; border: 1px solid var(--border); border-radius: 8px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 13px; text-decoration: none;
  }
  .btn-ghost.sm:disabled { opacity: .5; }
  .btn-test.sm {
    padding: 8px 16px; border-radius: 8px; font-size: 13px; font-weight: 700;
    cursor: pointer; border: 1px solid rgba(79,166,240,.55); color: #4FA6F0;
    background: rgba(79,166,240,.1);
  }
  .btn-test.sm:hover:not(:disabled) {
    background: rgba(79,166,240,.2); box-shadow: 0 0 14px rgba(79,166,240,.3);
  }
  .btn-test.sm:disabled { opacity: .5; cursor: default; }
  .notice {
    display: flex; align-items: center; gap: 10px;
    font-size: 13px; font-weight: 600; font-family: var(--font-mono);
    margin: 12px 0 0; padding: 10px 14px; border-radius: 10px;
    border: 1px solid; animation: pop-in .25s ease-out;
  }
  .notice.ok {
    color: #4ade80; background: rgba(34,197,94,.12);
    border-color: rgba(34,197,94,.45);
    box-shadow: 0 0 18px rgba(34,197,94,.18);
  }
  .notice.err {
    color: #f87171; background: rgba(239,68,68,.1);
    border-color: rgba(239,68,68,.45);
    box-shadow: 0 0 18px rgba(239,68,68,.15);
  }
  .notice-icon {
    display: inline-flex; align-items: center; justify-content: center;
    width: 22px; height: 22px; border-radius: 50%; font-size: 13px; font-weight: 800;
    flex-shrink: 0;
  }
  .notice.ok .notice-icon { background: #22c55e; color: #052e12; }
  .notice.err .notice-icon { background: #ef4444; color: #450a0a; }
  @keyframes pop-in {
    from { opacity: 0; transform: translateY(-4px) scale(.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
  .ok { color: #22c55e; font-size: 12px; font-family: var(--font-mono); margin: 8px 0 0; }
  .err { color: #ef4444; font-size: 12px; font-family: var(--font-mono); margin: 8px 0 0; }
</style>

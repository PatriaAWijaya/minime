<script lang="ts">
  // Hidden diagnostics view: shows recent app logs for bug reports.
  // Accessible via 5 clicks on the app version in Settings → Updates.
  import { readLogs, pruneLogs } from "../lib/logger";

  let logs = $state<string[]>([]);
  let loading = $state(false);
  let pruning = $state(false);
  let filter = $state<"all" | "info" | "warn" | "error">("all");
  let message = $state("");
  let orchHistory = $state<Array<[string, string, number, number, string, number | null]>>([]);
  let orchLoading = $state(false);

  async function loadOrchHistory() {
    orchLoading = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      orchHistory = await invoke("get_orchestration_history", { limit: 50 });
    } catch (e) {
      message = "Gagal muat riwayat: " + String(e);
    } finally {
      orchLoading = false;
    }
  }

  let parseHealth = $state<[number, number, number] | null>(null);

  async function loadParseHealth() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      parseHealth = await invoke("get_parse_health");
    } catch (e) {
      console.error("Gagal muat parse health:", e);
    }
  }

  // Muat saat komponen mount
  $effect(() => {
    loadOrchHistory();
    loadParseHealth();
  });

  async function load() {
    loading = true;
    message = "";
    try {
      logs = await readLogs(500);
    } catch (e) {
      message = `Gagal membaca log: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function prune() {
    if (!confirm("Hapus file log yang lebih lama dari 14 hari?")) return;
    pruning = true;
    try {
      const n = await pruneLogs(14);
      message = `${n} file log lama dihapus.`;
      await load();
    } catch (e) {
      message = `Gagal: ${e}`;
    } finally {
      pruning = false;
    }
  }

  function copyLogs() {
    const text = filtered.join("\n");
    navigator.clipboard.writeText(text).then(
      () => (message = `${filtered.length} baris disalin ke clipboard.`),
      () => (message = "Gagal menyalin.")
    );
  }

  const filtered = $derived(
    filter === "all" ? logs : logs.filter((l) => l.includes(`[${filter.toUpperCase()}]`))
  );

  function levelClass(line: string): string {
    if (line.includes("[ERROR]")) return "lv-error";
    if (line.includes("[WARN]")) return "lv-warn";
    return "lv-info";
  }

  $effect(() => {
    void load();
  });
</script>

<div class="diag">
  <h3>Diagnostik</h3>
  <p class="hint">Log aplikasi — salin dan kirim saat melaporkan bug. Log tersimpan di <code>~/.minime/logs/</code>.</p>

  <div class="row">
    <div class="seg">
      {#each ["all", "info", "warn", "error"] as f}
        <button class:active={filter === f} onclick={() => (filter = f as typeof filter)}>
          {f === "all" ? "Semua" : f.toUpperCase()}
        </button>
      {/each}
    </div>
    <span class="spacer"></span>
    <button class="btn-ghost sm" onclick={load} disabled={loading}>
      {loading ? "Memuat..." : "↻ Muat ulang"}
    </button>
    <button class="btn-ghost sm" onclick={copyLogs} disabled={filtered.length === 0}>
      Salin
    </button>
    <button class="btn-ghost sm danger" onclick={prune} disabled={pruning}>
      {pruning ? "..." : "Bersihkan log lama"}
    </button>
  </div>

  {#if message}<p class="msg">{message}</p>{/if}

  {#if parseHealth && parseHealth[2] > 20}
    <div class="warn-box">
      ⚠️ Parse failure rate tinggi: {parseHealth[2].toFixed(1)}% ({parseHealth[1]}/{parseHealth[0]} gagal dalam 24 jam).
      Prompt template mungkin perlu diperbaiki.
    </div>
  {/if}

  <!-- Tab Orkestrasi (saran #3) -->
  <section class="diag-section">
    <div class="section-head">
      <h3>Riwayat Orkestrasi</h3>
      <button class="btn-ghost sm" onclick={loadOrchHistory} disabled={orchLoading}>
        {orchLoading ? "Memuat..." : "Muat Ulang"}
      </button>
    </div>
    {#if orchHistory.length === 0}
      <p class="empty">Belum ada riwayat orkestrasi.</p>
    {:else}
      <table class="orch-table">
        <thead>
          <tr><th>Waktu</th><th>Agen</th><th>Prompt</th><th>Skor</th><th>Model</th><th>QC</th></tr>
        </thead>
        <tbody>
          {#each orchHistory as [waktu, agen, plen, cscore, model, qc]}
            <tr>
              <td class="mono">{waktu.slice(0, 16)}</td>
              <td>{agen}</td>
              <td class="mono">{plen} chr / skor {cscore}</td>
              <td class="mono">{model.replace("muse:", "")}</td>
              <td>{qc === null ? "—" : qc}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  <div class="logbox">
    {#if loading}
      <p class="dim">Memuat log...</p>
    {:else if filtered.length === 0}
      <p class="dim">Belum ada log. Log tercatat saat aplikasi berjalan.</p>
    {:else}
      {#each filtered as line}
        <div class="line {levelClass(line)}">{line}</div>
      {/each}
    {/if}
  </div>
  <p class="count">{filtered.length} baris</p>
</div>

<style>
  .diag { padding: 4px 2px; }
  .diag h3 { margin: 0 0 4px; font-size: 16px; }
  .hint { color: var(--dim); font-size: 13px; margin: 0 0 12px; }
  .hint code { font-family: var(--font-mono); font-size: 12px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin-bottom: 10px; }
  .spacer { flex: 1; }
  .seg { display: flex; border: 1px solid var(--border); border-radius: 8px; overflow: hidden; }
  .seg button {
    background: transparent; border: none; color: var(--dim);
    padding: 7px 12px; font-size: 12px; font-weight: 600; cursor: pointer;
  }
  .seg button.active { background: var(--hover); color: var(--text); }
  .btn-ghost.sm {
    padding: 7px 12px; border: 1px solid var(--border); border-radius: 8px;
    background: transparent; color: var(--text); cursor: pointer; font-size: 12px;
  }
  .btn-ghost.sm:disabled { opacity: .5; }
  .btn-ghost.sm.danger { color: #f87171; border-color: rgba(239,68,68,.4); }
  .msg { font-size: 12px; color: var(--dim); font-family: var(--font-mono); }
  .logbox {
    background: #0d0f12; border: 1px solid var(--border); border-radius: 10px;
    padding: 10px 12px; max-height: 420px; overflow-y: auto;
    font-family: var(--font-mono); font-size: 11.5px; line-height: 1.55;
  }
  .line { white-space: pre-wrap; word-break: break-all; padding: 1px 0; }
  .lv-error { color: #f87171; }
  .lv-warn { color: #fbbf24; }
  .lv-info { color: #9ca3af; }
  .dim { color: var(--dim); }
  .count { font-size: 11px; color: var(--dim); margin-top: 6px; }
</style>

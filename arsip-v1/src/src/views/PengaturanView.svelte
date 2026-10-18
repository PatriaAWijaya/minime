<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "../lib/daemon";
  import type { DaemonStatus } from "../lib/types";

  interface Provider {
    account: string;
    name: string;
    desc: string;
  }

  const PROVIDERS: Provider[] = [
    { account: "gemini", name: "Gemini", desc: "Orkestrasi, riset, creative — model default" },
    { account: "muse", name: "Muse", desc: "Model default berbayar (Meta Model API)" },
    { account: "deepseek", name: "DeepSeek", desc: "Strategi bisnis, code" },
    { account: "valyu", name: "Valyu", desc: "Riset finansial premium (fallback)" },
    { account: "fred", name: "FRED", desc: "Data ekonomi AS (gratis)" },
    { account: "postiz", name: "Viralin (postiz)", desc: "API key untuk draft & scheduling" },
  ];

  let status = $state<DaemonStatus | null>(null);
  let hasKey = $state<Record<string, boolean>>({});
  let inputs = $state<Record<string, string>>({});
  let saving = $state<Record<string, boolean>>({});
  let msg = $state<Record<string, string>>({});

  async function refreshKeys() {
    for (const p of PROVIDERS) {
      try {
        const r = await api.secretsHas(p.account);
        hasKey[p.account] = r.has;
      } catch {
        hasKey[p.account] = false;
      }
    }
  }

  async function saveKey(account: string) {
    const value = (inputs[account] ?? "").trim();
    if (!value) {
      msg[account] = "Key tidak boleh kosong.";
      return;
    }
    saving[account] = true;
    msg[account] = "";
    try {
      await api.secretsSet(account, value);
      hasKey[account] = true;
      inputs[account] = "";
      msg[account] = "Tersimpan di Keychain.";
    } catch (e) {
      msg[account] = `Gagal: ${e instanceof Error ? e.message : e}`;
    } finally {
      saving[account] = false;
    }
  }

  onMount(async () => {
    try {
      status = await api.daemonStatus();
    } catch {
      status = null;
    }
    await refreshKeys();
  });
</script>

<h1 class="section-title">Pengaturan</h1>
<p class="supporting">Pengaturan aplikasi, provider model, dan status daemon.</p>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 16px 0;">Daemon</h2>
  {#if status}
    <p class="agent-meta">Versi: {status.version}</p>
    <p class="agent-meta">Uptime: {status.uptime_secs} detik</p>
    <p class="agent-meta">Agen: {status.agents} · Antre: {status.queued_tasks} · Berjalan: {status.running_tasks}</p>
  {:else}
    <p class="agent-meta">Tidak terhubung ke daemon.</p>
  {/if}
</div>

<div class="card">
  <h2 style="font-size: 20px; margin: 0 0 8px 0;">Providers</h2>
  <p class="supporting" style="margin-bottom: 20px;">
    API key tersimpan di OS Keychain (write-only — nilai tidak pernah ditampilkan kembali).
  </p>
  {#if !isTauri}
    <p class="agent-meta" style="margin-bottom: 16px;">Mode demo — penyimpanan key hanya simulasi.</p>
  {/if}
  {#each PROVIDERS as p}
    <div class="provider-row">
      <div class="provider-info">
        <strong>{p.name}</strong>
        <span class="agent-meta">{p.desc}</span>
        <span class="key-status" class:has={hasKey[p.account]}>
          {hasKey[p.account] ? "● key tersimpan" : "○ belum ada key"}
        </span>
      </div>
      <div class="provider-form">
        <label for="key-{p.account}">API key {p.name}</label>
        <div class="input-row">
          <input
            id="key-{p.account}"
            type="password"
            placeholder={hasKey[p.account] ? "•••••••• (isi untuk ganti)" : "Tempel API key"}
            bind:value={inputs[p.account]}
            autocomplete="off"
          />
          <button
            class="btn-primary"
            disabled={saving[p.account]}
            onclick={() => saveKey(p.account)}
          >
            {saving[p.account] ? "Menyimpan…" : "Simpan"}
          </button>
        </div>
        {#if msg[p.account]}
          <p class="agent-meta">{msg[p.account]}</p>
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .provider-row {
    padding: 16px 0;
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .provider-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .provider-info strong {
    font-size: 16px;
  }
  .key-status {
    font-size: 14px;
    color: #999;
  }
  .key-status.has {
    color: #2e7d32;
  }
  .provider-form label {
    display: block;
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 8px;
  }
  .input-row {
    display: flex;
    gap: 12px;
  }
  .input-row input {
    flex: 1;
    min-height: 44px;
    font-size: 16px;
    padding: 0 12px;
    border: 1px solid var(--border, #ccc);
    border-radius: 8px;
  }
  .input-row .btn-primary {
    min-height: 44px;
    padding: 0 24px;
    font-size: 16px;
  }
</style>

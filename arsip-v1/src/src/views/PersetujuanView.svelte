<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/daemon";
  import type { Approval } from "../lib/types";

  let approvals = $state<Approval[]>([]);
  let loading = $state(true);
  let acting = $state<string | null>(null);
  let rejectNote = $state<Record<string, string>>({});
  let error = $state<string | null>(null);

  async function refresh() {
    loading = true;
    error = null;
    try {
      approvals = await api.approvalsList("pending");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function decide(a: Approval, decision: "approve" | "reject") {
    if (decision === "reject" && !(rejectNote[a.id] ?? "").trim()) {
      error = "Tulis alasan penolakan dulu — agen butuh instruksi perbaikan yang konkret.";
      return;
    }
    acting = a.id;
    error = null;
    try {
      await api.approvalsDecide(a.id, decision, rejectNote[a.id] ?? "");
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      acting = null;
    }
  }

  onMount(refresh);
</script>

<h1 class="section-title">Persetujuan</h1>
<p class="supporting">Output yang menunggu keputusan Patria. Tidak ada aksi eksternal tanpa persetujuan.</p>

{#if error}
  <div class="alert-error" role="alert">{error}</div>
{/if}

<div class="card">
  {#if loading}
    <div class="empty">Memuat…</div>
  {:else if approvals.length === 0}
    <div class="empty">Tidak ada yang menunggu persetujuan.</div>
  {:else}
    {#each approvals as a (a.id)}
      <article class="approval-item">
        <div class="approval-head">
          <p class="approval-stage">Stage: {a.stage}</p>
          {#if a.qc_score !== null && a.qc_score !== undefined}
            <span
              class="score-badge"
              class:score-pass={a.qc_score >= 75}
              class:score-fail={a.qc_score < 75}
            >
              Skor QC: {a.qc_score}/100
            </span>
          {/if}
        </div>
        <p class="agent-meta approval-summary">{a.payload_summary}</p>
        <p class="approval-time">Diminta: {a.created_at}</p>

        <label class="field-label" for={"note-" + a.id}>Catatan untuk agen (wajib bila menolak)</label>
        <textarea
          id={"note-" + a.id}
          class="note-input"
          rows="2"
          placeholder="Contoh: perbaiki bagian strategi harga — tambahkan angka proyeksi 3 tahun."
          bind:value={rejectNote[a.id]}
        ></textarea>

        <div class="approval-actions">
          <button
            class="btn-primary"
            disabled={acting === a.id}
            onclick={() => decide(a, "approve")}
          >
            {acting === a.id ? "Memproses…" : "Setujui"}
          </button>
          <button
            class="btn-secondary"
            disabled={acting === a.id}
            onclick={() => decide(a, "reject")}
          >
            Tolak & minta revisi
          </button>
        </div>
      </article>
    {/each}
  {/if}
</div>

<style>
  .approval-item {
    margin-bottom: 28px;
    padding-bottom: 28px;
    border-bottom: 1px solid var(--border);
  }
  .approval-item:last-child {
    border-bottom: none;
    margin-bottom: 0;
    padding-bottom: 0;
  }
  .approval-head {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 8px;
  }
  .approval-stage {
    font-size: 18px;
    font-weight: 700;
    margin: 0;
  }
  .score-badge {
    font-size: 15px;
    font-weight: 700;
    padding: 6px 14px;
    border-radius: 999px;
  }
  .score-pass {
    background: #e8f5e9;
    color: #1b5e20;
  }
  .score-fail {
    background: #fdecea;
    color: #b71c1c;
  }
  .approval-summary {
    white-space: pre-wrap;
    font-size: 15px;
    line-height: 1.6;
  }
  .approval-time {
    font-size: 13px;
    color: var(--muted);
  }
  .field-label {
    display: block;
    font-size: 14px;
    font-weight: 600;
    margin: 16px 0 6px;
  }
  .note-input {
    width: 100%;
    font-size: 15px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    resize: vertical;
    box-sizing: border-box;
  }
  .approval-actions {
    display: flex;
    gap: 12px;
    margin-top: 16px;
  }
  .approval-actions button {
    min-height: 44px;
    padding: 0 28px;
    font-size: 16px;
  }
  .alert-error {
    background: #fdecea;
    color: #b71c1c;
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 16px;
    font-size: 15px;
  }
  .empty {
    padding: 32px 0;
    text-align: center;
    font-size: 16px;
    color: var(--muted);
  }
</style>

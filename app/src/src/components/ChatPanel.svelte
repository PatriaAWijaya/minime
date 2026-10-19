<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import { getAgent } from "../lib/agents";  import type { Agent } from "../lib/agents";
  import {
    activeAgentId,
    agentModel,
    agentStatus,
    buildSystemPrompt,
    defaultModelFor,
    getModel,
    loadChatHistory,
    messagesByAgent,
    modelCatalog,
    newTaskSignal,
    pushMessage,
    sendChatViaRouter,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  let {
    agent: agentProp = null,
    header = true,
  }: { agent?: Agent | null; header?: boolean } = $props();

  let draft = $state("");
  let inputEl: HTMLTextAreaElement | null = $state(null);
  let timer: ReturnType<typeof setTimeout> | null = null;

  // Mock voice input (Phase 1): local recording state per panel.
  let recording = $state(false);
  let recSeconds = $state(0);
  let recTimer: ReturnType<typeof setInterval> | null = null;

  const agent = $derived(agentProp ?? getAgent($activeAgentId));
  const messages = $derived($messagesByAgent[$activeAgentId] ?? []);
  const status = $derived($agentStatus[$activeAgentId] ?? "idle");

  // "+ Tugas Baru" focuses the input so the user can type immediately.
  const unsub = newTaskSignal.subscribe(() => {
    queueMicrotask(() => inputEl?.focus());
  });

  // Muat riwayat chat dari SQLite saat panel dibuka / agen berganti.
  let lastHistoryAgent = $state("");
  $effect(() => {
    const aid = agent.id;
    if (aid && aid !== lastHistoryAgent) {
      lastHistoryAgent = aid;
      loadChatHistory(aid);
    }
  });
  onMount(() => {
    // Pulihkan status yang nyangkut "working" (mis. timer mock terhapus
    // saat HMR/reload sebelum sempat mengembalikan idle).
    agentStatus.update((st) => ({ ...st, [agent.id]: "idle" }));
    if (agent.id) loadChatHistory(agent.id);
  });
  onDestroy(() => {
    unsub();
    if (timer) clearTimeout(timer);
    if (recTimer) clearInterval(recTimer);
  });

  function startRecording() {
    if (recording || status === "working") return;
    recording = true;
    recSeconds = 0;
    recTimer = setInterval(() => { recSeconds += 1; }, 1000);
  }

  function stopRecording() {
    if (recTimer) clearInterval(recTimer);
    recTimer = null;
    if (!recording) return;
    recording = false;
    const transcript = $t("chat.mockTranscript");
    draft = draft.trim() ? `${draft.trim()} ${transcript}` : transcript;
    queueMicrotask(() => inputEl?.focus());
  }

  // Stops the timer without injecting a transcript (used when sending mid-record).
  function stopRecordingSilent() {
    if (recTimer) clearInterval(recTimer);
    recTimer = null;
    recording = false;
  }

  function formatTime(s: number) {
    const m = Math.floor(s / 60).toString().padStart(2, "0");
    const sec = (s % 60).toString().padStart(2, "0");
    return `${m}:${sec}`;
  }

  async function send() {
    const text = draft.trim();
    if (!text || status === "working") return;
    if (recording) stopRecordingSilent();
    const id = agent.id;
    const translate = $t;
    pushMessage(id, "user", text);
    draft = "";
    agentStatus.update((s) => ({ ...s, [id]: "working" }));

    try {
      // Tahap 2: coba model router nyata dulu; fallback ke mock lokal bila
      // backend belum tersedia (browser/preview) atau semua provider gagal.
      let mid = $agentModel[id];
      if (!mid || !get(modelCatalog).some((m) => m.id === mid)) {
        mid = defaultModelFor(id);
      }
      const mopt = getModel(mid);
      const tier = mopt.kind === "paid" ? "berbayar" : "gratis";
      const systemPrompt = buildSystemPrompt(id, translate(agent.roleKey));
      const history = ($messagesByAgent[id] ?? []).map((m) => ({
        role: m.role === "agent" ? "assistant" : "user",
        content: m.text,
      }));
      const res = await sendChatViaRouter(id, mid, tier, systemPrompt, history);
      if (res && res.error && res.error.trim()) {
        // Router gagal tapi menyampaikannya sebagai field (bukan rejection).
        pushMessage(id, "agent", res.error.trim());
      } else if (res && res.text.trim()) {
        pushMessage(id, "agent", res.text);
      } else if (res === null) {
        // Bukan di Tauri (browser/preview): pakai mock lokal.
        timer = setTimeout(() => {
          pushMessage(id, "agent", translate("chat.mockReply", { name: agent.name }));
          agentStatus.update((s) => ({ ...s, [id]: "idle" }));
        }, 1400);
        return;
      }
    } catch (e) {
      // Router gagal / timeout: tampilkan alasan aslinya agar jelas
      // apa yang kurang (mis. belum ada provider terhubung).
      const msg =
        typeof e === "string" && e.trim()
          ? e.trim()
          : e instanceof Error && e.message
            ? e.message
            : translate("chat.routerFailed");
      pushMessage(id, "agent", msg);
    } finally {
      agentStatus.update((s) => ({ ...s, [id]: "idle" }));
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }
</script>

<section class="chat">
  {#if header}
  <header class="chat-head">
    <span class="badge" style:background={agent.color}>{agent.badge}</span>
    <div class="who">
      <div class="name">{agent.name}</div>
      <div class="role">{$t(agent.roleKey)} · {agent.model}</div>
    </div>
    <span class="status"><span class="dot {status}"></span>{$t(`status.${status}`)}</span>
  </header>
  {/if}

  <div class="messages">
    {#if messages.length === 0}
      <div class="empty">
        <span class="badge big" style:background={agent.color}>{agent.badge}</span>
        <p>{$t("chat.empty", { name: agent.name })}</p>
      </div>
    {:else}
      {#each messages as m (m.id)}
        <div class="msg {m.role}">
          <div class="bubble">
            <p>{m.text}</p>
            <span class="time">{m.time}</span>
          </div>
        </div>
      {/each}
    {/if}
    {#if status === "working"}
      <div class="msg agent">
        <div class="bubble thinking">{$t("chat.thinking", { name: agent.name })}</div>
      </div>
    {/if}
  </div>

  <div class="composer">
    <div class="field">
      <textarea
        bind:this={inputEl}
        bind:value={draft}
        rows="1"
        placeholder={$t("chat.placeholder")}
        onkeydown={onKey}
        aria-label={$t("chat.placeholder")}
      ></textarea>
    </div>
    {#if recording}
      <div class="rec-indicator" role="status" aria-live="polite" aria-label={$t("chat.recording")}>
        <span class="rec-dot"></span>
        <span class="rec-time">{formatTime(recSeconds)}</span>
      </div>
      <button class="btn-ghost btn-stop" onclick={stopRecording} aria-label={$t("chat.stop")}>
        {$t("chat.stop")}
      </button>
    {:else}
      <button
        class="btn-mic"
        onclick={startRecording}
        disabled={status === "working"}
        aria-label={$t("chat.record")}
        title={$t("chat.record")}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="9" y="2" width="6" height="12" rx="3" />
          <path d="M5 10a7 7 0 0 0 14 0" />
          <line x1="12" y1="17" x2="12" y2="22" />
        </svg>
      </button>
    {/if}
    <button class="btn-primary" onclick={send} disabled={!draft.trim()}>
      {$t("chat.send")}
    </button>
  </div>
</section>

<style>
  .chat {
    display: flex; flex-direction: column;
    min-height: 0; min-width: 0;
    background: var(--bg);
  }
  .chat-head {
    display: flex; align-items: center; gap: 12px;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .badge {
    width: 44px; height: 44px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    color: #0a1f33; font-size: 20px; font-weight: 800; flex: none;
  }
  .badge.big { width: 64px; height: 64px; font-size: 30px; }
  .who { flex: 1; min-width: 0; }
  .who .name { font-size: 16px; font-weight: 800; }
  .who .role { font-size: 13px; color: var(--dim); }
  .status { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--dim); }
  .messages {
    flex: 1; overflow-y: auto;
    padding: 20px;
    display: flex; flex-direction: column; gap: 12px;
  }
  .empty {
    margin: auto; text-align: center; color: var(--dim);
    display: flex; flex-direction: column; align-items: center; gap: 12px;
    max-width: 320px;
  }
  .msg { display: flex; }
  .msg.user { justify-content: flex-end; }
  .bubble {
    max-width: 70%;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }
  .msg.user .bubble { background: var(--accent); border-color: var(--accent); color: var(--accent-ink); }
  .bubble p { font-size: 15px; }
  .time { display: block; font-size: 11px; opacity: 0.7; margin-top: 4px; text-align: right; }
  .thinking { font-style: italic; color: var(--dim); animation: pulse 1.2s infinite; }
  .composer {
    display: flex; gap: 10px; align-items: flex-end;
    padding: 12px 20px 16px;
    border-top: 1px solid var(--border);
    background: var(--panel);
  }
  .composer .field { flex: 1; }
  .composer textarea { resize: none; }
  .btn-mic {
    min-width: 44px; min-height: 44px;
    display: flex; align-items: center; justify-content: center;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: transparent;
    color: var(--dim);
    cursor: pointer;
    flex: none;
  }
  .btn-mic:hover { background: var(--panel-2); color: var(--text); }
  .btn-mic:disabled { opacity: 0.45; cursor: default; }
  .btn-stop { min-height: 44px; flex: none; }
  .rec-indicator {
    display: flex; align-items: center; gap: 8px;
    min-height: 44px; padding: 0 12px;
    font-size: 14px; color: var(--text);
    flex: none;
  }
  .rec-dot {
    width: 12px; height: 12px; border-radius: 50%;
    background: #ef4444;
    animation: recpulse 1.2s infinite;
    flex: none;
  }
  @keyframes recpulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.45; transform: scale(0.85); }
  }
</style>

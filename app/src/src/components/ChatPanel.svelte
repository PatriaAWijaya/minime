<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { get } from "svelte/store";
  import { getAgent, agentDisplayName, agentDisplayRole } from "../lib/agents";  import type { Agent } from "../lib/agents";
  import type { AttachmentMeta, Message } from "../lib/types";
  import {
    activeAgentId,
    agentModel,
    agentNames,
    agentStatus,
    agentTitles,
    buildSystemPrompt,
    defaultModelFor,
    getModel,
    loadChatHistory,
    messagesByAgent,
    modelCatalog,
    newTaskSignal,
    realProviders,
    refreshProviders,
    isTauri,
    settingsPrefs,
    deleteMessage,
    pushMessage,
    setMessageReaction,
    sendChatViaRouter,
    sendOrchestrated,
    shouldOrchestrate,
    addLibraryItem,
    addKanbanCard,
    moveKanbanCard,
    ensureFlowAgents,
  } from "../lib/store";
  import { t } from "../lib/i18n";
  import MetaStatus from "./MetaStatus.svelte";

  let {
    agent: agentProp = null,
    header = true,
  }: { agent?: Agent | null; header?: boolean } = $props();

  let draft = $state("");
  let inputEl: HTMLTextAreaElement | null = $state(null);
  let timer: ReturnType<typeof setTimeout> | null = null;
  let fileInput: HTMLInputElement | null = $state(null);

  // Lampiran: gambar, audio, video, dokumen — untuk pengaya skills / analisa project.
  let attachments = $state<AttachmentMeta[]>([]);

  function kindOf(file: File): AttachmentMeta["kind"] {
    if (file.type.startsWith("image/")) return "image";
    if (file.type.startsWith("audio/")) return "audio";
    if (file.type.startsWith("video/")) return "video";
    return "document";
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  function onFilesPicked(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const files = input.files;
    if (!files) return;
    for (const f of Array.from(files)) {
      if (attachments.length >= 5) break;
      attachments = [...attachments, { name: f.name, kind: kindOf(f), size: formatSize(f.size) }];
    }
    input.value = "";
  }

  function removeAttachment(i: number) {
    attachments = attachments.filter((_, idx) => idx !== i);
  }

  // Per-message action menu (reply / reaction / copy / delete).
  let openMenuId = $state<number | null>(null);
  let emojiForId = $state<number | null>(null);
  const REACTIONS = ["👍", "❤️", "😂", "😮", "😢", "🙏"];

  function toggleMenu(id: number) {
    openMenuId = openMenuId === id ? null : id;
    emojiForId = null;
  }

  function closeMenus() {
    openMenuId = null;
    emojiForId = null;
  }

  function doReply(m: Message) {
    const quote = m.text.length > 80 ? m.text.slice(0, 80) + "…" : m.text;
    draft = `> ${quote}\n`;
    closeMenus();
    queueMicrotask(() => inputEl?.focus());
  }

  async function doCopy(m: Message) {
    try {
      await navigator.clipboard.writeText(m.text);
    } catch {
      /* clipboard unavailable in this context */
    }
    closeMenus();
  }

  function doReact(m: Message, emoji: string) {
    setMessageReaction(agent.id, m.id, emoji);
    closeMenus();
  }

  function doDelete(m: Message) {
    deleteMessage(agent.id, m.id);
    closeMenus();
  }

  // STT nyata via meta_transcribe (24 Okt 2026): MediaRecorder → base64 → transcribe_audio_b64.
  let recording = $state(false);
  let recSeconds = $state(0);
  let recTimer: ReturnType<typeof setInterval> | null = null;
  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  let transcribing = $state(false);

  const agent = $derived(agentProp ?? getAgent($activeAgentId));
  const messages = $derived($messagesByAgent[$activeAgentId] ?? []);
  const status = $derived($agentStatus[$activeAgentId] ?? "idle");

  // Auto-scroll: tiap pesan baru / status berubah, gulir ke bawah agar
  // chat terbaru tidak terpotong di luar window.
  let messagesEl: HTMLDivElement | undefined = $state();
  $effect(() => {
    // Dependensi: panjang pesan + status (thinking) memicu scroll
    const _len = messages.length;
    const _status = status;
    if (messagesEl) {
      queueMicrotask(() => {
        if (messagesEl) {
          messagesEl.scrollTop = messagesEl.scrollHeight;
        }
      });
    }
  });

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

  async function startRecording() {
    if (recording || status === "working") return;
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      audioChunks = [];
      mediaRecorder = new MediaRecorder(stream);
      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) audioChunks.push(e.data);
      };
      mediaRecorder.start();
      recording = true;
      recSeconds = 0;
      recTimer = setInterval(() => { recSeconds += 1; }, 1000);
    } catch (e) {
      console.error("[STT] mic gagal:", e);
    }
  }

  async function stopRecording() {
    if (recTimer) clearInterval(recTimer);
    recTimer = null;
    if (!recording) return;
    recording = false;
    const rec = mediaRecorder;
    mediaRecorder = null;
    if (!rec) return;
    
    transcribing = true;
    try {
      const blob = await new Promise<Blob>((resolve) => {
        rec.onstop = () => resolve(new Blob(audioChunks, { type: rec.mimeType }));
        rec.stop();
        rec.stream.getTracks().forEach((t) => t.stop());
      });
      
      // Blob → base64
      const b64 = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
          const result = reader.result as string;
          resolve(result.split(",")[1]);
        };
        reader.onerror = reject;
        reader.readAsDataURL(blob);
      });
      
      const ext = (rec.mimeType.split("/")[1] || "webm").split(";")[0];
      const { invoke } = await import("@tauri-apps/api/core");
      const transcript = await invoke<string>("transcribe_audio_b64", {
        audioB64: b64,
        ext,
      });
      
      if (transcript && transcript.trim()) {
        draft = draft.trim() ? `${draft.trim()} ${transcript.trim()}` : transcript.trim();
      }
    } catch (e) {
      console.error("[STT] transkripsi gagal:", e);
    } finally {
      transcribing = false;
      queueMicrotask(() => inputEl?.focus());
    }
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
    const atts = attachments;
    if ((!text && atts.length === 0) || status === "working") return;
    if (recording) stopRecordingSilent();
    const id = agent.id;
    const translate = $t;
    pushMessage(id, "user", text || $t("chat.attachmentSent"), atts.length > 0 ? atts : undefined);
    // Kanban: kartu muncul SEJAK tugas diberikan (kolom Dikerjakan),
    // bukan hanya saat selesai.
    let kanbanId: number | null = null;
    try {
      const taskTitle = text.trim().length > 60
        ? text.trim().slice(0, 57) + "..."
        : text.trim() || "Tugas " + agent.name;
      kanbanId = addKanbanCard(taskTitle, id, "doing", "");
    } catch {}
    draft = "";
    attachments = [];
    agentStatus.update((s) => ({ ...s, [id]: "working" }));

    try {
      // Tahap 2: coba model router nyata dulu; fallback ke mock lokal bila
      // backend belum tersedia (browser/preview) atau semua provider gagal.
      // Muat provider nyata dulu agar validasi memakai katalog model asli.
      if (get(realProviders) === null && (await isTauri())) {
        await refreshProviders();
      }
      let mid = $agentModel[id];
      const catalog = get(modelCatalog);
      // Hanya reset jika katalog sudah dimuat (tidak kosong) DAN model tidak valid.
      // Jangan reset saat katalog belum siap — itu yang menyebabkan model "terkunci".
      if (!mid || (catalog.length > 0 && !catalog.some((m) => m.id === mid))) {
        mid = defaultModelFor(id);
        agentModel.update((m) => ({ ...m, [id]: mid }));
      }
      const mopt = getModel(mid);
      // Tier dari katalog BISA salah (getModel return "free" bila ID tak
      // ditemukan) → cek nama model langsung sebagai koreksi.
      const midLower = (mid || "").toLowerCase();
      const isPaidName = ["muse", "gemini", "deepseek", "claude", "gpt", "openai"].some((k) =>
        midLower.includes(k),
      );
      const isFreeName = ["ollama", "opencode"].some((k) => midLower.includes(k));
      const tier = isPaidName
        ? "berbayar"
        : isFreeName
          ? "gratis"
          : mopt.kind === "paid"
            ? "berbayar"
            : "gratis";
      // ORKESTRASI: bila pesan ke Mimi adalah tugas, delegasikan ke agen
      // yang tepat via orchestrate_task (bukan dijawab Mimi sendiri).
      if (shouldOrchestrate(id, text) && (await isTauri())) {
        try {
          const orch = await sendOrchestrated(text, mid);
          if (orch && !orch.error) {
            // Format hasil orkestrasi: agregasi + hasil per agen
            let out = orch.aggregated || "";
            if (orch.subtask_results && orch.subtask_results.length > 0) {
              const parts = orch.subtask_results
                .filter((st) => st.result && !st.error)
                .map((st) => `**${st.agent_id}**: ${st.result}`);
              if (parts.length > 0 && !out) {
                out = parts.join("\n\n");
              }
            }
            if (out.trim()) {
              pushMessage(id, "agent", out);
              // Simpan ke Library + Kanban Selesai (sama seperti chat sukses)
              const taskTitle = text.trim().length > 60
                ? text.trim().slice(0, 57) + "..."
                : text.trim();
              try {
                addLibraryItem({
                  title: taskTitle,
                  summary: out.length > 200 ? out.slice(0, 197) + "..." : out,
                  content: out,
                  type: "doc",
                  agentId: id,
                  taskKey: "orch-" + Date.now(),
                });
              } catch {}
              try {
                if (kanbanId !== null) moveKanbanCard(kanbanId, "done");
              } catch {}
              // Visualisasi Koneksi: agen yang didelegasikan otomatis muncul
              try {
                const delegated = (orch.subtask_results || [])
                  .map((st) => st.agent_id)
                  .filter(Boolean);
                if (delegated.length > 0) ensureFlowAgents(delegated);
              } catch {}
              return;
            }
          }
          // Bila orkestrasi gagal/kosong, lanjut ke chat langsung (fallback)
        } catch {
          // Fallback ke chat langsung
        }
      }
      const systemPrompt = buildSystemPrompt(id, translate(agent.roleKey));
      const history = ($messagesByAgent[id] ?? []).map((m) => ({
        role: m.role === "agent" ? "assistant" : "user",
        content: m.text,
      }));
      const res = await sendChatViaRouter(id, mid, tier, systemPrompt, history);
      if (res && res.error && res.error.trim()) {
        // Router gagal tapi menyampaikannya sebagai field (bukan rejection).
        // Bersihkan kode warna ANSI dari stderr provider.
        const clean = res.error.replace(/\x1b\[[0-9;]*m/g, "");
        pushMessage(id, "agent", clean.trim());
      } else if (res && res.text.trim()) {
        pushMessage(id, "agent", res.text);
        // Otomatis: tugas berhasil → output ke Library + proses ke Kanban
        // Judul dari prompt user (maks 60 char), tipe dari konten
        const userPrompt = text.trim();
        const title = userPrompt.length > 60
          ? userPrompt.slice(0, 57) + "..."
          : userPrompt || "Tugas " + agent.name;
        const summary = res.text.length > 200
          ? res.text.slice(0, 197) + "..."
          : res.text;
        try {
          addLibraryItem({
            title,
            summary,
            content: res.text,
            type: "doc",
            agentId: id,
            taskKey: "chat-" + Date.now(),
          });
        } catch {}
        // Sukses → pindah kartu ke Selesai
        try {
          if (kanbanId !== null) moveKanbanCard(kanbanId, "done");
        } catch {}
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
      // Gagal → pindah kartu ke Revisi (butuh perhatian)
      try {
        if (kanbanId !== null) moveKanbanCard(kanbanId, "revision");
      } catch {}
    } finally {
      agentStatus.update((s) => ({ ...s, [id]: "idle" }));
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key !== "Enter") return;
    // Send shortcut preference (Settings > General): "enter" sends on Enter,
    // "ctrlEnter" sends only on Ctrl/Cmd+Enter; Enter alone = new line.
    if ($settingsPrefs.sendShortcut === "ctrlEnter") {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        send();
      }
      return;
    }
    if (!e.shiftKey) {
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
      <div class="name">{agentDisplayName(agent, $agentNames, $t)}</div>
      <div class="role">{agentDisplayRole(agent, $agentTitles, $t)} · {agent.model}</div>
    </div>
    <span class="status"><span class="dot {status}"></span>{$t(`status.${status}`)}</span>
    <MetaStatus capability="Chat" compact={true} />
  </header>
  {/if}

  <div class="messages" bind:this={messagesEl}>
    {#if messages.length === 0}
      <div class="empty">
        <span class="badge big" style:background={agent.color}>{agent.badge}</span>
        <p>{$t("chat.empty", { name: agent.name })}</p>
      </div>
    {:else}
      {#each messages as m (m.id)}
        <div class="msg {m.role}">
          <div class="bubble">
            <button
              class="msg-menu-btn"
              onclick={(e) => { e.stopPropagation(); toggleMenu(m.id); }}
              aria-label={$t("chat.msgActions")}
              aria-haspopup="menu"
              aria-expanded={openMenuId === m.id}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <circle cx="12" cy="5" r="1.8" /><circle cx="12" cy="12" r="1.8" /><circle cx="12" cy="19" r="1.8" />
              </svg>
            </button>
            {#if openMenuId === m.id}
              <div class="msg-menu" role="menu">
                {#if emojiForId === m.id}
                  <div class="emoji-row" role="menuitem">
                    {#each REACTIONS as em (em)}
                      <button class="emoji-btn" onclick={(e) => { e.stopPropagation(); doReact(m, em); }} aria-label={em}>{em}</button>
                    {/each}
                  </div>
                {:else}
                  <button role="menuitem" class="msg-menu-item" onclick={(e) => { e.stopPropagation(); doReply(m); }}>
                    {$t("chat.msgReply")}
                  </button>
                  <button role="menuitem" class="msg-menu-item" onclick={(e) => { e.stopPropagation(); emojiForId = m.id; }}>
                    {$t("chat.msgReact")}
                  </button>
                  <button role="menuitem" class="msg-menu-item" onclick={(e) => { e.stopPropagation(); doCopy(m); }}>
                    {$t("chat.msgCopy")}
                  </button>
                  <button role="menuitem" class="msg-menu-item danger" onclick={(e) => { e.stopPropagation(); doDelete(m); }}>
                    {$t("chat.msgDelete")}
                  </button>
                {/if}
              </div>
            {/if}
            {#if m.replyTo}
              <div class="reply-quote">{m.replyTo}</div>
            {/if}
            {#if m.attachments && m.attachments.length > 0}
              <div class="msg-attach">
                {#each m.attachments as att (att.name)}
                  <span class="attach-chip sm">
                    <span class="attach-kind">{att.kind}</span>
                    <span class="attach-name">{att.name}</span>
                  </span>
                {/each}
              </div>
            {/if}
            <p>{m.text}</p>
            <span class="time">{m.time}</span>
            {#if m.reaction}
              <span class="msg-reaction" aria-label={$t("chat.msgReaction")}>{m.reaction}</span>
            {/if}
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
    {#if attachments.length > 0}
      <div class="attach-row" aria-label={$t("chat.attachments")}>
        {#each attachments as att, i (i)}
          <span class="attach-chip">
            <span class="attach-kind">{att.kind}</span>
            <span class="attach-name">{att.name}</span>
            <span class="attach-size">{att.size}</span>
            <button
              class="attach-x"
              onclick={() => removeAttachment(i)}
              aria-label={$t("chat.attachRemove")}
            >×</button>
          </span>
        {/each}
      </div>
    {/if}
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
    <input
      type="file"
      bind:this={fileInput}
      onchange={onFilesPicked}
      accept="image/*,audio/*,video/*,.pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt,.md,.csv"
      multiple
      hidden
      aria-hidden="true"
    />
    <button
      class="btn-clip"
      onclick={() => fileInput?.click()}
      disabled={status === "working"}
      aria-label={$t("chat.attach")}
      title={$t("chat.attach")}
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48" />
      </svg>
    </button>
    {#if recording}
      <div class="rec-indicator" role="status" aria-live="polite" aria-label={$t("chat.recording")}>
        <span class="rec-dot"></span>
        <span class="rec-time">{formatTime(recSeconds)}</span>
      </div>
      <button class="btn-ghost btn-stop" onclick={stopRecording} aria-label={$t("chat.stop")}>
        {$t("chat.stop")}
      </button>
    {:else if transcribing}
      <div class="rec-indicator" role="status" aria-live="polite">
        <span class="rec-dot"></span>
        <span class="rec-time">{$t("chat.transcribing")}</span>
      </div>
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
    <button class="btn-primary" onclick={send} disabled={!draft.trim() && attachments.length === 0}>
      {$t("chat.send")}
    </button>
  </div>
</section>

<style>
  .chat {
    display: flex; flex-direction: column;
    flex: 1;
    min-height: 0; min-width: 0;
    overflow: hidden;
    background: var(--bg);
  }
  .chat-head {
    display: flex; align-items: center; gap: 10px;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .badge {
    width: 36px; height: 36px; border-radius: 50%;
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
    /* Scrollbar selalu terlihat (bukan overlay macOS) */
    scrollbar-width: thin;
    scrollbar-color: var(--border) transparent;
  }
  .messages::-webkit-scrollbar { width: 8px; }
  .messages::-webkit-scrollbar-track { background: transparent; }
  .messages::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }
  .messages::-webkit-scrollbar-thumb:hover {
    background: var(--dim);
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
    display: flex; gap: 8px; align-items: flex-end;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    background: var(--panel);
    /* Menempel ke bawah window, bukan floating */
    margin-top: auto;
    flex: none;
  }
  .composer .field { flex: 1; }
  .composer textarea { resize: none; }
  .btn-mic {
    min-width: 38px; min-height: 38px;
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
  .btn-clip {
    min-width: 44px; min-height: 44px;
    display: flex; align-items: center; justify-content: center;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: transparent;
    color: var(--dim);
    cursor: pointer;
    flex: none;
  }
  .btn-clip:hover { background: var(--panel-2); color: var(--text); }
  .btn-clip:disabled { opacity: 0.45; cursor: default; }
  .attach-row {
    display: flex; flex-wrap: wrap; gap: 8px;
    padding: 8px 0;
  }
  .attach-chip {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 20px;
    font-size: 13px;
    background: var(--panel-2);
  }
  .attach-chip.sm { font-size: 12px; padding: 4px 8px; }
  .attach-kind {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    color: var(--accent);
  }
  .attach-name { max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .attach-size { color: var(--dim); font-size: 12px; }
  .attach-x {
    border: none; background: none; cursor: pointer;
    font-size: 16px; line-height: 1; color: var(--dim);
    padding: 0 2px;
  }
  .attach-x:hover { color: var(--text); }
  .msg-attach { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 6px; }
  .bubble { position: relative; }
  .msg-menu-btn {
    position: absolute; top: 4px; right: 4px;
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    border: none; border-radius: 50%;
    background: transparent; color: inherit;
    opacity: 0.55; cursor: pointer;
  }
  .msg-menu-btn:hover { opacity: 1; background: rgba(127, 127, 127, 0.2); }
  .msg-menu {
    position: absolute; top: 30px; right: 4px; z-index: 20;
    min-width: 150px;
    background: var(--panel-solid, var(--panel));
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    padding: 6px;
    display: flex; flex-direction: column;
  }
  .msg-menu-item {
    text-align: left;
    padding: 8px 10px;
    border: none; border-radius: 6px;
    background: transparent; color: var(--text);
    font-size: 14px; cursor: pointer;
  }
  .msg-menu-item:hover { background: var(--panel-2); }
  .msg-menu-item.danger { color: var(--danger); }
  .emoji-row { display: flex; gap: 2px; padding: 2px; }
  .emoji-btn {
    font-size: 18px; padding: 4px 6px;
    border: none; border-radius: 6px;
    background: transparent; cursor: pointer;
  }
  .emoji-btn:hover { background: var(--panel-2); }
  .reply-quote {
    font-size: 13px; color: var(--dim);
    border-left: 3px solid var(--border);
    padding: 2px 8px; margin-bottom: 6px;
  }
  .msg-reaction {
    position: absolute; bottom: -10px; right: 8px;
    font-size: 13px;
    background: var(--panel-solid, var(--panel));
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1px 6px;
  }
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

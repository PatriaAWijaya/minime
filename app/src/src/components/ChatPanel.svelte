<script lang="ts">
  import { onDestroy } from "svelte";
  import { getAgent } from "../lib/agents";
  import {
    activeAgentId,
    agentStatus,
    messagesByAgent,
    newTaskSignal,
    pushMessage,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  let draft = $state("");
  let inputEl: HTMLTextAreaElement | null = $state(null);
  let timer: ReturnType<typeof setTimeout> | null = null;

  const agent = $derived(getAgent($activeAgentId));
  const messages = $derived($messagesByAgent[$activeAgentId] ?? []);
  const status = $derived($agentStatus[$activeAgentId] ?? "idle");

  // "+ Tugas Baru" focuses the input so the user can type immediately.
  const unsub = newTaskSignal.subscribe(() => {
    queueMicrotask(() => inputEl?.focus());
  });
  onDestroy(() => {
    unsub();
    if (timer) clearTimeout(timer);
  });

  function send() {
    const text = draft.trim();
    if (!text || status === "working") return;
    const id = agent.id;
    const translate = $t;
    pushMessage(id, "user", text);
    draft = "";
    agentStatus.update((s) => ({ ...s, [id]: "working" }));
    timer = setTimeout(() => {
      pushMessage(id, "agent", translate("chat.mockReply", { name: agent.name }));
      agentStatus.update((s) => ({ ...s, [id]: "idle" }));
    }, 1400);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }
</script>

<section class="chat">
  <header class="chat-head">
    <span class="badge" style:background={agent.color}>{agent.badge}</span>
    <div class="who">
      <div class="name">{agent.name}</div>
      <div class="role">{$t(agent.roleKey)} · {agent.model}</div>
    </div>
    <span class="status"><span class="dot {status}"></span>{$t(`status.${status}`)}</span>
  </header>

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
    color: #fff; font-size: 20px; font-weight: 800; flex: none;
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
</style>

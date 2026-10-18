<script lang="ts">
  import { agents } from "../lib/agents";
  import {
    MODEL_CATALOG,
    activeAgentId,
    activeServer,
    addCustomAgent,
    addTaskFolder,
    agentFacts,
    agentInstructions,
    agentModel,
    agentNames,
    agentNotify,
    agentReasoning,
    agentStatus,
    customAgents,
    defaultModelFor,
    getModel,
    serverLayout,
    taskFolders,
    toggleFolder,
  } from "../lib/store";
  import type { Agent, TaskFolder } from "../lib/types";
  import { t } from "../lib/i18n";

  const reasoningOptions = ["low", "mid", "high"] as const;

  const allAgents = $derived([...agents, ...$customAgents]);
  const agent = $derived(
    allAgents.find((a) => a.id === $activeAgentId) ?? allAgents[0],
  );
  const facts = $derived(agentFacts[agent.id] ?? agentFacts.mimi);
  const modelId = $derived($agentModel[agent.id] ?? defaultModelFor(agent.id));
  const model = $derived(getModel(modelId));
  const reasoning = $derived($agentReasoning[agent.id] ?? "mid");
  const notify = $derived($agentNotify[agent.id] ?? true);
  const displayName = $derived(
    $agentNames[agent.id] ?? (agent.nameKey ? $t(agent.nameKey) : agent.name),
  );

  // Dropdown / menu state
  let panelOpen = $state(false);
  let serverOpen = $state(false);
  let addOpen = $state(false);
  let layoutOpen = $state(false);
  let detailOpen = $state(true);
  let query = $state("");

  function closeAll() {
    panelOpen = false;
    serverOpen = false;
    addOpen = false;
    layoutOpen = false;
    query = "";
  }

  function pickServer(kind: "local" | "cloud") {
    activeServer.set(kind);
    closeAll();
  }

  // Task folders: agents grouped into collapsible folders
  const groupedIds = $derived(
    new Set($taskFolders.flatMap((f) => f.agentIds)),
  );
  const looseAgents = $derived(allAgents.filter((a) => !groupedIds.has(a.id)));
  function folderAgents(f: TaskFolder): Agent[] {
    return allAgents.filter((a) => f.agentIds.includes(a.id));
  }
  function folderName(f: TaskFolder): string {
    return f.num ? $t(f.nameKey, { n: String(f.num) }) : $t(f.nameKey);
  }

  const ollamaModels = $derived(
    MODEL_CATALOG.filter((m) => m.providerId === "ollama"),
  );
  const opencodeModels = $derived(
    MODEL_CATALOG.filter((m) => m.providerId === "opencode"),
  );
  const paidModels = $derived(MODEL_CATALOG.filter((m) => m.kind === "paid"));

  const q = $derived(query.trim().toLowerCase());
  const matches = (label: string) => !q || label.toLowerCase().includes(q);
  const anyMatch = $derived(MODEL_CATALOG.some((m) => matches(m.label)));

  function setModel(id: string) {
    agentModel.update((m) => ({ ...m, [agent.id]: id }));
  }
  function setReasoning(value: "low" | "mid" | "high") {
    agentReasoning.update((m) => ({ ...m, [agent.id]: value }));
  }
  function setNotify(value: boolean) {
    agentNotify.update((m) => ({ ...m, [agent.id]: value }));
  }
</script>

{#snippet agentRow(a: Agent)}
  {@const status = $agentStatus[a.id] ?? "idle"}
  {@const baseName = a.nameKey ? $t(a.nameKey) : a.name}
  {@const rowName = $agentNames[a.id] ?? baseName}
  <button
    class="agent-row"
    class:active={$activeAgentId === a.id}
    onclick={() => activeAgentId.set(a.id)}
  >
    <span class="badge" style:background={a.color}>{a.badge}</span>
    <span class="meta">
      <span class="name">{rowName}</span>
      <span class="role">{$t(a.roleKey)}</span>
    </span>
    <span class="dot {status}" aria-hidden="true"></span>
  </button>
{/snippet}

<div class="agents-view">
  <!-- Top bar: server dropdown | add menu | active agent | model shortcut | panel controls -->
  <div class="topbar">
    <div class="tb-left">
      <!-- Server selector -->
      <div class="menu-wrap">
        <button
          class="pill"
          onclick={() => {
            const next = !serverOpen;
            closeAll();
            serverOpen = next;
          }}
          aria-haspopup="menu"
          aria-expanded={serverOpen}
          aria-label={$t("server.title")}
        >
          <span class="server-dot {$activeServer}" aria-hidden="true"></span>
          <span
            >{$t(
              $activeServer === "local" ? "server.local" : "server.cloud",
            )}</span
          >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M6 9l6 6 6-6" />
          </svg>
        </button>
        {#if serverOpen}
          <div class="scrim" onclick={closeAll} aria-hidden="true"></div>
          <div class="menu" role="menu" aria-label={$t("server.title")}>
            <span class="menu-label">{$t("server.title")}</span>
            <button class="menu-item" role="menuitemradio" aria-checked={$activeServer === "local"} onclick={() => pickServer("local")}>
              <span class="check" aria-hidden="true">{#if $activeServer === "local"}✓{/if}</span>
              <span class="server-dot local" aria-hidden="true"></span>
              <span>{$t("server.local")}</span>
            </button>
            <button class="menu-item" role="menuitemradio" aria-checked={$activeServer === "cloud"} onclick={() => pickServer("cloud")}>
              <span class="check" aria-hidden="true">{#if $activeServer === "cloud"}✓{/if}</span>
              <span class="server-dot cloud" aria-hidden="true"></span>
              <span>{$t("server.cloud")}</span>
            </button>
            <div class="menu-sep" aria-hidden="true"></div>
            <button class="menu-item" onclick={closeAll}>
              <span class="check" aria-hidden="true"></span>
              <span>{$t("server.add")}</span>
            </button>
            <button class="menu-item" onclick={closeAll}>
              <span class="check" aria-hidden="true"></span>
              <span>{$t("server.settings")}</span>
            </button>
            <div class="menu-sep" aria-hidden="true"></div>
            <div class="menu-wrap-sub">
              <button
                class="menu-item"
                onclick={() => (layoutOpen = !layoutOpen)}
                aria-haspopup="menu"
                aria-expanded={layoutOpen}
              >
                <span class="check" aria-hidden="true"></span>
                <span class="sub-label-text">{$t("server.layout")}</span>
                <span class="sub-caret" aria-hidden="true">›</span>
              </button>
              {#if layoutOpen}
                <div class="menu sub" role="menu" aria-label={$t("server.layout")}>
                  <button
                    class="menu-item"
                    role="menuitemradio"
                    aria-checked={$serverLayout === "rail"}
                    onclick={() => {
                      serverLayout.set("rail");
                      closeAll();
                    }}
                  >
                    <span class="check" aria-hidden="true">{#if $serverLayout === "rail"}✓{/if}</span>
                    <span>{$t("server.layout.rail")}</span>
                  </button>
                  <button
                    class="menu-item"
                    role="menuitemradio"
                    aria-checked={$serverLayout === "menu"}
                    onclick={() => {
                      serverLayout.set("menu");
                      closeAll();
                    }}
                  >
                    <span class="check" aria-hidden="true">{#if $serverLayout === "menu"}✓{/if}</span>
                    <span>{$t("server.layout.menu")}</span>
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Add menu: new agent / new task folder -->
      <div class="menu-wrap">
        <button
          class="icon-btn bordered"
          onclick={() => {
            const next = !addOpen;
            closeAll();
            addOpen = next;
          }}
          aria-haspopup="menu"
          aria-expanded={addOpen}
          aria-label={$t("agents.add")}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
        {#if addOpen}
          <div class="scrim" onclick={closeAll} aria-hidden="true"></div>
          <div class="menu" role="menu" aria-label={$t("agents.add")}>
            <button
              class="menu-item"
              onclick={() => {
                addCustomAgent();
                closeAll();
              }}
            >
              <span class="check" aria-hidden="true"></span>
              <span>{$t("agents.addAgent")}</span>
            </button>
            <button
              class="menu-item"
              onclick={() => {
                addTaskFolder();
                closeAll();
              }}
            >
              <span class="check" aria-hidden="true"></span>
              <span>{$t("agents.addFolder")}</span>
            </button>
          </div>
        {/if}
      </div>

      <!-- Active agent identity -->
      <div class="tb-agent">
        <span class="badge sm" style:background={agent.color}>{agent.badge}</span>
        <span class="tb-agent-meta">
          <span class="tb-agent-name">{displayName}</span>
          <span class="tb-agent-role">{$t(agent.roleKey)}</span>
        </span>
      </div>
    </div>

    <div class="tb-center">
      <!-- Model shortcut pill (like OpenBot's provider header) -->
      <button
        class="pill"
        onclick={() => {
          const next = !panelOpen;
          closeAll();
          panelOpen = next;
        }}
        aria-haspopup="dialog"
        aria-expanded={panelOpen}
        aria-label={$t("agents.model")}
      >
        <span class="pill-dot" style:background={agent.color} aria-hidden="true"></span>
        <span>{model.label} · {$t(`agents.reasoning.${reasoning}`)}</span>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>
      {#if panelOpen}
        <div class="scrim" onclick={closeAll} aria-hidden="true"></div>
        <div class="model-panel" role="dialog" aria-label={$t("agents.model")}>
          <div class="field">
            <input
              type="search"
              bind:value={query}
              placeholder={$t("model.search")}
              aria-label={$t("model.search")}
            />
          </div>

          <span class="group-label">{$t("providers.free")}</span>
          <span class="sub-label">Ollama</span>
          {#each ollamaModels as m}
            {#if matches(m.label)}
              <label class="model-item">
                <input
                  type="radio"
                  name="model-pick"
                  checked={modelId === m.id}
                  onchange={() => setModel(m.id)}
                />
                <span class="m-label">{m.label}</span>
                <span class="chip">{$t("providers.free")}</span>
              </label>
            {/if}
          {/each}
          <span class="sub-label">OpenCode</span>
          {#each opencodeModels as m}
            {#if matches(m.label)}
              <label class="model-item">
                <input
                  type="radio"
                  name="model-pick"
                  checked={modelId === m.id}
                  onchange={() => setModel(m.id)}
                />
                <span class="m-label">{m.label}</span>
                <span class="chip">{$t("providers.free")}</span>
              </label>
            {/if}
          {/each}

          <span class="group-label">{$t("providers.paid")}</span>
          {#each paidModels as m}
            {#if matches(m.label)}
              <label class="model-item">
                <input
                  type="radio"
                  name="model-pick"
                  checked={modelId === m.id}
                  onchange={() => setModel(m.id)}
                />
                <span class="m-label">{m.label}</span>
                <span class="chip paid">{$t("providers.paid")}</span>
              </label>
            {/if}
          {/each}

          {#if !anyMatch}
            <p class="empty-note">{$t("model.empty")}</p>
          {/if}

          <div class="panel-effort">
            <span class="group-label">{$t("agents.reasoning")}</span>
            <span class="seg" role="group" aria-label={$t("agents.reasoning")}>
              {#each reasoningOptions as r}
                <button
                  class:active={reasoning === r}
                  onclick={() => setReasoning(r)}
                >{$t(`agents.reasoning.${r}`)}</button>
              {/each}
            </span>
          </div>
        </div>
      {/if}
    </div>

    <div class="tb-right">
      <button
        class="settings-btn"
        class:on={detailOpen}
        onclick={() => (detailOpen = !detailOpen)}
        aria-pressed={detailOpen}
        aria-label={$t("agents.agentSettings")}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
        <span>{$t("agents.agentSettings")}</span>
      </button>
      <button
        class="icon-btn bordered"
        aria-label={$t("agents.panel.add")}
        onclick={closeAll}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="3" y="4" width="18" height="16" rx="2" />
          <path d="M12 4v16M15 9l3 3-3 3" />
        </svg>
      </button>
      <button
        class="icon-btn bordered"
        aria-label={$t("agents.panel.expand")}
        onclick={closeAll}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
        </svg>
      </button>
    </div>
  </div>

  <div class="split" class:single={!detailOpen}>
    <div class="list-col">
      <h2>{$t("agents.title")}</h2>
      {#each looseAgents as a (a.id)}
        {@render agentRow(a)}
      {/each}
      {#each $taskFolders as f (f.id)}
        <div class="folder">
          <button
            class="folder-head"
            onclick={() => toggleFolder(f.id)}
            aria-expanded={f.open}
          >
            <svg
              class="chev"
              class:open={f.open}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <path d="M9 6l6 6-6 6" />
            </svg>
            <span class="folder-name">{folderName(f)}</span>
            <span class="folder-count"
              >{$t("agents.folder.count", { n: String(folderAgents(f).length) })}</span
            >
          </button>
          {#if f.open}
            <div class="folder-body">
              {#each folderAgents(f) as a (a.id)}
                {@render agentRow(a)}
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if detailOpen}
      <div class="detail">
        <div class="detail-head">
          <span class="badge big" style:background={agent.color}>{agent.badge}</span>
          <div class="who">
            <input
              class="name-input"
              value={displayName}
              oninput={(e) => agentNames.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
              aria-label={$t("agents.name")}
            />
            <div class="role">{$t(agent.roleKey)}</div>
          </div>
        </div>

        <div class="field">
          <label for="instructions">{$t("agents.instructions")}</label>
          <textarea
            id="instructions"
            rows="2"
            placeholder={$t("agents.instructions.text")}
            value={$agentInstructions[agent.id] ?? ""}
            oninput={(e) =>
              agentInstructions.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
          ></textarea>
        </div>

        <div class="section">
          <h3>{$t("agents.brain")}</h3>
          <div class="field">
            <label for="model">{$t("agents.model")}</label>
            <select id="model" value={modelId} onchange={(e) => setModel(e.currentTarget.value)}>
              <optgroup label="Ollama · {$t('providers.free')}">
                {#each ollamaModels as m}
                  <option value={m.id} selected={modelId === m.id}>{m.label}</option>
                {/each}
              </optgroup>
              <optgroup label="OpenCode · {$t('providers.free')}">
                {#each opencodeModels as m}
                  <option value={m.id} selected={modelId === m.id}>{m.label}</option>
                {/each}
              </optgroup>
              <optgroup label={$t("providers.paid")}>
                {#each paidModels as m}
                  <option value={m.id} selected={modelId === m.id}>{m.label}</option>
                {/each}
              </optgroup>
            </select>
          </div>
          <div class="row">
            <span class="label">{$t("agents.reasoning")}</span>
            <span class="seg" role="group" aria-label={$t("agents.reasoning")}>
              {#each reasoningOptions as r}
                <button
                  class:active={reasoning === r}
                  onclick={() => setReasoning(r)}
                >{$t(`agents.reasoning.${r}`)}</button>
              {/each}
            </span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.backup")}</span>
            <span class="value">{$t("agents.backup.ask")}</span>
          </div>
        </div>

        <div class="section">
          <h3>{$t("agents.knows")}</h3>
          <div class="row">
            <span class="label">{$t("agents.memories")}</span>
            <span class="value">{$t("agents.memories.n", { n: String(facts.memories) })}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.skills")}</span>
            <span class="value">{$t("agents.skills.n", { n: String(facts.skills) })}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.files")}</span>
            <span class="value">{facts.files}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.tables")}</span>
            <span class="value">{$t("agents.tables.n", { n: String(facts.tables) })}</span>
          </div>
        </div>

        <div class="section">
          <h3>{$t("agents.does")}</h3>
          <div class="row">
            <span class="label">{$t("agents.routines")}</span>
            <span class="value">{$t("agents.routines.n", { n: String(facts.routines) })}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.usage")}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.publish")}</span>
          </div>
        </div>

        <div class="section">
          <h3>{$t("agents.rules")}</h3>
          <div class="row">
            <span class="label">{$t("agents.permissions")}</span>
            <span class="value">{$t("agents.permissions.full")}</span>
          </div>
          <div class="row">
            <span class="label">{$t("agents.notifications")}</span>
            <label class="switch">
              <input
                type="checkbox"
                checked={notify}
                onchange={(e) => setNotify(e.currentTarget.checked)}
                aria-label={$t("agents.notifications")}
              />
              <span class="track" aria-hidden="true"></span>
            </label>
          </div>
          <div class="row">
            <span class="label">{$t("agents.advanced")}</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .agents-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .topbar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
    flex: none;
    min-width: 0;
  }
  .tb-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: none;
  }
  .tb-center {
    margin-inline: auto;
    position: relative;
    min-width: 0;
  }
  .tb-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: none;
  }
  .agents-view .split {
    flex: 1;
    height: auto;
    min-height: 0;
  }
  .agents-view .split.single {
    grid-template-columns: minmax(0, 1fr);
  }

  /* Active agent identity in the top bar */
  .tb-agent {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin-left: 4px;
  }
  .tb-agent-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .tb-agent-name {
    font-size: 14px;
    font-weight: 800;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }
  .tb-agent-role {
    font-size: 12px;
    color: var(--dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }
  .badge.sm {
    width: 32px;
    height: 32px;
    font-size: 15px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 0 16px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--panel-2);
    color: var(--text);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .pill:hover {
    border-color: var(--accent);
  }
  .pill svg {
    width: 16px;
    height: 16px;
    color: var(--dim);
    flex: none;
  }
  .pill-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex: none;
  }
  .server-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex: none;
  }
  .server-dot.local { background: var(--ok); }
  .server-dot.cloud { background: #60a5fa; }

  .icon-btn.bordered {
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text);
  }
  .icon-btn.bordered:hover {
    border-color: var(--accent);
    background: var(--panel-2);
    color: var(--text);
  }

  .settings-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 0 16px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--panel-2);
    color: var(--text);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .settings-btn:hover {
    border-color: var(--accent);
  }
  .settings-btn.on {
    border-color: var(--accent);
    color: var(--accent);
  }
  .settings-btn svg {
    width: 18px;
    height: 18px;
    flex: none;
  }

  /* Dropdown menus */
  .menu-wrap {
    position: relative;
  }
  .menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 250px;
    max-width: calc(100vw - 60px);
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    padding: 6px;
    z-index: 60;
  }
  .menu.sub {
    top: 0;
    left: calc(100% + 6px);
  }
  .menu-wrap-sub {
    position: relative;
  }
  .menu-label {
    display: block;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--dim);
    padding: 8px 12px 4px;
  }
  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 44px;
    padding: 8px 12px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
  }
  .menu-item:hover {
    background: var(--panel-2);
  }
  .menu-item .check {
    width: 20px;
    flex: none;
    color: var(--accent);
    font-weight: 800;
  }
  .menu-item .sub-label-text {
    flex: 1;
  }
  .menu-item .sub-caret {
    color: var(--dim);
    font-size: 18px;
    line-height: 1;
  }
  .menu-sep {
    height: 1px;
    background: var(--border);
    margin: 6px 4px;
  }

  .scrim {
    position: fixed;
    inset: 0;
    z-index: 55;
  }
  .model-panel {
    position: absolute;
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    width: 340px;
    max-width: calc(100vw - 120px);
    max-height: min(70vh, 560px);
    overflow-y: auto;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    padding: 12px;
    z-index: 60;
  }
  .model-panel .field {
    margin-bottom: 4px;
  }
  .group-label {
    display: block;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--dim);
    margin: 12px 4px 4px;
  }
  .sub-label {
    display: block;
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
    margin: 8px 4px 2px;
  }
  .model-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 44px;
    padding: 8px 10px;
    border-radius: 8px;
    cursor: pointer;
  }
  .model-item:hover {
    background: var(--panel-2);
  }
  .model-item input[type="radio"] {
    width: 18px;
    height: 18px;
    margin: 0;
    flex: none;
    accent-color: var(--accent);
  }
  .m-label {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
  }
  .model-item .chip { align-self: center; }
  .chip.paid {
    border-color: var(--yellow);
    color: var(--yellow);
  }
  .panel-effort {
    border-top: 1px solid var(--border);
    margin-top: 12px;
    padding-top: 4px;
  }
  .panel-effort .seg {
    margin: 4px;
  }
  .empty-note {
    color: var(--dim);
    font-size: 14px;
    padding: 12px 4px;
  }

  /* Task folders in the agent list */
  .folder {
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
    flex: none;
  }
  .folder-head {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 44px;
    padding: 0 12px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    text-align: left;
  }
  .folder-head:hover {
    background: var(--panel-2);
  }
  .folder-head .chev {
    width: 16px;
    height: 16px;
    color: var(--dim);
    flex: none;
    transition: transform 0.15s ease;
  }
  .folder-head .chev.open {
    transform: rotate(90deg);
  }
  .folder-name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .folder-count {
    font-size: 12px;
    font-weight: 600;
    color: var(--dim);
    flex: none;
  }
  .folder-body {
    padding: 2px 6px 8px;
  }

  @media (max-width: 860px) {
    .tb-agent-role,
    .settings-btn span {
      display: none;
    }
    .tb-agent-name {
      max-width: 110px;
    }
  }
</style>

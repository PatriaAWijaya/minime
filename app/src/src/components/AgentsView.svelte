<script lang="ts">
  import { agents, getAgent } from "../lib/agents";
  import {
    MODEL_CATALOG,
    activeAgentId,
    agentFacts,
    agentInstructions,
    agentModel,
    agentNames,
    agentNotify,
    agentReasoning,
    agentStatus,
    defaultModelFor,
    getModel,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  const reasoningOptions = ["low", "mid", "high"] as const;

  const agent = $derived(getAgent($activeAgentId));
  const facts = $derived(agentFacts[agent.id] ?? agentFacts.mimi);
  const modelId = $derived($agentModel[agent.id] ?? defaultModelFor(agent.id));
  const model = $derived(getModel(modelId));
  const reasoning = $derived($agentReasoning[agent.id] ?? "mid");
  const notify = $derived($agentNotify[agent.id] ?? true);
  const displayName = $derived($agentNames[agent.id] ?? agent.name);

  let panelOpen = $state(false);
  let query = $state("");

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
  function closePanel() {
    panelOpen = false;
    query = "";
  }
</script>

<div class="agents-view">
  <!-- Model shortcut bar (like OpenBot's provider header) -->
  <div class="topbar">
    <button
      class="pill"
      onclick={() => (panelOpen = !panelOpen)}
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
      <div class="scrim" onclick={closePanel} aria-hidden="true"></div>
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

  <div class="split">
  <div class="list-col">
    <h2>{$t("agents.title")}</h2>
    {#each agents as a}
      {@const status = $agentStatus[a.id] ?? "idle"}
      <button
        class="agent-row"
        class:active={$activeAgentId === a.id}
        onclick={() => activeAgentId.set(a.id)}
      >
        <span class="badge" style:background={a.color}>{a.badge}</span>
        <span class="meta">
          <span class="name">{a.name}</span>
          <span class="role">{$t(a.roleKey)}</span>
        </span>
        <span class="dot {status}" aria-hidden="true"></span>
      </button>
    {/each}
  </div>

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
    justify-content: flex-end;
    align-items: center;
    padding: 8px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
    flex: none;
  }
  .agents-view .split {
    flex: 1;
    height: auto;
    min-height: 0;
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
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 55;
  }
  .model-panel {
    position: absolute;
    top: calc(100% + 8px);
    right: 16px;
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
</style>

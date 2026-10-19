<script lang="ts">
  import { agents } from "../lib/agents";
  import ChatPanel from "./ChatPanel.svelte";
  import {
    PROVIDERS,
    activeAgentId,
    activeServer,
    addCustomAgent,
    addCustomModel,
    addAgentFile,
    addKanbanCard,
    addTaskFolder,
    agentBrowserOn,
    agentFacts,
    agentFiles,
    agentGender,
    agentAge,
    agentHobbies,
    agentInstructions,
    agentModel,
    agentNames,
    agentNotify,
    agentPosition,
    agentReasoning,
    agentStatus,
    agentTitles,
    agentVoice,
    agentVoiceOn,
    customAgents,
    defaultModelFor,
    getModel,
    modelCatalog,
    projects,
    removeAgentFile,
    removeCustomModel,
    serverLayout,
    taskFolders,
    toggleFolder,
  } from "../lib/store";
  import type { ModelSection } from "../lib/store";
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
  const providerName = $derived(
    PROVIDERS.find((p) => p.id === model.providerId)?.name ?? model.providerId,
  );
  const reasoning = $derived($agentReasoning[agent.id] ?? "mid");
  const notify = $derived($agentNotify[agent.id] ?? true);
  const gender = $derived($agentGender[agent.id]);
  const ageVal = $derived(
    $agentAge[agent.id] !== undefined ? String($agentAge[agent.id]) : "",
  );
  const position = $derived($agentPosition[agent.id]);
  const hobbies = $derived($agentHobbies[agent.id] ?? "");
  const voiceOn = $derived($agentVoiceOn[agent.id] ?? false);
  const voiceChoice = $derived($agentVoice[agent.id] ?? "v1");
  const displayName = $derived(
    $agentNames[agent.id] ?? (agent.nameKey ? $t(agent.nameKey) : agent.name),
  );
  const agentTitle = $derived($agentTitles[agent.id] ?? "");
  const chatAgent = $derived({ ...agent, name: displayName });

  // Dropdown / menu state
  let panelOpen = $state(false);
  let serverOpen = $state(false);
  let addOpen = $state(false);
  let layoutOpen = $state(false);
  let detailOpen = $state(true);
  let query = $state("");
  let agentSearch = $state("");
  // Agent extras menu (+) + dialogs
  let agentMenuOpen = $state(false);
  let agentDlg = $state<"browser" | "files" | "task" | null>(null);
  let taskTitle = $state("");
  let taskProject = $state("");
  let taskAdded = $state(false);

  function closeAll() {
    panelOpen = false;
    serverOpen = false;
    addOpen = false;
    layoutOpen = false;
    agentMenuOpen = false;
    agentDlg = null;
    query = "";
  }

  function openModelPanel() {
    closeAll();
    syncModelSection();
    panelOpen = true;
  }

  function pickServer(kind: "local" | "cloud") {
    activeServer.set(kind);
    closeAll();
  }

  // Agent extras: browser toggle, files, task creation (per active agent)
  const browserOn = $derived($agentBrowserOn[agent.id] ?? false);
  const files = $derived($agentFiles[agent.id] ?? []);

  function setBrowser(v: boolean) {
    agentBrowserOn.update((m) => ({ ...m, [agent.id]: v }));
  }

  function openAgentDlg(kind: "browser" | "files" | "task") {
    agentMenuOpen = false;
    taskAdded = false;
    agentDlg = kind;
  }

  function addFile() {
    addAgentFile(agent.id, $t("agents.files.sample"), "120 KB");
  }

  function dropFile(i: number) {
    removeAgentFile(agent.id, i);
  }

  function submitTask() {
    const title = taskTitle.trim();
    if (!title) return;
    const pid = taskProject || $projects[0]?.id || "";
    addKanbanCard(title, agent.id, "queue", pid);
    taskTitle = "";
    taskAdded = true;
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

  // Agent-list search
  const aq = $derived(agentSearch.trim().toLowerCase());
  const labelOf = (a: Agent) =>
    (
      ($agentNames[a.id] ?? (a.nameKey ? $t(a.nameKey) : a.name)) +
      " " +
      $t(a.roleKey)
    ).toLowerCase();
  const filteredLoose = $derived(
    looseAgents.filter((a) => !aq || labelOf(a).includes(aq)),
  );
  const folderView = $derived(
    $taskFolders
      .map((f) => ({
        f,
        agents: folderAgents(f).filter((a) => !aq || labelOf(a).includes(aq)),
      }))
      .filter(
        ({ f, agents }) =>
          !aq ||
          folderName(f).toLowerCase().includes(aq) ||
          agents.length > 0,
      ),
  );

  const ollamaModels = $derived(
    $modelCatalog.filter((m) => m.providerId === "ollama"),
  );
  const opencodeModels = $derived(
    $modelCatalog.filter((m) => m.providerId === "opencode"),
  );
  const paidModels = $derived($modelCatalog.filter((m) => m.kind === "paid"));

  const q = $derived(query.trim().toLowerCase());
  const matches = (label: string) => !q || label.toLowerCase().includes(q);
  const anyMatch = $derived($modelCatalog.some((m) => matches(m.label)));

  // Model dropdown sections (keputusan Patria 19 Okt)
  const MODEL_SECTIONS: Array<{ id: ModelSection; labelKey: string }> = [
    { id: "opencode", labelKey: "model.freeOpenCode" },
    { id: "ollama", labelKey: "model.freeOllama" },
    { id: "paid", labelKey: "model.premium" },
  ];
  const sectionOf = (id: string): ModelSection => {
    const m = getModel(id);
    return m.kind === "paid" ? "paid" : (m.providerId as ModelSection);
  };
  const sectionModels = (s: ModelSection) =>
    s === "paid" ? paidModels : s === "ollama" ? ollamaModels : opencodeModels;

  let modelSectionOpen = $state<ModelSection | null>("opencode");
  let modelAddFor = $state<ModelSection | null>(null);
  let newModelName = $state("");

  function syncModelSection() {
    modelSectionOpen = sectionOf(modelId);
    modelAddFor = null;
    newModelName = "";
  }

  function submitCustomModel(section: ModelSection) {
    addCustomModel(section, newModelName);
    newModelName = "";
    modelAddFor = null;
    modelSectionOpen = section;
  }

  function dropCustomModel(section: ModelSection, name: string) {
    removeCustomModel(section, name);
  }

  function setModel(id: string) {
    agentModel.update((m) => ({ ...m, [agent.id]: id }));
  }
  function setReasoning(value: "low" | "mid" | "high") {
    agentReasoning.update((m) => ({ ...m, [agent.id]: value }));
  }
  function setNotify(value: boolean) {
    agentNotify.update((m) => ({ ...m, [agent.id]: value }));
  }

  // Personalization setters (persisted per agent)
  let ageError = $state(false);
  $effect(() => {
    // reset validation state when switching agents
    void agent.id;
    ageError = false;
  });
  function setGender(value: "male" | "female" | undefined) {
    agentGender.update((m) => {
      const c = { ...m };
      if (value === undefined) delete c[agent.id];
      else c[agent.id] = value;
      return c;
    });
  }
  function setAge(raw: string) {
    const t = raw.trim();
    if (t === "") {
      agentAge.update((m) => {
        const c = { ...m };
        delete c[agent.id];
        return c;
      });
      ageError = false;
      return;
    }
    const n = parseInt(t, 10);
    if (Number.isNaN(n) || n < 1 || n > 120) {
      ageError = true;
      return;
    }
    ageError = false;
    agentAge.update((m) => ({ ...m, [agent.id]: n }));
  }
  function setPosition(value: string) {
    agentPosition.update((m) => {
      const c = { ...m };
      if (!value) delete c[agent.id];
      else c[agent.id] = value as "clevel" | "senior" | "manager";
      return c;
    });
  }
  function setVoiceOn(value: boolean) {
    agentVoiceOn.update((m) => ({ ...m, [agent.id]: value }));
  }
  function setVoice(value: string) {
    agentVoice.update((m) => ({ ...m, [agent.id]: value }));
  }

  // Single-d SVG icon paths (multi-subpath where needed)
  const ICONS = {
    cpu: "M7 7h10v10H7z M4 9h3 M4 15h3 M17 9h3 M17 15h3 M9 4v3 M15 4v3 M9 17v3 M15 17v3",
    backup: "M7 8h11 M14 4.5L17.5 8 14 11.5 M17 16H6 M10 12.5L6.5 16 10 19.5",
    memory: "M7 3h10a1 1 0 0 1 1 1v17l-6-4-6 4V4a1 1 0 0 1 1-1z",
    skills:
      "M12 3l2.7 5.6 6.1.8-4.5 4.2 1.1 6-5.4-3-5.4 3 1.1-6L3.2 9.4l6.1-.8z",
    files: "M6 2h8l5 5v13a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z M14 2v5h5",
    tables: "M4 4h16v16H4z M4 10h16 M4 15h16 M10 4v16 M15 4v16",
    routines: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z M12 7v5l3.5 2",
    usage: "M4 19a9 9 0 1 1 16 0 M12 14l4.5-4.5 M2 19h20",
    publish: "M12 15V3 M7 8l5-5 5 5 M4 21h16",
    shield: "M12 3l7 2.8v5.4c0 4.8-3.4 7.6-7 9.3-3.6-1.7-7-4.5-7-9.3V5.8z",
    sliders: "M4 7h16 M4 17h16 M9 4.5v5 M15 14.5v5",
  } as const;
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

{#snippet navRow(icon: string, label: string, valueText: string)}
  <button class="row nav-row" onclick={closeAll}>
    <span class="nav-left">
      <svg
        class="nav-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d={icon} />
      </svg>
      <span class="label">{label}</span>
    </span>
    <span class="nav-right">
      {#if valueText}<span class="value">{valueText}</span>{/if}
      <svg
        class="chev-r"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        aria-hidden="true"
      >
        <path d="M9 6l6 6-6 6" />
      </svg>
    </span>
  </button>
{/snippet}

<div class="agents-view">
  <!-- Top bar: server dropdown | active agent | model shortcut | panel controls -->
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
          if (next) syncModelSection();
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

          {#each MODEL_SECTIONS as sec}
            <button
              class="sec-toggle"
              aria-expanded={modelSectionOpen === sec.id || q !== ""}
              onclick={() =>
                (modelSectionOpen =
                  modelSectionOpen === sec.id ? null : sec.id)}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M6 9l6 6 6-6" />
              </svg>
              <span>{$t(sec.labelKey)}</span>
            </button>
            {#if modelSectionOpen === sec.id || q !== ""}
              {#each sectionModels(sec.id) as m}
                {#if matches(m.label)}
                  <label class="model-item">
                    <input
                      type="radio"
                      name="model-pick"
                      checked={modelId === m.id}
                      onchange={() => setModel(m.id)}
                    />
                    <span class="m-label">{m.label}</span>
                    <span class="chip {sec.id === 'paid' ? 'paid' : ''}">
                      {sec.id === "paid" ? $t("providers.paid") : $t("providers.free")}
                    </span>
                    {#if m.custom}
                      <button
                        class="icon-btn del-model"
                        type="button"
                        title={$t("model.removeCustom")}
                        aria-label={$t("model.removeCustom")}
                        onclick={(e) => {
                          e.preventDefault();
                          dropCustomModel(sec.id, m.label);
                        }}
                      >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                          <path d="M18 6L6 18M6 6l12 12" />
                        </svg>
                      </button>
                    {/if}
                  </label>
                {/if}
              {/each}
              {#if modelAddFor === sec.id}
                <div class="add-model">
                  <input
                    type="text"
                    bind:value={newModelName}
                    placeholder={$t("model.namePlaceholder")}
                    aria-label={$t("model.namePlaceholder")}
                    onkeydown={(e) => {
                      if (e.key === "Enter") submitCustomModel(sec.id);
                      if (e.key === "Escape") {
                        modelAddFor = null;
                        newModelName = "";
                      }
                    }}
                  />
                  <button
                    class="btn-primary"
                    type="button"
                    onclick={() => submitCustomModel(sec.id)}
                    disabled={!newModelName.trim()}
                  >
                    {$t("model.add")}
                  </button>
                  <button
                    class="btn-ghost sm"
                    type="button"
                    onclick={() => {
                      modelAddFor = null;
                      newModelName = "";
                    }}
                  >
                    {$t("providers.cancel")}
                  </button>
                </div>
              {:else}
                <button
                  class="add-model-btn"
                  type="button"
                  onclick={() => {
                    modelAddFor = sec.id;
                    newModelName = "";
                  }}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M12 5v14M5 12h14" />
                  </svg>
                  <span>{$t("model.addModel")}</span>
                </button>
              {/if}
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
      <div class="menu-wrap">
        <button
          class="icon-btn bordered"
          aria-label={$t("agents.menu.title")}
          aria-haspopup="menu"
          aria-expanded={agentMenuOpen}
          onclick={() => {
            const next = !agentMenuOpen;
            closeAll();
            agentMenuOpen = next;
          }}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
        {#if agentMenuOpen}
          <div class="scrim" onclick={closeAll} aria-hidden="true"></div>
          <div class="menu" role="menu" aria-label={$t("agents.menu.title")}>
            <button class="menu-item" role="menuitem" onclick={() => openAgentDlg("browser")}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" width="20" height="20">
                <rect x="2" y="4" width="20" height="13" rx="2" />
                <path d="M8 21h8M12 17v4" />
              </svg>
              <span>{$t("agents.menu.browser")}</span>
            </button>
            <button class="menu-item" role="menuitem" onclick={() => openAgentDlg("files")}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" width="20" height="20">
                <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              </svg>
              <span>{$t("agents.menu.files")}</span>
            </button>
            <button class="menu-item" role="menuitem" onclick={() => openAgentDlg("task")}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" width="20" height="20">
                <path d="M9 6h11M9 12h11M9 18h11" />
                <path d="M4 5l1 1 2-2M4 11l1 1 2-2M4 17l1 1 2-2" />
              </svg>
              <span>{$t("agents.menu.tasks")}</span>
            </button>
          </div>
        {/if}
      </div>
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

  {#if agentDlg}
    <div class="scrim" onclick={() => (agentDlg = null)} aria-hidden="true"></div>
    <div class="agent-dlg" role="dialog" aria-modal="true" aria-label={$t("agents.menu.title")}>
      {#if agentDlg === "browser"}
        <h3>{$t("agents.browser.title")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <div class="row">
          <span class="label">{$t("agents.browser.allow")}</span>
          <label class="switch">
            <input
              type="checkbox"
              checked={browserOn}
              onchange={(e) => setBrowser(e.currentTarget.checked)}
              aria-label={$t("agents.browser.allow")}
            />
            <span class="track" aria-hidden="true"></span>
          </label>
        </div>
        <p class="note">{$t("agents.browser.note")}</p>
        <div class="dlg-actions">
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
        </div>
      {:else if agentDlg === "files"}
        <h3>{$t("agents.files.title")}</h3>
        <p class="dlg-agent">{displayName}</p>
        {#if files.length === 0}
          <p class="note">{$t("agents.files.empty")}</p>
        {:else}
          <ul class="file-list">
            {#each files as f, i (i)}
              <li>
                <span class="fname">{f.name}</span>
                <span class="fsize">{f.size}</span>
                <button
                  class="mini-btn danger"
                  onclick={() => dropFile(i)}
                  aria-label="{$t("agents.files.remove")}: {f.name}"
                >×</button>
              </li>
            {/each}
          </ul>
        {/if}
        <button class="btn-ghost" onclick={addFile}>+ {$t("agents.files.add")}</button>
        <div class="dlg-actions">
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
        </div>
      {:else}
        <h3>{$t("agents.task.title")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <label class="field-label" for="dlg-task-title">{$t("agents.task.name")}</label>
        <div class="field">
          <input
            id="dlg-task-title"
            type="text"
            bind:value={taskTitle}
            maxlength={120}
          />
        </div>
        <label class="field-label" for="dlg-task-project">{$t("agents.task.project")}</label>
        <div class="field">
          <select id="dlg-task-project" bind:value={taskProject}>
            {#each $projects as p (p.id)}
              <option value={p.id}>{p.name}</option>
            {/each}
          </select>
        </div>
        {#if taskAdded}
          <p class="ok-note" role="status">{$t("agents.task.done")}</p>
        {/if}
        <div class="dlg-actions">
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
          <button
            class="btn-primary"
            disabled={!taskTitle.trim()}
            onclick={submitTask}
          >{$t("agents.task.add")}</button>
        </div>
      {/if}
    </div>
  {/if}

  <div class="split" class:no-desc={!detailOpen}>
    <!-- Left column: agent list -->
    <div class="list-col">
      <div class="list-head">
        <h2>{$t("agents.title")}</h2>
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
      </div>
      <input
        class="list-search"
        type="search"
        bind:value={agentSearch}
        placeholder={$t("agents.search")}
        aria-label={$t("agents.search")}
      />
      <div class="list-body">
        {#each filteredLoose as a (a.id)}
          {@render agentRow(a)}
        {/each}
        {#each folderView as { f, agents: fAgents } (f.id)}
          <div class="folder">
            <button
              class="folder-head"
              onclick={() => toggleFolder(f.id)}
              aria-expanded={f.open || !!aq}
            >
              <svg
                class="chev"
                class:open={f.open || !!aq}
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
                >{$t("agents.folder.count", { n: String(fAgents.length) })}</span
              >
            </button>
            {#if f.open || aq}
              <div class="folder-body">
                {#each fAgents as a (a.id)}
                  {@render agentRow(a)}
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Middle column: chat area (widest) -->
    <div class="chat-col">
      <ChatPanel agent={chatAgent} header={false} />
    </div>

    <!-- Right column: agent description -->
    {#if detailOpen}
      <div class="desc-col">
        <div class="avatar-wrap">
          <span
            class="badge avatar-big"
            style:background={agent.color}
          >{agent.badge}</span>
          <button
            class="avatar-edit"
            aria-label={$t("agents.editAvatar")}
            onclick={closeAll}
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M17 3a2.8 2.8 0 1 1 4 4L8 20l-5 1 1-5z" />
            </svg>
          </button>
        </div>

        <div class="field">
          <label for="d-name">{$t("agents.name")}</label>
          <input
            id="d-name"
            value={displayName}
            oninput={(e) =>
              agentNames.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
          />
        </div>

        <div class="field">
          <label for="d-title">{$t("agents.agentTitle")}</label>
          <input
            id="d-title"
            value={agentTitle}
            placeholder={$t(agent.roleKey)}
            oninput={(e) =>
              agentTitles.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
          />
        </div>

        <div class="field">
          <label for="d-instructions">{$t("agents.instructions")}</label>
          <textarea
            id="d-instructions"
            rows="3"
            placeholder={$t("agents.instructions.text")}
            value={$agentInstructions[agent.id] ?? ""}
            oninput={(e) =>
              agentInstructions.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
          ></textarea>
        </div>

        <div class="section">
          <h3>{$t("agents.brain")}</h3>
          <button class="row nav-row" onclick={openModelPanel}>
            <span class="nav-left">
              <svg
                class="nav-icon"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d={ICONS.cpu} />
              </svg>
              <span class="label">{$t("agents.model")}</span>
            </span>
            <span class="nav-right">
              <span class="prov-chip">{providerName}</span>
              <span class="value">{model.label}</span>
              <svg
                class="chev-r"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                aria-hidden="true"
              >
                <path d="M9 6l6 6-6 6" />
              </svg>
            </span>
          </button>
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
          {@render navRow(ICONS.backup, $t("agents.backup"), $t("agents.backup.ask"))}
        </div>

        <div class="section">
          <h3>{$t("agents.knows")}</h3>
          {@render navRow(ICONS.memory, $t("agents.memories"), $t("agents.memories.n", { n: String(facts.memories) }))}
          {@render navRow(ICONS.skills, $t("agents.skills"), $t("agents.skills.n", { n: String(facts.skills) }))}
          {@render navRow(ICONS.files, $t("agents.files"), facts.files)}
          {@render navRow(ICONS.tables, $t("agents.tables"), $t("agents.tables.n", { n: String(facts.tables) }))}
        </div>

        <div class="section">
          <h3>{$t("agents.does")}</h3>
          {@render navRow(ICONS.routines, $t("agents.routines"), $t("agents.routines.n", { n: String(facts.routines) }))}
          {@render navRow(ICONS.usage, $t("agents.usage"), "")}
          {@render navRow(ICONS.publish, $t("agents.publish"), "")}
        </div>

        <div class="section">
          <h3>{$t("agents.rules")}</h3>
          {@render navRow(ICONS.shield, $t("agents.permissions"), $t("agents.permissions.full"))}
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
          {@render navRow(ICONS.sliders, $t("agents.advanced"), "")}
        </div>

        <div class="section">
          <h3>{$t("agents.personal")}</h3>

          <div class="row">
            <span class="label">{$t("agents.gender")}</span>
            <span class="seg" role="group" aria-label={$t("agents.gender")}>
              <button
                class:active={gender === "male"}
                onclick={() => setGender(gender === "male" ? undefined : "male")}
              >{$t("agents.gender.male")}</button>
              <button
                class:active={gender === "female"}
                onclick={() => setGender(gender === "female" ? undefined : "female")}
              >{$t("agents.gender.female")}</button>
            </span>
          </div>

          <div class="row">
            <span class="label">{$t("agents.age")}</span>
            <input
              class="age-input"
              type="number"
              min="1"
              max="120"
              inputmode="numeric"
              value={ageVal}
              placeholder={$t("agents.age.placeholder")}
              aria-label={$t("agents.age")}
              onchange={(e) => setAge(e.currentTarget.value)}
            />
          </div>
          {#if ageError}
            <p class="err" role="alert">{$t("agents.age.invalid")}</p>
          {/if}

          <div class="row">
            <span class="label">{$t("agents.position")}</span>
            <select
              value={position ?? ""}
              onchange={(e) => setPosition(e.currentTarget.value)}
              aria-label={$t("agents.position")}
            >
              <option value="">{$t("agents.position.unset")}</option>
              <option value="clevel">{$t("agents.position.clevel")}</option>
              <option value="senior">{$t("agents.position.senior")}</option>
              <option value="manager">{$t("agents.position.manager")}</option>
            </select>
          </div>

          <div class="field">
            <label for="d-hobbies">{$t("agents.hobbies")}</label>
            <input
              id="d-hobbies"
              value={hobbies}
              placeholder={$t("agents.hobbies.placeholder")}
              oninput={(e) =>
                agentHobbies.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
            />
          </div>

          <div class="row">
            <span class="label">{$t("agents.voice.allow")}</span>
            <label class="switch">
              <input
                type="checkbox"
                checked={voiceOn}
                onchange={(e) => setVoiceOn(e.currentTarget.checked)}
                aria-label={$t("agents.voice.allow")}
              />
              <span class="track" aria-hidden="true"></span>
            </label>
          </div>

          <div class="row">
            <span class="label">{$t("agents.voice.choose")}</span>
            <select
              value={voiceChoice}
              disabled={!voiceOn}
              onchange={(e) => setVoice(e.currentTarget.value)}
              aria-label={$t("agents.voice.choose")}
            >
              <option value="v1">{$t("agents.voice.1")}</option>
              <option value="v2">{$t("agents.voice.2")}</option>
              <option value="v3">{$t("agents.voice.3")}</option>
            </select>
          </div>
          <p class="note">{$t("agents.voice.note")}</p>
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
  .tb-right .menu {
    left: auto;
    right: 0;
  }
  /* Agent extras dialogs (Browser / Files / Tugas) */
  .agent-dlg {
    position: fixed;
    top: 64px;
    right: 16px;
    width: 330px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 96px);
    overflow-y: auto;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    padding: 16px;
    z-index: 60;
  }
  .agent-dlg h3 {
    margin: 0 0 2px;
    font-size: 16px;
    font-weight: 800;
  }
  .dlg-agent {
    font-size: 13px;
    color: var(--dim);
    margin: 0 0 12px;
  }
  .agent-dlg .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 44px;
  }
  .agent-dlg .label {
    font-size: 14px;
    font-weight: 600;
  }
  .agent-dlg .note {
    font-size: 13px;
    color: var(--dim);
    margin: 8px 0 0;
  }
  .agent-dlg .field-label {
    display: block;
    font-size: 13px;
    font-weight: 700;
    margin: 12px 0 4px;
  }
  .file-list {
    list-style: none;
    margin: 0 0 8px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .file-list li {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 6px 6px 6px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-2);
  }
  .file-list .fname {
    flex: 1;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-list .fsize {
    font-size: 12px;
    color: var(--dim);
    flex: none;
  }
  .mini-btn {
    width: 36px;
    height: 36px;
    flex: none;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .mini-btn.danger:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
  .dlg-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 14px;
  }
  .ok-note {
    font-size: 13px;
    color: var(--accent);
    margin: 8px 0 0;
  }

  /* 3-column split: list | chat (widest) | description */
  .agents-view .split {
    flex: 1;
    height: auto;
    min-height: 0;
    grid-template-columns: 300px minmax(0, 1fr) 360px;
  }
  .agents-view .split.no-desc {
    grid-template-columns: 300px minmax(0, 1fr);
  }

  /* Left column: agent list */
  .list-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .list-head h2 {
    flex: 1;
    font-size: 18px;
    margin: 0;
    padding: 0 4px;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .list-search {
    width: 100%;
    min-height: 44px;
    padding: 0 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--panel-2);
    color: var(--text);
    font-size: 14px;
    flex: none;
  }
  .list-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow-y: auto;
    min-height: 0;
    flex: 1;
    padding: 2px;
  }
  /* Middle column: chat area */
  .chat-col {
    min-width: 0;
    min-height: 0;
    display: flex;
    border-right: 1px solid var(--border);
    background: var(--bg);
  }
  .chat-col > :global(.chat) {
    flex: 1;
    min-width: 0;
  }

  /* Right column: agent description */
  .desc-col {
    background: var(--panel);
    padding: 24px;
    overflow-y: auto;
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .avatar-wrap {
    position: relative;
    width: 96px;
    margin: 0 auto;
    flex: none;
  }
  .badge.avatar-big {
    width: 96px;
    height: 96px;
    font-size: 44px;
    border-radius: 24px;
  }
  .avatar-edit {
    position: absolute;
    right: -6px;
    bottom: -6px;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--panel-2);
    border: 1px solid var(--border);
    color: var(--text);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex: none;
  }
  .avatar-edit:hover {
    border-color: var(--accent);
  }
  .avatar-edit svg {
    width: 20px;
    height: 20px;
  }

  /* Navigable rows (mock detail links) */
  .nav-row {
    width: 100%;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
    color: var(--text);
    text-align: left;
    padding: 10px 0;
  }
  button.nav-row:hover {
    background: var(--panel-2);
  }
  .nav-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .nav-icon {
    width: 20px;
    height: 20px;
    color: var(--dim);
    flex: none;
  }
  .nav-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    min-width: 0;
    max-width: 65%;
  }
  .nav-right .value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .chev-r {
    width: 16px;
    height: 16px;
    color: var(--dim);
    flex: none;
  }
  .prov-chip {
    font-size: 11px;
    font-weight: 700;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--accent);
    color: var(--accent);
    white-space: nowrap;
    flex: none;
  }

  /* Personalization fields */
  .desc-col .row select {
    min-height: 44px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--panel-2);
    color: var(--text);
    font-size: 14px;
  }
  .desc-col .row select:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .age-input {
    width: 120px;
    min-height: 44px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--panel-2);
    color: var(--text);
    font-size: 15px;
  }
  .desc-col .note {
    font-size: 13px;
    color: var(--dim);
    margin: 6px 0 0;
  }
  .desc-col .err {
    font-size: 13px;
    color: var(--danger);
    margin: 4px 0 0;
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
  .server-dot.cloud { background: var(--accent); }

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
  .list-head .menu {
    left: auto;
    right: 0;
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
  .sec-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 44px;
    padding: 8px 10px;
    margin-top: 6px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-2);
    color: var(--text);
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
  }
  .sec-toggle svg {
    width: 14px;
    height: 14px;
    flex: none;
    transition: transform 0.15s ease;
  }
  .sec-toggle[aria-expanded="true"] svg {
    transform: rotate(180deg);
  }
  .add-model-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 44px;
    padding: 8px 10px;
    margin: 2px 0 8px;
    border: 1px dashed var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--dim);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .add-model-btn:hover {
    color: var(--accent);
    border-color: var(--accent);
  }
  .add-model-btn svg {
    width: 14px;
    height: 14px;
    flex: none;
  }
  .add-model {
    display: flex;
    gap: 8px;
    margin: 2px 0 8px;
    padding: 8px;
    border: 1px dashed var(--border);
    border-radius: 8px;
  }
  .add-model input {
    flex: 1;
    min-width: 0;
    min-height: 44px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg);
    color: var(--text);
    font-size: 14px;
  }
  .del-model {
    width: 36px;
    height: 36px;
    flex: none;
  }
  .del-model svg {
    width: 14px;
    height: 14px;
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
    border-color: var(--warn);
    color: var(--warn);
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

  @media (max-width: 1280px) {
    .agents-view .split {
      grid-template-columns: 260px minmax(0, 1fr) 320px;
    }
    .agents-view .split.no-desc {
      grid-template-columns: 260px minmax(0, 1fr);
    }
  }
  @media (max-width: 1024px) {
    .agents-view .split,
    .agents-view .split.no-desc {
      grid-template-columns: 240px minmax(0, 1fr);
    }
    .desc-col {
      display: none;
    }
  }
  @media (max-width: 860px) {
    .topbar {
      flex-wrap: wrap;
      row-gap: 8px;
    }
    .tb-left {
      flex: 1 1 auto;
    }
    .tb-right {
      margin-left: auto;
    }
    .tb-center {
      order: 3;
      flex: 1 1 100%;
      margin-inline: 0;
      display: flex;
      justify-content: center;
    }
    .tb-center .pill {
      max-width: 100%;
    }
    .tb-center .pill span {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .tb-agent-role,
    .settings-btn span {
      display: none;
    }
    .tb-agent-name {
      max-width: 110px;
    }
  }
</style>

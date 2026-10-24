<script lang="ts">
  import { agents, agentDisplayName, agentDisplayRole, agentAvatar, AVATAR_CHOICES } from "../lib/agents";
  import ChatPanel from "./ChatPanel.svelte";
  import AgentStatusIcon from "./AgentStatusIcon.svelte";
  import AgentChatPanel from "./AgentChatPanel.svelte";
  import {
    PROVIDERS,
    activeAgentId,
    activeView,
    addCustomAgent,
    addCustomModel,
    addAgentFile,
    addKanbanCard,
    addAgentToProject,
    removeAgentFromProject,
    renameProject,
    removeFlowProject,
    addFlowProject,
    agentBrowserOn,
    agentFiles,
    agentGender,
    agentAge,
    agentHobbies,
    agentInstructions,
    agentEnabled,
    fetchAgentTools,
    saveAgentTools,
    createSocialDraft,
    fetchToolStatus,
    activeFlowProjectId,
    socialMode,
    setSocialMode,
    requestAutopost,
    confirmAutopost,
    cancelAutopost,
    autopostPending,
    skillCatalog,
    agentModel,
    setMimiModel,
    setAgentModelExplicit,
    agentProvider,
    agentNames,
    agentNotify,
    agentAvatars,
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
    folderOpen,
    activateAgent,
    deactivateAgent,
    moveAgentInProject,
    agentColWidths,
    flowRunning,
    runningProjectId,
    startFlowRun,
    stopFlowRun,
    toggleFolder,
    isFolderOpen,
  } from "../lib/store";
  import type { ModelSection } from "../lib/store";
  import type { Agent, FlowProject } from "../lib/types";
  import { t } from "../lib/i18n";
  import { get } from "svelte/store";

  const reasoningOptions = ["low", "mid", "high"] as const;

  const allAgents = $derived([...agents, ...$customAgents]);
  const agent = $derived(
    allAgents.find((a) => a.id === $activeAgentId) ?? allAgents[0],
  );
  /* Real per-agent stats (Patria 20 Okt): skill count from the backend
     catalog, file count from the agent's own attachments. Nothing fabricated. */
  const skillCount = $derived(
    ($skillCatalog ?? []).filter((s) => s.agents.includes(agent.id)).length,
  );
  const fileCount = $derived(($agentFiles[agent.id] ?? []).length);
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
  const displayName = $derived(agentDisplayName(agent, $agentNames, $t));
  const displayRole = $derived(agentDisplayRole(agent, $agentTitles, $t));
  const agentTitle = $derived($agentTitles[agent.id] ?? "");
  const chatAgent = $derived({ ...agent, name: displayName });

  // Dropdown / menu state
  let panelOpen = $state(false);
  // Menu flyouts: one at a time — "add" | "layout"; subFly = nested submenu.
  let flyout = $state<string | null>(null);
  let subFly = $state<string | null>(null);
  let addOpen = $state(false);

  let detailOpen = $state(true);
  let query = $state("");
  let agentSearch = $state("");
  // Agent extras menu (+) + dialogs
  let agentMenuOpen = $state(false);
  let agentDlg = $state<"browser" | "files" | "task" | "memories" | "skills" | "tables" | "publish" | "advanced" | "permissions" | null>(null);
  let autopostAgree = $state(false);
  let avatarPickerOpen = $state(false);
  let taskTitle = $state("");
  let taskProject = $state("");
  let taskAdded = $state(false);

  function closeAll() {
    panelOpen = false;
    addOpen = false;
    flyout = null;
    subFly = null;
    agentMenuOpen = false;
    agentDlg = null;
    query = "";
  }

  function openModelPanel() {
    closeAll();
    syncModelSection();
    panelOpen = true;
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

  // Tool permissions (Phase 3.1): allowlist editor backed by the Rust
  // tools gateway. KNOWN_TOOLS mirrors the backend registry (browser is
  // internal and not offered here).
  const KNOWN_TOOLS: { id: string; name: string; draft_only?: boolean }[] = [
    { id: "meta", name: "Meta", draft_only: true },
    { id: "postiz", name: "Postiz", draft_only: true },
    { id: "pireel", name: "Pireel" },
    { id: "drive", name: "Google Drive" },
    { id: "slack", name: "Slack" },
    { id: "discord", name: "Discord" },
    { id: "telegram", name: "Telegram" },
    { id: "github", name: "GitHub" },
    { id: "notion", name: "Notion" },
    { id: "canva", name: "Canva" },
    { id: "ruangapi", name: "RuangAPI" },
  ];
  let permAllow = $state<string[]>([]);
  let permLoadedFor = $state("");

  $effect(() => {
    if (agentDlg === "permissions" && permLoadedFor !== agent.id) {
      permLoadedFor = agent.id;
      void fetchAgentTools(agent.id).then((list) => {
        if (list) permAllow = list;
      });
    }
  });

  function togglePerm(toolId: string) {
    const next = permAllow.includes(toolId)
      ? permAllow.filter((x) => x !== toolId)
      : [...permAllow, toolId];
    permAllow = next;
    void saveAgentTools(agent.id, next);
  }

  function openPermissions() {
    permLoadedFor = "";
    agentDlg = "permissions";
  }

  // Social draft form (Phase 3.3): Meta/Postiz are draft-only. The create
  // button stays disabled until the platform tool is connected.
  let draftPlatform = $state<"meta" | "postiz">("meta");
  let draftTitle = $state("");
  let draftContent = $state("");
  let draftToolConnected = $state(false);

  function checkDraftTool() {
    void fetchToolStatus(draftPlatform).then((v) => {
      draftToolConnected = v ?? false;
    });
  }

  $effect(() => {
    if (agentDlg === "publish") checkDraftTool();
  });

  function submitSocialDraft() {
    createSocialDraft(
      draftTitle.trim(),
      draftContent.trim(),
      draftPlatform,
      agent.id,
      $activeFlowProjectId || "inbox",
    );
    draftTitle = "";
    draftContent = "";
    agentDlg = null;
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
    const pid = taskProject || $projects[0]?.id || "proj-a";
    addKanbanCard(title, agent.id, "queue", pid);
    taskTitle = "";
    taskAdded = true;
  }

  // Project folders: agents grouped into collapsible folders. The folder IS
  // the project (Patria 20 Okt): agents in a folder are the agents assigned
  // to work on that project (Tasks, Kanban, Brief).
  // Only ACTIVE agents are listed (Patria 20 Okt): the user activates agents
  // from the Minime defaults (or creates new ones); deactivated agents leave
  // the list but stay in Tools and Tasks. Mimi is always active.
  const isEnabled = (a: Agent) => a.id === "mimi" || ($agentEnabled[a.id] ?? false);
  const groupedIds = $derived(
    new Set($projects.flatMap((p) => p.agentIds ?? [])),
  );
  const looseAgents = $derived(
    allAgents.filter((a) => !groupedIds.has(a.id) && isEnabled(a)),
  );
  function folderAgents(p: FlowProject): Agent[] {
    return allAgents.filter((a) => (p.agentIds ?? []).includes(a.id) && isEnabled(a));
  }
  // Inactive registry agents available for activation (Patria 20 Okt):
  // built-in defaults not yet active plus deactivated custom agents.
  const inactiveAgents = $derived(
    allAgents.filter((a) => a.id !== "mimi" && !($agentEnabled[a.id] ?? false)),
  );
  const folderIsOpen = (id: string) => isFolderOpen(id, $folderOpen);

  // "+ Add agent" menu per folder: pick from agents not in any folder.
  let addAgentMenuFor = $state<string | null>(null);
  // Inline rename state per folder.
  let renamingFolder = $state<string | null>(null);
  let renameValue = $state("");
  function startRename(p: FlowProject) {
    renamingFolder = p.id;
    renameValue = p.name;
  }
  function commitRename(p: FlowProject) {
    if (renamingFolder === p.id) renameProject(p.id, renameValue);
    renamingFolder = null;
  }
  function addFolderProject() {
    const n = $projects.length + 1;
    const id = addFlowProject($t("tasks.projectNew", { n: String(n) }));
    folderOpen.update((m) => ({ ...m, [id]: true }));
  }
  function addNewAgent() {
    const id = addCustomAgent();
    activeAgentId.set(id);
    detailOpen = true;
    closeAll();
  }

  // Manual column resize via mouse drag (Patria 20 Okt).
  let resizing = $state<"list" | "desc" | null>(null);
  let resizeStartX = 0;
  let resizeStartW = 0;
  function startResize(which: "list" | "desc", e: PointerEvent) {
    resizing = which;
    resizeStartX = e.clientX;
    resizeStartW = $agentColWidths[which];
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function onResizeMove(e: PointerEvent) {
    if (!resizing) return;
    const dx = e.clientX - resizeStartX;
    const [lo, hi] = resizing === "list" ? [180, 480] : [240, 560];
    const w = Math.min(hi, Math.max(lo, Math.round(resizeStartW + dx)));
    agentColWidths.update((v) => ({ ...v, [resizing as "list" | "desc"]: w }));
  }
  function endResize() {
    resizing = null;
  }

  // Agent-list search
  const aq = $derived(agentSearch.trim().toLowerCase());
  const labelOf = (a: Agent) =>
    (
      agentDisplayName(a, $agentNames, $t) +
      " " +
      agentDisplayRole(a, $agentTitles, $t)
    ).toLowerCase();
  const filteredLoose = $derived(
    looseAgents.filter((a) => !aq || labelOf(a).includes(aq)),
  );
  const folderView = $derived(
    $projects
      .map((p) => ({
        p,
        agents: folderAgents(p).filter((a) => !aq || labelOf(a).includes(aq)),
      }))
      .filter(
        ({ p, agents }) =>
          !aq ||
          p.name.toLowerCase().includes(aq) ||
          agents.length > 0,
      ),
  );

  function agentFolderId(agentId: string): string {
    return $projects.find((p) => (p.agentIds ?? []).includes(agentId))?.id ?? "";
  }
  /** Svelte action: focus the element as soon as it mounts. */
  function autofocus(node: HTMLElement) {
    node.focus();
  }
  function moveAgentToFolder(agentId: string, projectId: string) {
    const cur = agentFolderId(agentId);
    if (cur) removeAgentFromProject(cur, agentId);
    if (projectId) addAgentToProject(projectId, agentId);
  }

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
  // Filter sections: hanya tampilkan provider yang dipilih (pola OpenBot)
  const visibleSections = $derived(
    MODEL_SECTIONS.filter((sec) => {
      if (sec.id === "paid") return PROVIDERS.find((p) => p.id === agentProvId)?.kind === "paid";
      return sec.id === agentProvId;
    })
  );
  const sectionOf = (id: string): ModelSection => {
    const m = getModel(id);
    return m.kind === "paid" ? "paid" : (m.providerId as ModelSection);
  };
  const sectionModels = (s: ModelSection) =>
    s === "paid" ? paidModels : s === "ollama" ? ollamaModels : opencodeModels;

  let modelSectionOpen = $state<ModelSection | null>("opencode");
  let modelAddFor = $state<ModelSection | null>(null);
  let newModelName = $state("");
  let ollamaInstalled = $state<Array<{id: string; size: string; modified: string}>>([]);
  let opencodeFree = $state<Array<{id: string; name: string; context: string}>>([]);
  let pullingModel = $state<string | null>(null);
  let modelsLoading = $state(false);
  let ollamaOpen = $state(true);
  let opencodeOpen = $state(false);
  let premiumExpanded = $state<string | null>(null);
  let premiumKey = $state("");
  let premiumEndpoint = $state("");
  let premiumSaving = $state(false);
  let premiumMsg = $state("");

  async function loadRealModels() {
    modelsLoading = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const [oll, opc] = await Promise.all([
        invoke<Array<{id: string; size: string; modified: string}>>("ollama_installed_models").catch(() => []),
        invoke<Array<{id: string; name: string; context: string}>>("opencode_free_models").catch(() => []),
      ]);
      ollamaInstalled = oll;
      opencodeFree = opc;
    } catch {} finally {
      modelsLoading = false;
    }
  }
  $effect(() => { if (modelSectionOpen) loadRealModels(); });

  async function pullModel(id: string) {
    pullingModel = id;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("ollama_pull_model", { modelName: id });
      await loadRealModels();
    } catch (e) {
      alert(String(e));
    } finally {
      pullingModel = null;
    }
  }

  async function savePremiumKey(providerId: string) {
    const k = premiumKey.trim();
    if (!k) return;
    premiumSaving = true;
    premiumMsg = "";
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("provider_connect", { providerId, apiKey: k });
      premiumMsg = "Tersimpan!";
      premiumKey = "";
    } catch (e) {
      premiumMsg = String(e);
    } finally {
      premiumSaving = false;
    }
  }

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
    // Default: agen mengikuti model Mimi; bila yang diubah adalah Mimi,
    // propagasi ke semua agen non-override. Bila agen lain, tandai override.
    if (agent.id === "mimi") {
      setMimiModel(id);
    } else {
      setAgentModelExplicit(agent.id, id);
    }
  }
  // Pola OpenBot (clean-room): pilih provider dulu, model mengikuti.
  // Ganti provider → otomatis pilih model pertama provider itu.
  function setProvider(pid: string) {
    agentProvider.update((p) => ({ ...p, [agent.id]: pid }));
    const catalog = get(modelCatalog);
    const first = catalog.find((m) => m.providerId === pid);
    if (first) {
      if (agent.id === "mimi") {
        setMimiModel(first.id);
      } else {
        setAgentModelExplicit(agent.id, first.id);
      }
    }
    // Local-first: jika belum konek → tandai agar SettingsView
    // otomatis trigger setup (download/API key dialog)
    const prov = PROVIDERS.find((x) => x.id === pid);
    const st = get(providerState)[pid] ?? { connected: false };
    if (prov && !st.connected && (prov.kind === "paid" || prov.local)) {
      try {
        localStorage.setItem("minime.pending-connect", pid);
      } catch {}
    }
  }
  const agentProvId = $derived(
    $agentProvider[agent.id] ?? getModel($agentModel[agent.id] ?? defaultModelFor(agent.id)).providerId
  );
  const providerModels = $derived($modelCatalog.filter((m) => m.providerId === agentProvId));
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
  {@const rowName = agentDisplayName(a, $agentNames, $t)}
  {@const rowRole = agentDisplayRole(a, $agentTitles, $t)}
  <button
    class="agent-row"
    class:active={$activeAgentId === a.id}
    onclick={() => activeAgentId.set(a.id)}
  >
    {#if agentAvatar(a, $agentAvatars)}<img class="badge-img" src={agentAvatar(a, $agentAvatars)} alt={a.name} />{:else}<span class="badge" style:background={a.color}>{a.badge}</span>{/if}
    <span class="meta">
      <span class="name">{rowName}</span>
      <span class="role">{rowRole}</span>
    </span>
    <AgentStatusIcon status={status} size={14} />
  </button>
{/snippet}

{#snippet navRow(icon: string, label: string, valueText: string, onclick?: () => void)}
  <button class="row nav-row" onclick={onclick ?? closeAll}>
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
  <!-- Top bar: active agent | model shortcut | panel controls -->
  <div class="topbar">
    <div class="tb-left">
<!-- Active agent identity -->
      <div class="tb-agent">
        {#if agentAvatar(agent, $agentAvatars)}<img class="badge-img sm" src={agentAvatar(agent, $agentAvatars)} alt={agent.name} />{:else}<span class="badge sm" style:background={agent.color}>{agent.badge}</span>{/if}
        <span class="tb-agent-meta">
          <span class="tb-agent-name">{displayName}</span>
          <span class="tb-agent-role">{displayRole}</span>
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
          <button
            class="panel-x"
            onclick={closeAll}
            aria-label={$t("common.close")}
            title={$t("common.close")}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <line x1="6" y1="6" x2="18" y2="18" />
              <line x1="18" y1="6" x2="6" y2="18" />
            </svg>
          </button>
          <div class="field">
            <label for="agent-provider">{$t("model.provider")}</label>
            <select
              id="agent-provider"
              value={agentProvId}
              onchange={(e) => setProvider(e.currentTarget.value)}
              aria-label={$t("model.provider")}
            >
              {#each PROVIDERS as pv}
                <option value={pv.id}>
                  {pv.name} — {$t(pv.kind === "free" ? "providers.free" : "providers.paid")}
                </option>
              {/each}
            </select>
          </div>
          <div class="field field-search">
            <input
              type="search"
              bind:value={query}
              placeholder={$t("model.search")}
              aria-label={$t("model.search")}
            />
          </div>

          <!-- OLLAMA: dropdown list real -->
          <div class="xiryn-prov">
            <button class="xiryn-prov-head" onclick={() => (ollamaOpen = !ollamaOpen)}>
              <span class="xiryn-prov-name">Ollama</span>
              <span class="chip">Gratis</span>
              <span class="spacer"></span>
              <span class="hint-sm">{ollamaInstalled.length} model</span>
              <span class="chev">{ollamaOpen ? "▾" : "▸"}</span>
            </button>
            {#if ollamaOpen}
            <div class="xiryn-prov-body">
            {#if ollamaInstalled.length === 0}
              <p class="hint-sm">{modelsLoading ? "Memuat..." : "Ollama tidak jalan atau belum ada model terinstall."}</p>
            {/if}
            {#each ollamaInstalled as m}
              {@const mid = `ollama:${m.id}`}
              {@const isUsed = modelId === mid}
              <div class="xiryn-model-row">
                <div class="xiryn-model-meta">
                  <span class="xiryn-model-name">{m.id}</span>
                  <span class="xiryn-model-desc">{m.size}{m.modified ? " · " + m.modified : ""}</span>
                </div>
                {#if isUsed}
                  <button class="btn-used" disabled>Terpakai</button>
                {:else}
                  <span class="badge-ok">✓ Siap pakai</span>
                  <button class="btn-ghost sm" onclick={() => setModel(mid)}>
                    Pakai
                  </button>
                {/if}
              </div>
            {/each}
            <div class="field" style="margin-top:10px">
              <label>Download model baru (nama dari ollama.com/library)</label>
              <div class="row">
                <input type="text" id="ollama-pull-name" placeholder="cth: llama3.1:8b" style="flex:1" />
                <button class="btn-primary sm" disabled={!!pullingModel}
                  onclick={() => {
                    const el = document.getElementById("ollama-pull-name") as HTMLInputElement;
                    if (el?.value.trim()) pullModel(el.value.trim());
                  }}>
                  {pullingModel ? "Mengunduh..." : "⬇ Download"}
                </button>
              </div>
            </div>
            </div>
            {/if}
          </div>

          <!-- OPENCODE: dropdown list real -->
          <div class="xiryn-prov">
            <button class="xiryn-prov-head" onclick={() => (opencodeOpen = !opencodeOpen)}>
              <span class="xiryn-prov-name">OpenCode</span>
              <span class="chip">Gratis</span>
              <span class="spacer"></span>
              <span class="hint-sm">{opencodeFree.length} model</span>
              <span class="chev">{opencodeOpen ? "▾" : "▸"}</span>
            </button>
            {#if opencodeOpen}
            <div class="xiryn-prov-body">
            <p class="hint-sm">Model free dari OpenCode Zen, tanpa API key. Daftar real dari opencode.ai.</p>
            {#if opencodeFree.length === 0}
              <p class="hint-sm">{modelsLoading ? "Memuat..." : "Gagal memuat daftar."}</p>
            {/if}
            {#each opencodeFree as m}
              {@const mid = `opencode:${m.id}`}
              {@const isUsed = modelId === mid}
              <div class="xiryn-model-row">
                <div class="xiryn-model-meta">
                  <span class="xiryn-model-name">{m.id}</span>
                  <span class="xiryn-model-desc">{m.name}{m.context ? " · ctx " + m.context : ""}</span>
                </div>
                {#if isUsed}
                  <button class="btn-used" disabled>Terpakai</button>
                {:else}
                  <span class="badge-ok">Gratis</span>
                  <button class="btn-ghost sm" onclick={() => setModel(mid)}>
                    Pakai
                  </button>
                {/if}
              </div>
            {/each}
            </div>
            {/if}
          </div>

          <!-- PREMIUM: klik untuk input API key ala Xiryn -->
          <div class="xiryn-sec">
            <span class="group-label">Premium — API Key</span>
            <p class="hint-sm">Pilih provider, masukkan API key, simpan.</p>
            {#each ["muse", "gemini", "deepseek"] as pid}
              <div class="xiryn-prov">
                <button
                  class="xiryn-prov-head"
                  onclick={() => {
                    premiumExpanded = premiumExpanded === pid ? null : pid;
                    premiumKey = "";
                    premiumEndpoint = "";
                    premiumMsg = "";
                  }}
                >
                  <span class="xiryn-prov-name">{pid === "muse" ? "Muse" : pid === "gemini" ? "Gemini" : "DeepSeek"}</span>
                  <span class="chip paid">Berbayar</span>
                  <span class="spacer"></span>
                  <span class="chev">{premiumExpanded === pid ? "▾" : "▸"}</span>
                </button>
                {#if premiumExpanded === pid}
                  <div class="xiryn-prov-body">
                    <div class="field">
                      <label>API Key</label>
                      <input type="password" bind:value={premiumKey}
                        placeholder="Tempel API key" autocomplete="off" spellcheck={false} />
                    </div>
                    <div class="field">
                      <label>Endpoint (opsional)</label>
                      <input type="text" bind:value={premiumEndpoint}
                        placeholder="https://..." spellcheck={false} />
                    </div>
                    <div class="row">
                      <button class="btn-primary sm"
                        onclick={() => savePremiumKey(pid)}
                        disabled={premiumSaving || !premiumKey.trim()}>
                        {premiumSaving ? "..." : "Save"}
                      </button>
                      {#if premiumMsg}<span class="hint-sm">{premiumMsg}</span>{/if}
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>

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
      <button
        class="panel-x"
        onclick={() => (agentDlg = null)}
        aria-label={$t("common.close")}
        title={$t("common.close")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <line x1="6" y1="6" x2="18" y2="18" />
          <line x1="18" y1="6" x2="6" y2="18" />
        </svg>
      </button>
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
      {:else if agentDlg === "permissions"}
        <h3>{$t("agents.permissions")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <p class="note">{$t("agents.permissions.note")}</p>
        <div class="perm-list">
          {#each KNOWN_TOOLS as tool (tool.id)}
            <label class="perm-item">
              <input
                type="checkbox"
                checked={permAllow.includes(tool.id) || permAllow.includes("*")}
                disabled={permAllow.includes("*")}
                onchange={() => togglePerm(tool.id)}
              />
              <span>{tool.name}</span>
              {#if tool.draft_only}<span class="perm-badge">draft</span>{/if}
            </label>
          {/each}
        </div>
        <div class="dlg-actions">
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
        </div>
      {:else if agentDlg === "skills"}
        <h3>{$t("agents.skills")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <p class="note">{$t("agents.skills.n", { n: String(skillCount) })}</p>
        <div class="dlg-actions">
          <button class="btn-ghost" onclick={() => { agentDlg = null; activeView.set("tools"); }}>{$t("agents.skills.manage")}</button>
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
        </div>
      {:else if agentDlg === "publish"}
        {#if $autopostPending}
          <h3>{$t("agents.publish.consent.title")}</h3>
          <p class="note warn" role="alert">
            {$t("agents.publish.consent.body", {
              platform: $autopostPending === "meta" ? "Meta" : "Postiz",
            })}
          </p>
          <ul class="consent-list">
            <li>{$t("agents.publish.consent.c1")}</li>
            <li>{$t("agents.publish.consent.c2")}</li>
            <li>{$t("agents.publish.consent.c3")}</li>
          </ul>
          <label class="consent-check">
            <input type="checkbox" bind:checked={autopostAgree} />
            <span>{$t("agents.publish.consent.agree")}</span>
          </label>
          <div class="dlg-actions">
            <button
              class="btn-primary"
              disabled={!autopostAgree}
              onclick={() => { confirmAutopost(); autopostAgree = false; }}
            >{$t("agents.publish.consent.enable")}</button>
            <button
              class="btn-ghost"
              onclick={() => { cancelAutopost(); autopostAgree = false; }}
            >{$t("agents.task.cancel")}</button>
          </div>
        {:else}
        <h3>{$t("agents.publish")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <p class="note">{$t("agents.publish.note")}</p>
        <p class="note draft-rule">{$t("agents.publish.draftrule")}</p>
        <label class="fld">
          <span>{$t("agents.publish.platform")}</span>
          <select
            bind:value={draftPlatform}
            onchange={() => checkDraftTool()}
          >
            <option value="meta">Meta</option>
            <option value="postiz">Postiz</option>
          </select>
        </label>
        <label class="fld">
          <span>{$t("agents.publish.mode")}</span>
          <select
            value={$socialMode[draftPlatform] ?? "draft"}
            onchange={(e) => {
              const v = (e.currentTarget as HTMLSelectElement).value as "draft" | "autopost";
              if (v === "autopost") requestAutopost(draftPlatform);
              else setSocialMode(draftPlatform, "draft");
            }}
          >
            <option value="draft">{$t("agents.publish.mode.draft")}</option>
            <option value="autopost">{$t("agents.publish.mode.autopost")}</option>
          </select>
        </label>
        {#if ($socialMode[draftPlatform] ?? "draft") === "autopost"}
          <p class="note warn">{$t("agents.publish.autopost.warn")}</p>
        {/if}
        <label class="fld">
          <span>{$t("agents.publish.title")}</span>
          <input type="text" bind:value={draftTitle} maxlength="120" />
        </label>
        <label class="fld">
          <span>{$t("agents.publish.content")}</span>
          <textarea bind:value={draftContent} rows="4" maxlength="2000"></textarea>
        </label>
        {#if !draftToolConnected}
          <p class="note warn">{$t("agents.publish.notconnected", { platform: draftPlatform === "meta" ? "Meta" : "Postiz" })}</p>
        {/if}
        <div class="dlg-actions">
          <button
            class="btn-primary"
            disabled={!draftToolConnected || !draftTitle.trim() || !draftContent.trim()}
            onclick={submitSocialDraft}
          >{$t("agents.publish.create")}</button>
          <button class="btn-ghost" onclick={() => (agentDlg = null)}>{$t("agents.task.cancel")}</button>
        </div>
        {/if}
      {:else if agentDlg === "advanced"}
        <h3>{$t("agents.advanced")}</h3>
        <p class="dlg-agent">{displayName}</p>
        <p class="note">{$t("agents.advanced.note")}</p>
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

  <div
    class="split"
    class:no-desc={!detailOpen}
    style:grid-template-columns={detailOpen
      ? `${$agentColWidths.list}px 8px minmax(0,1fr) 8px ${$agentColWidths.desc}px`
      : `${$agentColWidths.list}px 8px minmax(0,1fr)`}
  >
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
                  addNewAgent();
                }}
              >
                <span class="check" aria-hidden="true"></span>
                <span>{$t("agents.addAgent")}</span>
              </button>
              <button
                class="menu-item"
                onclick={() => {
                  addFolderProject();
                  closeAll();
                }}
              >
                <span class="check" aria-hidden="true"></span>
                <span>{$t("agents.addFolder")}</span>
              </button>
              {#if inactiveAgents.length > 0}
                <div class="menu-sep" aria-hidden="true"></div>
                <div class="menu-label">{$t("agents.addFromDefaults")}</div>
                {#each inactiveAgents as a (a.id)}
                  <button
                    class="menu-item"
                    onclick={() => {
                      activateAgent(a.id);
                      activeAgentId.set(a.id);
                      detailOpen = true;
                      closeAll();
                    }}
                  >
                    {#if agentAvatar(a, $agentAvatars)}<img class="badge-img sm" src={agentAvatar(a, $agentAvatars)} alt={a.name} />{:else}<span class="badge sm" style:background={a.color}>{a.badge}</span>{/if}
                    <span>{agentDisplayName(a, $agentNames, $t)}</span>
                  </button>
                {/each}
              {/if}
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
          <div class="agent-row-wrap">
            {@render agentRow(a)}
            {#if a.id !== "mimi"}
              <button
                class="row-del"
                onclick={(e) => {
                  e.stopPropagation();
                  deactivateAgent(a.id);
                }}
                aria-label={$t("agents.removeFromList")}
                title={$t("agents.removeFromList")}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                  <path d="M6 6l12 12M18 6L6 18" />
                </svg>
              </button>
            {/if}
          </div>
        {/each}
        {#each folderView as { p, agents: fAgents } (p.id)}
          {@const open = folderIsOpen(p.id) || !!aq}
          <div class="folder">
            <div class="folder-head">
              <button
                class="folder-toggle"
                onclick={() => toggleFolder(p.id)}
                aria-expanded={open}
                aria-label={p.name}
              >
                <svg
                  class="chev"
                  class:open={open}
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  aria-hidden="true"
                >
                  <path d="M9 6l6 6-6 6" />
                </svg>
                {#if renamingFolder === p.id}
                  <input
                    class="folder-rename"
                    bind:value={renameValue}
                    onclick={(e) => e.stopPropagation()}
                    onkeydown={(e) => {
                      if (e.key === "Enter") commitRename(p);
                      else if (e.key === "Escape") renamingFolder = null;
                    }}
                    onblur={() => commitRename(p)}
                    aria-label={$t("agents.folder.rename")}
                    use:autofocus
                  />
                {:else}
                  <span class="folder-name">{p.name}</span>
                {/if}
                <span class="folder-count"
                  >{$t("agents.folder.count", { n: String(fAgents.length) })}</span
                >
              </button>
              <div class="folder-actions">
                {#if $flowRunning && $runningProjectId === p.id}
                  <button
                    class="icon-btn sm"
                    aria-label={$t("agents.folder.pause")}
                    title={$t("agents.folder.pause")}
                    onclick={(e) => {
                      e.stopPropagation();
                      stopFlowRun();
                    }}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <rect x="7" y="5" width="3.4" height="14" rx="1" fill="currentColor" stroke="none" />
                      <rect x="13.6" y="5" width="3.4" height="14" rx="1" fill="currentColor" stroke="none" />
                    </svg>
                  </button>
                {:else}
                  <button
                    class="icon-btn sm"
                    disabled={fAgents.length === 0 || $flowRunning}
                    aria-label={$t("agents.folder.play")}
                    title={$t("agents.folder.play")}
                    onclick={(e) => {
                      e.stopPropagation();
                      startFlowRun(p.id);
                    }}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <path d="M8 5v14l11-7z" fill="currentColor" stroke="none" />
                    </svg>
                  </button>
                {/if}
                <button
                  class="icon-btn sm"
                  aria-label={$t("agents.folder.addAgent")}
                  title={$t("agents.folder.addAgent")}
                  onclick={(e) => {
                    e.stopPropagation();
                    addAgentMenuFor = addAgentMenuFor === p.id ? null : p.id;
                  }}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M12 5v14M5 12h14" />
                  </svg>
                </button>
                <button
                  class="icon-btn sm"
                  aria-label={$t("agents.folder.rename")}
                  title={$t("agents.folder.rename")}
                  onclick={(e) => {
                    e.stopPropagation();
                    startRename(p);
                  }}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M17 3a2.8 2.8 0 1 1 4 4L8 20l-5 1 1-5z" />
                  </svg>
                </button>
                <button
                    class="icon-btn sm danger"
                    aria-label={$t("agents.folder.delete")}
                    title={$t("agents.folder.delete")}
                    onclick={(e) => {
                      e.stopPropagation();
                      removeFlowProject(p.id);
                    }}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                      <path d="M6 6l12 12M18 6L6 18" />
                    </svg>
                  </button>
              </div>
            </div>
            {#if addAgentMenuFor === p.id}
              {@const candidates = allAgents.filter(
                (a) =>
                  a.id !== "mimi" && isEnabled(a) && !(p.agentIds ?? []).includes(a.id),
              )}
              <div class="folder-add-menu" role="menu" aria-label={$t("agents.folder.addAgent")}>
                {#if candidates.length === 0}
                  <p class="folder-add-empty">{$t("agents.folder.noLooseAgents")}</p>
                {:else}
                  {#each candidates as a (a.id)}
                    <button
                      class="menu-item"
                      onclick={() => {
                        addAgentToProject(p.id, a.id);
                        addAgentMenuFor = null;
                      }}
                    >
                      {#if agentAvatar(a, $agentAvatars)}<img class="badge-img sm" src={agentAvatar(a, $agentAvatars)} alt={a.name} />{:else}<span class="badge sm" style:background={a.color}>{a.badge}</span>{/if}
                      <span>{agentDisplayName(a, $agentNames, $t)}</span>
                    </button>
                  {/each}
                {/if}
              </div>
            {/if}
            {#if open}
              <div class="folder-body">
                {#each fAgents as a, i (a.id)}
                  <div class="folder-agent">
                    {@render agentRow(a)}
                    <div class="order-btns" role="group" aria-label={$t("agents.folder.order")}>                      <button
                        class="order-btn"
                        disabled={i === 0}
                        onclick={() => moveAgentInProject(p.id, a.id, -1)}
                        aria-label={$t("agents.folder.moveUp")}
                        title={$t("agents.folder.moveUp")}
                      >↑</button>
                      <button
                        class="order-btn"
                        disabled={i === fAgents.length - 1}
                        onclick={() => moveAgentInProject(p.id, a.id, 1)}
                        aria-label={$t("agents.folder.moveDown")}
                        title={$t("agents.folder.moveDown")}
                      >↓</button>
                    </div>
                    {#if a.id !== "mimi"}
                      <button
                        class="row-del"
                        onclick={(e) => {
                          e.stopPropagation();
                          deactivateAgent(a.id);
                        }}
                        aria-label={$t("agents.removeFromList")}
                        title={$t("agents.removeFromList")}
                      >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                          <path d="M6 6l12 12M18 6L6 18" />
                        </svg>
                      </button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Column resizer: list | chat -->
    <div
      class="col-resizer"
      class:dragging={resizing === "list"}
      onpointerdown={(e) => startResize("list", e)}
      onpointermove={onResizeMove}
      onpointerup={endResize}
      onpointercancel={endResize}
      role="separator"
      aria-orientation="vertical"
      aria-label={$t("agents.resizeList")}
    ></div>

    <AgentChatPanel chatAgent={chatAgent} />

    {#if detailOpen}
      <!-- Column resizer: chat | detail -->
      <div
        class="col-resizer"
        class:dragging={resizing === "desc"}
        onpointerdown={(e) => startResize("desc", e)}
        onpointermove={onResizeMove}
        onpointerup={endResize}
        onpointercancel={endResize}
        role="separator"
        aria-orientation="vertical"
        aria-label={$t("agents.resizeDesc")}
      ></div>
    {/if}

    <!-- Right column: agent description -->
    {#if detailOpen}
      <div class="desc-col">
        <div class="avatar-wrap">
          {#if agentAvatar(agent, $agentAvatars)}
            <img
              class="badge-img avatar-big"
              src={agentAvatar(agent, $agentAvatars)}
              alt={displayName}
            />
          {:else}
            <span
              class="badge avatar-big"
              style:background={agent.color}
            >{agent.badge}</span>
          {/if}
          <button
            class="avatar-edit"
            aria-label={$t("agents.editAvatar")}
            onclick={() => (avatarPickerOpen = true)}
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
        {#if avatarPickerOpen}
          <div class="avatar-picker" role="dialog" aria-label={$t("agents.editAvatar")}>
            {#each AVATAR_CHOICES as choice}
              <button
                class="avatar-choice"
                class:selected={agentAvatar(agent, $agentAvatars) === choice}
                onclick={() => {
                  agentAvatars.update((m) => ({ ...m, [agent.id]: choice }));
                  avatarPickerOpen = false;
                }}
              >
                <img src={choice} alt={choice} />
              </button>
            {/each}
            <button
              class="btn-ghost"
              onclick={() => (avatarPickerOpen = false)}
            >{$t("agents.task.cancel")}</button>
          </div>
        {/if}

        {#if agent.id !== "mimi"}
          <div class="row">
            <span class="label">{$t("agents.enabled")}</span>
            <label class="switch">
              <input
                type="checkbox"
                checked={isEnabled(agent)}
                onchange={(e) => {
                  const on = e.currentTarget.checked;
                  if (on) activateAgent(agent.id);
                  else {
                    deactivateAgent(agent.id);
                    // Leave the detail panel on an active agent.
                    activeAgentId.set("mimi");
                  }
                }}
                aria-label={$t("agents.enabled")}
              />
              <span class="track" aria-hidden="true"></span>
            </label>
          </div>
        {/if}

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

        <div class="field">
          <label for="d-folder">{$t("agents.folder.label")}</label>
          <select
            id="d-folder"
            value={agentFolderId(agent.id)}
            onchange={(e) => moveAgentToFolder(agent.id, e.currentTarget.value)}
          >
            <option value="">{$t("agents.folder.none")}</option>
            {#each $projects as p (p.id)}
              <option value={p.id}>{p.name}</option>
            {/each}
          </select>
          <p class="hint">{$t("agents.folder.hint")}</p>
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
          {@render navRow(ICONS.backup, $t("agents.backup"), $t("agents.backup.ask"), openModelPanel)}
        </div>

        <div class="section">
          <h3>{$t("agents.knows")}</h3>
          {@render navRow(ICONS.skills, $t("agents.skills"), $t("agents.skills.n", { n: String(skillCount) }), () => (agentDlg = "skills"))}
          {@render navRow(ICONS.files, $t("agents.files"), $t("agents.files.n", { n: String(fileCount) }), () => activeView.set("library"))}
        </div>

        <div class="section">
          <h3>{$t("agents.does")}</h3>
          {@render navRow(ICONS.usage, $t("agents.usage"), "", () => activeView.set("cost"))}
          {@render navRow(ICONS.publish, $t("agents.publish"), "", () => (agentDlg = "publish"))}
        </div>

        <div class="section">
          <h3>{$t("agents.rules")}</h3>
          {@render navRow(ICONS.shield, $t("agents.permissions"), $t("agents.permissions.full"), openPermissions)}
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
          {@render navRow(ICONS.sliders, $t("agents.advanced"), "", () => (agentDlg = "advanced"))}
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
    z-index: 50;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
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
    background: var(--panel-solid);
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
  .fld {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-top: 10px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-2);
  }
  .fld input, .fld select, .fld textarea {
    padding: 9px 11px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--panel);
    color: var(--text);
    font: inherit;
    font-weight: 400;
  }
  .fld textarea { resize: vertical; }
  .note.warn { color: var(--danger); }
  .note.draft-rule { color: var(--accent); font-weight: 600; }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }
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
    grid-template-columns: 262px minmax(0, 1fr) 322px;
  }
  .agents-view .split.no-desc {
    grid-template-columns: 262px minmax(0, 1fr);
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
.badge-img {
  width: 40px; height: 40px; border-radius: 50%; object-fit: cover;
  flex: none;
}
.badge-img.sm { width: 28px; height: 28px; }
.avatar-picker {
  display: flex; flex-wrap: wrap; gap: 8px; align-items: center;
  padding: 12px; margin-top: 8px;
  background: var(--dlg-bg, #1a1a1a); border-radius: 12px;
}
.avatar-choice {
  width: 56px; height: 56px; border-radius: 50%; overflow: hidden;
  border: 2px solid transparent; padding: 0; cursor: pointer; background: none;
}
.avatar-choice img { width: 100%; height: 100%; object-fit: cover; }
.avatar-choice.selected { border-color: #4fa6f0; }
.badge-img.avatar-big { width: 72px; height: 72px; }
  .badge.sm {
    width: 32px;
    height: 32px;
    font-size: 15px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-height: 33px;
    padding: 0 12px;
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
    gap: 7px;
    min-height: 33px;
    padding: 0 12px;
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
    background: var(--panel-solid);
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
  .pill .srv-icon {
    width: 17px;
    height: 17px;
    flex: none;
  }
  .srv-kind-icon {
    width: 22px;
    height: 22px;
    flex: none;
    color: var(--text-2);
  }
  .srv-kind-icon.local { color: #7ac74f; }
  .srv-kind-icon.cloud { color: var(--accent); }
  .srv-kind-icon.independent { color: #c9a44f; }
  .menu-item .sub-note {
    display: block;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--dim);
  }
  .menu-item .sub-label-text.col {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .menu-item .sub-note-inline {
    font-size: 13px;
    font-weight: 500;
    color: var(--dim);
    margin-left: 2px;
  }
  .menu-item .radio-empty {
    display: inline-block;
    width: 15px;
    height: 15px;
    border: 1.8px solid var(--dim);
    border-radius: 50%;
    vertical-align: -2px;
  }
  .menu-item:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .scrim {
    position: fixed;
    inset: 0;
    z-index: 55;
  }
  .xiryn-sec { margin: 16px 0; }
  .xiryn-sec-head { display: flex; align-items: center; justify-content: space-between; }
  .hint-sm { font-size: 12px; color: var(--dim); margin: 4px 0 10px; }
  .xiryn-model-row {
    display: flex; align-items: center; gap: 10px;
    padding: 10px 12px; border: 1px solid var(--border); border-radius: 8px; margin-bottom: 8px;
  }
  .xiryn-model-meta { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .xiryn-model-name { font-family: var(--font-mono); font-size: 13px; font-weight: 600; }
  .xiryn-model-desc { font-size: 11px; color: var(--dim); }
  .badge-ok { font-size: 11px; color: #4FA6F0; font-weight: 600; }
  .btn-used {
    padding: 6px 14px; border-radius: 8px; font-size: 13px; font-weight: 600;
    background: #4FA6F0; color: #fff; border: none; cursor: default;
  }
  .xiryn-prov { border: 1px solid var(--border); border-radius: 8px; margin-bottom: 8px; overflow: hidden; }
  .xiryn-prov-head {
    width: 100%; display: flex; align-items: center; gap: 10px;
    padding: 12px; background: none; border: none; cursor: pointer; color: var(--text); text-align: left;
  }
  .xiryn-prov-head:hover { background: var(--hover); }
  .xiryn-prov-name { font-weight: 600; font-size: 14px; }
  .xiryn-prov-body { padding: 0 12px 12px; border-top: 1px solid var(--border); }
  .model-panel {
    position: absolute;
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    width: 340px;
    max-width: calc(100vw - 120px);
    max-height: min(70vh, 560px);
    overflow-y: auto;
    background: var(--panel-solid);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    padding: 12px;
    z-index: 100;
  }
  .panel-x {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--dim);
    cursor: pointer;
  }
  .panel-x:hover { background: var(--panel-2); color: var(--text); }
  .perm-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 10px 0;
  }
  .perm-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 14px;
    cursor: pointer;
  }
  .perm-item:hover { background: var(--panel-2); }
  .perm-badge {
    margin-left: auto;
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 1px 6px;
  }
  .model-panel .field-search { margin-right: 40px; }
  .model-panel .field {    margin-bottom: 4px;
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
    gap: 4px;
    width: 100%;
    min-height: 44px;
    padding: 0 8px 0 12px;
    background: transparent;
    color: var(--text);
  }
  .folder-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    min-height: 44px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    text-align: left;
    padding: 0;
  }
  .folder-toggle:hover {
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
  .folder-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: none;
  }
  .icon-btn.sm {
    width: 30px;
    height: 30px;
    min-height: 30px;
    border-radius: 8px;
  }
  .icon-btn.sm svg {
    width: 15px;
    height: 15px;
  }
  .icon-btn.sm.danger:hover {
    color: #ff7a7a;
    background: rgba(255, 90, 90, 0.12);
  }
  .folder-rename {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 700;
    color: var(--text);
    background: var(--panel-2);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 6px 10px;
    outline: none;
  }
  .folder-add-menu {
    margin: 2px 8px 6px;
    padding: 6px;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 12px;
    max-height: 220px;
    overflow-y: auto;
  }
  .folder-add-menu .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
  }
  .folder-add-menu .menu-item:hover {
    background: var(--panel-3);
  }
  .folder-add-empty {
    margin: 0;
    padding: 10px;
    font-size: 12px;
    color: var(--dim);
  }
  .badge.sm {
    width: 26px;
    height: 26px;
    font-size: 12px;
  }
  .field .hint {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--dim);
    line-height: 1.5;
  }
  .field select {
    width: 100%;
    min-height: 44px;
    padding: 0 12px;
    font-size: 14px;
    color: var(--text);
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 10px;
    outline: none;
  }
  .folder-body {
    padding: 2px 6px 8px;
  }
  .folder-agent {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .folder-agent .agent-row {
    flex: 1;
    min-width: 0;
  }
  .agent-row-wrap {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .agent-row-wrap .agent-row {
    flex: 1;
    min-width: 0;
  }
  .row-del {
    width: 26px;
    height: 26px;
    flex: none;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--dim);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }
  .row-del svg {
    width: 14px;
    height: 14px;
  }
  .row-del:hover {
    background: var(--panel-2);
    color: #ff6b6b;
  }
  .order-btns {
    display: flex;
    flex-direction: column;
    flex: none;
  }
  .order-btn {
    width: 22px;
    height: 18px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--dim);
    font-size: 11px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }
  .order-btn:hover:not(:disabled) {
    background: var(--panel-2);
    color: var(--text);
  }
  .order-btn:disabled {
    opacity: 0.25;
    cursor: default;
  }
  /* Manually resizable column edges (Patria 20 Okt): drag with the mouse. */
  .col-resizer {
    width: 8px;
    margin: 0 -4px;
    cursor: col-resize;
    z-index: 5;
    background: transparent;
    border: 0;
    padding: 0;
  }
  .col-resizer:hover,
  .col-resizer.dragging {
    background: color-mix(in srgb, var(--accent) 45%, transparent);
  }

  @media (max-width: 1280px) {
    .agents-view .split {
      grid-template-columns: 232px minmax(0, 1fr) 292px;
    }
    .agents-view .split.no-desc {
      grid-template-columns: 232px minmax(0, 1fr);
    }
  }
  @media (max-width: 1024px) {
    .agents-view .split,
    .agents-view .split.no-desc {
      grid-template-columns: 216px minmax(0, 1fr);
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

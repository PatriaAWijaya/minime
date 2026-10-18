<script lang="ts">
  import { writable, get } from "svelte/store";
  import {
    SvelteFlow,
    Background,
    BackgroundVariant,
    Controls,
    MiniMap,
    type Node,
    type Edge,
    type Connection,
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
  import { agents } from "../lib/agents";
  import {
    agentStatus,
    customAgents,
    tasksView,
    kanbanCards,
    kanbanFilterProject,
    kanbanFilterAgent,
    kanbanFilterPeriod,
    projects,
    activeFlowProjectId,
    switchFlowProject,
    addFlowProject,
    flowNodes,
    flowEdges,
    addKanbanCard,
    moveKanbanCard,
    addFlowNode,
    addFlowEdge,
    moveFlowNode,
    removeFlowNode,
    disconnectFlowNode,
    type TasksViewKind,
  } from "../lib/store";
  import type { KanbanCol } from "../lib/types";
  import { t } from "../lib/i18n";
  import FlowAgentNode from "./FlowAgentNode.svelte";

  const cols: KanbanCol[] = ["queue", "doing", "approval", "revision", "done"];
  const allAgents = $derived([...agents, ...$customAgents]);

  function agentOf(id: string) {
    return allAgents.find((a) => a.id === id) ?? allAgents[0];
  }

  function setView(v: TasksViewKind) {
    tasksView.set(v);
  }

  // ---------- Kanban filters ----------
  const filteredCards = $derived(
    $kanbanCards.filter((c) => {
      if ($kanbanFilterProject !== "all" && c.projectId !== $kanbanFilterProject)
        return false;
      if ($kanbanFilterAgent !== "all" && c.agentId !== $kanbanFilterAgent)
        return false;
      if ($kanbanFilterPeriod !== "all") {
        const days = Number($kanbanFilterPeriod);
        if (Date.now() - c.createdAtTs > days * 86400000) return false;
      }
      return true;
    }),
  );

  // ---------- Kanban: add-card form ----------
  let addingCol = $state<KanbanCol | null>(null);
  let draftTitle = $state("");
  let draftAgent = $state("mimi");
  let draftProject = $state("proj-a");

  function openForm(col: KanbanCol) {
    addingCol = col;
    draftTitle = "";
    draftAgent = "mimi";
    draftProject = $projects[0]?.id ?? "proj-a";
  }

  function submitForm() {
    const title = draftTitle.trim();
    if (!title || !addingCol) return;
    addKanbanCard(title, draftAgent, addingCol, draftProject);
    addingCol = null;
  }

  // ---------- Kanban: HTML5 drag & drop ----------
  function onDragStart(e: DragEvent, id: number) {
    if (e.dataTransfer) {
      e.dataTransfer.setData("text/plain", String(id));
      e.dataTransfer.effectAllowed = "move";
    }
  }

  function onDropCol(e: DragEvent, col: KanbanCol) {
    e.preventDefault();
    const id = Number(e.dataTransfer?.getData("text/plain"));
    if (!Number.isNaN(id)) moveKanbanCard(id, col);
  }

  // ---------- Flow canvas ----------
  const nodeTypes = { agent: FlowAgentNode };

  function handleAddChild(sourceId: string, agentId: string) {
    const src = get(flowNodes).find((n) => n.id === sourceId);
    const id = addFlowNode(agentId, (src?.x ?? 80) + 320, src?.y ?? 120);
    addFlowEdge(sourceId, id);
    reloadFlow();
  }

  function toSfNode(n: { id: string; agentId: string; x: number; y: number }): Node {
    return {
      id: n.id,
      type: "agent",
      position: { x: n.x, y: n.y },
      data: { agentId: n.agentId, nodeId: n.id, onAddChild: handleAddChild },
    };
  }

  const sfNodes = writable<Node[]>(get(flowNodes).map(toSfNode));
  const sfEdges = writable<Edge[]>(
    get(flowEdges).map((e) => ({ id: e.id, source: e.source, target: e.target })),
  );

  function reloadFlow() {
    sfNodes.set(get(flowNodes).map(toSfNode));
    sfEdges.set(
      get(flowEdges).map((e) => ({ id: e.id, source: e.source, target: e.target })),
    );
  }

  function handleConnect(conn: Connection) {
    if (!conn.source || !conn.target) return;
    addFlowEdge(conn.source, conn.target);
    const id = `edge-${conn.source}-${conn.target}`;
    sfEdges.update((list) =>
      list.some((e) => e.source === conn.source && e.target === conn.target)
        ? list
        : [...list, { id, source: conn.source as string, target: conn.target as string }],
    );
  }

  function persistNodePositions() {
    for (const n of get(sfNodes)) moveFlowNode(n.id, n.position.x, n.position.y);
  }

  let addMenuOpen = $state(false);

  function addNode(agentId: string) {
    addFlowNode(agentId);
    reloadFlow();
    addMenuOpen = false;
  }

  // ---------- Flow: node context menu (right-click) ----------
  let ctxMenu = $state<{ x: number; y: number; nodeId: string } | null>(null);

  function onNodeContextMenu({ node, event }: { node: Node; event: MouseEvent }) {
    event.preventDefault();
    ctxMenu = {
      x: Math.min(event.clientX, window.innerWidth - 240),
      y: Math.min(event.clientY, window.innerHeight - 160),
      nodeId: node.id,
    };
  }

  function closeCtx() {
    ctxMenu = null;
  }

  function ctxDisconnect() {
    if (ctxMenu) {
      disconnectFlowNode(ctxMenu.nodeId);
      reloadFlow();
    }
    ctxMenu = null;
  }

  function ctxDelete() {
    if (ctxMenu) {
      removeFlowNode(ctxMenu.nodeId);
      reloadFlow();
    }
    ctxMenu = null;
  }

  // ---------- Flow: projects ----------
  let projDialogOpen = $state(false);
  let projName = $state("");

  function doSwitchProject(id: string) {
    switchFlowProject(id);
    reloadFlow();
  }

  function createProject() {
    const name = projName.trim();
    if (!name) return;
    addFlowProject(name);
    reloadFlow();
    projName = "";
    projDialogOpen = false;
  }
</script>

<div class="tasks">
  <div class="tasks-topbar">
    <h2>{$t("tasks.title")}</h2>
    <div class="seg" role="tablist" aria-label={$t("tasks.title")}>
      <button
        role="tab"
        aria-selected={$tasksView === "kanban"}
        class:active={$tasksView === "kanban"}
        onclick={() => setView("kanban")}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="3" y="4" width="7" height="16" rx="1.5" />
          <rect x="14" y="4" width="7" height="10" rx="1.5" />
        </svg>
        {$t("tasks.kanban")}
      </button>
      <button
        role="tab"
        aria-selected={$tasksView === "flow"}
        class:active={$tasksView === "flow"}
        onclick={() => setView("flow")}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="6" cy="6" r="2.5" />
          <circle cx="18" cy="6" r="2.5" />
          <circle cx="12" cy="18" r="2.5" />
          <path d="M8 7.5 10.5 15.5M16 7.5 13.5 15.5M8.5 6h7" />
        </svg>
        {$t("tasks.flow")}
      </button>
    </div>
  </div>

  {#if $tasksView === "kanban"}
    <div class="filterbar" role="group" aria-label={$t("tasks.filterBar")}>
      <label>
        <span>{$t("tasks.filterProject")}</span>
        <select bind:value={$kanbanFilterProject}>
          <option value="all">{$t("tasks.allProjects")}</option>
          {#each $projects as p (p.id)}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </label>
      <label>
        <span>{$t("tasks.agent")}</span>
        <select bind:value={$kanbanFilterAgent}>
          <option value="all">{$t("tasks.allAgents")}</option>
          {#each allAgents as a (a.id)}
            <option value={a.id}>{a.name}</option>
          {/each}
        </select>
      </label>
      <label>
        <span>{$t("tasks.filterPeriod")}</span>
        <select bind:value={$kanbanFilterPeriod}>
          <option value="all">{$t("tasks.period.all")}</option>
          <option value="7">{$t("tasks.period.7")}</option>
          <option value="30">{$t("tasks.period.30")}</option>
          <option value="90">{$t("tasks.period.90")}</option>
          <option value="365">{$t("tasks.period.365")}</option>
        </select>
      </label>
    </div>
    <div class="board">
      {#each cols as col (col)}
        {@const cards = filteredCards.filter((c) => c.col === col)}
        <section
          class="col"
          ondragover={(e) => e.preventDefault()}
          ondrop={(e) => onDropCol(e, col)}
          aria-label={$t(`tasks.col.${col}`)}
        >
          <header class="col-head">
            <h3>{$t(`tasks.col.${col}`)}</h3>
            <span class="count">{cards.length}</span>
          </header>
          <div class="cards">
            {#each cards as card (card.id)}
              {@const a = agentOf(card.agentId)}
              {@const st = $agentStatus[a.id] ?? "idle"}
              <article
                class="card"
                draggable="true"
                ondragstart={(e) => onDragStart(e, card.id)}
              >
                <p class="card-title">{card.title}</p>
                <div class="card-agent">
                  <span class="badge" style:background={a.color}>{a.badge}</span>
                  <span class="name">{a.name}</span>
                  <span class="dot {st}" aria-hidden="true"></span>
                </div>
                <span class="card-date">{card.createdAt}</span>
              </article>
            {/each}
          </div>
          {#if addingCol === col}
            <div class="add-form">
              <label>
                <span>{$t("tasks.cardTitle")}</span>
                <input
                  type="text"
                  bind:value={draftTitle}
                  placeholder={$t("tasks.cardTitlePh")}
                  maxlength="120"
                />
              </label>
              <label>
                <span>{$t("tasks.agent")}</span>
                <select bind:value={draftAgent}>
                  {#each allAgents as a (a.id)}
                    <option value={a.id}>{a.name}</option>
                  {/each}
                </select>
              </label>
              <label>
                <span>{$t("tasks.filterProject")}</span>
                <select bind:value={draftProject}>
                  {#each $projects as p (p.id)}
                    <option value={p.id}>{p.name}</option>
                  {/each}
                </select>
              </label>
              <div class="form-actions">
                <button class="btn-primary" onclick={submitForm} disabled={!draftTitle.trim()}>
                  {$t("tasks.add")}
                </button>
                <button class="btn-ghost" onclick={() => (addingCol = null)}>
                  {$t("tasks.cancel")}
                </button>
              </div>
            </div>
          {:else}
            <button class="btn-ghost add-btn" onclick={() => openForm(col)}>
              {$t("tasks.addTask")}
            </button>
          {/if}
        </section>
      {/each}
    </div>
  {:else}
    <div class="flow-bar">
      <div class="proj-tabs" role="tablist" aria-label={$t("tasks.flow.projects")}>
        {#each $projects as p (p.id)}
          <button
            role="tab"
            aria-selected={$activeFlowProjectId === p.id}
            class:active={$activeFlowProjectId === p.id}
            onclick={() => doSwitchProject(p.id)}
          >
            {p.name}
          </button>
        {/each}
        <button class="btn-ghost add-proj" onclick={() => (projDialogOpen = true)}>
          {$t("tasks.flow.addProject")}
        </button>
      </div>
      {#if projDialogOpen}
        <div class="proj-dialog" role="dialog" aria-label={$t("tasks.flow.newProject")}>
          <label>
            <span>{$t("tasks.flow.projectName")}</span>
            <input
              type="text"
              bind:value={projName}
              placeholder={$t("tasks.flow.projectNamePh")}
              maxlength="60"
            />
          </label>
          <div class="form-actions">
            <button class="btn-primary" onclick={createProject} disabled={!projName.trim()}>
              {$t("tasks.flow.create")}
            </button>
            <button class="btn-ghost" onclick={() => (projDialogOpen = false)}>
              {$t("tasks.cancel")}
            </button>
          </div>
        </div>
      {/if}
    </div>
    <div class="flow-wrap">
      <SvelteFlow
        nodes={sfNodes}
        edges={sfEdges}
        {nodeTypes}
        onconnect={handleConnect}
        onnodedragstop={persistNodePositions}
        onnodecontextmenu={onNodeContextMenu}
        colorMode="dark"
        fitView
      >
        <Background variant={BackgroundVariant.Dots} gap={24} size={1.5} />
        <Controls />
        <MiniMap pannable zoomable />
      </SvelteFlow>
      <div class="flow-add">
        <button
          class="btn-primary add-node"
          onclick={() => (addMenuOpen = !addMenuOpen)}
          aria-expanded={addMenuOpen}
          aria-label={$t("tasks.addNode")}
        >
          +
        </button>
        {#if addMenuOpen}
          <div class="add-menu" role="menu" aria-label={$t("tasks.addNode")}>
            {#each allAgents as a (a.id)}
              <button role="menuitem" onclick={() => addNode(a.id)}>
                <span class="badge" style:background={a.color}>{a.badge}</span>
                <span class="name">{a.name}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
      {#if ctxMenu}
        <div
          class="ctx-backdrop"
          onclick={closeCtx}
          onkeydown={(e) => e.key === "Escape" && closeCtx()}
          role="presentation"
        ></div>
        <div
          class="ctx-menu"
          role="menu"
          style:left={`${ctxMenu.x}px`}
          style:top={`${ctxMenu.y}px`}
        >
          <button role="menuitem" onclick={ctxDisconnect}>
            {$t("tasks.flow.disconnect")}
          </button>
          <button role="menuitem" class="danger" onclick={ctxDelete}>
            {$t("tasks.flow.deleteNode")}
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tasks {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 20px 24px;
    gap: 16px;
  }
  .tasks-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex: none;
  }
  .tasks-topbar h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
  }
  .seg button svg {
    width: 16px;
    height: 16px;
    flex: none;
  }
  .seg button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  /* ---------- Kanban ---------- */
  .filterbar {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    flex: none;
  }
  .filterbar label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
  }
  .filterbar select {
    min-height: 44px;
    border-radius: 10px;
    border: 1px solid var(--border, #2a2e36);
    background: var(--panel, #121419);
    color: var(--text, #e8eaed);
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
    max-width: 220px;
  }
  .board {
    display: grid;
    grid-template-columns: repeat(5, minmax(230px, 1fr));
    gap: 16px;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .col {
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--panel, #121419);
    border: 1px solid var(--border, #262a32);
    border-radius: 14px;
    padding: 12px;
    gap: 10px;
  }
  .col-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 4px;
  }
  .col-head h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }
  .count {
    min-width: 28px;
    height: 28px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    background: var(--border, #262a32);
    font-size: 13px;
    font-weight: 600;
  }
  .cards {
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
    min-height: 60px;
  }
  .card {
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    padding: 12px;
    cursor: grab;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card:active { cursor: grabbing; }
  .card-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    line-height: 1.4;
  }
  .card-agent {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .badge {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 13px;
    color: #fff;
    flex: none;
  }
  .card-agent .name {
    font-size: 13px;
    color: var(--muted, #9aa0aa);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex: none;
  }
  .dot.idle { background: #22c55e; }
  .dot.working { background: #eab308; }
  .dot.attention { background: #ef4444; }
  .card-date {
    font-size: 12px;
    color: var(--muted, #9aa0aa);
  }
  .add-btn { align-self: stretch; }
  .add-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--card, #16181d);
    border: 1px dashed var(--border, #2a2e36);
    border-radius: 12px;
    padding: 12px;
  }
  .add-form label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
  }
  .add-form input,
  .add-form select {
    min-height: 44px;
    border-radius: 10px;
    border: 1px solid var(--border, #2a2e36);
    background: var(--bg, #0b0d10);
    color: var(--text, #e8eaed);
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  .form-actions {
    display: flex;
    gap: 8px;
  }
  .form-actions .btn-primary,
  .form-actions .btn-ghost {
    flex: 1;
  }

  /* ---------- Flow canvas ---------- */
  .flow-bar {
    position: relative;
    flex: none;
  }
  .proj-tabs {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow-x: auto;
    padding-bottom: 2px;
  }
  .proj-tabs [role="tab"] {
    flex: none;
    min-height: 44px;
    padding: 0 16px;
    border-radius: 10px;
    border: 1px solid var(--border, #2a2e36);
    background: var(--panel, #121419);
    color: var(--muted, #9aa0aa);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }
  .proj-tabs [role="tab"].active {
    background: var(--card, #16181d);
    color: var(--text, #e8eaed);
    border-color: var(--accent, #10b981);
  }
  .add-proj {
    flex: none;
  }
  .proj-dialog {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 30;
    width: 280px;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }
  .proj-dialog label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
  }
  .proj-dialog input {
    min-height: 44px;
    border-radius: 10px;
    border: 1px solid var(--border, #2a2e36);
    background: var(--bg, #0b0d10);
    color: var(--text, #e8eaed);
    padding: 0 12px;
    font-size: 14px;
    font-family: inherit;
  }
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
  }
  .ctx-menu {
    position: fixed;
    z-index: 41;
    min-width: 200px;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }
  .ctx-menu button {
    min-height: 44px;
    padding: 6px 12px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text, #e8eaed);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }
  .ctx-menu button:hover {
    background: var(--border, #262a32);
  }
  .ctx-menu button.danger {
    color: #f87171;
  }
  .flow-wrap {
    position: relative;
    flex: 1;
    min-height: 0;
    border: 1px solid var(--border, #262a32);
    border-radius: 14px;
    overflow: hidden;
    background: #0b0d10;
  }
  .flow-wrap :global(.svelte-flow) {
    background: #0b0d10;
  }
  .flow-wrap :global(.svelte-flow__controls) {
    border: 1px solid var(--border, #2a2e36);
    border-radius: 10px;
    overflow: hidden;
  }
  .flow-wrap :global(.svelte-flow__controls-button) {
    background: var(--card, #16181d);
    border-bottom: 1px solid var(--border, #2a2e36);
    width: 36px;
    height: 36px;
  }
  .flow-wrap :global(.svelte-flow__controls-button svg) {
    fill: var(--text, #e8eaed);
  }
  .flow-wrap :global(.svelte-flow__minimap) {
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 10px;
    overflow: hidden;
  }
  .flow-wrap :global(.svelte-flow__edge path) {
    stroke: var(--accent, #10b981);
    stroke-width: 2;
  }
  .flow-wrap :global(.svelte-flow__connection-path) {
    stroke: var(--accent, #10b981);
    stroke-width: 2;
  }
  .flow-add {
    position: absolute;
    top: 12px;
    right: 12px;
    z-index: 10;
  }
  .add-node {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    font-size: 24px;
    line-height: 1;
    padding: 0;
  }
  .add-menu {
    position: absolute;
    top: 52px;
    right: 0;
    min-width: 220px;
    background: var(--card, #16181d);
    border: 1px solid var(--border, #2a2e36);
    border-radius: 12px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
  }
  .add-menu button {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    padding: 6px 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text, #e8eaed);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }
  .add-menu button:hover {
    background: var(--border, #262a32);
  }
  .add-menu .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

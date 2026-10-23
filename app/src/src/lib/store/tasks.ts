import { writable, get, derived, type Writable, type Readable } from "svelte/store";
import { loadRecord, persistRecord, loadString, loadBool, loadJSON } from "./_helpers";
import type {
  Agent, AgentFile, AgentStatus, CostRow, FlowEdge, FlowNode, FlowProject,
  KanbanCard, KanbanCol, LibraryItem, Message, ProviderInfo, ScheduledItem,
  ToolInfo, LibraryType,
} from "../types";
import { invoke } from "@tauri-apps/api/core";
import { setAgentEnabled } from "./agents";

export type TasksViewKind = "kanban" | "flow" | "brief";

export const tasksView = writable<TasksViewKind>("kanban");

/** Small helper: load/persist a JSON list in localStorage. */
function loadList<T>(key: string, seed: () => T[]): T[] {
  try {
    const raw = localStorage.getItem(key);
    if (raw) {
      const parsed = JSON.parse(raw);
      // Defensive: corrupt data may be {} instead of [].
      if (Array.isArray(parsed)) return parsed as T[];
    }
  } catch {
    /* fall through to seed */
  }
  return seed();
}

function persistList<T>(key: string, store: Writable<T[]>) {
  store.subscribe((v) => {
    try {
      localStorage.setItem(key, JSON.stringify(v));
    } catch {
      /* ignore */
    }
  });
}

let kanbanSeq = 1;

/** Kanban seed. Empty by default (Patria 20 Okt): no dummy cards — the user
    creates real cards via Brief "Jadikan tugas" or the Kanban add button. */
function seedKanban(): KanbanCard[] {
  return [];
}

/** Backfill new card fields for cards persisted before they existed. */
function normalizeKanban(list: KanbanCard[]): KanbanCard[] {
  const now = Date.now();
  return list.map((c) => ({
    ...c,
    projectId: c.projectId ?? "proj-a",
    createdAtTs: c.createdAtTs ?? now,
  }));
}

/** Kanban cards (persisted). Created by the user via Brief or the board. */
export const kanbanCards = writable<KanbanCard[]>(
  normalizeKanban(loadList("minime.kanban", seedKanban)),
);
persistList("minime.kanban", kanbanCards);
// Continue the id sequence past any persisted cards.
kanbanSeq =
  get(kanbanCards).reduce((m, c) => Math.max(m, c.id), 0) + 1;

export function addKanbanCard(
  title: string,
  agentId: string,
  col: KanbanCol,
  projectId: string,
) {
  const now = new Date();
  kanbanCards.update((list) => [
    ...list,
    {
      id: kanbanSeq++,
      title,
      agentId,
      col,
      projectId,
      createdAt: now.toLocaleDateString("id-ID", { day: "numeric", month: "short" }),
      createdAtTs: now.getTime(),
    },
  ]);
}

export function moveKanbanCard(id: number, col: KanbanCol) {
  let cardTitle = "";
  kanbanCards.update((list) =>
    list.map((c) => {
      if (c.id === id) {
        cardTitle = c.title;
        return { ...c, col };
      }
      return c;
    }),
  );
  // Native notification when a card needs approval or is finished (Phase 3.2),
  // gated by the user's notification preferences (Settings > Notifications).
  const prefs = get(settingsPrefs);
  if (col === "approval" && prefs.notifApproval) {
    void sendNotification("Butuh approval", `"${cardTitle}" menunggu persetujuanmu`);
  } else if (col === "done" && prefs.notifDone) {
    void sendNotification("Tugas selesai", `"${cardTitle}" selesai`);
  }
}

/** Approval decisions from the Kanban approval column (Phase 3.2):
    approve -> done, revise -> revision, reject -> back to queue. */
export function decideKanbanCard(id: number, decision: "approve" | "revise" | "reject") {
  const col: KanbanCol =
    decision === "approve" ? "done" : decision === "revise" ? "revision" : "queue";
  moveKanbanCard(id, col);
}

export interface ToolRunArgs {
  agentId: string;
  toolId: string;
  action: string;
  args: Record<string, string>;
}

/** Tool ids with a real backend implementation, for one agent. */
export async function allowedRuntimeTools(agentId: string): Promise<string[]> {
  const list = await tauriInvoke<string[]>("runtime_allowed_tools", { agentId });
  return list ?? [];
}

function toolErrorLabel(code: string): string {
  const known = [
    "not_allowed",
    "not_connected",
    "not_implemented",
    "unknown_tool",
    "agent_inactive",
  ] as const;
  return (known as readonly string[]).includes(code) ? code : "error";
}

export async function runToolTask(
  cardId: number,
  run: ToolRunArgs,
  opts: { title?: string } = {},
): Promise<{ ok: boolean; error?: string }> {
  // Patria 20 Okt: a deactivated agent keeps its cards, but its function is
  // off — tool runs are refused at the store level (not just hidden in UI).
  if (!isAgentEnabled(run.agentId)) {
    return { ok: false, error: "agent_inactive" };
  }
  moveKanbanCard(cardId, "doing");
  try {
    const res = await tauriInvoke<{ ok: boolean; summary: string }>("tool_run", {
      agentId: run.agentId,
      toolId: run.toolId,
      action: run.action,
      args: run.args,
    });
    if (!res || !res.ok) {
      // Not running inside Tauri (e.g. web preview) — card returns to queue.
      moveKanbanCard(cardId, "queue");
      return { ok: false, error: "error" };
    }
    addLibraryItem({
      title: opts.title ?? res.summary,
      summary: res.summary,
      type: "doc",
      agentId: run.agentId,
    });
    moveKanbanCard(cardId, "approval");
    void sendNotification(
      "Tool selesai",
      `Hasil dari ${run.toolId} menunggu persetujuanmu`,
    );
    return { ok: true };
  } catch (e) {
    const code = toolErrorLabel(e instanceof Error ? e.message : String(e));
    moveKanbanCard(cardId, "queue");
    void sendNotification("Tool gagal", `Tool ${run.toolId}: ${code}`);
    return { ok: false, error: code };
  }
}

/** Turn a Brief todo into a real Kanban task card (Tahap 3.6 wiring). */
export function briefTodoToCard(todoId: string): void {
  const todos = get(projectTodos);
  const td = todos.find((t) => t.id === todoId);
  if (!td || !td.text.trim()) return;
  addKanbanCard(td.text.trim(), td.agentId, "queue", td.projectId);
}

/* ---------- Social drafts (Phase 3.3: Meta/Postiz) ----------
   Default mode is DRAFT (Patria's rule): content waits in the approval
   column. The user may explicitly switch a platform to AUTOPOST in the
   publish dialog — at their own risk — and drafts for that platform then
   skip approval. The choice is per platform and persisted. */

export type SocialMode = "draft" | "autopost";

export const socialMode = writable<Record<"meta" | "postiz", SocialMode>>(
  loadRecord<SocialMode>("minime.social-mode"),
);
persistRecord("minime.social-mode", socialMode);

export function setSocialMode(platform: "meta" | "postiz", mode: SocialMode): void {
  socialMode.update((m) => ({ ...m, [platform]: mode }));
}

/* ---------- Autopost consent gate (Patria 21 Okt) ----------
   Autopost mode requires explicit first-time consent per platform.
   The dialog shows consequences; only a manual confirmation enables it. */
export const autopostConsent = writable<Record<"meta" | "postiz", boolean>>(
  loadRecord<boolean>("minime.autopost-consent"),
);
persistRecord("minime.autopost-consent", autopostConsent);

/** Platform awaiting consent dialog, null when no dialog open. */
export const autopostPending = writable<"meta" | "postiz" | null>(null);

export function hasAutopostConsent(platform: "meta" | "postiz"): boolean {
  return get(autopostConsent)[platform] === true;
}

/** Called when the user picks "autopost" in the UI. Opens the consent
    dialog on first use; enables directly when already consented. */
export function requestAutopost(platform: "meta" | "postiz"): void {
  if (hasAutopostConsent(platform)) {
    setSocialMode(platform, "autopost");
  } else {
    autopostPending.set(platform);
  }
}

/** User confirmed the consequences — record consent and enable. */
export function confirmAutopost(): void {
  const p = get(autopostPending);
  if (p) {
    autopostConsent.update((c) => ({ ...c, [p]: true }));
    setSocialMode(p, "autopost");
  }
  autopostPending.set(null);
}

/** User declined — stay on draft. */
export function cancelAutopost(): void {
  autopostPending.set(null);
}

export function createSocialDraft(
  title: string,
  content: string,
  platform: "meta" | "postiz",
  agentId: string,
  projectId: string,
): void {
  const mode = get(socialMode)[platform] ?? "draft";
  const auto = mode === "autopost";
  const now = new Date();
  kanbanCards.update((list) => [
    ...list,
    {
      id: kanbanSeq++,
      title,
      agentId,
      col: (auto ? "done" : "approval") as KanbanCol,
      projectId,
      createdAt: now.toLocaleDateString("id-ID", { day: "numeric", month: "short" }),
      createdAtTs: now.getTime(),
      draftContent: content,
      draftPlatform: platform,
      autoPosted: auto,
    },
  ]);
  const platformName = platform === "meta" ? "Meta" : "Postiz";
  const prefs2 = get(settingsPrefs);
  if (auto) {
    if (prefs2.notifDone) {
      void sendNotification(
        "Terposting otomatis",
        `"${title}" langsung diposting ke ${platformName} (mode autoposting aktif)`,
      );
    }
  } else if (prefs2.notifApproval) {
    void sendNotification(
      "Draft siap direview",
      `"${title}" menunggu approval sebelum diposting ke ${platformName}`,
    );
  }
}

/** Kanban filter state (persisted). "all" = no filter. */
export const kanbanFilterProject = writable<string>(
  loadString("minime.kanban-fproj", "all"),
);
kanbanFilterProject.subscribe((v) => {
  try {
    localStorage.setItem("minime.kanban-fproj", v);
  } catch {
    /* ignore */
  }
});
export const kanbanFilterAgent = writable<string>(
  loadString("minime.kanban-fagent", "all"),
);
kanbanFilterAgent.subscribe((v) => {
  try {
    localStorage.setItem("minime.kanban-fagent", v);
  } catch {
    /* ignore */
  }
});
/** "all" | "7" | "30" | "90" | "365" (days). */
export const kanbanFilterPeriod = writable<string>(
  loadString("minime.kanban-fperiod", "all"),
);
kanbanFilterPeriod.subscribe((v) => {
  try {
    localStorage.setItem("minime.kanban-fperiod", v);
  } catch {
    /* ignore */
  }
});

let flowSeq = 1;

/** Nodes + edges of one project on the connection canvas (mock). */
interface FlowProjectCanvas {
  nodes: FlowNode[];
  edges: FlowEdge[];
}

function loadFlowMap(): Record<string, FlowProjectCanvas> {
  try {
    const raw = localStorage.getItem("minime.flow-data");
    if (raw) {
      const parsed = JSON.parse(raw) as Record<string, FlowProjectCanvas>;
      if (parsed && typeof parsed === "object") {
        // Self-heal: drop corrupt project entries (nodes/edges must be arrays).
        let healed = false;
        for (const [pid, canvas] of Object.entries(parsed)) {
          if (
            !canvas ||
            typeof canvas !== "object" ||
            !Array.isArray((canvas as FlowProjectCanvas).nodes) ||
            !Array.isArray((canvas as FlowProjectCanvas).edges)
          ) {
            delete parsed[pid];
            healed = true;
          }
        }
        if (healed) {
          try {
            localStorage.setItem("minime.flow-data", JSON.stringify(parsed));
          } catch {
            /* ignore */
          }
        }
        return parsed;
      }
    }
  } catch {
    /* fall through to seed */
  }
  return {};
}

const flowMap: Record<string, FlowProjectCanvas> = loadFlowMap();

function saveFlowMap(): void {
  try {
    localStorage.setItem("minime.flow-data", JSON.stringify(flowMap));
  } catch {
    /* ignore */
  }
}

/** All projects, shared by the Connection canvas, the Kanban filter, and
    the Agents-view folders (the folder IS the project — Patria 20 Okt).
    Empty by default: dummy projects removed; the user creates the first
    real project themselves. */
export const projects = writable<FlowProject[]>(
  loadList<FlowProject>("minime.projects", () => []).map((p) => ({
    ...p,
    // Patria 20 Okt: Mimi is the main agent — she delegates, never joins
    // folders. Strip any legacy "mimi" roster entries on load.
    agentIds: (Array.isArray(p.agentIds) ? p.agentIds : []).filter(
      (id) => id !== "mimi",
    ),
  })),
);
persistList("minime.projects", projects);

/** Rename a project (also renames its Agents-view folder). */
export function renameProject(id: string, name: string): void {
  const n = name.trim();
  if (!n) return;
  projects.update((list) => list.map((p) => (p.id === id ? { ...p, name: n } : p)));
}

/** Agents assigned to a project = agents in its folder (Patria 20 Okt: one
    agent may serve several folders, but never twice in one folder). Mimi
    never joins folders — she is the main agent and delegates to the team. */
/** Add an agent id to a folder's roster (dedup per folder). No canvas
 *  side-effects — callers decide about regen. */
function folderAddAgent(projectId: string, agentId: string): void {
  // Patria 20 Okt: Mimi is the main agent — she delegates, never joins a folder.
  if (agentId === "mimi") return;
  projects.update((list) =>
    list.map((p) =>
      p.id === projectId && !(p.agentIds ?? []).includes(agentId)
        ? { ...p, agentIds: [...(p.agentIds ?? []), agentId] }
        : p,
    ),
  );
}

export function addAgentToProject(projectId: string, agentId: string): void {
  // Patria 20 Okt: the same agent MAY serve several folders/projects, but it
  // cannot appear twice inside one folder — dedup is per-folder only.
  const wasAuto = isCanvasAuto(projectId);
  folderAddAgent(projectId, agentId);
  // An agent plotted for a project is meant to work: make it active.
  setAgentEnabled(agentId, true);
  if (wasAuto) regenChain(projectId);
}

/** Sync an agent node added on the Koneksi canvas back to the folder roster
 *  (Patria 20 Okt): the folder appears with the project's agents. No chain
 *  regen — the canvas was just edited manually. */
export function canvasAddAgent(projectId: string, agentId: string): void {
  folderAddAgent(projectId, agentId);
  setAgentEnabled(agentId, true);
}

/** Reorder an agent inside a folder (Patria 20 Okt): folder order = task
 *  order reflected by the auto-generated Koneksi chain. */
export function moveAgentInProject(
  projectId: string,
  agentId: string,
  dir: -1 | 1,
): void {
  const wasAuto = isCanvasAuto(projectId);
  projects.update((list) =>
    list.map((p) => {
      if (p.id !== projectId) return p;
      const ids = [...(p.agentIds ?? [])];
      const i = ids.indexOf(agentId);
      const j = i + dir;
      if (i < 0 || j < 0 || j >= ids.length) return p;
      [ids[i], ids[j]] = [ids[j], ids[i]];
      return { ...p, agentIds: ids };
    }),
  );
  if (wasAuto) regenChain(projectId);
}

export function removeAgentFromProject(projectId: string, agentId: string): void {
  const wasAuto = isCanvasAuto(projectId);
  projects.update((list) =>
    list.map((p) =>
      p.id === projectId
        ? { ...p, agentIds: (p.agentIds ?? []).filter((x) => x !== agentId) }
        : p,
    ),
  );
  if (wasAuto) regenChain(projectId);
}

/** Agent ids assigned to a project (empty when the project has no folder team). */
export function projectAgentIds(projectId: string): string[] {
  return get(projects).find((p) => p.id === projectId)?.agentIds ?? [];
}


/** Project briefs + todos (workflow Patria 20 Okt). Brief awal dibuat user
    di Mimi; Mimi auto-delegasi menjadi todo per agen. */
export const projectBriefs = writable<Record<string, import("./types").ProjectBrief>>(
  loadRecord<import("./types").ProjectBrief>("minime.project-briefs"),
);
persistRecord("minime.project-briefs", projectBriefs);

export const projectTodos = writable<import("./types").ProjectTodo[]>(
  loadList("minime.project-todos", () => []),
);
persistList("minime.project-todos", projectTodos);

let todoSeq = 1;

export function getProjectBrief(projectId: string): import("./types").ProjectBrief {
  const rec = get(projectBriefs);
  return rec[projectId] ?? { projectId, brief: "", goals: "", initiated: false };
}

export function setProjectBrief(projectId: string, brief: string, goals: string): void {
  projectBriefs.update((m) => ({
    ...m,
    [projectId]: { ...(m[projectId] ?? { projectId, brief: "", goals: "", initiated: false }), brief, goals },
  }));
}

export function addProjectTodo(projectId: string, text: string, agentId: string): void {
  const t = text.trim();
  if (!t) return;
  projectTodos.update((list) => [
    ...list,
    { id: `todo-${todoSeq++}`, projectId, text: t, agentId, done: false },
  ]);
}

export function toggleProjectTodo(id: string): void {
  projectTodos.update((list) =>
    list.map((t) => (t.id === id ? { ...t, done: !t.done } : t)),
  );
}

export function removeProjectTodo(id: string): void {
  projectTodos.update((list) => list.filter((t) => t.id !== id));
}

/** Auto-delegation: Mimi turns the brief into per-agent todos + activates
    the assigned agents, then marks the project initiated. */
export function initiateProject(projectId: string, assignments: { agentId: string; task: string }[]): void {
  for (const a of assignments) {
    addProjectTodo(projectId, a.task, a.agentId);
    setAgentEnabled(a.agentId, true);
  }
  projectBriefs.update((m) => ({
    ...m,
    [projectId]: { ...(m[projectId] ?? { projectId, brief: "", goals: "", initiated: false }), initiated: true },
  }));
}

/** Active project on the Connection canvas. */
export const activeFlowProjectId = writable<string>(
  loadString("minime.flow-project-active", ""),
);
activeFlowProjectId.subscribe((v) => {
  try {
    localStorage.setItem("minime.flow-project-active", v);
  } catch {
    /* ignore */
  }
});

function seedFlowMap(): void {
  if (Object.keys(flowMap).length > 0) return;
  // Migrate the old flat keys (pre-project) into Project A when present.
  let aNodes: FlowNode[] | null = null;
  let aEdges: FlowEdge[] | null = null;
  try {
    const n = localStorage.getItem("minime.flow-nodes");
    const e = localStorage.getItem("minime.flow-edges");
    if (n) {
      const pn = JSON.parse(n);
      if (Array.isArray(pn)) aNodes = pn as FlowNode[];
    }
    if (e) {
      const pe = JSON.parse(e);
      if (Array.isArray(pe)) aEdges = pe as FlowEdge[];
    }
  } catch {
    /* ignore */
  }
  flowMap["proj-a"] = {
    nodes: aNodes ?? [
      { id: "node-1", agentId: "mimi", x: 80, y: 120 },
      { id: "node-2", agentId: "kreatif", x: 400, y: 120 },
    ],
    edges: aEdges ?? [{ id: "edge-1", source: "node-1", target: "node-2" }],
  };
  flowMap["proj-b"] = {
    nodes: [{ id: "node-b1", agentId: "mimi", x: 80, y: 120 }],
    edges: [],
  };
  saveFlowMap();
}
seedFlowMap();

/**
 * Nodes/edges of the active project. These are buffers: every change is
 * written back into the per-project map (and localStorage) automatically.
 */
export const flowNodes = writable<FlowNode[]>([]);
export const flowEdges = writable<FlowEdge[]>([]);

function resolveProjectId(): string {
  const list = get(projects);
  const id = get(activeFlowProjectId);
  if (list.some((p) => p.id === id)) return id;
  return list[0]?.id ?? "proj-a";
}

/** Fingerprint of the pre-Mimi auto format (Patria 20 Okt, superseded):
 *  linear chain, folder order, x = 80+i*320, y = 140, sequential edges.
 *  Exact match ⇒ auto-generated, safe to migrate to the Mimi-parent star. */
function isLegacyLinearAuto(
  nodes: FlowNode[],
  edges: FlowEdge[],
  ids: string[],
): boolean {
  if (nodes.length !== ids.length || nodes.length === 0) return false;
  for (let i = 0; i < nodes.length; i++) {
    if (nodes[i].agentId !== ids[i]) return false;
    if (nodes[i].x !== 80 + i * 320 || nodes[i].y !== 140) return false;
  }
  if (edges.length !== nodes.length - 1) return false;
  for (let i = 0; i + 1 < nodes.length; i++) {
    const e = edges[i];
    if (e.source !== nodes[i].id || e.target !== nodes[i + 1].id) return false;
  }
  return true;
}

function loadActiveCanvas(): void {
  // Patria 20 Okt: with no projects, the Connection view shows its default —
  // an empty canvas (no project name) with just Mimi as parent.
  if (get(projects).length === 0) {
    const { nodes, edges } = buildChain([]);
    flowNodes.set(nodes);
    flowEdges.set(edges);
    return;
  }
  const pid = resolveProjectId();
  const d = flowMap[pid];
  // Defensive: old/corrupt localStorage may hold {} without nodes/edges.
  const nodes = Array.isArray(d?.nodes) ? d.nodes : [];
  const edges = Array.isArray(d?.edges) ? d.edges : [];
  flowNodes.set(nodes);
  flowEdges.set(edges);
  // Patria 20 Okt — Folder → Koneksi: an empty canvas auto-generates Mimi
  // as the parent node; folder agents appear as her team (star). Legacy
  // linear auto-canvases (pre-Mimi) migrate to the star exactly once.
  const ids = get(projects).find((p) => p.id === pid)?.agentIds ?? [];
  if (nodes.length === 0 || isLegacyLinearAuto(nodes, edges, ids)) {
    const { nodes: cn, edges: ce } = buildChain(ids);
    flowNodes.set(cn);
    flowEdges.set(ce);
  }
}

function saveActiveCanvas(): void {
  // No project to save to (Patria 20 Okt: default empty canvas) — skip.
  if (get(projects).length === 0) return;
  flowMap[resolveProjectId()] = {
    nodes: get(flowNodes),
    edges: get(flowEdges),
  };
  saveFlowMap();
}

flowNodes.subscribe(() => saveActiveCanvas());
flowEdges.subscribe(() => saveActiveCanvas());
loadActiveCanvas();

/** Build the auto canvas (Patria 20 Okt): Mimi is the main agent — she never
 *  joins folders; she delegates. She is auto-placed as the FIRST node
 *  (parent), with one edge to every folder agent (the team). */
function buildChain(ids: string[]): { nodes: FlowNode[]; edges: FlowEdge[] } {
  const mimi: FlowNode = {
    id: `node-${flowSeq++}-${Date.now() % 100000}`,
    agentId: "mimi",
    x: Math.round(80 + Math.max(0, ids.length - 1) * 160),
    y: 60,
  };
  const nodes: FlowNode[] = [mimi];
  const edges: FlowEdge[] = [];
  ids.forEach((agentId, i) => {
    const n: FlowNode = {
      id: `node-${flowSeq++}-${Date.now() % 100000}`,
      agentId,
      x: 80 + i * 320,
      y: 300,
    };
    nodes.push(n);
    edges.push({
      id: `edge-${mimi.id}-${n.id}`,
      source: mimi.id,
      target: n.id,
    });
  });
  return { nodes, edges };
}

/** True when the canvas is a pure reflection of the project (Patria 20 Okt):
 *  Mimi as parent (exactly once) + every folder agent exactly once, with
 *  star edges Mimi → team. Any manual restructuring breaks auto status. */
export function isCanvasAuto(pid: string): boolean {
  const ids = get(projects).find((p) => p.id === pid)?.agentIds ?? [];
  const active = pid === get(activeFlowProjectId);
  const nodes = active ? get(flowNodes) : (flowMap[pid]?.nodes ?? []);
  const edges = active ? get(flowEdges) : (flowMap[pid]?.edges ?? []);
  if (nodes.length !== ids.length + 1) return false;
  const counts = new Map<string, number>();
  for (const n of nodes) counts.set(n.agentId, (counts.get(n.agentId) ?? 0) + 1);
  if (counts.get("mimi") !== 1) return false;
  if (!ids.every((id) => counts.get(id) === 1)) return false;
  const mimiNode = nodes.find((n) => n.agentId === "mimi");
  const teamNodeIds = new Set(
    nodes.filter((n) => n.agentId !== "mimi").map((n) => n.id),
  );
  if (!mimiNode || edges.length !== ids.length) return false;
  return edges.every(
    (e) => e.source === mimiNode.id && teamNodeIds.has(e.target),
  );
}

/** Regenerate the auto-chain for a project from its current folder order. */
export function regenChain(pid: string): void {
  const ids = get(projects).find((p) => p.id === pid)?.agentIds ?? [];
  const { nodes, edges } = buildChain(ids);
  if (pid === get(activeFlowProjectId)) {
    flowNodes.set(nodes); // auto-saved via subscribe
    flowEdges.set(edges);
  } else {
    flowMap[pid] = { nodes, edges };
    saveFlowMap();
  }
}

/** Switch the Connection canvas to another project. */
export function switchFlowProject(id: string): void {
  if (id === resolveProjectId()) return;
  activeFlowProjectId.set(id);
  loadActiveCanvas();
}

/** Create a project (empty canvas) and switch to it. Returns the new id.
 *  The project also appears as a folder in the Agents view. */
export function addFlowProject(name: string): string {
  const id = `proj-${Date.now()}`;
  projects.update((list) => [...list, { id, name, agentIds: [] }]);
  flowMap[id] = { nodes: [], edges: [] };
  saveFlowMap();
  activeFlowProjectId.set(id);
  loadActiveCanvas();
  return id;
}

/**
 * Delete a project and its whole connection canvas (Patria 20 Okt: the folder
 * IS the project — one delete removes both, from either view). Deleting the
 * last project is allowed: the Connection view falls back to its default —
 * an empty canvas with just Mimi as parent.
 */
export function removeFlowProject(id: string): void {
  const list = get(projects);
  if (!list.some((p) => p.id === id)) return;
  projects.update((cur) => cur.filter((p) => p.id !== id));
  delete flowMap[id];
  saveFlowMap();
  if (get(activeFlowProjectId) === id) {
    const rest = get(projects);
    activeFlowProjectId.set(rest[0]?.id ?? "");
    loadActiveCanvas();
  }
}

export function addFlowNode(agentId: string, x?: number, y?: number): string {
  const id = `node-${flowSeq++}-${Date.now() % 100000}`;
  flowNodes.update((list) => {
    const px = x ?? 80 + (list.length % 4) * 320;
    const py = y ?? 80 + Math.floor(list.length / 4) * 220;
    return [...list, { id, agentId, x: Math.round(px), y: Math.round(py) }];
  });
  return id;
}

export function addFlowEdge(source: string, target: string) {
  if (source === target) return;
  flowEdges.update((list) => {
    if (list.some((e) => e.source === source && e.target === target)) return list;
    return [...list, { id: `edge-${source}-${target}`, source, target }];
  });
}

export function moveFlowNode(id: string, x: number, y: number) {
  flowNodes.update((list) =>
    list.map((n) => (n.id === id ? { ...n, x: Math.round(x), y: Math.round(y) } : n)),
  );
}

/** Remove a node and every edge connected to it. */
export function removeFlowNode(id: string) {
  flowNodes.update((list) => list.filter((n) => n.id !== id));
  flowEdges.update((list) =>
    list.filter((e) => e.source !== id && e.target !== id),
  );
}

/** Remove every edge connected to a node (the node itself stays). */
export function disconnectFlowNode(id: string) {
  flowEdges.update((list) =>
    list.filter((e) => e.source !== id && e.target !== id),
  );
}

/** A project workflow run (Patria 20 Okt): the folder's agent chain runs in
 *  order — agent status dots animate working → idle, visible in every view.
 *  Runs at store level so it can start from the Agents folder or Koneksi. */
export const flowRunning = writable(false);
export const runningProjectId = writable<string | null>(null);
let runTimers: ReturnType<typeof setTimeout>[] = [];
let runPrevStatuses: Record<string, AgentStatus> | null = null;

export function startFlowRun(projectId: string): boolean {
  const d =
    projectId === get(activeFlowProjectId)
      ? { nodes: get(flowNodes) }
      : flowMap[projectId];
  const nodes = d?.nodes ?? [];
  if (get(flowRunning) || nodes.length === 0) return false;
  stopFlowRun();
  flowRunning.set(true);
  runningProjectId.set(projectId);
  runPrevStatuses = { ...get(agentStatus) };
  const step = 900;
  nodes.forEach((n, i) => {
    runTimers.push(
      setTimeout(() => {
        agentStatus.update((s) => {
          const next = { ...s };
          for (const m of nodes) next[m.agentId] = "idle";
          next[n.agentId] = "working";
          return next;
        });
      }, i * step),
    );
  });
  runTimers.push(setTimeout(() => stopFlowRun(), nodes.length * step + 700));
  return true;
}

export function stopFlowRun(): void {
  for (const t of runTimers) clearTimeout(t);
  runTimers = [];
  flowRunning.set(false);
  runningProjectId.set(null);
  if (runPrevStatuses) {
    agentStatus.set(runPrevStatuses);
    runPrevStatuses = null;
  }
}

/** Add a user-created agent. Returns the new id. The agent is enabled right
    away (Patria 20 Okt): clicking "Agen baru" must visibly add the agent to
    the list — the "only Mimi by default" rule covers the initial roster,
    not an explicit user action. */

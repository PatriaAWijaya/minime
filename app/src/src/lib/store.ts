// Central UI state (Phase 1: all local, no backend yet).

import { writable, get, type Writable } from "svelte/store";
import type {
  Agent,
  AgentStatus,
  AgentTemplate,
  CostRow,
  FlowEdge,
  FlowNode,
  FlowProject,
  KanbanCard,
  KanbanCol,
  LibraryItem,
  Message,
  ProviderInfo,
  ScheduledItem,
  ServerKind,
  ServerLayout,
  Skill,
  TaskFolder,
} from "./types";

export type View =
  | "agents"
  | "tasks"
  | "schedule"
  | "library"
  | "cost"
  | "tools"
  | "settings";

/** Which main view is visible. */
export const activeView = writable<View>("agents");

/** Selected agent (shared by the Agents detail panel and the Tasks chat). */
export const activeAgentId = writable<string>("mimi");

/** Live status per agent id (yellow = working, green = idle, red = attention). */
export const agentStatus = writable<Record<string, AgentStatus>>({
  mimi: "idle",
  kreatif: "idle",
  riset: "working",
  finance: "idle",
  strategis: "idle",
  code: "idle",
});

/** Chat history per agent id (local mock). */
export const messagesByAgent = writable<Record<string, Message[]>>({});

let msgSeq = 1;

export function pushMessage(agentId: string, role: Message["role"], text: string) {
  const msg: Message = {
    id: msgSeq++,
    role,
    text,
    time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
  };
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: [...(m[agentId] ?? []), msg],
  }));
}

/** Incremented to focus the chat composer (e.g. from "+ Tugas Baru"). */
export const newTaskSignal = writable(0);

/** Small helper: load/persist a per-agent record in localStorage. */
function loadRecord<T>(key: string): Record<string, T> {
  try {
    return JSON.parse(localStorage.getItem(key) ?? "{}");
  } catch {
    return {};
  }
}

function persistRecord<T>(key: string, store: Writable<Record<string, T>>) {
  store.subscribe((v) => {
    try {
      localStorage.setItem(key, JSON.stringify(v));
    } catch {
      /* ignore */
    }
  });
}

/** Specific model per agent (mock, persisted).
    Default: Mimi = Muse, others = Ollama/Llama 3.1 (free-first principle). */
export const agentModel = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-model"),
);
persistRecord("minime.agent-model", agentModel);

export interface ModelOption {
  id: string;
  providerId: string;
  /** Display label (brand/model name, not translated) */
  label: string;
  kind: "free" | "paid";
}

/** Model catalog (mock). Free: Ollama local models + OpenCode free models.
    Paid: Muse, Gemini 3.8, DeepSeek. */
export const MODEL_CATALOG: ModelOption[] = [
  { id: "ollama:llama3.1", providerId: "ollama", label: "Llama 3.1", kind: "free" },
  { id: "ollama:qwen2.5", providerId: "ollama", label: "Qwen 2.5", kind: "free" },
  { id: "ollama:mistral", providerId: "ollama", label: "Mistral", kind: "free" },
  { id: "ollama:gemma2", providerId: "ollama", label: "Gemma 2", kind: "free" },
  { id: "opencode:deepseek-v3", providerId: "opencode", label: "DeepSeek V3", kind: "free" },
  { id: "opencode:qwen2.5-coder", providerId: "opencode", label: "Qwen 2.5 Coder", kind: "free" },
  { id: "claude", providerId: "claude", label: "Muse", kind: "paid" },
  { id: "gemini", providerId: "gemini", label: "Gemini 3.8", kind: "paid" },
  { id: "deepseek", providerId: "deepseek", label: "DeepSeek", kind: "paid" },
];

export function getModel(id: string): ModelOption {
  return MODEL_CATALOG.find((m) => m.id === id) ?? MODEL_CATALOG[0];
}

export function defaultModelFor(agentId: string): string {
  return agentId === "mimi" ? "claude" : "ollama:llama3.1";
}

/** Reasoning effort per agent (mock, persisted). */
export const agentReasoning = writable<Record<string, "low" | "mid" | "high">>(
  loadRecord<"low" | "mid" | "high">("minime.agent-reasoning"),
);
persistRecord("minime.agent-reasoning", agentReasoning);

/** Per-agent notification toggle (mock, persisted). */
export const agentNotify = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.agent-notify"),
);
persistRecord("minime.agent-notify", agentNotify);

/** Custom display-name override per agent (mock, persisted). */
export const agentNames = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-names"),
);
persistRecord("minime.agent-names", agentNames);

/** Custom title override per agent (mock, persisted). */
export const agentTitles = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-titles"),
);
persistRecord("minime.agent-titles", agentTitles);

/** Custom instructions per agent (mock, persisted). */
export const agentInstructions = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-instructions"),
);
persistRecord("minime.agent-instructions", agentInstructions);

/** Personalization: gender per agent (optional). */
export const agentGender = writable<Record<string, "male" | "female">>(
  loadRecord<"male" | "female">("minime.agent-gender"),
);
persistRecord("minime.agent-gender", agentGender);

/** Personalization: age per agent in years (1–120, optional). */
export const agentAge = writable<Record<string, number>>(
  loadRecord<number>("minime.agent-age"),
);
persistRecord("minime.agent-age", agentAge);

/** Personalization: position level per agent (optional). */
export const agentPosition = writable<
  Record<string, "clevel" | "senior" | "manager">
>(loadRecord<"clevel" | "senior" | "manager">("minime.agent-position"));
persistRecord("minime.agent-position", agentPosition);

/** Personalization: hobbies free text per agent. */
export const agentHobbies = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-hobbies"),
);
persistRecord("minime.agent-hobbies", agentHobbies);

/** Personalization: allow agent voice per agent. */
export const agentVoiceOn = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.agent-voice-on"),
);
persistRecord("minime.agent-voice-on", agentVoiceOn);

/** Personalization: chosen voice per agent (mock). */
export const agentVoice = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-voice"),
);
persistRecord("minime.agent-voice", agentVoice);

/** Mock facts shown in the Agents detail panel. */
export const agentFacts: Record<
  string,
  { memories: number; skills: number; files: string; tables: number; routines: number }
> = {
  mimi: { memories: 24, skills: 5, files: "312 KB", tables: 3, routines: 2 },
  kreatif: { memories: 12, skills: 4, files: "1.2 MB", tables: 1, routines: 1 },
  riset: { memories: 18, skills: 3, files: "856 KB", tables: 4, routines: 2 },
  finance: { memories: 9, skills: 3, files: "402 KB", tables: 5, routines: 1 },
  strategis: { memories: 11, skills: 4, files: "230 KB", tables: 2, routines: 1 },
  code: { memories: 7, skills: 6, files: "98 KB", tables: 0, routines: 0 },
};

/** Active-task indicator per agent (mock). */
export const activeTaskByAgent: Record<string, { titleKey: string; progress: number }> = {
  riset: { titleKey: "tasks.demo1", progress: 45 },
  kreatif: { titleKey: "tasks.demo2", progress: 80 },
};

/** Scheduled / recurring tasks (mock). */
let scheduleSeq = 5;
export const scheduleItems = writable<ScheduledItem[]>([
  { id: 1, nameKey: "schedule.mock2.name", whenKey: "schedule.mock2.when", range: "day", active: true },
  { id: 2, nameKey: "schedule.mock1.name", whenKey: "schedule.mock1.when", range: "week", active: true },
  { id: 3, nameKey: "schedule.mock4.name", whenKey: "schedule.mock4.when", range: "week", active: false },
  { id: 4, nameKey: "schedule.mock3.name", whenKey: "schedule.mock3.when", range: "month", active: true },
]);

export function addScheduleItem() {
  scheduleItems.update((items) => [
    ...items,
    {
      id: scheduleSeq++,
      nameKey: "schedule.mockNew.name",
      whenKey: "schedule.mockNew.when",
      range: "day",
      active: true,
    },
  ]);
}

/** Library outputs (mock). */
export const libraryItems: LibraryItem[] = [
  { id: 1, titleKey: "library.mock1.title", dateKey: "library.mock1.date", daysAgo: 1, type: "doc", agentId: "kreatif", taskKey: "schedule.mock1.name" },
  { id: 2, titleKey: "library.mock2.title", dateKey: "library.mock2.date", daysAgo: 2, type: "image", agentId: "kreatif", taskKey: "library.task.once" },
  { id: 3, titleKey: "library.mock3.title", dateKey: "library.mock3.date", daysAgo: 4, type: "table", agentId: "riset", taskKey: "schedule.mock4.name" },
  { id: 4, titleKey: "library.mock4.title", dateKey: "library.mock4.date", daysAgo: 7, type: "deck", agentId: "strategis", taskKey: "library.task.once" },
  { id: 5, titleKey: "library.mock5.title", dateKey: "library.mock5.date", daysAgo: 9, type: "video", agentId: "kreatif", taskKey: "schedule.mock3.name" },
  { id: 6, titleKey: "library.mock6.title", dateKey: "library.mock6.date", daysAgo: 14, type: "doc", agentId: "finance", taskKey: "library.task.once" },
  { id: 7, titleKey: "library.mock7.title", dateKey: "library.mock7.date", daysAgo: 21, type: "app", agentId: "code", taskKey: "library.task.once" },
  { id: 8, titleKey: "library.mock8.title", dateKey: "library.mock8.date", daysAgo: 17, type: "table", agentId: "finance", taskKey: "schedule.mock2.name" },
];

/** Cost rows (mock). */
export const costRows: CostRow[] = [
  { id: 1, agentId: "kreatif", tokens: 48200, costUsd: 0.38, dateKey: "cost.date.1", daysAgo: 1 },
  { id: 2, agentId: "riset", tokens: 96500, costUsd: 0.77, dateKey: "cost.date.1", daysAgo: 1 },
  { id: 3, agentId: "mimi", tokens: 12400, costUsd: 0.31, dateKey: "cost.date.2", daysAgo: 4 },
  { id: 4, agentId: "finance", tokens: 31200, costUsd: 0.25, dateKey: "cost.date.3", daysAgo: 9 },
  { id: 5, agentId: "strategis", tokens: 27800, costUsd: 0.22, dateKey: "cost.date.3", daysAgo: 9 },
  { id: 6, agentId: "code", tokens: 61400, costUsd: 0.49, dateKey: "cost.date.4", daysAgo: 17 },
  { id: 7, agentId: "riset", tokens: 44100, costUsd: 0.35, dateKey: "cost.date.5", daysAgo: 29 },
  { id: 8, agentId: "kreatif", tokens: 22900, costUsd: 0.18, dateKey: "cost.date.5", daysAgo: 29 },
];

/** Agent templates (mock). */
export const templates = writable<AgentTemplate[]>([
  { id: 1, nameKey: "tools.tpl1.name", descKey: "tools.tpl1.desc", enabled: true },
  { id: 2, nameKey: "tools.tpl2.name", descKey: "tools.tpl2.desc", enabled: true },
  { id: 3, nameKey: "tools.tpl3.name", descKey: "tools.tpl3.desc", enabled: false },
  { id: 4, nameKey: "tools.tpl4.name", descKey: "tools.tpl4.desc", enabled: false },
  { id: 5, nameKey: "tools.tpl5.name", descKey: "tools.tpl5.desc", enabled: false },
  { id: 6, nameKey: "tools.tpl6.name", descKey: "tools.tpl6.desc", enabled: false },
]);

/** Skills (mock). */
let skillSeq = 7;
export const skills = writable<Skill[]>([
  { id: 1, nameKey: "tools.skill1.name", descKey: "tools.skill1.desc", installed: true },
  { id: 2, nameKey: "tools.skill2.name", descKey: "tools.skill2.desc", installed: true },
  { id: 3, nameKey: "tools.skill3.name", descKey: "tools.skill3.desc", installed: false },
  { id: 4, nameKey: "tools.skill4.name", descKey: "tools.skill4.desc", installed: false },
  { id: 5, nameKey: "tools.skill5.name", descKey: "tools.skill5.desc", installed: false },
  { id: 6, nameKey: "tools.skill6.name", descKey: "tools.skill6.desc", installed: false },
]);

export function addSkill(name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  skills.update((list) => [
    ...list,
    { id: skillSeq++, customName: trimmed, installed: true },
  ]);
}

/* ---------- Model providers (Patria's decision 18 Oct ~22:40) ----------
   Free: Ollama = default, OpenCode = fallback.
   Paid: Muse = default, fallback = other popular models (Gemini 3.8, DeepSeek).
   All state below is local mock, persisted on this device. */

/** Built-in providers. */
export const PROVIDERS: ProviderInfo[] = [
  { id: "ollama", name: "Ollama", kind: "free", tagKey: "providers.defaultFree", local: true },
  { id: "opencode", name: "OpenCode", kind: "free", tagKey: "providers.backupFree" },
  { id: "claude", name: "Muse", kind: "paid", tagKey: "providers.defaultPaid" },
  { id: "gemini", name: "Gemini 3.8", kind: "paid" },
  { id: "deepseek", name: "DeepSeek", kind: "paid" },
];

/** Per-provider mock state: connected + active toggle. */
export const providerState = writable<Record<string, { connected: boolean; active: boolean }>>(
  loadRecord("minime.providers"),
);
persistRecord("minime.providers", providerState);

export function connectProvider(id: string) {
  providerState.update((s) => ({
    ...s,
    [id]: { connected: true, active: s[id]?.active ?? true },
  }));
}

export function toggleProvider(id: string, active: boolean) {
  providerState.update((s) => ({
    ...s,
    [id]: { connected: s[id]?.connected ?? false, active },
  }));
}

function loadString(key: string, fallback: string): string {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    return fallback;
  }
}

/** Default provider id (radio selection). */
export const defaultProviderId = writable<string>(
  loadString("minime.default-provider", "ollama"),
);
defaultProviderId.subscribe((v) => {
  try {
    localStorage.setItem("minime.default-provider", v);
  } catch {
    /* ignore */
  }
});

function loadBool(key: string, fallback: boolean): boolean {
  try {
    const v = localStorage.getItem(key);
    return v === null ? fallback : v === "1";
  } catch {
    return fallback;
  }
}

/** Auto-discover local models & agents (mock). */
export const autoScan = writable<boolean>(loadBool("minime.auto-scan", true));
autoScan.subscribe((v) => {
  try {
    localStorage.setItem("minime.auto-scan", v ? "1" : "0");
  } catch {
    /* ignore */
  }
});

export interface CustomProvider {
  id: number;
  name: string;
  endpoint: string;
  active: boolean;
}

/** Custom providers added by the user (mock). */
let customProvSeq = 1;
export const customProviders = writable<CustomProvider[]>([]);

export function addCustomProvider(name: string, endpoint: string) {
  const n = name.trim();
  const e = endpoint.trim();
  if (!n || !e) return;
  customProviders.update((list) => [
    ...list,
    { id: customProvSeq++, name: n, endpoint: e, active: true },
  ]);
}

export function toggleCustomProvider(id: number, active: boolean) {
  customProviders.update((list) =>
    list.map((p) => (p.id === id ? { ...p, active } : p)),
  );
}

/* ---------- Server selector (mock, persisted) ---------- */

/** Which AI model server is in use: local device or cloud. */
export const activeServer = writable<ServerKind>(
  loadString("minime.server", "local") as ServerKind,
);
activeServer.subscribe((v) => {
  try {
    localStorage.setItem("minime.server", v);
  } catch {
    /* ignore */
  }
});

/** Server menu layout preference (mock). */
export const serverLayout = writable<ServerLayout>(
  loadString("minime.server-layout", "menu") as ServerLayout,
);
serverLayout.subscribe((v) => {
  try {
    localStorage.setItem("minime.server-layout", v);
  } catch {
    /* ignore */
  }
});

/* ---------- Task folders + custom agents (mock, in-memory) ---------- */

let folderSeq = 3;

/** Example task folders; each groups agents that work together. */
export const taskFolders = writable<TaskFolder[]>([
  { id: 1, nameKey: "agents.folder1.name", agentIds: ["kreatif", "riset"], open: true },
  { id: 2, nameKey: "agents.folder2.name", agentIds: ["finance", "strategis"], open: false },
]);

export function addTaskFolder() {
  const id = folderSeq++;
  taskFolders.update((list) => [
    ...list,
    { id, nameKey: "agents.folderNew.name", num: id, agentIds: [], open: true },
  ]);
}

export function toggleFolder(id: number) {
  taskFolders.update((list) =>
    list.map((f) => (f.id === id ? { ...f, open: !f.open } : f)),
  );
}

let customAgentSeq = 1;
const CUSTOM_AGENT_COLORS = ["#64748b", "#0ea5e9", "#f43f5e", "#a855f7"];

/** Agents created in-app via the "+" menu (mock). */
export const customAgents = writable<Agent[]>([]);

/** Tasks sub-view: Kanban board or connection canvas (mock). */
export type TasksViewKind = "kanban" | "flow";

export const tasksView = writable<TasksViewKind>("kanban");

/** Small helper: load/persist a JSON list in localStorage. */
function loadList<T>(key: string, seed: () => T[]): T[] {
  try {
    const raw = localStorage.getItem(key);
    if (raw) return JSON.parse(raw) as T[];
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

function seedKanban(): KanbanCard[] {
  const day = 24 * 60 * 60 * 1000;
  const now = Date.now();
  const seed: KanbanCard[] = [
    { id: kanbanSeq++, title: "Riset kompetitor kopi susu", agentId: "riset", col: "queue", projectId: "proj-a", createdAt: "18 Okt", createdAtTs: now - 1 * day },
    { id: kanbanSeq++, title: "Draf kalender konten November", agentId: "kreatif", col: "queue", projectId: "proj-a", createdAt: "19 Okt", createdAtTs: now },
    { id: kanbanSeq++, title: "Laporan arus kas Q3", agentId: "finance", col: "doing", projectId: "proj-a", createdAt: "9 Okt", createdAtTs: now - 10 * day },
    { id: kanbanSeq++, title: "Naskah video profil perusahaan", agentId: "kreatif", col: "doing", projectId: "proj-b", createdAt: "18 Okt", createdAtTs: now - 1 * day },
    { id: kanbanSeq++, title: "Materi kampanye Ramadhan", agentId: "kreatif", col: "approval", projectId: "proj-a", createdAt: "19 Okt", createdAtTs: now },
    { id: kanbanSeq++, title: "Perbaiki headline brosur", agentId: "strategis", col: "revision", projectId: "proj-b", createdAt: "8 Sep", createdAtTs: now - 40 * day },
    { id: kanbanSeq++, title: "Audit biaya iklan", agentId: "strategis", col: "done", projectId: "proj-a", createdAt: "15 Okt", createdAtTs: now - 4 * day },
    { id: kanbanSeq++, title: "Evaluasi vendor cetak tahunan", agentId: "finance", col: "done", projectId: "proj-a", createdAt: "10 Jun", createdAtTs: now - 130 * day },
  ];
  return seed;
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

/** Kanban cards (mock, persisted). */
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
  kanbanCards.update((list) =>
    list.map((c) => (c.id === id ? { ...c, col } : c)),
  );
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
      if (parsed && typeof parsed === "object") return parsed;
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

function seedProject(id: string, name: string): FlowProject {
  return { id, name };
}

/** All projects, shared by the Connection canvas and the Kanban filter. */
export const projects = writable<FlowProject[]>(
  loadList("minime.projects", () => [
    seedProject("proj-a", "Project A"),
    seedProject("proj-b", "Project B"),
  ]),
);
persistList("minime.projects", projects);

/** Active project on the Connection canvas. */
export const activeFlowProjectId = writable<string>(
  loadString("minime.flow-project-active", "proj-a"),
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
    if (n) aNodes = JSON.parse(n) as FlowNode[];
    if (e) aEdges = JSON.parse(e) as FlowEdge[];
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

function loadActiveCanvas(): void {
  const d = flowMap[resolveProjectId()] ?? { nodes: [], edges: [] };
  flowNodes.set(d.nodes);
  flowEdges.set(d.edges);
}

function saveActiveCanvas(): void {
  flowMap[resolveProjectId()] = {
    nodes: get(flowNodes),
    edges: get(flowEdges),
  };
  saveFlowMap();
}

flowNodes.subscribe(() => saveActiveCanvas());
flowEdges.subscribe(() => saveActiveCanvas());
loadActiveCanvas();

/** Switch the Connection canvas to another project. */
export function switchFlowProject(id: string): void {
  if (id === resolveProjectId()) return;
  activeFlowProjectId.set(id);
  loadActiveCanvas();
}

/** Create a project (empty canvas) and switch to it. Returns the new id. */
export function addFlowProject(name: string): string {
  const id = `proj-${Date.now()}`;
  projects.update((list) => [...list, { id, name }]);
  flowMap[id] = { nodes: [], edges: [] };
  saveFlowMap();
  activeFlowProjectId.set(id);
  loadActiveCanvas();
  return id;
}

/**
 * Delete a project and its whole connection canvas. At least one project
 * must remain (callers hide/disable the delete affordance when only one is
 * left). If the deleted project was active, switch to the first remaining.
 */
export function removeFlowProject(id: string): void {
  const list = get(projects);
  if (list.length <= 1) return;
  const rest = list.filter((p) => p.id !== id);
  if (rest.length === list.length) return;
  projects.set(rest);
  delete flowMap[id];
  saveFlowMap();
  if (get(activeFlowProjectId) === id) {
    activeFlowProjectId.set(rest[0].id);
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

export function addCustomAgent() {
  const n = customAgentSeq++;
  customAgents.update((list) => [
    ...list,
    {
      id: `custom-${n}`,
      name: "",
      nameKey: "agents.agentNew.name",
      roleKey: "agents.agentNew.role",
      model: "Ollama",
      badge: "A",
      color: CUSTOM_AGENT_COLORS[(n - 1) % CUSTOM_AGENT_COLORS.length],
    },
  ]);
}

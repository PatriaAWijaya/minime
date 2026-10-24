import { writable, get, type Writable } from "svelte/store";
import { loadRecord, persistRecord, loadList, persistList } from "./_helpers";
import type {
  Agent, AgentFile, AgentStatus, CostRow, FlowEdge, FlowNode, FlowProject,
  KanbanCard, KanbanCol, LibraryItem, Message, ProviderInfo, ScheduledItem,
  ToolInfo, LibraryType,
} from "../types";
import { agents } from "../agents";
import { flowNodes, flowEdges } from "./tasks";

export interface AgentColWidths {
  list: number;
  desc: number;
}
function loadColWidths(): AgentColWidths {
  const clamp = (v: number, lo: number, hi: number) =>
    Math.min(hi, Math.max(lo, v));
  try {
    const raw = JSON.parse(
      localStorage.getItem("minime.agent-col-widths") ?? "{}",
    );
    return {
      list: clamp(typeof raw.list === "number" ? raw.list : 262, 180, 480),
      desc: clamp(typeof raw.desc === "number" ? raw.desc : 322, 240, 560),
    };
  } catch {
    return { list: 262, desc: 322 };
  }
}
export const agentColWidths = writable<AgentColWidths>(loadColWidths());
agentColWidths.subscribe((w) => {
  try {
    localStorage.setItem("minime.agent-col-widths", JSON.stringify(w));
  } catch {
    /* ignore */
  }
});

/** Selected agent (shared by the Agents detail panel and the Tasks chat). */
export const activeAgentId = writable<string>("mimi");

/** Live status per agent id (yellow = working, green = idle, red = attention). */
export const agentStatus = writable<Record<string, AgentStatus>>({
  mimi: "idle",
  kreatif: "idle",
  riset: "idle",
  finance: "idle",
  strategis: "idle",
  code: "idle",
});

/** Agent active/enabled state (Patria 20 Okt): by default ONLY the main
    agent (Mimi) is active. Other agents activate when the user enables
    them or when Mimi delegates work to them. */
export const agentEnabled = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.agent-enabled"),
);
persistRecord("minime.agent-enabled", agentEnabled);

export function isAgentEnabled(id: string): boolean {
  if (id === "mimi") return true;
  return get(agentEnabled)[id] ?? false;
}

export function setAgentEnabled(id: string, enabled: boolean): void {
  if (id === "mimi") return;
  agentEnabled.update((m) => ({ ...m, [id]: enabled }));
}

/** Activate an agent: it joins the active Agents list (Patria 20 Okt).
 *  New agents automatically appear in Tasks and Tools via the shared registry. */
export function activateAgent(id: string): void {
  setAgentEnabled(id, true);
}

/** Deactivate an agent (Patria 20 Okt): it disappears from the active Agents
 *  list, but stays in Tools (manageable) and in Tasks (its cards remain —
 *  only its function is disabled, enforced in runToolTask). Folder
 *  memberships are kept so reactivation restores them. */
export function deactivateAgent(id: string): void {
  setAgentEnabled(id, false);
}

/** Chat history per agent id (local mock). */
export const agentModel = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-model"),
);
persistRecord("minime.agent-model", agentModel);

/** Per-agent provider choice (Patria 21 Okt: pola OpenBot — provider dulu, model mengikuti). */
export const agentProvider = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-provider"),
);
persistRecord("minime.agent-provider", agentProvider);

/* ---------- Brand Kit (Media Studio) ---------- */
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

/** Custom avatar override per agent (persisted). */
export const agentAvatars = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-avatars"),
);
persistRecord("minime.agent-avatars", agentAvatars);

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

/** Per-agent: allow the agent to open a browser on its own (mock, Tahap 1).
    Real browser via Playwright comes in a later phase. */
export const agentBrowserOn = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.agent-browser-on"),
);
persistRecord("minime.agent-browser-on", agentBrowserOn);

/** Per-agent attached files (mock, persisted). */
export const agentFiles = writable<Record<string, AgentFile[]>>(
  loadRecord<AgentFile[]>("minime.agent-files"),
);
persistRecord("minime.agent-files", agentFiles);

export function addAgentFile(agentId: string, name: string, size: string) {
  agentFiles.update((m) => ({
    ...m,
    [agentId]: [...(m[agentId] ?? []), { name, size }],
  }));
}

export function removeAgentFile(agentId: string, index: number) {
  agentFiles.update((m) => ({
    ...m,
    [agentId]: (m[agentId] ?? []).filter((_, i) => i !== index),
  }));
}

/** Real per-agent stats for the detail panel (Patria 20 Okt: no fabricated
    numbers). Skills come from the backend catalog; files from the agent's
    own attachments. Memories/tables/routines have no real backing and are
    not shown. */
export function agentSkillCount(agentId: string): number {
  return (get(skillCatalog) ?? []).filter((s) => s.agents.includes(agentId)).length;
}

export function agentFileCount(agentId: string): number {
  return (get(agentFiles)[agentId] ?? []).length;
}

/** Scheduled / recurring tasks. Empty by default (Patria 20 Okt): no dummy
    items — the user creates their own via the Schedule view. */
export async function fetchAgentTools(agentId: string): Promise<string[] | null> {
  return await tauriInvoke<string[]>("get_agent_tools", { agentId });
}

/** Replace an agent's tool allowlist in the backend. */
export async function saveAgentTools(agentId: string, toolIds: string[]): Promise<void> {
  await tauriInvoke("set_agent_tools", { agentId, toolIds });
}

/** Gate check: may this agent use this tool? */
export async function fetchAgentCanUseTool(agentId: string, toolId: string): Promise<boolean | null> {
  return await tauriInvoke<boolean>("agent_can_use_tool", { agentId, toolId });
}

/* ---------- Marketplace > Skills (real catalog in the Rust backend) ---------- */

export interface SkillCatalogEntry {
  id: string;
  name: string;
  description: string;
  /** "bundled" (ships with the app) or "user" (created in-app). */
  source: "bundled" | "user";
  /** Agent ids whose effective skill set includes this skill. */
  agents: string[];
}

/** Catalog snapshot from the backend (null = not loaded / browser). */
export const skillCatalog = writable<SkillCatalogEntry[] | null>(null);

/** Refresh the skill catalog for every known agent (seed + custom). */
export async function refreshSkillCatalog(): Promise<SkillCatalogEntry[] | null> {
  try {
    const allIds = [
      ...agents.map((a) => a.id),
      ...get(customAgents).map((a) => a.id),
    ];
    const list = await tauriInvoke<SkillCatalogEntry[]>("list_skill_catalog", {
      agentIds: allIds,
    });
    if (list) skillCatalog.set(list);
    return list;
  } catch {
    return null;
  }
}

/** Replace an agent's skill set. The backend persists it to SQLite and the
 *  skills take effect on that agent's next chat message. */
export async function saveAgentSkills(
  agentId: string,
  skillIds: string[],
): Promise<boolean> {
  try {
    await tauriInvoke("set_agent_skills", { agentId, skills: skillIds });
    return true;
  } catch {
    return false;
  }
}

/** Create a user skill (writes a real SKILL.md into the user skills dir). */
export async function createSkillFile(
  name: string,
  description: string,
): Promise<SkillCatalogEntry | null> {
  try {
    return await tauriInvoke<SkillCatalogEntry>("create_skill", {
      name,
      description,
    });
  } catch {
    return null;
  }
}

/* ---------- Native notifications (Phase 3.2: approval gate) ---------- */

/** Send a native OS notification via the Rust backend. No-op in browser. */
/* ---------- Dynamic Island sync (Settings > Dynamic Island) ----------
   The main window composes the island payload (mode, texts, config) and
   sends it to the Rust `island_sync` command, which shows/hides/positions
   the real floating island window(s) and forwards the state to them. */

/** Live island state reported back by the backend (for the settings page). */
export const customAgents = writable<Agent[]>(
  loadList<Agent>("minime.custom-agents", () => []),
);
persistList("minime.custom-agents", customAgents);
// Restore the id sequence so a restart never reuses an id.
for (const a of get(customAgents)) {
  const m = /^custom-(\d+)$/.exec(a.id);
  if (m) customAgentSeq = Math.max(customAgentSeq, parseInt(m[1], 10) + 1);
}

/** Tasks sub-view: Kanban board or connection canvas (mock). */
export function addCustomAgent(): string {
  const n = customAgentSeq++;
  const id = `custom-${n}`;
  customAgents.update((list) => [
    ...list,
    {
      id,
      name: "",
      nameKey: "agents.agentNew.name",
      roleKey: "agents.agentNew.role",
      model: "Ollama",
      badge: "A",
      color: CUSTOM_AGENT_COLORS[(n - 1) % CUSTOM_AGENT_COLORS.length],
    },
  ]);
  setAgentEnabled(id, true);
  return id;
}

/** Drop one key from a per-agent record store. */
function dropAgentKey<T>(store: Writable<Record<string, T>>, id: string): void {
  store.update((m) => {
    if (!(id in m)) return m;
    const next = { ...m };
    delete next[id];
    return next;
  });
}

/** Permanently delete a custom agent (Patria 20 Okt): ONLY from the Tools
 *  menu; the 6 built-in Minime agents cannot be deleted (only deactivated).
 *  Cleans the registry, per-agent config, folder memberships, backend skill
 *  overrides and chat history; reassigns its cards/todos to Mimi so no
 *  orphaned work remains. */
export async function deleteCustomAgent(id: string): Promise<void> {
  if (!get(customAgents).some((a) => a.id === id)) return;
  customAgents.update((list) => list.filter((a) => a.id !== id));
  for (const s of [
    agentEnabled, agentModel, agentReasoning, agentNotify, agentNames,
    agentTitles, agentInstructions, agentGender, agentAge, agentPosition,
    agentHobbies, agentVoiceOn, agentVoice, agentBrowserOn, agentFiles,
  ]) {
    dropAgentKey(s as Writable<Record<string, unknown>>, id);
  }
  projects.update((list) =>
    list.map((p) => ({
      ...p,
      agentIds: (p.agentIds ?? []).filter((x) => x !== id),
    })),
  );
  // Remove the agent's nodes from every Connection canvas (Patria 20 Okt:
  // folder ↔ Koneksi stay in sync).
  const stripNodes = (nodes: FlowNode[], edges: FlowEdge[]) => {
    const dead = new Set(nodes.filter((n) => n.agentId === id).map((n) => n.id));
    if (dead.size === 0) return null;
    return {
      nodes: nodes.filter((n) => !dead.has(n.id)),
      edges: edges.filter((e) => !dead.has(e.source) && !dead.has(e.target)),
    };
  };
  const active = stripNodes(get(flowNodes), get(flowEdges));
  if (active) {
    flowNodes.set(active.nodes);
    flowEdges.set(active.edges);
  }
  for (const pid of Object.keys(flowMap)) {
    const d = flowMap[pid];
    if (!d) continue;
    const clean = stripNodes(d.nodes ?? [], d.edges ?? []);
    if (clean) flowMap[pid] = clean;
  }
  saveFlowMap();
  kanbanCards.update((list) =>
    list.map((c) => (c.agentId === id ? { ...c, agentId: "mimi" } : c)),
  );
  projectTodos.update((list) =>
    list.map((t) => (t.agentId === id ? { ...t, agentId: "mimi" } : t)),
  );
  messagesByAgent.update((m) => {
    if (!(id in m)) return m;
    const next = { ...m };
    delete next[id];
    return next;
  });
  try {
    await tauriInvoke("set_agent_skills", { agentId: id, skills: [] });
  } catch {
    /* backend unavailable (preview) — local state is already clean */
  }
  await refreshSkillCatalog();
}

/* ---------- Tauri bridge — model router nyata (Tahap 2) ----------
   Di aplikasi desktop, status provider & chat memakai Tauri commands
   (Rust). Di browser (preview), invoke() tidak ada → fallback ke mock
   lokal agar UI tetap bisa diklik-tayang. */


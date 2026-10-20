// Central UI state (Phase 1: all local, no backend yet).

import { writable, get, derived, type Writable, type Readable } from "svelte/store";
import { agents } from "./agents";
import type {
  Agent,
  AgentFile,
  AgentStatus,
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
  ToolInfo,
  LibraryType,
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

/** Manually resizable Agents-view column widths (Patria 20 Okt): drag the
 *  column edges with the mouse. Persisted per device. */
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
export const messagesByAgent = writable<Record<string, Message[]>>({});

let msgSeq = 1;

export function pushMessage(agentId: string, role: Message["role"], text: string, attachments?: Message["attachments"], replyTo?: string) {
  const msg: Message = {
    id: msgSeq++,
    role,
    text,
    time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    ...(attachments && attachments.length > 0 ? { attachments } : {}),
    ...(replyTo ? { replyTo } : {}),
  };
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: [...(m[agentId] ?? []), msg],
  }));
}

/** Remove a chat message (per-bubble delete). */
export function deleteMessage(agentId: string, msgId: number): void {
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: (m[agentId] ?? []).filter((msg) => msg.id !== msgId),
  }));
}

/** Toggle an emoji reaction on a chat message. */
export function setMessageReaction(agentId: string, msgId: number, emoji: string): void {
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: (m[agentId] ?? []).map((msg) =>
      msg.id === msgId
        ? { ...msg, reaction: msg.reaction === emoji ? undefined : emoji }
        : msg,
    ),
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
  /** True for models the user added themselves. */
  custom?: boolean;
}

/** Free-tier catalog: real model names only (no fictitious versions).
    Ollama models must exist in the Ollama library; OpenCode free models
    are verified against OpenCode's actual offerings. */
const FREE_TIER: Array<[string, string]> = [
  ["llama3.1:8b", "Llama 3.1 8B"],
  ["qwen2.5:7b", "Qwen 2.5 7B"],
  ["deepseek-r1:8b", "DeepSeek R1 8B"],
  ["mistral:7b", "Mistral 7B"],
  ["gemma2:9b", "Gemma 2 9B"],
  ["phi3:mini", "Phi-3 Mini"],
  ["llama3.2:3b", "Llama 3.2 3B"],
  ["qwen2.5:3b", "Qwen 2.5 3B"],
];

/** Section id for the model dropdown groups. */
export type ModelSection = "opencode" | "ollama" | "paid";

/** Custom models added by the user, persisted per section. */
export const customModels = writable<Record<ModelSection, string[]>>(
  (() => {
    const rec = loadRecord<string[]>("minime.custom-models");
    return {
      opencode: rec.opencode ?? [],
      ollama: rec.ollama ?? [],
      paid: rec.paid ?? [],
    };
  })(),
);
persistRecord("minime.custom-models", customModels);

function slugify(name: string): string {
  return (
    name
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "model"
  );
}

export function addCustomModel(section: ModelSection, name: string): void {
  const clean = name.trim();
  if (!clean) return;
  customModels.update((m) => {
    const list = m[section];
    if (list.some((n) => n.toLowerCase() === clean.toLowerCase())) return m;
    return { ...m, [section]: [...list, clean] };
  });
}

export function removeCustomModel(section: ModelSection, name: string): void {
  customModels.update((m) => ({
    ...m,
    [section]: m[section].filter((n) => n !== name),
  }));
}

/** Status provider dari backend Rust; null = belum dicek / browser.
    Didefinisikan di sini (bukan di bawah) karena modelCatalog memakainya. */
export const realProviders = writable<RealProviderStatus[] | null>(null);
/** True bila aplikasi berjalan di Tauri dengan backend Tahap 2. */
export const usingRealBackend = writable<boolean>(false);

/** Full catalog = built-in models + user-added custom models.
    Bila backend Tahap 2 tersedia dan Ollama terhubung, daftar model Ollama
    diganti dengan model yang benar-benar terinstal (dari /api/tags). */
export const modelCatalog: Readable<ModelOption[]> = derived(
  [customModels, realProviders],
  ([$custom, $real]): ModelOption[] => {
    const ollamaReal = $real?.find((p) => p.id === "ollama");
    const ollamaModels: ModelOption[] =
      ollamaReal && ollamaReal.state === "connected" && ollamaReal.models.length > 0
        ? ollamaReal.models.map(
            (m): ModelOption => ({
              id: m.id,
              providerId: "ollama",
              label: m.label,
              kind: "free",
            }),
          )
        : FREE_TIER.map(
            ([slug, label]): ModelOption => ({
              id: `ollama:${slug}`,
              providerId: "ollama",
              label,
              kind: "free",
            }),
          );
    return [
      ...ollamaModels,
      ...FREE_TIER.map(
        ([slug, label]): ModelOption => ({
          id: `opencode:${slug}`,
          providerId: "opencode",
          label,
          kind: "free",
        }),
      ),
      { id: "claude", providerId: "muse", label: "Muse", kind: "paid" },
      { id: "gemini", providerId: "gemini", label: "Gemini", kind: "paid" },
      { id: "gpt", providerId: "gpt", label: "GPT", kind: "paid" },
      { id: "claude", providerId: "claude", label: "Claude", kind: "paid" },
      { id: "deepseek", providerId: "deepseek", label: "DeepSeek", kind: "paid" },
    ...$custom.ollama.map(
      (label): ModelOption => ({
        id: `custom:ollama:${slugify(label)}`,
        providerId: "ollama",
        label,
        kind: "free",
        custom: true,
      }),
    ),
    ...$custom.opencode.map(
      (label): ModelOption => ({
        id: `custom:opencode:${slugify(label)}`,
        providerId: "opencode",
        label,
        kind: "free",
        custom: true,
      }),
    ),
    ...$custom.paid.map(
      (label): ModelOption => ({
        id: `custom:paid:${slugify(label)}`,
        providerId: "custom",
        label,
        kind: "paid",
        custom: true,
      }),
    ),
    ];
  },
);

export function getModel(id: string): ModelOption {
  const catalog = get(modelCatalog);
  // Legacy migration: old "claude" id was Muse (renamed to "muse" when
  // "Claude" became its own option).
  const resolved = id === "claude" ? "muse" : id;
  return catalog.find((m) => m.id === resolved) ?? catalog[0];
}

export function defaultModelFor(agentId: string): string {
  if (agentId === "mimi") return "muse";
  // Bila Ollama terhubung, pakai model chat asli pertama (bukan embedding).
  const ollama = get(realProviders)?.find((p) => p.id === "ollama");
  if (ollama && ollama.state === "connected") {
    const chat = ollama.models.find((m) => !/embed/i.test(m.label));
    if (chat) return chat.id;
    if (ollama.models.length > 0) return ollama.models[0].id;
  }
  return "ollama:qwen2.5:3b";
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
let scheduleSeq = 1;
export const scheduleItems = writable<ScheduledItem[]>([]);

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

/** Library outputs. Empty by default (Patria 20 Okt): no dummy items — real
 *  tool results are appended by addLibraryItem (Tahap 3.6) and persist
 *  across restarts. */
let librarySeq = 1000;
export const libraryItems = writable<LibraryItem[]>(
  loadList<LibraryItem>("minime.library", () => []),
);
persistList("minime.library", libraryItems);
for (const i of get(libraryItems)) {
  librarySeq = Math.max(librarySeq, i.id + 1);
}

/** Append a REAL tool result to the Library (Tahap 3.6). */
export function addLibraryItem(input: {
  title: string;
  summary?: string;
  type: LibraryType;
  agentId: string;
  taskKey?: string;
}): number {
  const now = new Date();
  const id = librarySeq++;
  libraryItems.update((list) => [
    {
      id,
      titleKey: "",
      dateKey: "",
      daysAgo: 0,
      type: input.type,
      agentId: input.agentId,
      taskKey: input.taskKey ?? "library.task.once",
      title: input.title,
      summary: input.summary,
      dateLabel: now.toLocaleDateString("id-ID", {
        day: "numeric",
        month: "short",
        year: "numeric",
      }),
    },
    ...list,
  ]);
  return id;
}

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

/** Installed apps (by app id). RuangAPI connects separately via credentials. */
export const installedApps = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.installed-apps"),
);
persistRecord("minime.installed-apps", installedApps);

export function toggleAppInstalled(id: string) {
  installedApps.update((m) => ({ ...m, [id]: !m[id] }));
  // Mirror the install state into the Rust tools gateway (Phase 3.1) so
  // the backend registry stays the source of truth. Fire-and-forget.
  const now = get(installedApps)[id] ?? false;
  void tauriInvoke("set_tool_connected", { toolId: id, connected: now }).catch(
    () => undefined,
  );
}

/* ---------- Tools gateway (Phase 3.1, Rust registry) ---------- */

/** Tool registry snapshot from the Rust gateway (null = not loaded / browser). */
export const toolInfos = writable<ToolInfo[] | null>(null);

/** Refresh the tool registry from the backend. Returns the list or null. */
export async function refreshTools(): Promise<ToolInfo[] | null> {
  const list = await tauriInvoke<ToolInfo[]>("list_tools");
  if (list) toolInfos.set(list);
  return list;
}

/** Is a tool connected in the backend registry? */
export async function fetchToolStatus(toolId: string): Promise<boolean | null> {
  return await tauriInvoke<boolean>("get_tool_status", { toolId });
}

/* ---------- Pireel (Tahap 3.4 — real OAuth, not the generic toggle) ---------- */

export type PireelOAuthPhase = "idle" | "pending" | "connected";

export interface PireelConnStatus {
  connected: boolean;
  /** "not_connected" | "live" | "offline" | "fresh" | "unknown" |
   *  "reauth_needed" | "network_error" */
  detail: string;
}

/** Last known Pireel status (null = not checked yet). */
export const pireelStatus = writable<PireelConnStatus | null>(null);
/** OAuth flow phase for the card UI. */
export const pireelOAuth = writable<PireelOAuthPhase>("idle");
/** Authorization URL of the in-flight flow (fallback reopen link). */
export const pireelAuthUrl = writable<string | null>(null);

/** Start OAuth: backend registers a client, opens the system browser. */
export async function pireelConnect(): Promise<void> {
  try {
    const url = await tauriInvoke<string>("pireel_connect_url");
    if (url) {
      pireelAuthUrl.set(url);
      pireelOAuth.set("pending");
    } else {
      pireelOAuth.set("idle");
    }
  } catch {
    pireelOAuth.set("idle");
  }
}

/** Poll the in-flight OAuth flow; refresh status once connected. */
export async function pireelPollOAuth(): Promise<void> {
  try {
    const s = await tauriInvoke<{ state: string }>("pireel_oauth_poll");
    if (!s) return;
    if (s.state === "connected") {
      pireelOAuth.set("connected");
      await pireelRefreshStatus();
    } else if (s.state === "idle") {
      pireelOAuth.set("idle");
    }
  } catch {
    /* keep polling */
  }
}

/** Refresh Pireel status from the backend (free get_state check). */
export async function pireelRefreshStatus(): Promise<void> {
  try {
    const s = await tauriInvoke<PireelConnStatus>("pireel_status");
    if (s) {
      pireelStatus.set(s);
      pireelOAuth.set(s.connected ? "connected" : "idle");
      // Mirror into the tools registry so the Apps tab + allowlists agree.
      void tauriInvoke("set_tool_connected", {
        toolId: "pireel",
        connected: s.connected,
      }).catch(() => undefined);
    }
  } catch {
    /* backend unreachable (browser preview) */
  }
}

/** Disconnect: tokens removed from the OS keychain. */
export async function pireelDisconnect(): Promise<void> {
  try {
    await tauriInvoke("pireel_disconnect");
  } catch {
    /* ignore */
  }
  pireelStatus.set({ connected: false, detail: "not_connected" });
  pireelOAuth.set("idle");
  pireelAuthUrl.set(null);
  void tauriInvoke("set_tool_connected", {
    toolId: "pireel",
    connected: false,
  }).catch(() => undefined);
}

/* ---------- Google Drive (Tahap 3.5 — OAuth client ID milik user) ---------- */

export type DriveOAuthPhase = "idle" | "pending" | "connected";

export interface DriveConnStatus {
  connected: boolean;
  /** "not_connected" | "connected" | "reauth_needed" | "network_error" |
   *  "invalid_client_id" */
  detail: string;
  email: string | null;
}

/** Last known Drive status (null = not checked yet). */
export const driveStatus = writable<DriveConnStatus | null>(null);
/** OAuth flow phase for the card UI. */
export const driveOAuth = writable<DriveOAuthPhase>("idle");
/** Authorization URL of the in-flight flow (fallback reopen link). */
export const driveAuthUrl = writable<string | null>(null);
/** The user's own Google OAuth client ID (Desktop app type). */
export const driveClientId = writable<string>(
  loadString("minime.drive.client-id", ""),
);
driveClientId.subscribe((v) => {
  try {
    localStorage.setItem("minime.drive.client-id", v);
  } catch {
    /* ignore */
  }
});

/** Start OAuth with the user's own client ID (opens the system browser). */
export async function driveConnect(): Promise<void> {
  const clientId = get(driveClientId).trim();
  if (!clientId) {
    driveStatus.set({ connected: false, detail: "invalid_client_id", email: null });
    return;
  }
  try {
    const url = await tauriInvoke<string>("drive_connect_url", {
      clientId,
    });
    if (url) {
      driveAuthUrl.set(url);
      driveOAuth.set("pending");
    } else {
      driveOAuth.set("idle");
    }
  } catch {
    driveOAuth.set("idle");
  }
}

/** Poll the in-flight OAuth flow; refresh status once connected. */
export async function drivePollOAuth(): Promise<void> {
  try {
    const s = await tauriInvoke<{ state: string }>("drive_oauth_poll");
    if (!s) return;
    if (s.state === "connected") {
      driveOAuth.set("connected");
      await driveRefreshStatus();
    } else if (s.state === "idle") {
      driveOAuth.set("idle");
    }
  } catch {
    /* keep polling */
  }
}

/** Refresh Drive status from the backend (free about.get check). */
export async function driveRefreshStatus(): Promise<void> {
  try {
    const s = await tauriInvoke<DriveConnStatus>("drive_status");
    if (s) {
      driveStatus.set(s);
      driveOAuth.set(s.connected ? "connected" : "idle");
      void tauriInvoke("set_tool_connected", {
        toolId: "drive",
        connected: s.connected,
      }).catch(() => undefined);
    }
  } catch {
    /* backend unreachable (browser preview) */
  }
}

/** Disconnect: tokens removed from the OS keychain. */
export async function driveDisconnect(): Promise<void> {
  try {
    await tauriInvoke("drive_disconnect");
  } catch {
    /* ignore */
  }
  driveStatus.set({ connected: false, detail: "not_connected", email: null });
  driveOAuth.set("idle");
  driveAuthUrl.set(null);
  void tauriInvoke("set_tool_connected", {
    toolId: "drive",
    connected: false,
  }).catch(() => undefined);
}

export interface DriveUploadDone {
  id: string;
  name: string;
}

/** Upload a deliverable into the MIMI folder. `contentBase64` is the raw
 *  file bytes encoded as base64. Returns the created file or null. */
export async function driveUploadFile(
  fileName: string,
  contentBase64: string,
  mimeType: string,
): Promise<DriveUploadDone | null> {
  try {
    const r = await tauriInvoke<DriveUploadDone>("drive_upload", {
      fileName,
      contentBase64,
      mimeType,
    });
    return r ?? null;
  } catch {
    return null;
  }
}

/** Agent tool allowlist from the backend. */
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
export const islandStatus = writable<{ mode: string; displays: number } | null>(
  null,
);

export interface IslandPayload {
  notch: boolean;
  idle: boolean;
  extraDisplays: boolean;
  haptics: boolean;
  position: string;
  widthPct: number;
  heightPct: number;
  mode: string;
  title: string;
  sub: string;
  attention: boolean;
  logoColor: string;
}

export async function syncIsland(payload: IslandPayload): Promise<void> {
  try {
    const res = await tauriInvoke<{ mode: string; displays: number }>(
      "island_sync",
      { payload },
    );
    if (res) islandStatus.set(res);
  } catch {
    /* island unavailable — never block the app */
  }
}

/** Fetch the last island payload (the island window asks on mount). */
export async function fetchIslandState(): Promise<IslandPayload | null> {
  try {
    return await tauriInvoke<IslandPayload>("island_get_state");
  } catch {
    return null;
  }
}

/* ---------- macOS system permissions (Settings > Computer Use) ---------- */

export interface SystemPermissions {
  screen_recording: boolean;
  accessibility: boolean;
  /** False on platforms without this permission model. */
  supported: boolean;
}

/** Real preflight check (never prompts). Null outside the desktop app. */
export async function fetchSystemPermissions(): Promise<SystemPermissions | null> {
  try {
    return await tauriInvoke<SystemPermissions>("system_permissions");
  } catch {
    return null;
  }
}

/** Open the macOS Privacy & Security pane for one permission kind. */
export async function openPrivacySettings(kind: "screen" | "accessibility"): Promise<void> {
  try {
    await tauriInvoke<void>("open_privacy_settings", { kind });
  } catch {
    /* non-macOS or open failed — the settings page explains the state */
  }
}

/* ---------- Device info (Settings > Profile sessions) ---------- */

export interface DeviceInfo {
  name: string;
  os: string;
}

/** This device's name + OS label from the backend. Null in the browser. */
export async function fetchDeviceInfo(): Promise<DeviceInfo | null> {
  try {
    return await tauriInvoke<DeviceInfo>("device_info");
  } catch {
    return null;
  }
}

/* ---------- App updates (Settings > Updates) ---------- */

export interface UpdateInfo {
  current: string;
  latest: string | null;
  update_available: boolean;
  notes: string;
  url: string;
}

/** Result of the last update check (GitHub releases). */
export const updateInfo = writable<UpdateInfo | null>(null);

/** Check GitHub releases for a newer Minime version on the chosen track. */
export async function checkForUpdate(): Promise<void> {
  try {
    const res = await tauriInvoke<UpdateInfo>("check_update", {
      track: get(settingsPrefs).updateTrack,
    });
    if (res) updateInfo.set(res);
  } catch {
    /* offline or no releases — keep the previous state */
  }
}

/** Restart the app (applies a downloaded update). */
export async function restartApp(): Promise<void> {
  try {
    await tauriInvoke<void>("restart_app");
  } catch {
    /* not in the desktop app */
  }
}

export async function sendNotification(title: string, body: string): Promise<void> {
  try {
    await tauriInvoke("send_notification", { title, body });
  } catch {
    /* notifications unavailable — never block the workflow */
  }
}

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
   Paid: Muse = default, fallback = Gemini, GPT, Claude (in that order).
   All state below is local mock, persisted on this device. */

/** Built-in providers. */
export const PROVIDERS: ProviderInfo[] = [
  { id: "ollama", name: "Ollama", kind: "free", tagKey: "providers.defaultFree", local: true },
  { id: "opencode", name: "OpenCode", kind: "free", tagKey: "providers.backupFree" },
  { id: "muse", name: "Muse", kind: "paid", tagKey: "providers.defaultPaid" },
  { id: "gemini", name: "Gemini", kind: "paid" },
  { id: "gpt", name: "GPT", kind: "paid" },
  { id: "claude", name: "Claude", kind: "paid" },
  { id: "deepseek", name: "DeepSeek", kind: "paid" },
  { id: "routerku", name: "RouterKu", kind: "paid", tagKey: "providers.routerKuTag" },
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

/* ---------- RouterKu gateway (mock, local only) ----------
   Real connection = Phase 2. The key never leaves this device. */
export const routerkuKey = writable<string>(loadString("minime.routerku.key", ""));
export const routerkuBaseUrl = writable<string>(loadString("minime.routerku.baseurl", ""));
routerkuKey.subscribe((v) => {
  try { localStorage.setItem("minime.routerku.key", v); } catch { /* ignore */ }
});
routerkuBaseUrl.subscribe((v) => {
  try { localStorage.setItem("minime.routerku.baseurl", v); } catch { /* ignore */ }
});
/** Save RouterKu credentials; mark connected only when a key is present. */
export function saveRouterku(key: string, baseUrl: string) {
  routerkuKey.set(key);
  routerkuBaseUrl.set(baseUrl);
  const ok = key.trim().length > 0;
  providerState.update((s) => ({
    ...s,
    routerku: { connected: ok, active: s["routerku"]?.active ?? true },
  }));
}

/* ---------- RuangAPI gateway (mock, local only) ----------
   Real connection = Phase 3. The key never leaves this device. */
export const ruangapiKey = writable<string>(loadString("minime.ruangapi.key", ""));
export const ruangapiMcpUrl = writable<string>(
  loadString("minime.ruangapi.mcp-url", "https://mcp.ruangapi.id/mcp"),
);
export const ruangapiConnected = writable<boolean>(
  loadString("minime.ruangapi.connected", "false") === "true",
);
ruangapiKey.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.key", v); } catch { /* ignore */ }
});
ruangapiMcpUrl.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.mcp-url", v); } catch { /* ignore */ }
});
ruangapiConnected.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.connected", String(v)); } catch { /* ignore */ }
});
/** Mock-connect RuangAPI; mark connected only when a key is present. */
export function connectRuangapi(key: string) {
  ruangapiKey.set(key);
  ruangapiConnected.set(key.trim().length > 0);
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

/** Servers the user has added (Phase: server menu per Patria's reference).
    "local" always exists; Cloud/VPS and Independent are added on demand. */
export const serverList = writable<ServerKind[]>(
  (() => {
    try {
      const raw = localStorage.getItem("minime.server-list");
      const parsed: unknown = raw ? JSON.parse(raw) : null;
      if (Array.isArray(parsed)) {
        const valid = parsed.filter(
          (k): k is ServerKind => k === "local" || k === "cloud" || k === "independent",
        );
        if (!valid.includes("local")) valid.unshift("local");
        return valid.length ? valid : ["local"];
      }
    } catch {
      /* fall through to default */
    }
    return ["local"];
  })(),
);
serverList.subscribe((v) => {
  try {
    localStorage.setItem("minime.server-list", JSON.stringify(v));
  } catch {
    /* ignore */
  }
});

/** Add a server location and switch to it. */
export function addServer(kind: ServerKind): void {
  serverList.update((list) => (list.includes(kind) ? list : [...list, kind]));
  activeServer.set(kind);
}

/** Per-server mute state (server menu, Local submenu). */
export const serverMuted = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.server-muted"),
);
persistRecord("minime.server-muted", serverMuted);

export function setServerMuted(kind: ServerKind, muted: boolean): void {
  serverMuted.update((m) => ({ ...m, [kind]: muted }));
}

/** Per-server notification level: all | mentions | nothing. */
export type ServerNotifLevel = "all" | "mentions" | "nothing";export const serverNotif = writable<Record<string, ServerNotifLevel>>(
  loadRecord<ServerNotifLevel>("minime.server-notif"),
);
persistRecord("minime.server-notif", serverNotif);

export function setServerNotif(kind: ServerKind, level: ServerNotifLevel): void {
  serverNotif.update((m) => ({ ...m, [kind]: level }));
}

/* ---------- App settings (Settings view, two-column layout) ---------- */

export interface SettingsPrefs {
  langMode: "system" | "id" | "en" | "zh";
  logoColor: string;
  sendShortcut: "enter" | "ctrlEnter";
  steerAgents: boolean;
  memoriesPerAgent: number;
  externalLinks: "browser" | "inapp";
  launchAtLogin: boolean;
  fastConnect: boolean;
  keepBackground: boolean;
  restoreWorkspace: boolean;
  shareAnalytics: boolean;
  notifApproval: boolean;
  notifDone: boolean;
  notifSound: boolean;
  voiceEnabled: boolean;
  voiceLang: string;
  dictationModel: "parakeet" | "whisper" | "elevenlabs";
  dictationLang: string;
  dictationDownloads: string[];
  elevenlabsKeySaved: boolean;
  /** Permanent Mobile Connect link token ("" = no link generated). */
  mobileLinkToken: string;
  islandNotch: boolean;
  islandIdle: boolean;
  islandExtra: boolean;
  islandHaptics: boolean;
  islandPosition: "top" | "bottom";
  islandServers: Record<string, boolean>;
  islandFutureServers: boolean;
  islandWidth: number;
  islandHeight: number;
  computerBrowser: boolean;
  computerHidden: boolean;
  computerMirror: boolean;
  autoUpdate: boolean;
  updateTrack: "stable" | "beta";
  updateAutoInstall: boolean;
  updateMembers: boolean;
}

const SETTINGS_DEFAULTS: SettingsPrefs = {
  langMode: "system",
  logoColor: "#B18CFF",
  sendShortcut: "enter",
  steerAgents: false,
  memoriesPerAgent: 64,
  externalLinks: "browser",
  launchAtLogin: true,
  fastConnect: true,
  keepBackground: false,
  restoreWorkspace: true,
  shareAnalytics: true,
  notifApproval: true,
  notifDone: true,
  notifSound: false,
  voiceEnabled: true,
  voiceLang: "id",
  dictationModel: "parakeet",
  dictationLang: "auto",
  dictationDownloads: [],
  elevenlabsKeySaved: false,
  mobileLinkToken: "",
  islandNotch: true,
  islandIdle: true,
  islandExtra: true,
  islandHaptics: true,
  islandPosition: "top",
  islandServers: {},
  islandFutureServers: true,
  islandWidth: 100,
  islandHeight: 100,
  computerBrowser: true,
  computerHidden: false,
  computerMirror: true,
  autoUpdate: true,
  updateTrack: "stable",
  updateAutoInstall: false,
  updateMembers: true,
};

export const settingsPrefs = writable<SettingsPrefs>({
  ...SETTINGS_DEFAULTS,
  ...loadRecord<unknown>("minime.settings-prefs"),
} as SettingsPrefs);
settingsPrefs.subscribe((v) => {
  try {
    localStorage.setItem("minime.settings-prefs", JSON.stringify(v));
  } catch {
    /* ignore */
  }
});

export function setPref<K extends keyof SettingsPrefs>(key: K, value: SettingsPrefs[K]): void {
  settingsPrefs.update((p) => ({ ...p, [key]: value }));
}

/** User profile (Settings > Profile). `photo` is a small data-URL
    (downscaled at upload); `signedInAt` stamps the local session start. */
export interface ProfileState {
  name: string;
  email: string;
  photo: string;
  signedInAt: number;
}

export const profileStore = writable<ProfileState>({
  name: "",
  email: "",
  photo: "",
  signedInAt: 0,
  ...(loadRecord<string>("minime.profile") as Partial<ProfileState>),
} as ProfileState);
profileStore.subscribe((v) => {
  try {
    localStorage.setItem("minime.profile", JSON.stringify(v));
  } catch {
    /* ignore */
  }
});
// Stamp the local session the first time the app runs with a profile.
if (get(profileStore).signedInAt === 0) {
  profileStore.update((p) => ({ ...p, signedInAt: Date.now() }));
}

/* ---------- Theme (keputusan Patria 19 Okt): system = follow OS,
   or force dark/light. Applied as data-theme on <html> by App.svelte. ---------- */

export type Theme = "system" | "dark" | "light";

export const theme = writable<Theme>(
  loadString("minime.theme", "system") as Theme,
);
theme.subscribe((v) => {
  try {
    localStorage.setItem("minime.theme", v);
  } catch {
    /* ignore */
  }
});

/** Folder open/collapse UI state per project (the Agents-view folder IS the
    project — Patria 20 Okt). Not persisted; defaults to open. */
export const folderOpen = writable<Record<string, boolean>>({});

export function toggleFolder(id: string) {
  folderOpen.update((m) => ({ ...m, [id]: !(m[id] ?? true) }));
}

export function isFolderOpen(id: string, openMap: Record<string, boolean>): boolean {
  return openMap[id] ?? true;
}

let customAgentSeq = 1;
const CUSTOM_AGENT_COLORS = ["#F4A7C3", "#A8D8FF", "#B5E8C0", "#FFD6A5", "#D3BFF0", "#FFF3B0"];

/** Agents created in-app via the "+" menu or the Marketplace.
 *  Persisted so template agents survive restarts. */
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

/* ---------- Agent runtime (Tahap 3.6) ----------
   Runs a real tool for an agent, with the Kanban card moving along
   queue → doing → approval and the result landing in the Library.
   Errors move the card back to queue and notify — never a silent dummy. */

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
    agentIds: Array.isArray(p.agentIds) ? p.agentIds : [],
  })),
);
persistList("minime.projects", projects);

/** Rename a project (also renames its Agents-view folder). */
export function renameProject(id: string, name: string): void {
  const n = name.trim();
  if (!n) return;
  projects.update((list) => list.map((p) => (p.id === id ? { ...p, name: n } : p)));
}

/** Agents assigned to a project = agents in its folder. An agent belongs to
    at most one folder; adding it to a project removes it from the others. */
/** Add an agent id to a folder's roster (dedup per folder). No canvas
 *  side-effects — callers decide about regen. */
function folderAddAgent(projectId: string, agentId: string): void {
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

function loadActiveCanvas(): void {
  const pid = resolveProjectId();
  const d = flowMap[pid];
  // Defensive: old/corrupt localStorage may hold {} without nodes/edges.
  const nodes = Array.isArray(d?.nodes) ? d.nodes : [];
  const edges = Array.isArray(d?.edges) ? d.edges : [];
  flowNodes.set(nodes);
  flowEdges.set(edges);
  // Patria 20 Okt — Folder → Koneksi: empty canvas + folder has agents ⇒
  // auto-generate the agent chain in folder order.
  if (nodes.length === 0) {
    const ids = get(projects).find((p) => p.id === pid)?.agentIds ?? [];
    if (ids.length > 0) {
      const { nodes: cn, edges: ce } = buildChain(ids);
      flowNodes.set(cn);
      flowEdges.set(ce);
    }
  }
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

/** Build a sequential agent chain (Patria 20 Okt): one node per agent in
 *  folder order, linked source → target. */
function buildChain(ids: string[]): { nodes: FlowNode[]; edges: FlowEdge[] } {
  const nodes: FlowNode[] = ids.map((agentId, i) => ({
    id: `node-${flowSeq++}-${Date.now() % 100000}`,
    agentId,
    x: 80 + i * 320,
    y: 140,
  }));
  const edges: FlowEdge[] = [];
  for (let i = 0; i + 1 < nodes.length; i++) {
    edges.push({
      id: `edge-${nodes[i].id}-${nodes[i + 1].id}`,
      source: nodes[i].id,
      target: nodes[i + 1].id,
    });
  }
  return { nodes, edges };
}

/** True when the canvas is a pure reflection of the folder team (same agents,
 *  each exactly once) — i.e. still auto-generated, not manually edited. */
function isCanvasAuto(pid: string): boolean {
  const ids = get(projects).find((p) => p.id === pid)?.agentIds ?? [];
  const nodes =
    pid === get(activeFlowProjectId) ? get(flowNodes) : (flowMap[pid]?.nodes ?? []);
  if (nodes.length === 0 || nodes.length !== ids.length) return false;
  const nodeAgents = nodes.map((n) => n.agentId);
  return (
    ids.every((id) => nodeAgents.includes(id)) &&
    nodeAgents.every((id) => ids.includes(id))
  );
}

/** Regenerate the auto-chain for a project from its current folder order. */
function regenChain(pid: string): void {
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

/** Add a user-created agent. Returns the new id. The agent is enabled right
    away (Patria 20 Okt): clicking "Agen baru" must visibly add the agent to
    the list — the "only Mimi by default" rule covers the initial roster,
    not an explicit user action. */
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

export interface RealModelInfo {
  id: string;
  label: string;
}

export interface RealProviderStatus {
  id: string;
  label: string;
  kind: "free" | "paid";
  /** "connected" | "no_key" | "unreachable" | "unknown" */
  state: string;
  detail: string;
  models: RealModelInfo[];
}

export interface ChatWireMessage {
  role: string;
  content: string;
}

export interface ChatWireResult {
  text: string;
  served_by: string;
}

/** True bila berjalan di Tauri (backend Rust tersedia). Di-cache per sesi. */
let tauriAvailable: boolean | null = null;
export async function isTauri(): Promise<boolean> {
  if (tauriAvailable !== null) return tauriAvailable;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("get_app_info");
    tauriAvailable = true;
  } catch {
    tauriAvailable = false;
  }
  return tauriAvailable;
}

async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | null> {
  if (!(await isTauri())) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return await invoke<T>(cmd, args ?? {});
}

function mirrorToMock(list: RealProviderStatus[]) {
  providerState.update((s) => {
    const next = { ...s };
    for (const p of list) {
      next[p.id] = {
        connected: p.state === "connected",
        active: next[p.id]?.active ?? true,
      };
    }
    return next;
  });
}

/** Pindai ulang semua provider via backend. Dipanggil saat layar
    Penyedia Model dibuka dan setelah connect/disconnect. */
export async function refreshProviders(): Promise<void> {
  try {
    const res = await tauriInvoke<RealProviderStatus[]>("provider_probe");
    if (res) {
      usingRealBackend.set(true);
      realProviders.set(res);
      mirrorToMock(res);
      await migrateRouterkuKey();
    }
  } catch {
    /* backend gagal — biarkan mock */
  }
}

/** Simpan API key provider berbayar ke OS Keychain (atau mock di browser). */
export async function connectProviderKey(
  id: string,
  apiKey: string,
): Promise<boolean> {
  const key = apiKey.trim();
  if (!key) return false;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: id,
      apiKey: key,
    });
    if (await isTauri()) {
      await refreshProviders();
      return true;
    }
  } catch {
    return false;
  }
  connectProvider(id); // fallback browser: mock
  return true;
}

/** Hapus API key provider dari OS Keychain. */
export async function disconnectProviderKey(id: string): Promise<void> {
  try {
    await tauriInvoke<void>("provider_disconnect", { providerId: id });
  } catch {
    /* abaikan */
  }
  providerState.update((s) => ({
    ...s,
    [id]: { connected: false, active: s[id]?.active ?? true },
  }));
  await refreshProviders();
}

/** Simpan API key ElevenLabs Scribe (Pengaturan > Suara) ke OS Keychain.
    Hanya status tersimpan yang dicatat di preferensi — kuncinya tidak. */
export async function saveElevenlabsKey(key: string): Promise<boolean> {
  const k = key.trim();
  if (!k) return false;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: "elevenlabs",
      apiKey: k,
    });
  } catch {
    return false;
  }
  setPref("elevenlabsKeySaved", true);
  return true;
}

/** Hapus API key ElevenLabs dari OS Keychain. */
export async function removeElevenlabsKey(): Promise<void> {
  try {
    await tauriInvoke<void>("provider_disconnect", {
      providerId: "elevenlabs",
    });
  } catch {
    /* abaikan */
  }
  setPref("elevenlabsKeySaved", false);
}

/** Migrasi sekali: pindahkan API key RouterKu yang dulu tersimpan di
    localStorage (mock Tahap 1) ke OS Keychain, lalu hapus dari sana. */
let routerkuMigrated = false;
async function migrateRouterkuKey(): Promise<void> {
  if (routerkuMigrated) return;
  routerkuMigrated = true;
  let raw = "";
  try {
    raw = localStorage.getItem("minime.routerku.key") ?? "";
  } catch {
    return;
  }
  if (!raw.trim()) return;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: "routerku",
      apiKey: raw.trim(),
    });
    if (await isTauri()) {
      try {
        localStorage.removeItem("minime.routerku.key");
      } catch {
        /* ignore */
      }
      routerkuKey.set("");
    }
  } catch {
    /* abaikan */
  }
}

/** Simpan kredensial RouterKu: key → Keychain, base URL → settings.
/// Tidak lagi menyimpan raw key di localStorage. */
export async function saveRouterkuSecure(
  key: string,
  baseUrl: string,
): Promise<boolean> {
  const k = key.trim();
  const u = baseUrl.trim();
  try {
    if (u) {
      await tauriInvoke<void>("routerku_set_base_url", { baseUrl: u });
      routerkuBaseUrl.set(u);
    }
    if (!k) return false;
    await tauriInvoke<void>("provider_connect", {
      providerId: "routerku",
      apiKey: k,
    });
    if (await isTauri()) {
      await refreshProviders();
      return true;
    }
  } catch {
    return false;
  }
  saveRouterku(k, u); // fallback browser: mock lama
  return true;
}

/* ---------- System prompt builder (Tahap 2) ----------
   Persona agen = peran + nama + instruksi kustom + personalisasi.
   Playbook skill disuntikkan di backend Rust (skills.rs). */

/** Susun system prompt untuk agen. `roleLabel` = peran terterjemah dari UI. */
export function buildSystemPrompt(agentId: string, roleLabel: string): string {
  const name = (get(agentNames)[agentId] ?? "").trim();
  const title = (get(agentTitles)[agentId] ?? "").trim();
  const instructions = (get(agentInstructions)[agentId] ?? "").trim();
  const gender = get(agentGender)[agentId];
  const age = get(agentAge)[agentId];
  const position = get(agentPosition)[agentId];
  const hobbies = (get(agentHobbies)[agentId] ?? "").trim();

  const lines: string[] = [];
  lines.push(
    `Kamu adalah ${name || agentId}, asisten AI di aplikasi Minime.`,
    `Peranmu: ${title || roleLabel}.`,
  );
  if (agentId === "mimi") {
    lines.push(
      "Kamu adalah CIO (Chief Intelligent Officer) dan orkestrator tim: " +
        "koordinasikan 5 agen senior (Kreatif, Riset, Finance, Bisnis Strategis, Code Builder), " +
        "pecah permintaan kompleks menjadi tugas yang jelas, dan rangkum hasilnya untuk pengguna.",
    );
  }
  if (gender || age || position || hobbies) {
    const parts: string[] = [];
    if (gender) parts.push(gender === "male" ? "laki-laki" : "perempuan");
    if (age) parts.push(`${age} tahun`);
    if (position) parts.push(position);
    if (hobbies) parts.push(`hobi: ${hobbies}`);
    lines.push(`Kepribadian: ${parts.join(", ")}.`);
  }
  lines.push(
    "Jawab dalam Bahasa Indonesia yang ringkas dan jelas, kecuali pengguna memakai bahasa lain.",
    "Jangan mengarang fakta; bila tidak tahu, katakan terus terang.",
  );
  if (instructions) {
    lines.push(`Instruksi khusus dari pengguna:\n${instructions}`);
  }
  return lines.join("\n");
}

/* ---------- Chat history + usage nyata (Tahap 2) ---------- */

export interface RealStoredMessage {
  id: number;
  agent_id: string;
  role: string;
  text: string;
  served_by: string | null;
  created_at: string;
}

export interface RealUsageRow {
  agent_id: string;
  provider_id: string;
  model_id: string;
  calls: number;
  input_tokens: number;
  output_tokens: number;
}

/** Race janji melawan timeout — menjamin tidak ada hang diam-diam. */
function withTimeout<T>(p: Promise<T>, ms: number, label: string): Promise<T> {
  return Promise.race([
    p,
    new Promise<never>((_, rej) =>
      setTimeout(() => rej(`${label}: timeout setelah ${Math.round(ms / 1000)} detik`), ms),
    ),
  ]);
}

/** Kirim chat lewat model router (dengan fallback otomatis).
    - Return null → bukan di Tauri (browser/preview), pakai mock.
    - Throw string → di Tauri tapi router gagal / timeout (pesan Indonesia). */
export async function sendChatViaRouter(
  agentId: string,
  modelId: string,
  tier: "gratis" | "berbayar",
  systemPrompt: string,
  messages: ChatWireMessage[],
): Promise<ChatWireResult | null> {
  const tauri = await withTimeout(isTauri(), 10000, "deteksi backend");
  if (!tauri) return null;
  // Pastikan daftar provider/model nyata sudah dimuat (untuk model dinamis).
  if (get(realProviders) === null) {
    await refreshProviders();
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return await withTimeout(
    invoke<ChatWireResult>("agent_chat", {
      agentId,
      modelId,
      tier,
      systemPrompt,
      messages,
    }),
    180000,
    "model router",
  );
}

const historyLoadedFor = new Set<string>();

/** Muat riwayat chat agen dari SQLite (sekali per agen per sesi). */
export async function loadChatHistory(agentId: string): Promise<void> {
  if (historyLoadedFor.has(agentId)) return;
  historyLoadedFor.add(agentId);
  let rows: RealStoredMessage[] | null = null;
  try {
    rows = await tauriInvoke<RealStoredMessage[]>("chat_history", {
      agentId,
      limit: 100,
    });
  } catch {
    return;
  }
  if (!rows || rows.length === 0) return;
  messagesByAgent.update((m) => {
    if ((m[agentId] ?? []).length > 0) return m; // sudah ada pesan sesi ini
    return {
      ...m,
      [agentId]: rows.map((r) => {
        let time = r.created_at;
        const d = new Date(r.created_at);
        if (!isNaN(d.getTime())) {
          time = d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        }
        return {
          id: r.id,
          role: r.role === "assistant" ? "agent" : "user",
          text: r.text,
          time,
        };
      }),
    };
  });
}

/** Ringkasan usage nyata untuk tab Cost; null = belum dimuat / browser. */
export const realUsage = writable<RealUsageRow[] | null>(null);

export async function refreshUsage(): Promise<void> {
  try {
    const rows = await tauriInvoke<RealUsageRow[]>("usage_summary");
    if (rows) realUsage.set(rows);
  } catch {
    /* abaikan */
  }
}

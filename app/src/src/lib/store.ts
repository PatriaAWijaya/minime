// Central UI state (Phase 1: all local, no backend yet).

import { writable, get, derived, type Writable, type Readable } from "svelte/store";
import type {
  Agent,
  AgentFile,
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
  ToolInfo,
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

let folderSeq = 3;

/** Task folders group agents that work together. Empty by default
    (Patria 20 Okt): the user creates their own folders. */
export const taskFolders = writable<TaskFolder[]>([]);

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
const CUSTOM_AGENT_COLORS = ["#F4A7C3", "#A8D8FF", "#B5E8C0", "#FFD6A5", "#D3BFF0", "#FFF3B0"];

/** Agents created in-app via the "+" menu (mock). */
export const customAgents = writable<Agent[]>([]);

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

function seedProject(id: string, name: string): FlowProject {
  return { id, name };
}

/** All projects, shared by the Connection canvas and the Kanban filter.
    Empty by default (Patria 20 Okt): dummy projects removed; the user
    creates the first real project themselves. */
export const projects = writable<FlowProject[]>(
  loadList("minime.projects", () => []),
);
persistList("minime.projects", projects);

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
  const d = flowMap[resolveProjectId()];
  // Defensive: old/corrupt localStorage may hold {} without nodes/edges.
  const nodes = Array.isArray(d?.nodes) ? d.nodes : [];
  const edges = Array.isArray(d?.edges) ? d.edges : [];
  flowNodes.set(nodes);
  flowEdges.set(edges);
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

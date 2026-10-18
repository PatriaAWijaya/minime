// Central UI state (Phase 1: all local, no backend yet).

import { writable, type Writable } from "svelte/store";
import type {
  AgentStatus,
  AgentTemplate,
  CostRow,
  LibraryItem,
  Message,
  ScheduledItem,
  Skill,
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

/** Per-agent AI choice (placeholder; real routing lands in Phase 2). */
export const agentAiChoice = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-ai"),
);
persistRecord("minime.agent-ai", agentAiChoice);

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

/** Custom instructions per agent (mock, persisted). */
export const agentInstructions = writable<Record<string, string>>(
  loadRecord<string>("minime.agent-instructions"),
);
persistRecord("minime.agent-instructions", agentInstructions);

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

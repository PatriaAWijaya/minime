// Central UI state (Phase 1: all local, no backend yet).

import { writable } from "svelte/store";
import type { AgentStatus, Message, Task } from "./types";

export type View = "chat" | "settings" | "usage" | "tools";

/** Which main view is visible. */
export const activeView = writable<View>("chat");

/** Which agent the chat panel is talking to. */
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

/** Incremented whenever "+ Tugas Baru" is pressed so ChatPanel can focus input. */
export const newTaskSignal = writable(0);

/** Per-agent AI choice (placeholder; real routing lands in Phase 2). */
const AI_KEY = "minime.agent-ai";
function loadAiChoice(): Record<string, string> {
  try {
    return JSON.parse(localStorage.getItem(AI_KEY) ?? "{}");
  } catch {
    return {};
  }
}
export const agentAiChoice = writable<Record<string, string>>(loadAiChoice());
agentAiChoice.subscribe((v) => {
  try {
    localStorage.setItem(AI_KEY, JSON.stringify(v));
  } catch {
    /* ignore */
  }
});

/** Seeded demo tasks for the running-tasks monitor. */
let taskSeq = 1;
export const tasks = writable<Task[]>([
  { id: taskSeq++, titleKey: "monitor.mock1", agentId: "riset", progress: 45, done: false },
  { id: taskSeq++, titleKey: "monitor.mock2", agentId: "kreatif", progress: 80, done: false },
]);

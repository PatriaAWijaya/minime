// Adapter daemon: pakai Tauri invoke bila tersedia, fallback ke mock (mode demo).
// Di aplikasi Tauri, src-tauri mem-proxy perintah ini ke daemon via Unix socket.

import type { Agent, Approval, BrowserSession, DaemonStatus, Task, ToolStatus } from "./types";

declare global {
  interface Window {
    __TAURI__?: unknown;
  }
}

export const isTauri: boolean =
  typeof window !== "undefined" && "__TAURI__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }
  return mockInvoke<T>(cmd, args);
}

// ---------- Mock untuk mode demo (vite dev tanpa Tauri) ----------

const MOCK_AGENTS: Agent[] = [
  {
    id: "mimi", name: "Mimi", role: "Orkestrator", model: "gemini-2.5-pro",
    provider_id: "gemini", avatar_path: "assets/avatars/minime-standard.jpg",
    color: "#B71C1C", status: "idle", skills: "[]", tools_allowlist: '["pipeline"]',
    effort: "medium", paused: false, daily_budget_usd: 5,
  },
  {
    id: "riset", name: "Riset", role: "Senior Researcher", model: "gemini-2.5-pro",
    provider_id: "gemini", avatar_path: "assets/avatars/riset.webp",
    color: "#8E24AA", status: "working", skills: '["riset-web"]', tools_allowlist: '["browser"]',
    effort: "medium", paused: false, daily_budget_usd: 3,
  },
  {
    id: "strategi-bisnis", name: "Strategi Bisnis", role: "Senior Strategist", model: "deepseek-chat",
    provider_id: "deepseek", avatar_path: "assets/avatars/strategi-bisnis.webp",
    color: "#1A237E", status: "idle",
    skills: '["branding-playbook","marketing-playbook","selling-playbook","operational-playbook","finance-playbook","code-builder-playbook"]',
    tools_allowlist: "[]", effort: "high", paused: false, daily_budget_usd: 5,
  },
  {
    id: "creative-director", name: "Creative Director", role: "Senior Creative", model: "gemini-2.5-pro",
    provider_id: "gemini", avatar_path: "assets/avatars/creative.webp",
    color: "#1B5E20", status: "idle",
    skills: '["storytelling","copywriting","content","smo-playbook"]',
    tools_allowlist: '["canva","affinity","open-creator","voice-studio","hyperframes","instagram-cli"]',
    effort: "high", paused: false, daily_budget_usd: 8,
  },
  {
    id: "qc", name: "QC", role: "Senior Judge", model: "gemini-2.5-flash",
    provider_id: "gemini", avatar_path: "assets/avatars/qc.webp",
    color: "#F9A825", status: "idle", skills: '["qc-rubrik"]', tools_allowlist: "[]",
    effort: "low", paused: false, daily_budget_usd: 2,
  },
];

async function mockInvoke<T>(cmd: string, _args?: Record<string, unknown>): Promise<T> {
  await new Promise((r) => setTimeout(r, 150)); // simulasi latensi IPC
  switch (cmd) {
    case "agents_list":
      return MOCK_AGENTS as unknown as T;
    case "daemon_status":
      return {
        version: "0.1.0-demo", uptime_secs: 0, agents: 5, queued_tasks: 0, running_tasks: 1,
      } as unknown as T;
    case "tasks_list":
      return [] as unknown as T;
    case "approvals_list":
      return [] as unknown as T;
    case "approvals_decide":
      return { ok: true } as unknown as T;
    case "tools_status":
      return MOCK_TOOLS as unknown as T;
    case "browser_list":
      return { sessions: [] } as unknown as T;
    case "secrets_set":
      return { stored: (_args as Record<string, unknown>)?.account ?? "" } as unknown as T;
    case "secrets_has":
      return { account: (_args as Record<string, unknown>)?.account ?? "", has: false } as unknown as T;
    case "usage_summary":
      return MOCK_USAGE as unknown as T;
    default:
      throw new Error(`Perintah ${cmd} belum tersedia di mode demo (Fase 1)`);
  }
}

const MOCK_USAGE = [
  { agent_id: "mimi", agent_name: "Mimi", daily_budget_usd: 5, spent_today_usd: 0.12, spent_total_usd: 1.4, tokens_total: 52000 },
  { agent_id: "riset", agent_name: "Riset", daily_budget_usd: 3, spent_today_usd: 0.08, spent_total_usd: 0.9, tokens_total: 38000 },
  { agent_id: "strategi-bisnis", agent_name: "Strategi Bisnis", daily_budget_usd: 5, spent_today_usd: 0, spent_total_usd: 0.3, tokens_total: 12000 },
  { agent_id: "creative-director", agent_name: "Creative Director", daily_budget_usd: 8, spent_today_usd: 0.21, spent_total_usd: 2.1, tokens_total: 84000 },
  { agent_id: "qc", agent_name: "QC", daily_budget_usd: 2, spent_today_usd: 0.02, spent_total_usd: 0.4, tokens_total: 15000 },
];

// ---------- API publik ----------

const MOCK_TOOLS: ToolStatus[] = [
  { id: "canva", label: "Canva", kind: "cli", ready: true, state: "ready", detail: "Terhubung (mode demo).", fallback: "Render lokal via HyperFrames." },
  { id: "affinity", label: "Affinity", kind: "mcp-http", ready: false, state: "unreachable", detail: "Bridge di MacBook (mode demo).", fallback: "Olah desain via Canva / HyperFrames." },
  { id: "open-creator", label: "OpenCreator", kind: "sidecar", ready: false, state: "not_installed", detail: "Sidecar belum dipasang (mode demo).", fallback: "Render video via HyperFrames." },
  { id: "voice-studio", label: "VoiceStudio", kind: "rest", ready: false, state: "unreachable", detail: "Instalasi MacBook pending (mode demo).", fallback: "Fallback: edge-tts." },
  { id: "hyperframes", label: "HyperFrames", kind: "cli", ready: true, state: "ready", detail: "Siap render MP4 lokal (mode demo).", fallback: "Render visual via Canva." },
  { id: "instagram-cli", label: "Instagram CLI", kind: "cli", ready: true, state: "ready", detail: "Mode DRAFT-ONLY (mode demo).", fallback: "Draft diserahkan ke Patria." },
  { id: "browser", label: "Browser bawaan", kind: "browser", ready: true, state: "ready", detail: "Chromium headless (mode demo).", fallback: "Riset via Gemini." },
];

export const api = {
  daemonStatus: () => invoke<DaemonStatus>("daemon_status"),
  agentsList: () => invoke<Agent[]>("agents_list"),
  tasksList: (filter?: { agent_id?: string; status?: string }) =>
    invoke<Task[]>("tasks_list", filter),
  tasksEnqueue: (agent_id: string, title: string) =>
    invoke<{ task_id: string }>("tasks_enqueue", { agent_id, title }),
  approvalsList: (status?: string) =>
    invoke<Approval[]>("approvals_list", { status }),
  approvalsDecide: (approval_id: string, decision: "approve" | "reject", note?: string) =>
    invoke<{ approval_id: string; decision: string }>("approvals_decide", {
      approval_id,
      decision,
      note: note ?? "",
    }),
  toolsStatus: () => invoke<ToolStatus[]>("tools_status"),
  browserList: () => invoke<{ sessions: BrowserSession[] }>("browser_list"),
  secretsSet: (account: string, value: string) =>
    invoke<{ stored: string }>("secrets_set", { account, value }),
  secretsHas: (account: string) =>
    invoke<{ account: string; has: boolean }>("secrets_has", { account }),
  usageSummary: () => invoke<UsageRow[]>("usage_summary"),
};

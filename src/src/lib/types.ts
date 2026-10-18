// Tipe data Minime — mirror skema SQLite daemon (arsitektur §7).

export interface Agent {
  id: string;
  name: string;
  role: string;
  model: string;
  provider_id: string;
  avatar_path: string;
  color: string;
  status: "idle" | "working" | "attention" | string;
  skills: string; // JSON array
  tools_allowlist: string; // JSON array
  effort: string;
  paused: boolean;
  daily_budget_usd: number;
}

export interface Task {
  id: string;
  agent_id: string;
  run_id: string | null;
  title: string;
  status: "queued" | "running" | "paused" | "done" | "failed" | "cancelled" | string;
  progress: number;
  result_summary: string;
  error: string;
  created_at: string;
}

export interface Approval {
  id: string;
  run_id: string | null;
  stage: string;
  payload_summary: string;
  qc_score: number | null;
  status: "pending" | "approved" | "rejected" | string;
  created_at: string;
}

export interface DaemonStatus {
  version: string;
  uptime_secs: number;
  agents: number;
  queued_tasks: number;
  running_tasks: number;
}

export function statusLabel(status: string): string {
  switch (status) {
    case "working":
      return "Aktif";
    case "idle":
      return "Idle";
    case "attention":
      return "Butuh perhatian";
    default:
      return status;
  }
}

export function avatarUrl(avatarPath: string): string {
  const base = avatarPath.split("/").pop() ?? "minime-standard.jpg";
  return `/avatars/${base}`;
}

export interface ToolStatus {
  id: string;
  label: string;
  kind: string;
  ready: boolean;
  state: string;
  detail: string;
  fallback: string;
}

export interface BrowserSession {
  id: string;
  agent_id: string;
  created_at: string;
  last_url: string;
  shots: number;
}

export function toolStateLabel(state: string): string {
  switch (state) {
    case "ready":
      return "Siap";
    case "not_installed":
      return "Belum terinstal";
    case "not_configured":
      return "Belum dikonfigurasi";
    case "no_key":
      return "Butuh kunci API";
    case "unreachable":
      return "Tidak terjangkau";
    default:
      return state;
  }
}

export interface UsageRow {
  agent_id: string;
  agent_name: string;
  daily_budget_usd: number;
  spent_today_usd: number;
  spent_total_usd: number;
  tokens_total: number;
}

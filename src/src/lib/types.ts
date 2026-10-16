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
  const base = avatarPath.split("/").pop() ?? "mimi-orkestrator.webp";
  return `/avatars/${base}`;
}

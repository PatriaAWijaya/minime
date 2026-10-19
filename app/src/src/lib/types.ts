// Shared domain types for Minime v2 Phase 1 (UI mock).

export type AgentStatus = "idle" | "working" | "attention";

export interface Agent {
  id: string;
  /** Plain display name (empty when nameKey is used instead). */
  name: string;
  /** i18n key for the display name — used by custom agents created in-app. */
  nameKey?: string;
  /** i18n key for the agent's plain-language role description */
  roleKey: string;
  /** Display label of the AI model behind this agent */
  model: string;
  /** Single initial letter shown on the placeholder badge */
  badge: string;
  /** Solid badge background color (flat, no gradients) */
  color: string;
}

export interface Message {
  id: number;
  role: "user" | "agent";
  text: string;
  time: string;
}

export type ScheduleRange = "day" | "week" | "month";

export interface ScheduledItem {
  id: number;
  nameKey: string;
  whenKey: string;
  range: ScheduleRange;
  active: boolean;
}

export type LibraryType = "doc" | "table" | "deck" | "image" | "video" | "app";

export interface LibraryItem {
  id: number;
  titleKey: string;
  dateKey: string;
  /** Approximate age in days, used by the mock time filter */
  daysAgo: number;
  type: LibraryType;
  agentId: string;
  taskKey: string;
}

export interface CostRow {
  id: number;
  agentId: string;
  tokens: number;
  /** USD, mock */
  costUsd: number;
  dateKey: string;
  /** Approximate age in days, used by the mock range filter */
  daysAgo: number;
}

export interface AgentTemplate {
  id: number;
  nameKey: string;
  descKey: string;
  enabled: boolean;
}

export interface Skill {
  id: number;
  /** i18n key, or customName when created by the user */
  nameKey?: string;
  customName?: string;
  descKey?: string;
  installed: boolean;
}

export interface ProviderInfo {
  id: string;
  /** Display name (brand name, not translated) */
  name: string;
  kind: "free" | "paid";
  /** i18n key for the tag chip, e.g. "providers.defaultFree" */
  tagKey?: string;
  /** true → status shows "available on this computer" */
  local?: boolean;
}

/** Which model server is in use (mock). */
export type ServerKind = "local" | "cloud";

/** Server menu layout preference (mock). */
export type ServerLayout = "rail" | "menu";

/** A task folder groups several agents that work together (mock). */
export interface TaskFolder {
  id: number;
  /** i18n key for the folder name; {n} replaced by num when present */
  nameKey: string;
  num?: number;
  agentIds: string[];
  open: boolean;
}

/** Kanban column id (mock task board). */
export interface AgentFile {
  name: string;
  size: string;
}

export type KanbanCol = "queue" | "doing" | "approval" | "revision" | "done";

/** A task card on the Kanban board (mock). */
export interface KanbanCard {
  id: number;
  /** Card title, plain text entered by the user */
  title: string;
  agentId: string;
  col: KanbanCol;
  /** Project this card belongs to (shared with the Connection view). */
  projectId: string;
  /** Display date string, e.g. "18 Okt" */
  createdAt: string;
  /** Creation timestamp (ms) used by the period filter. */
  createdAtTs: number;
  /** Draft konten untuk Meta/Postiz (Tahap 3: draft-only, tidak auto-post). */
  draftContent?: string;
  /** Target platform draft: "meta" | "postiz" | null. */
  draftPlatform?: string;
}

/** A node on the connection canvas (mock). */
export interface FlowNode {
  id: string;
  agentId: string;
  x: number;
  y: number;
}

/** A connection between two canvas nodes (mock). */
export interface FlowEdge {
  id: string;
  source: string;
  target: string;
}

/** A named project owning its own connection-canvas content (mock). */
export interface FlowProject {
  id: string;
  name: string;
}

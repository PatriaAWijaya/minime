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
  /** 3D animal avatar image path (Patria's direction, 21 Oct 2026) */
  avatar?: string;
}

export interface AttachmentMeta {
  name: string;
  kind: "image" | "audio" | "video" | "document";
  size: string;
}

/** Tool registry entry from the Rust tools gateway (Phase 3.1). */
export interface ToolInfo {
  id: string;
  name: string;
  category: string;
  connected: boolean;
  requires_auth: boolean;
  draft_only: boolean;
}

export interface Message {
  id: number;
  role: "user" | "agent";
  text: string;
  time: string;
  attachments?: AttachmentMeta[];
  reaction?: string;
  replyTo?: string;
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
  /** Plain title for REAL items (Tahap 3.6); falls back to titleKey. */
  title?: string;
  /** Plain one-line summary of a real tool result. */
  summary?: string;
  /** Full output content for viewing/downloading/reuse. */
  content?: string;
  /** Plain date label for real items; falls back to dateKey. */
  dateLabel?: string;
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

/** Server menu layout preference (mock). */

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
  /** Social draft content (Phase 3.3: Meta/Postiz default to draft-only). */
  draftContent?: string;
  /** Target platform for social drafts: "meta" | "postiz". */
  draftPlatform?: string;
  /** True when the user enabled autoposting for the platform and this
      card skipped the approval column (Phase 3.3, user opt-in). */
  autoPosted?: boolean;
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

/** A named project owning its own connection-canvas content.
 *  Patria 20 Okt: the Agents-view folder IS the project — agents placed in
 *  the folder are the agents assigned to work on this project. */
export interface FlowProject {
  id: string;
  name: string;
  /** Agent ids assigned to this project (the folder members). */
  agentIds: string[];
}

/** Project workflow (Patria 20 Okt): user buat project → brief awal +
    to-do list + goals di Mimi → Mimi auto-delegasi ke agen → agen kerja
    mandiri & lapor progres ke Library → user pantau selesai di Kanban. */
export interface ProjectBrief {
  projectId: string;
  brief: string;
  goals: string;
  initiated: boolean;
}

export interface ProjectTodo {
  id: string;
  projectId: string;
  text: string;
  agentId: string;
  done: boolean;
}

// Shared domain types for Minime v2 Phase 1 (UI mock).

export type AgentStatus = "idle" | "working" | "attention";

export interface Agent {
  id: string;
  name: string;
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

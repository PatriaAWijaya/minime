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

export interface Task {
  id: number;
  titleKey: string;
  agentId: string;
  progress: number; // 0..100
  done: boolean;
}

// Seeded agent roster (Patria's decision, 18 Oct 2026).
// Avatars are placeholder initial-letter badges — v1 character avatars were
// rejected by Patria and no new direction has been given yet.

import type { Agent } from "./types";

// Agent badge colors: pastel palette (Patria's decision, 19 Oct 2026) —
// each agent keeps its own pastel color on the monochrome UI.
export const agents: Agent[] = [
  { id: "mimi", name: "Mimi", roleKey: "agent.mimi.role", model: "Muse", badge: "M", color: "#F4A7C3" },
  { id: "kreatif", name: "Kreatif", roleKey: "agent.kreatif.role", model: "Ollama", badge: "K", color: "#FFD6A5" },
  { id: "riset", name: "Riset", roleKey: "agent.riset.role", model: "Ollama", badge: "R", color: "#A8D8FF" },
  { id: "finance", name: "Finance", roleKey: "agent.finance.role", model: "Ollama", badge: "F", color: "#B5E8C0" },
  { id: "strategis", name: "Bisnis Strategis", roleKey: "agent.strategis.role", model: "Ollama", badge: "B", color: "#D3BFF0" },
  { id: "code", name: "Code Builder", roleKey: "agent.code.role", model: "Ollama", badge: "C", color: "#FFF3B0" },
];

export function getAgent(id: string): Agent {
  return agents.find((a) => a.id === id) ?? agents[0];
}

/** Single source of truth for agent display name (Patria 20 Okt): user
 *  overrides win, so an edit in the Agents panel instantly reflects in
 *  Tools, Tasks, Brief, Kanban — everywhere. */
export function agentDisplayName(
  a: Pick<Agent, "id" | "name" | "nameKey">,
  names: Record<string, string>,
  t: (key: string) => string,
): string {
  return names[a.id] ?? (a.nameKey ? t(a.nameKey) : a.name);
}

/** Single source of truth for agent display role (same sync rule). */
export function agentDisplayRole(
  a: Pick<Agent, "id" | "roleKey">,
  titles: Record<string, string>,
  t: (key: string) => string,
): string {
  return titles[a.id] ?? t(a.roleKey);
}

// Seeded agent roster (Patria's decision, 18 Oct 2026).
// Avatars are placeholder initial-letter badges — v1 character avatars were
// rejected by Patria and no new direction has been given yet.

import type { Agent } from "./types";

export const agents: Agent[] = [
  { id: "mimi", name: "Mimi", roleKey: "agent.mimi.role", model: "Muse", badge: "M", color: "#e11d48" },
  { id: "kreatif", name: "Kreatif", roleKey: "agent.kreatif.role", model: "Gemini 3.8", badge: "K", color: "#d97706" },
  { id: "riset", name: "Riset", roleKey: "agent.riset.role", model: "Gemini 3.8", badge: "R", color: "#2563eb" },
  { id: "finance", name: "Finance", roleKey: "agent.finance.role", model: "Gemini 3.8", badge: "F", color: "#059669" },
  { id: "strategis", name: "Bisnis Strategis", roleKey: "agent.strategis.role", model: "Gemini 3.8", badge: "B", color: "#7c3aed" },
  { id: "code", name: "Code Builder", roleKey: "agent.code.role", model: "Gemini 3.8", badge: "C", color: "#0891b2" },
];

export function getAgent(id: string): Agent {
  return agents.find((a) => a.id === id) ?? agents[0];
}

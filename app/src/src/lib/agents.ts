// Seeded agent roster (Patria's decision, 18 Oct 2026).
// Agent avatars: 3D animal renders designed by Patria (19 Oct 2026).
// Mapping: mimi=lion (leader), kreatif=fox (orange, clever),
// riset=elephant (blue, wise), finance=dog (watchdog),
// strategis=cat (strategic hunter), code=monkey (tinkerer).

import type { Agent } from "./types";

// Agent badge colors: pastel palette (Patria's decision, 19 Oct 2026) —
// each agent keeps its own pastel color on the monochrome UI.
export const agents: Agent[] = [
  { id: "mimi", name: "Mimi", roleKey: "agent.mimi.role", model: "Muse", badge: "M", color: "#F4A7C3", avatar: "avatars/lion.png" },
  { id: "kreatif", name: "Kreatif", roleKey: "agent.kreatif.role", model: "Ollama", badge: "K", color: "#FFD6A5", avatar: "avatars/fox.png" },
  { id: "riset", name: "Riset", roleKey: "agent.riset.role", model: "Ollama", badge: "R", color: "#A8D8FF", avatar: "avatars/elephant.png" },
  { id: "finance", name: "Finance", roleKey: "agent.finance.role", model: "Ollama", badge: "F", color: "#B5E8C0", avatar: "avatars/dog.png" },
  { id: "strategis", name: "Bisnis Strategis", roleKey: "agent.strategis.role", model: "Ollama", badge: "B", color: "#D3BFF0", avatar: "avatars/cat.png" },
  { id: "code", name: "Code Builder", roleKey: "agent.code.role", model: "Ollama", badge: "C", color: "#FFF3B0", avatar: "avatars/monkey.png" },
];

/** Preset avatar choices for the avatar picker (Patria's 3D renders). */
export const AVATAR_PRESETS = [
  "avatars/lion.png",
  "avatars/fox.png",
  "avatars/elephant.png",
  "avatars/dog.png",
  "avatars/cat.png",
  "avatars/monkey.png",
];

export function getAgent(id: string): Agent {
  return agents.find((a) => a.id === id) ?? agents[0];
}

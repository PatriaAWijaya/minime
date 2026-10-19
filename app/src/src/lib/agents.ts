// Seeded agent roster (Patria's decision, 18 Oct 2026).
// Agent avatars: 3D animal renders designed by Patria (19 Oct 2026).
// Mapping: mimi=lion (leader), kreatif=fox (orange, clever),
// riset=elephant (blue, wise), finance=dog (watchdog),
// strategis=cat (strategic hunter), code=monkey (tinkerer).

import type { Agent } from "./types";
import lionPng from "../assets/avatars/lion.png";
import foxPng from "../assets/avatars/fox.png";
import elephantPng from "../assets/avatars/elephant.png";
import dogPng from "../assets/avatars/dog.png";
import catPng from "../assets/avatars/cat.png";
import monkeyPng from "../assets/avatars/monkey.png";

// Agent badge colors: pastel palette (Patria's decision, 19 Oct 2026) —
// each agent keeps its own pastel color on the monochrome UI.
export const agents: Agent[] = [
  { id: "mimi", name: "Mimi", roleKey: "agent.mimi.role", model: "Muse", badge: "M", color: "#F4A7C3", avatar: lionPng },
  { id: "kreatif", name: "Kreatif", roleKey: "agent.kreatif.role", model: "Ollama", badge: "K", color: "#FFD6A5", avatar: foxPng },
  { id: "riset", name: "Riset", roleKey: "agent.riset.role", model: "Ollama", badge: "R", color: "#A8D8FF", avatar: elephantPng },
  { id: "finance", name: "Finance", roleKey: "agent.finance.role", model: "Ollama", badge: "F", color: "#B5E8C0", avatar: dogPng },
  { id: "strategis", name: "Bisnis Strategis", roleKey: "agent.strategis.role", model: "Ollama", badge: "B", color: "#D3BFF0", avatar: catPng },
  { id: "code", name: "Code Builder", roleKey: "agent.code.role", model: "Ollama", badge: "C", color: "#FFF3B0", avatar: monkeyPng },
];

/** Preset avatar choices for the avatar picker (Patria's 3D renders). */
export const AVATAR_PRESETS = [
  lionPng,
  foxPng,
  elephantPng,
  dogPng,
  catPng,
  monkeyPng,
];

export function getAgent(id: string): Agent {
  return agents.find((a) => a.id === id) ?? agents[0];
}

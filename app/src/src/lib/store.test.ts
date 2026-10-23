import { describe, it, expect, beforeEach, vi } from "vitest";
import { get } from "svelte/store";

// Mock browser globals before importing the store
const ls: Record<string, string> = {};
vi.stubGlobal("localStorage", {
  getItem: (k: string) => ls[k] ?? null,
  setItem: (k: string, v: string) => {
    ls[k] = v;
  },
  removeItem: (k: string) => {
    delete ls[k];
  },
});

const {
  agentEnabled,
  isAgentEnabled,
  activateAgent,
  deactivateAgent,
} = await import("./store");

describe("agent activation", () => {
  beforeEach(() => {
    for (const k of Object.keys(ls)) delete ls[k];
    agentEnabled.set({});
  });

  it("mimi is always enabled", () => {
    expect(isAgentEnabled("mimi")).toBe(true);
    // deactivating mimi is a no-op
    deactivateAgent("mimi");
    expect(isAgentEnabled("mimi")).toBe(true);
  });

  it("agents start disabled", () => {
    expect(isAgentEnabled("kreatif")).toBe(false);
    expect(isAgentEnabled("riset")).toBe(false);
  });

  it("activateAgent enables the agent", () => {
    activateAgent("kreatif");
    expect(isAgentEnabled("kreatif")).toBe(true);
    expect(get(agentEnabled)["kreatif"]).toBe(true);
  });

  it("deactivateAgent disables the agent", () => {
    activateAgent("kreatif");
    deactivateAgent("kreatif");
    expect(isAgentEnabled("kreatif")).toBe(false);
  });

  it("activation persists to localStorage", () => {
    activateAgent("riset");
    const raw = ls["minime.agent-enabled"];
    expect(raw).toBeDefined();
    expect(JSON.parse(raw)["riset"]).toBe(true);
  });

  it("re-activation is idempotent", () => {
    activateAgent("finance");
    activateAgent("finance");
    expect(isAgentEnabled("finance")).toBe(true);
  });

  it("unknown agent ids are treated as disabled", () => {
    expect(isAgentEnabled("no-such-agent")).toBe(false);
  });
});

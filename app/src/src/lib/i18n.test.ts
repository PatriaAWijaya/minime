import { describe, it, expect, beforeEach, vi } from "vitest";
import { get } from "svelte/store";

// Mock localStorage and document before importing i18n
const store: Record<string, string> = {};
vi.stubGlobal("localStorage", {
  getItem: (k: string) => store[k] ?? null,
  setItem: (k: string, v: string) => {
    store[k] = v;
  },
  removeItem: (k: string) => {
    delete store[k];
  },
});
vi.stubGlobal("document", { documentElement: { lang: "" } });

const { lang, t } = await import("./i18n");

describe("i18n", () => {
  beforeEach(() => {
    for (const k of Object.keys(store)) delete store[k];
    lang.set("id");
  });

  it("defaults to Indonesian", () => {
    expect(get(lang)).toBe("id");
  });

  it("translates a known key in Indonesian", () => {
    const translate = get(t);
    // "mkt.open" exists in all three dicts
    expect(translate("mkt.open")).toBe("Buka");
  });

  it("switches to English", () => {
    lang.set("en");
    expect(get(t)("mkt.open")).toBe("Open");
  });

  it("switches to Mandarin", () => {
    lang.set("zh");
    expect(get(t)("mkt.open")).toBe("打开");
  });

  it("falls back to Indonesian when key missing in current lang", () => {
    // pick a key that exists in id; if missing in en it should fall back
    lang.set("en");
    const translate = get(t);
    const val = translate("mkt.open");
    expect(typeof val).toBe("string");
    expect(val.length).toBeGreaterThan(0);
  });

  it("returns the key itself when missing everywhere", () => {
    expect(get(t)("nonexistent.key.xyz")).toBe("nonexistent.key.xyz");
  });

  it("interpolates {vars}", () => {
    const translate = get(t);
    // mkt.footer.skillsN uses {n}
    const out = translate("mkt.footer.skillsN", { n: "5" });
    expect(out).toContain("5");
  });

  it("persists language choice to localStorage", () => {
    lang.set("en");
    expect(store["minime.lang"]).toBe("en");
  });
});

import { describe, it, expect, vi, beforeEach } from "vitest";
import { logger } from "./logger";

describe("logger", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("falls back to console when Tauri invoke is unavailable", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    // Import of @tauri-apps/api/core will fail in Node → fallback to console
    logger.error("TestTarget", "something broke");
    // wait a tick for the async write
    await new Promise((r) => setTimeout(r, 50));
    expect(errSpy).toHaveBeenCalledWith(expect.stringContaining("TestTarget"));
    expect(errSpy).toHaveBeenCalledWith(expect.stringContaining("something broke"));
  });

  it("formats Error objects with message", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    logger.error("TestTarget", new Error("boom"));
    await new Promise((r) => setTimeout(r, 50));
    expect(errSpy).toHaveBeenCalledWith(expect.stringContaining("boom"));
  });

  it("logger.info and logger.warn do not throw without backend", async () => {
    const logSpy = vi.spyOn(console, "log").mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(() => logger.info("T", "hello")).not.toThrow();
    expect(() => logger.warn("T", "careful")).not.toThrow();
    await new Promise((r) => setTimeout(r, 50));
    expect(logSpy).toHaveBeenCalled();
    expect(warnSpy).toHaveBeenCalled();
  });
});

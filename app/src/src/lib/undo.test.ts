import { describe, it, expect } from "vitest";
import { createUndo } from "./undo";

describe("createUndo", () => {
  it("starts empty with no undo/redo available", () => {
    const u = createUndo<string>(20);
    expect(u.size).toBe(0);
    expect(u.canUndo).toBe(false);
    expect(u.canRedo).toBe(false);
    expect(u.peek()).toBeUndefined();
    expect(u.undo()).toBeUndefined();
    expect(u.redo()).toBeUndefined();
  });

  it("pushes snapshots and undoes to the previous one", () => {
    const u = createUndo<string>(20);
    u.push("a");
    expect(u.canUndo).toBe(false); // single snapshot: nothing to undo to
    u.push("b");
    expect(u.canUndo).toBe(true);
    expect(u.undo()).toBe("a");
    expect(u.canUndo).toBe(false);
    expect(u.canRedo).toBe(true);
  });

  it("redoes after undo", () => {
    const u = createUndo<string>(20);
    u.push("a");
    u.push("b");
    u.undo();
    expect(u.redo()).toBe("b");
    expect(u.canRedo).toBe(false);
    expect(u.canUndo).toBe(true);
  });

  it("truncates the redo branch on new push", () => {
    const u = createUndo<string>(20);
    u.push("a");
    u.push("b");
    u.undo(); // back at "a", "b" is redo
    u.push("c"); // "b" must be dropped
    expect(u.canRedo).toBe(false);
    expect(u.peek()).toBe("c");
    expect(u.undo()).toBe("a");
    expect(u.undo()).toBeUndefined();
  });

  it("caps history at maxDepth (default 20)", () => {
    const u = createUndo<number>(20);
    for (let i = 0; i < 25; i++) u.push(i);
    expect(u.size).toBe(20);
    // oldest kept should be 5 (0..4 shifted out)
    // walk back 19 steps to the oldest
    let val: number | undefined;
    for (let i = 0; i < 19; i++) val = u.undo();
    expect(val).toBe(5);
    expect(u.canUndo).toBe(false);
  });

  it("respects a custom maxDepth", () => {
    const u = createUndo<number>(3);
    u.push(1);
    u.push(2);
    u.push(3);
    u.push(4);
    expect(u.size).toBe(3);
    expect(u.undo()).toBe(3);
    expect(u.undo()).toBe(2);
    expect(u.undo()).toBeUndefined();
  });

  it("clear() resets everything", () => {
    const u = createUndo<string>(20);
    u.push("a");
    u.push("b");
    u.clear();
    expect(u.size).toBe(0);
    expect(u.canUndo).toBe(false);
    expect(u.canRedo).toBe(false);
    expect(u.peek()).toBeUndefined();
  });

  it("rejects invalid maxDepth", () => {
    expect(() => createUndo(0)).toThrow();
    expect(() => createUndo(-5)).toThrow();
    expect(() => createUndo(1.5)).toThrow();
  });

  it("works with object snapshots (serialized)", () => {
    const u = createUndo<string>(20);
    u.push(JSON.stringify({ tracks: 1 }));
    u.push(JSON.stringify({ tracks: 2 }));
    const prev = u.undo();
    expect(JSON.parse(prev!).tracks).toBe(1);
  });

  it("reset() jumps back to the first snapshot", () => {
    const u = createUndo<string>(20);
    expect(u.reset()).toBeUndefined();
    u.push("a");
    u.push("b");
    u.push("c");
    expect(u.reset()).toBe("a");
    expect(u.canUndo).toBe(false);
    expect(u.canRedo).toBe(true);
    expect(u.redo()).toBe("b");
  });
});

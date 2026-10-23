/**
 * Generic undo history with a fixed maximum depth.
 *
 * Usage:
 * ```ts
 * const undo = createUndo<string>(20);
 * undo.push(currentValue);   // snapshot after each mutation
 * if (undo.canUndo) {
 *   const prev = undo.undo(); // returns previous snapshot
 * }
 * undo.clear();
 * ```
 *
 * Snapshots are stored as-is; for objects, pass a serialized copy
 * (e.g. JSON.stringify) or a structured clone to avoid mutation leaks.
 */
export interface UndoStack<T> {
  /** Push a new snapshot. Truncates any redo branch. */
  push(snapshot: T): void;
  /** Step back one snapshot. Returns the snapshot, or undefined if empty. */
  undo(): T | undefined;
  /** Step forward one snapshot. Returns the snapshot, or undefined if at tip. */
  redo(): T | undefined;
  /** Whether undo() would return a snapshot. */
  readonly canUndo: boolean;
  /** Whether redo() would return a snapshot. */
  readonly canRedo: boolean;
  /** Number of stored snapshots. */
  readonly size: number;
  /** Drop all snapshots. */
  clear(): void;
  /** Peek at the current (tip) snapshot without moving. */
  peek(): T | undefined;
  /** Jump back to the first snapshot. Returns it, or undefined if empty. */
  reset(): T | undefined;
}

export function createUndo<T>(maxDepth = 20): UndoStack<T> {
  if (!Number.isInteger(maxDepth) || maxDepth < 1) {
    throw new Error(`maxDepth must be a positive integer, got ${maxDepth}`);
  }
  let history: T[] = [];
  let idx = -1;

  return {
    push(snapshot: T): void {
      history = history.slice(0, idx + 1);
      history.push(snapshot);
      if (history.length > maxDepth) history.shift();
      idx = history.length - 1;
    },
    undo(): T | undefined {
      if (idx <= 0) return undefined;
      idx--;
      return history[idx];
    },
    redo(): T | undefined {
      if (idx >= history.length - 1) return undefined;
      idx++;
      return history[idx];
    },
    get canUndo(): boolean {
      return idx > 0;
    },
    get canRedo(): boolean {
      return idx < history.length - 1;
    },
    get size(): number {
      return history.length;
    },
    clear(): void {
      history = [];
      idx = -1;
    },
    peek(): T | undefined {
      return idx >= 0 ? history[idx] : undefined;
    },
    reset(): T | undefined {
      if (history.length === 0) return undefined;
      idx = 0;
      return history[0];
    },
  };
}

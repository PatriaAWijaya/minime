// Shared persistence helpers untuk semua modul store.
import { writable, type Writable } from "svelte/store";

export function loadString(key: string, fallback: string): string {
  try {
    const v = localStorage.getItem(key);
    return v !== null ? v : fallback;
  } catch {
    return fallback;
  }
}

export function loadBool(key: string, fallback: boolean): boolean {
  try {
    const v = localStorage.getItem(key);
    if (v === "1") return true;
    if (v === "0") return false;
    return fallback;
  } catch {
    return fallback;
  }
}

export function loadJSON<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    if (!v) return fallback;
    return JSON.parse(v) as T;
  } catch {
    return fallback;
  }
}

export function loadRecord<T>(key: string): Record<string, T> {
  return loadJSON<Record<string, T>>(key, {});
}

export function persistRecord<T>(key: string, store: Writable<Record<string, T>>) {
  store.subscribe((v) => {
    try {
      localStorage.setItem(key, JSON.stringify(v));
    } catch {
      /* ignore */
    }
  });
}

export function persistString(key: string, store: Writable<string>) {
  store.subscribe((v) => {
    try {
      localStorage.setItem(key, v);
    } catch {
      /* ignore */
    }
  });
}

export function loadList<T>(key: string, seed: () => T[]): T[] {
  try {
    const raw = localStorage.getItem(key);
    if (raw) {
      const parsed = JSON.parse(raw);
      // Defensive: corrupt data may be {} instead of [].
      if (Array.isArray(parsed)) return parsed as T[];
    }
  } catch {
    /* fall through to seed */
  }
  return seed();
}

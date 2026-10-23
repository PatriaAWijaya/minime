/**
 * Lightweight frontend logger — writes to ~/.minime/logs/ via the backend.
 *
 * Levels: info, warn, error. Falls back to console when the Tauri
 * backend is unavailable (e.g. in tests or web preview).
 */

export type LogLevel = "info" | "warn" | "error";

async function write(level: LogLevel, target: string, message: string): Promise<void> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("app_log", { level, target, message });
  } catch {
    // Backend unavailable — fall back to console so nothing is lost.
    const fn = level === "error" ? console.error : level === "warn" ? console.warn : console.log;
    fn(`[${target}] ${message}`);
  }
}

export const logger = {
  info(target: string, message: string): void {
    void write("info", target, message);
  },
  warn(target: string, message: string): void {
    void write("warn", target, message);
  },
  error(target: string, message: string | unknown): void {
    const msg = message instanceof Error ? `${message.message}\n${message.stack ?? ""}` : String(message);
    void write("error", target, msg);
  },
};

/** Read the last N log lines (for the diagnostics view). */
export async function readLogs(lines = 200): Promise<string[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string[]>("read_logs", { lines });
}

/** Delete log files older than keepDays (default 14). Returns count removed. */
export async function pruneLogs(keepDays = 14): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<number>("prune_logs", { keepDays });
}

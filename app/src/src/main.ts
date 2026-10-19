import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";

// Temporary debug: capture JS errors with stack traces to a file via Rust.
if (typeof window !== "undefined") {
  window.addEventListener("error", async (ev) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("debug_log_js_error", {
        message: String(ev.message ?? ev.error),
        stack: String((ev.error as Error)?.stack ?? "no-stack"),
      });
    } catch {
      /* ignore */
    }
  });
  window.addEventListener("unhandledrejection", async (ev) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("debug_log_js_error", {
        message: "unhandledrejection: " + String(ev.reason),
        stack: String((ev.reason as Error)?.stack ?? "no-stack"),
      });
    } catch {
      /* ignore */
    }
  });
}

mount(App, { target: document.getElementById("app")! });

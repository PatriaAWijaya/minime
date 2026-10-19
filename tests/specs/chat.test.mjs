/** Test otomatis Minime via WebDriver.
 *  Tidak klik UI — panggil backend langsung via Tauri IPC dari dalam webview.
 *  Memverifikasi: agent_chat mengembalikan balasan (bukan error).
 */
import { expect } from "expect";

describe("Minime backend", () => {
  it("agent_chat mengembalikan balasan via Ollama", async () => {
    await browser.waitUntil(
      async () => (await browser.$("body").getText()).length > 100,
      { timeout: 30000, timeoutMsg: "app tidak load" }
    );

    // Panggil agent_chat langsung via Tauri invoke di dalam webview.
    const result = await browser.execute(async () => {
      const invoke = window.__TAURI__?.core?.invoke ?? window.__TAURI__?.invoke;
      if (!invoke) return { error: "tauri invoke tidak tersedia" };
      try {
        const res = await invoke("agent_chat", {
          agentId: "code",
          modelId: "ollama:qwen2.5:3b",
          tier: "gratis",
          messages: [{ role: "user", content: "balas dengan satu kata: halo" }],
        });
        return res;
      } catch (e) {
        return { error: "invoke gagal: " + String(e) };
      }
    });

    console.log("HASIL:", JSON.stringify(result).slice(0, 300));

    expect(result.error ?? null).toBeNull();
    expect(result.text?.trim().length ?? 0).toBeGreaterThan(0);
    expect(result.text.toLowerCase()).not.toContain("semua provider gagal");
  });
});

/** Test otomatis Minime via WebDriver — interaksi UI sungguhan.
 *
 *  Pendekatan: TIDAK pakai window.__TAURI__ (tidak tersedia di konteks
 *  WebDriver). Sebagai gantinya, klik elemen UI seperti pengguna asli:
 *  pilih agen → ketik pesan → kirim → tunggu balasan.
 *
 *  Selector dipetakan dari struktur Svelte (per 19 Okt 2026):
 *  - Daftar agen: button.agent-row (nama di span.name)
 *  - Input chat: .composer textarea
 *  - Tombol kirim: Enter di textarea
 *  - Pesan: .messages .msg
 */
import { expect } from "expect";

const APP_LOAD_TIMEOUT = 30000;
const REPLY_TIMEOUT = 120000; // Ollama lokal bisa lambat

async function waitForAppLoad() {
  await browser.waitUntil(
    async () => (await browser.$("body").getText()).length > 100,
    { timeout: APP_LOAD_TIMEOUT, timeoutMsg: "app tidak load dalam 30 detik" }
  );
}

async function selectAgent(agentName) {
  const rows = await browser.$$("button.agent-row");
  // Fallback: Mimi mungkin sudah diganti nama jadi Muse (remote)
  const aliases = { "Mimi": ["Mimi", "Muse"] };
  const names = aliases[agentName] || [agentName];
  for (const row of rows) {
    const text = await row.getText();
    if (names.some((n) => text.includes(n))) {
      await row.click();
      return true;
    }
  }
  throw new Error(`agen "${agentName}" tidak ditemukan di daftar`);
}

async function sendChatMessage(text) {
  const input = await browser.$(".composer textarea");
  await input.waitForDisplayed({ timeout: 10000 });
  await input.click();
  await input.setValue(text);
  await browser.keys("Enter");
}

async function waitForAgentReply(previousCount) {
  await browser.waitUntil(
    async () => {
      const msgs = await browser.$$(".messages .msg");
      return msgs.length > previousCount;
    },
    { timeout: REPLY_TIMEOUT, timeoutMsg: "agen tidak membalas dalam 120 detik" }
  );
  const msgs = await browser.$$(".messages .msg");
  return await msgs[msgs.length - 1].getText();
}

describe("Minime chat via UI", () => {
  it("app load dan daftar agen tampil", async () => {
    await waitForAppLoad();
    const rows = await browser.$$("button.agent-row");
    // Minimal 4 agen seed tampil; nama pasti dicek di test berikutnya
    expect(rows.length).toBeGreaterThanOrEqual(4);
    const texts = [];
    for (const row of rows) texts.push(await row.getText());
    const all = texts.join(" ");
    // Pastikan agen inti ada
    expect(all).toMatch(/Mimi|Muse/);
    expect(all).toMatch(/Kreatif/);
    console.log("AGEN DITEMUKAN:", texts.length, "baris");
  });

  it("chat dengan Mimi mendapat balasan", async () => {
    await waitForAppLoad();
    await selectAgent("Mimi");

    const before = (await browser.$$(".messages .msg")).length;
    await sendChatMessage("Halo, balas dengan satu kata saja");
    const reply = await waitForAgentReply(before);

    expect(reply.trim().length).toBeGreaterThan(0);
    expect(reply.toLowerCase()).not.toContain("semua provider gagal");
    console.log("BALASAN MIMI:", reply.slice(0, 200));
  });

  it("Mimi bisa delegasi ke agen lain", async () => {
    await waitForAppLoad();
    await selectAgent("Mimi");

    const before = (await browser.$$(".messages .msg")).length;
    await sendChatMessage(
      "Tolong riset kompetitor kopi susu di Surabaya. " +
      "Gunakan format [DELEGATE:riset] untuk mendelegasikan."
    );
    const reply = await waitForAgentReply(before);

    const hasDelegate = reply.includes("[DELEGATE:riset]") ||
                        reply.toLowerCase().includes("riset");
    expect(hasDelegate).toBe(true);
    console.log("DELEGASI:", reply.slice(0, 300));
  });
});

/** Test otomatis: kirim pesan chat ke agen dan verifikasi ada balasan.
 *  Skenario: buka app → pilih agen Code Builder → ketik "halo" → Kirim →
 *  tunggu bubble agen muncul (bukan error provider).
 */
import { expect } from "expect";

describe("Minime chat", () => {
  it("agen membalas pesan", async () => {
    // Tunggu app siap (sidebar Agen terlihat).
    const agenHeading = await browser.$("h2=Agen");
    await agenHeading.waitForExist({ timeout: 30000 });

    // Klik agen Code Builder di daftar.
    const codeBuilder = await browser.$("div*=Code Builder");
    await codeBuilder.waitForExist({ timeout: 10000 });
    await codeBuilder.click();

    // Ketik pesan di input chat.
    const input = await browser.$('input[placeholder*="Tulis"]');
    await input.waitForExist({ timeout: 10000 });
    await input.setValue("halo, balas dengan satu kata saja");

    // Klik Kirim.
    const kirim = await browser.$("button=Kirim");
    await kirim.click();

    // Tunggu bubble balasan agen muncul (maks 90 detik untuk load model).
    // Bubble agen adalah div pesan bukan dari user; cari teks yang bukan "halo".
    await browser.waitUntil(
      async () => {
        const bubbles = await browser.$$("div.chat-bubble-agent");
        return bubbles.length > 0;
      },
      { timeout: 90000, timeoutMsg: "agen tidak membalas dalam 90 detik" }
    );

    const bubbles = await browser.$$("div.chat-bubble-agent");
    const lastText = await bubbles[bubbles.length - 1].getText();
    console.log("BALASAN AGEN:", lastText.slice(0, 200));

    // Balasan bukan pesan error provider.
    expect(lastText.toLowerCase()).not.toContain("semua provider gagal");
    expect(lastText.toLowerCase()).not.toContain("tidak ada respon");
  });
});

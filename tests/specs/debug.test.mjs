/** Debug: lihat struktur DOM aplikasi. */
describe("debug", () => {
  it("tampilkan struktur", async () => {
    await browser.waitUntil(
      async () => (await browser.$("body").getText()).length > 100,
      { timeout: 30000 }
    );
    const html = await browser.execute(() => document.body.innerHTML.slice(0, 3000));
    console.log("HTML:", html.replace(/\n/g, " ").slice(0, 2000));
    const tauri = await browser.execute(() => ({
      hasTauri: !!window.__TAURI__,
      keys: window.__TAURI__ ? Object.keys(window.__TAURI__) : [],
      hasCore: !!window.__TAURI__?.core,
    }));
    console.log("TAURI:", JSON.stringify(tauri));
  });
});

/** Dump struktur DOM untuk perbaikan selector. */
describe("dom dump", () => {
  it("tampilkan elemen kunci", async () => {
    await browser.waitUntil(
      async () => (await browser.$("body").getText()).length > 100,
      { timeout: 30000 }
    );
    const info = await browser.execute(() => {
      const out = {};
      // Cari semua elemen yang mengandung teks agen
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT);
      const hits = [];
      let n;
      while ((n = walker.nextNode())) {
        const t = (n.textContent || "").trim();
        if (/^(Mimi|Code Builder|Kreatif|Riset)$/.test(t) && n.children.length === 0) {
          hits.push(n.tagName + "." + (n.className || "").toString().slice(0, 60));
        }
      }
      out.agentLabels = [...new Set(hits)].slice(0, 10);
      // Input chat
      const inputs = [...document.querySelectorAll("input, textarea")].map((el) => ({
        tag: el.tagName,
        ph: (el.placeholder || "").slice(0, 40),
        cls: (el.className || "").toString().slice(0, 60),
      }));
      out.inputs = inputs;
      // Tombol
      const btns = [...document.querySelectorAll("button")].map((el) => ({
        txt: (el.textContent || "").trim().slice(0, 30),
        cls: (el.className || "").toString().slice(0, 60),
      }));
      out.buttons = btns.slice(0, 15);
      return out;
    });
    console.log("DOM-INFO:" + JSON.stringify(info));
  });
});

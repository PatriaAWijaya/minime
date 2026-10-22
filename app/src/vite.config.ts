import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  // WAJIB untuk Tauri: path relatif agar asset ter-load dari bundle/dist.
  base: "./",
  clearScreen: false,
  publicDir: "static",
  server: { port: 1420, strictPort: true },
});

/** WebdriverIO config untuk automated testing Minime (Tauri v2, macOS).
 *  Dijalankan di MacBook Patria via SSH saat ia AFK.
 *  Menggunakan @wdio/tauri-service dengan embedded driver (tanpa driver eksternal).
 */
import path from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const appRoot = path.resolve(__dirname, "..");

export const config = {
  runner: "local",
  specs: ["./specs/chat.test.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: path.resolve(appRoot, "app/src-tauri/target/debug/minime"),
      },
    },
  ],
  services: [
    [
      "tauri",
      {
        // Embedded WebDriver server di dalam app (via tauri-plugin-wdio-webdriver).
        driverProvider: "embedded",
      },
    ],
  ],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },
  reporters: ["spec"],
  logLevel: "warn",
};

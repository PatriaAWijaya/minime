import { writable, get, derived, type Writable, type Readable } from "svelte/store";
import { loadRecord, persistRecord, loadString, loadBool, loadJSON, loadList, persistList } from "./_helpers";
import type {
  Agent, AgentFile, AgentStatus, CostRow, FlowEdge, FlowNode, FlowProject,
  KanbanCard, KanbanCol, LibraryItem, Message, ProviderInfo, ScheduledItem,
  ToolInfo, LibraryType,
} from "../types";
import { invoke } from "@tauri-apps/api/core";

export type View =
  | "agents"
  | "tasks"
  | "studio"
  | "schedule"
  | "library"
  | "cost"
  | "tools"
  | "settings";

/** Which main view is visible. */
export const activeView = writable<View>("agents");

/** Manually resizable Agents-view column widths (Patria 20 Okt): drag the
 *  column edges with the mouse. Persisted per device. */
export interface BrandKit {
  primary: string;
  secondary: string;
  font: string;
  tone: string;
  photo: string;        // data URL foto/logo brand
  visualElement: string; // deskripsi elemen visual/grafis
  iconStyle: string;    // gaya ikon
  layout: string;       // preferensi layout
  template: string;     // template konten
}
const defaultBrand: BrandKit = {
  primary: "#4FA6F0",
  secondary: "#1a1a1a",
  font: "Inter",
  tone: "profesional dan hangat",
  photo: "",
  visualElement: "",
  iconStyle: "minimal",
  layout: "center",
  template: "carousel",
};
function loadBrand(): BrandKit {
  try {
    const raw = localStorage.getItem("minime.brand-kit");
    if (raw) return { ...defaultBrand, ...JSON.parse(raw) };
  } catch {}
  return { ...defaultBrand };
}
export const brandKit = writable<BrandKit>(loadBrand());
brandKit.subscribe((v) => {
  try {
    localStorage.setItem("minime.brand-kit", JSON.stringify(v));
  } catch {}
});

/* ---------- Custom fonts (Media Studio) ---------- */
const defaultFonts = ["Inter", "Poppins", "Montserrat", "Playfair Display", "Roboto"];
function loadFonts(): string[] {
  try {
    const raw = localStorage.getItem("minime.brand-fonts");
    if (raw) return [...defaultFonts, ...JSON.parse(raw)];
  } catch {}
  return [...defaultFonts];
}
export const brandFonts = writable<string[]>(loadFonts());
brandFonts.subscribe((v) => {
  try {
    localStorage.setItem("minime.brand-fonts", JSON.stringify(v.filter((f) => !defaultFonts.includes(f))));
  } catch {}
});
export function addBrandFont(name: string): void {
  const n = name.trim();
  if (!n) return;
  brandFonts.update((list) => (list.includes(n) ? list : [...list, n]));
}
export function removeBrandFont(name: string): void {
  if (defaultFonts.includes(name)) return;
  brandFonts.update((list) => list.filter((f) => f !== name));
}

/* ---------- Research & Scripts (Media Studio) ---------- */
export interface UserInput {
  id: number;
  kind: string; // riset | goal | preferensi | materi | lain
  title: string;
  content: string;
  createdAt: string;
}
export interface ProdScript {
  id: number;
  researchId: number | null;
  title: string;
  kind: string; // reels | carousel | poster | story | thumbnail | artikel
  content: string;
  status: "draft" | "in_production" | "done";
  createdAt: string;
}
export const userInputs = writable<UserInput[]>(loadJSON("minime.inputs", loadJSON("minime.research", [])));
userInputs.subscribe((v) => {
  try { localStorage.setItem("minime.inputs", JSON.stringify(v)); } catch {}
});
export const prodScripts = writable<ProdScript[]>(loadJSON("minime.scripts", []));
prodScripts.subscribe((v) => {
  try { localStorage.setItem("minime.scripts", JSON.stringify(v)); } catch {}
});

/* ---------- Storyboard (Media Studio) ---------- */
export interface Scene {
  id: number;
  duration: number;
  visual: string;
  narration: string;
  dialogue: string;   // Storyboarder: dialog
  action: string;     // Storyboarder: aksi
  notes: string;      // Storyboarder: catatan
  shotType: string;   // Storyboarder: wide | medium | closeup | extreme
}
function loadScenes(): Scene[] {
  try {
    const raw = localStorage.getItem("minime.storyboard");
    if (raw) {
      const arr = JSON.parse(raw);
      // Migrasi: isi field baru bila belum ada
      return arr.map((sc: any) => ({
        dialogue: "", action: "", notes: "", shotType: "medium",
        ...sc,
      }));
    }
  } catch {}
  return [];
}
export const storyboard = writable<Scene[]>(loadScenes());
storyboard.subscribe((v) => {
  try {
    localStorage.setItem("minime.storyboard", JSON.stringify(v));
  } catch {}
});
let sceneSeq = 1;

/* ---------- Scheduler (Media Studio) ----------
   Aturan keras: TIDAK ADA auto-posting. Scheduler hanya membuat DRAFT. */
export interface ScheduledItem {
  id: number;
  title: string;
  platform: string;
  scheduledAt: string;
  status: "draft" | "scheduled";
}
function loadSchedule(): ScheduledItem[] {
  try {
    const raw = localStorage.getItem("minime.schedule-queue");
    if (raw) return JSON.parse(raw);
  } catch {}
  return [];
}
export const scheduleQueue = writable<ScheduledItem[]>(loadSchedule());
scheduleQueue.subscribe((v) => {
  try {
    localStorage.setItem("minime.schedule-queue", JSON.stringify(v));
  } catch {}
});
export interface ModelOption {
  id: string;
  providerId: string;
  /** Display label (brand/model name, not translated) */
  label: string;
  kind: "free" | "paid";
  /** True for models the user added themselves. */
  custom?: boolean;
}

/** Free-tier catalog: real model names only (no fictitious versions).
    Ollama models must exist in the Ollama library; OpenCode free models
    are verified against OpenCode's actual offerings. */
const FREE_TIER: Array<[string, string]> = [
  ["llama3.1:8b", "Llama 3.1 8B"],
  ["qwen2.5:7b", "Qwen 2.5 7B"],
  ["deepseek-r1:8b", "DeepSeek R1 8B"],
  ["mistral:7b", "Mistral 7B"],
  ["gemma2:9b", "Gemma 2 9B"],
  ["phi3:mini", "Phi-3 Mini"],
  ["llama3.2:3b", "Llama 3.2 3B"],
  ["qwen2.5:3b", "Qwen 2.5 3B"],
];

/** Section id for the model dropdown groups. */
export type ModelSection = "opencode" | "ollama" | "paid";

/** Custom models added by the user, persisted per section. */
export const customModels = writable<Record<ModelSection, string[]>>(
  (() => {
    const rec = loadRecord<string[]>("minime.custom-models");
    return {
      opencode: rec.opencode ?? [],
      ollama: rec.ollama ?? [],
      paid: rec.paid ?? [],
    };
  })(),
);
persistRecord("minime.custom-models", customModels);

function slugify(name: string): string {
  return (
    name
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "model"
  );
}

export function addCustomModel(section: ModelSection, name: string): void {
  const clean = name.trim();
  if (!clean) return;
  customModels.update((m) => {
    const list = m[section];
    if (list.some((n) => n.toLowerCase() === clean.toLowerCase())) return m;
    return { ...m, [section]: [...list, clean] };
  });
}

export function removeCustomModel(section: ModelSection, name: string): void {
  customModels.update((m) => ({
    ...m,
    [section]: m[section].filter((n) => n !== name),
  }));
}

/** Status provider dari backend Rust; null = belum dicek / browser.
    Didefinisikan di sini (bukan di bawah) karena modelCatalog memakainya. */
export const realProviders = writable<RealProviderStatus[] | null>(null);
/** True bila aplikasi berjalan di Tauri dengan backend Tahap 2. */
export const usingRealBackend = writable<boolean>(false);

/** Full catalog = built-in models + user-added custom models.
    Bila backend Tahap 2 tersedia dan Ollama terhubung, daftar model Ollama
    diganti dengan model yang benar-benar terinstal (dari /api/tags). */
export const modelCatalog: Readable<ModelOption[]> = derived(
  [customModels, realProviders],
  ([$custom, $real]): ModelOption[] => {
    const ollamaReal = $real?.find((p) => p.id === "ollama");
    const ollamaModels: ModelOption[] =
      ollamaReal && ollamaReal.state === "connected" && ollamaReal.models.length > 0
        ? ollamaReal.models.map(
            (m): ModelOption => ({
              id: m.id,
              providerId: "ollama",
              label: m.label,
              kind: "free",
            }),
          )
        : FREE_TIER.map(
            ([slug, label]): ModelOption => ({
              id: `ollama:${slug}`,
              providerId: "ollama",
              label,
              kind: "free",
            }),
          );
    return [
      ...ollamaModels,
      ...FREE_TIER.map(
        ([slug, label]): ModelOption => ({
          id: `opencode:${slug}`,
          providerId: "opencode",
          label,
          kind: "free",
        }),
      ),
      { id: "claude", providerId: "muse", label: "Muse", kind: "paid" },
      { id: "gemini", providerId: "gemini", label: "Gemini", kind: "paid" },
      { id: "gpt", providerId: "gpt", label: "GPT", kind: "paid" },
      { id: "claude", providerId: "claude", label: "Claude", kind: "paid" },
      { id: "deepseek", providerId: "deepseek", label: "DeepSeek", kind: "paid" },
    ...$custom.ollama.map(
      (label): ModelOption => ({
        id: `custom:ollama:${slugify(label)}`,
        providerId: "ollama",
        label,
        kind: "free",
        custom: true,
      }),
    ),
    ...$custom.opencode.map(
      (label): ModelOption => ({
        id: `custom:opencode:${slugify(label)}`,
        providerId: "opencode",
        label,
        kind: "free",
        custom: true,
      }),
    ),
    ...$custom.paid.map(
      (label): ModelOption => ({
        id: `custom:paid:${slugify(label)}`,
        providerId: "custom",
        label,
        kind: "paid",
        custom: true,
      }),
    ),
    ];
  },
);

export function getModel(id: string): ModelOption {
  const catalog = get(modelCatalog);
  // Legacy migration: old "claude" id was Muse (renamed to "muse" when
  // "Claude" became its own option).
  const resolved = id === "claude" ? "muse" : id;
  const found = catalog.find((m) => m.id === resolved);
  if (found) return found;
  // Dynamic ID (e.g. opencode:xxx-free dari Zen) — buat entri agar bar & list sinkron.
  const [providerId, ...rest] = resolved.split(":");
  const label = rest.join(":") || resolved;
  return { id: resolved, providerId, label, kind: "free" as const };
}

export function defaultModelFor(agentId: string): string {
  // 24 Okt 2026: default Mimi = Gemini (Muse billing belum aktif sampai
  // hackathon 26 Okt). Sebelumnya "muse" → selalu 402.
  if (agentId === "mimi") return "gemini:gemini-3.8-flash";
  // Bila Ollama terhubung, pakai model chat asli pertama (bukan embedding).
  const ollama = get(realProviders)?.find((p) => p.id === "ollama");
  if (ollama && ollama.state === "connected") {
    const chat = ollama.models.find((m) => !/embed/i.test(m.label));
    if (chat) return chat.id;
    if (ollama.models.length > 0) return ollama.models[0].id;
  }
  return "ollama:qwen2.5:3b";
}

/** Reasoning effort per agent (mock, persisted). */
export const scheduleItems = writable<ScheduledItem[]>([]);

export function addScheduleItem() {
  scheduleItems.update((items) => [
    ...items,
    {
      id: scheduleSeq++,
      nameKey: "schedule.mockNew.name",
      whenKey: "schedule.mockNew.when",
      range: "day",
      active: true,
    },
  ]);
}

/** Library outputs. Empty by default (Patria 20 Okt): no dummy items — real
 *  tool results are appended by addLibraryItem (Tahap 3.6) and persist
 *  across restarts. */
let librarySeq = 1000;
export const libraryItems = writable<LibraryItem[]>(
  loadList<LibraryItem>("minime.library", () => []),
);
persistList("minime.library", libraryItems);
for (const i of get(libraryItems)) {
  librarySeq = Math.max(librarySeq, i.id + 1);
}

/** Append a REAL tool result to the Library (Tahap 3.6). */
export function addLibraryItem(input: {
  title: string;
  summary?: string;
  content?: string;
  images?: string[];
  type: LibraryType;
  agentId: string;
  taskKey?: string;
}): number {
  const now = new Date();
  const id = librarySeq++;
  libraryItems.update((list) => [
    {
      id,
      titleKey: "",
      dateKey: "",
      daysAgo: 0,
      type: input.type,
      agentId: input.agentId,
      taskKey: input.taskKey ?? "library.task.once",
      title: input.title,
      summary: input.summary,
      content: input.content,
      images: input.images,
      dateLabel: now.toLocaleDateString("id-ID", {
        day: "numeric",
        month: "short",
        year: "numeric",
      }),
    },
    ...list,
  ]);
  return id;
}

/** Hapus satu item Library (24 Okt 2026): user bisa membuang output yang
 *  sudah tidak dibutuhkan. Persist otomatis via persistList. */
export function deleteLibraryItem(id: number): void {
  libraryItems.update((list) => list.filter((i) => i.id !== id));
}

/** Cost rows (mock). */
export const costRows: CostRow[] = [
  { id: 1, agentId: "kreatif", tokens: 48200, costUsd: 0.38, dateKey: "cost.date.1", daysAgo: 1 },
  { id: 2, agentId: "riset", tokens: 96500, costUsd: 0.77, dateKey: "cost.date.1", daysAgo: 1 },
  { id: 3, agentId: "mimi", tokens: 12400, costUsd: 0.31, dateKey: "cost.date.2", daysAgo: 4 },
  { id: 4, agentId: "finance", tokens: 31200, costUsd: 0.25, dateKey: "cost.date.3", daysAgo: 9 },
  { id: 5, agentId: "strategis", tokens: 27800, costUsd: 0.22, dateKey: "cost.date.3", daysAgo: 9 },
  { id: 6, agentId: "code", tokens: 61400, costUsd: 0.49, dateKey: "cost.date.4", daysAgo: 17 },
  { id: 7, agentId: "riset", tokens: 44100, costUsd: 0.35, dateKey: "cost.date.5", daysAgo: 29 },
  { id: 8, agentId: "kreatif", tokens: 22900, costUsd: 0.18, dateKey: "cost.date.5", daysAgo: 29 },
];

/** Installed apps (by app id). RuangAPI connects separately via credentials. */
export const installedApps = writable<Record<string, boolean>>(
  loadRecord<boolean>("minime.installed-apps"),
);
persistRecord("minime.installed-apps", installedApps);

export function toggleAppInstalled(id: string) {
  installedApps.update((m) => ({ ...m, [id]: !m[id] }));
  // Mirror the install state into the Rust tools gateway (Phase 3.1) so
  // the backend registry stays the source of truth. Fire-and-forget.
  const now = get(installedApps)[id] ?? false;
  void tauriInvoke("set_tool_connected", { toolId: id, connected: now }).catch(
    () => undefined,
  );
}

/* ---------- Tools gateway (Phase 3.1, Rust registry) ---------- */

/** Tool registry snapshot from the Rust gateway (null = not loaded / browser). */
export const toolInfos = writable<ToolInfo[] | null>(null);

/** Refresh the tool registry from the backend. Returns the list or null. */
export async function refreshTools(): Promise<ToolInfo[] | null> {
  const list = await tauriInvoke<ToolInfo[]>("list_tools");
  if (list) toolInfos.set(list);
  return list;
}

/** Is a tool connected in the backend registry? */
export async function fetchToolStatus(toolId: string): Promise<boolean | null> {
  return await tauriInvoke<boolean>("get_tool_status", { toolId });
}

/* ---------- Pireel (Tahap 3.4 — real OAuth, not the generic toggle) ---------- */

export type PireelOAuthPhase = "idle" | "pending" | "connected";

export interface PireelConnStatus {
  connected: boolean;
  /** "not_connected" | "live" | "offline" | "fresh" | "unknown" |
   *  "reauth_needed" | "network_error" */
  detail: string;
}

/** Last known Pireel status (null = not checked yet). */
export const pireelStatus = writable<PireelConnStatus | null>(null);
/** OAuth flow phase for the card UI. */
export const pireelOAuth = writable<PireelOAuthPhase>("idle");
/** Last OAuth failure reason (null = none). Patria 20 Okt. */
export const pireelOAuthError = writable<string | null>(null);
/** Authorization URL of the in-flight flow (fallback reopen link). */
export const pireelAuthUrl = writable<string | null>(null);

/** Start OAuth: backend registers a client, opens the system browser. */
export async function pireelConnect(): Promise<void> {
  pireelOAuthError.set(null);
  try {
    const url = await tauriInvoke<string>("pireel_connect_url");
    if (url) {
      pireelAuthUrl.set(url);
      pireelOAuth.set("pending");
    } else {
      pireelOAuth.set("idle");
    }
  } catch {
    pireelOAuth.set("idle");
  }
}

/** Poll the in-flight OAuth flow; refresh status once connected. */
export async function pireelPollOAuth(): Promise<void> {
  try {
    const s = await tauriInvoke<{ state: string; error?: string | null }>(
      "pireel_oauth_poll",
    );
    if (!s) return;
    if (s.state === "connected") {
      pireelOAuth.set("connected");
      pireelOAuthError.set(null);
      await pireelRefreshStatus();
    } else if (s.state === "idle") {
      pireelOAuth.set("idle");
      pireelOAuthError.set(s.error ?? null);
    }
  } catch {
    /* keep polling */
  }
}

/** Refresh Pireel status from the backend (free get_state check). */
export async function pireelRefreshStatus(): Promise<void> {
  try {
    const s = await tauriInvoke<PireelConnStatus>("pireel_status");
    if (s) {
      pireelStatus.set(s);
      pireelOAuth.set(s.connected ? "connected" : "idle");
      // Mirror into the tools registry so the Apps tab + allowlists agree.
      void tauriInvoke("set_tool_connected", {
        toolId: "pireel",
        connected: s.connected,
      }).catch(() => undefined);
    }
  } catch {
    /* backend unreachable (browser preview) */
  }
}

/** Create a dummy Pireel project (Patria 21 Okt: end-to-end test). */
export const pireelProjectResult = writable<string | null>(null);
export const pireelProjectBusy = writable(false);
export async function pireelCreateProject(): Promise<void> {
  pireelProjectBusy.set(true);
  pireelProjectResult.set(null);
  try {
    const r = await tauriInvoke<string>("pireel_create_project", {
      title: "Minime Dummy Project",
    });
    pireelProjectResult.set(r ?? "ok");
  } catch (e) {
    pireelProjectResult.set(`gagal: ${e}`);
  } finally {
    pireelProjectBusy.set(false);
  }
}

/** Disconnect: tokens removed from the OS keychain. */
export async function pireelDisconnect(): Promise<void> {
  try {
    await tauriInvoke("pireel_disconnect");
  } catch {
    /* ignore */
  }
  pireelStatus.set({ connected: false, detail: "not_connected" });
  pireelOAuth.set("idle");
  pireelAuthUrl.set(null);
  void tauriInvoke("set_tool_connected", {
    toolId: "pireel",
    connected: false,
  }).catch(() => undefined);
}

/* ---------- Google Drive (Tahap 3.5 — OAuth client ID milik user) ---------- */

export type DriveOAuthPhase = "idle" | "pending" | "connected";

export interface DriveConnStatus {
  connected: boolean;
  /** "not_connected" | "connected" | "reauth_needed" | "network_error" |
   *  "invalid_client_id" */
  detail: string;
  email: string | null;
}

/** Last known Drive status (null = not checked yet). */
export const driveStatus = writable<DriveConnStatus | null>(null);
/** OAuth flow phase for the card UI. */
export const driveOAuth = writable<DriveOAuthPhase>("idle");
/** Last OAuth failure reason (null = none). Patria 20 Okt. */
export const driveOAuthError = writable<string | null>(null);
/** Authorization URL of the in-flight flow (fallback reopen link). */
export const driveAuthUrl = writable<string | null>(null);
/** The user's own Google OAuth client ID (Desktop app type). */
export const driveClientId = writable<string>(
  loadString("minime.drive.client-id", ""),
);
driveClientId.subscribe((v) => {
  try {
    localStorage.setItem("minime.drive.client-id", v);
  } catch {
    /* ignore */
  }
});
/** Optional client secret (hanya bila Client ID tipe Web application). */
export const driveClientSecret = writable<string>(
  loadString("minime.drive.client-secret", ""),
);
driveClientSecret.subscribe((v) => {
  try {
    localStorage.setItem("minime.drive.client-secret", v);
  } catch {
    /* ignore */
  }
});

/** Start OAuth with the user's own client ID (opens the system browser). */
export async function driveConnect(): Promise<void> {
  const clientId = get(driveClientId).trim();
  const clientSecret = get(driveClientSecret).trim();
  driveOAuthError.set(null);
  if (!clientId) {
    driveStatus.set({ connected: false, detail: "invalid_client_id", email: null });
    return;
  }
  try {
    const url = await tauriInvoke<string>("drive_connect_url", {
      clientId,
      clientSecret: clientSecret || null,
    });
    if (url) {
      driveAuthUrl.set(url);
      driveOAuth.set("pending");
    } else {
      driveOAuth.set("idle");
    }
  } catch {
    driveOAuth.set("idle");
  }
}

/** Poll the in-flight OAuth flow; refresh status once connected. */
export async function drivePollOAuth(): Promise<void> {
  try {
    const s = await tauriInvoke<{ state: string; error?: string | null }>(
      "drive_oauth_poll",
    );
    if (!s) return;
    if (s.state === "connected") {
      driveOAuth.set("connected");
      driveOAuthError.set(null);
      await driveRefreshStatus();
    } else if (s.state === "idle") {
      driveOAuth.set("idle");
      driveOAuthError.set(s.error ?? null);
    }
  } catch {
    /* keep polling */
  }
}

/** Refresh Drive status from the backend (free about.get check). */
export async function driveRefreshStatus(): Promise<void> {
  try {
    const s = await tauriInvoke<DriveConnStatus>("drive_status");
    if (s) {
      driveStatus.set(s);
      driveOAuth.set(s.connected ? "connected" : "idle");
      void tauriInvoke("set_tool_connected", {
        toolId: "drive",
        connected: s.connected,
      }).catch(() => undefined);
    }
  } catch {
    /* backend unreachable (browser preview) */
  }
}

/** Disconnect: tokens removed from the OS keychain. */
export async function driveDisconnect(): Promise<void> {
  try {
    await tauriInvoke("drive_disconnect");
  } catch {
    /* ignore */
  }
  driveStatus.set({ connected: false, detail: "not_connected", email: null });
  driveOAuth.set("idle");
  driveAuthUrl.set(null);
  void tauriInvoke("set_tool_connected", {
    toolId: "drive",
    connected: false,
  }).catch(() => undefined);
}

export interface DriveUploadDone {
  id: string;
  name: string;
}

/** Upload a deliverable into the MIMI folder. `contentBase64` is the raw
 *  file bytes encoded as base64. Returns the created file or null. */
export async function driveUploadFile(
  fileName: string,
  contentBase64: string,
  mimeType: string,
): Promise<DriveUploadDone | null> {
  try {
    const r = await tauriInvoke<DriveUploadDone>("drive_upload", {
      fileName,
      contentBase64,
      mimeType,
    });
    return r ?? null;
  } catch {
    return null;
  }
}

/** Agent tool allowlist from the backend. */
export const islandStatus = writable<{ mode: string; displays: number } | null>(
  null,
);

export interface IslandPayload {
  notch: boolean;
  idle: boolean;
  extraDisplays: boolean;
  haptics: boolean;
  position: string;
  widthPct: number;
  heightPct: number;
  mode: string;
  title: string;
  sub: string;
  attention: boolean;
  logoColor: string;
}

export async function syncIsland(payload: IslandPayload): Promise<void> {
  try {
    const res = await tauriInvoke<{ mode: string; displays: number }>(
      "island_sync",
      { payload },
    );
    if (res) islandStatus.set(res);
  } catch {
    /* island unavailable — never block the app */
  }
}

/** Fetch the last island payload (the island window asks on mount). */
export async function fetchIslandState(): Promise<IslandPayload | null> {
  try {
    return await tauriInvoke<IslandPayload>("island_get_state");
  } catch {
    return null;
  }
}

/* ---------- macOS system permissions (Settings > Computer Use) ---------- */

export interface SystemPermissions {
  screen_recording: boolean;
  accessibility: boolean;
  /** False on platforms without this permission model. */
  supported: boolean;
}

/** Real preflight check (never prompts). Null outside the desktop app. */
export async function fetchSystemPermissions(): Promise<SystemPermissions | null> {
  try {
    return await tauriInvoke<SystemPermissions>("system_permissions");
  } catch {
    return null;
  }
}

/** Open the macOS Privacy & Security pane for one permission kind. */
export async function openPrivacySettings(kind: "screen" | "accessibility"): Promise<void> {
  try {
    await tauriInvoke<void>("open_privacy_settings", { kind });
  } catch {
    /* non-macOS or open failed — the settings page explains the state */
  }
}

/* ---------- Device info (Settings > Profile sessions) ---------- */

export interface DeviceInfo {
  name: string;
  os: string;
}

/** This device's name + OS label from the backend. Null in the browser. */
export async function fetchDeviceInfo(): Promise<DeviceInfo | null> {
  try {
    return await tauriInvoke<DeviceInfo>("device_info");
  } catch {
    return null;
  }
}

/* ---------- App updates (Settings > Updates) ---------- */

export interface UpdateInfo {
  current: string;
  latest: string | null;
  update_available: boolean;
  notes: string;
  url: string;
}

/** Result of the last update check (GitHub releases). */
export const updateInfo = writable<UpdateInfo | null>(null);

/** Check GitHub releases for a newer Minime version on the chosen track. */
export async function checkForUpdate(): Promise<void> {
  try {
    const res = await tauriInvoke<UpdateInfo>("check_update", {
      track: get(settingsPrefs).updateTrack,
    });
    if (res) updateInfo.set(res);
  } catch {
    /* offline or no releases — keep the previous state */
  }
}

/** Restart the app (applies a downloaded update). */
export async function restartApp(): Promise<void> {
  try {
    await tauriInvoke<void>("restart_app");
  } catch {
    /* not in the desktop app */
  }
}

export async function sendNotification(title: string, body: string): Promise<void> {
  try {
    await tauriInvoke("send_notification", { title, body });
  } catch {
    /* notifications unavailable — never block the workflow */
  }
}

export function addSkill(name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  skills.update((list) => [
    ...list,
    { id: skillSeq++, customName: trimmed, installed: true },
  ]);
}

/* ---------- Model providers (Patria's decision 18 Oct ~22:40) ----------
   Free: Ollama = default, OpenCode = fallback.
   Paid: Muse = default, fallback = Gemini, GPT, Claude (in that order).
   All state below is local mock, persisted on this device. */

/** Built-in providers. */
export const PROVIDERS: ProviderInfo[] = [
  { id: "ollama", name: "Ollama", kind: "free", tagKey: "providers.defaultFree", local: true },
  { id: "opencode", name: "OpenCode", kind: "free", tagKey: "providers.backupFree" },
  { id: "muse", name: "Muse", kind: "paid", tagKey: "providers.defaultPaid" },
  { id: "gemini", name: "Gemini", kind: "paid" },
  { id: "gpt", name: "GPT", kind: "paid" },
  { id: "claude", name: "Claude", kind: "paid" },
  { id: "deepseek", name: "DeepSeek", kind: "paid" },
  { id: "routerku", name: "RouterKu", kind: "paid", tagKey: "providers.routerKuTag" },
];

/** Per-provider mock state: connected + active toggle. */
export const providerState = writable<Record<string, { connected: boolean; active: boolean }>>(
  loadRecord("minime.providers"),
);
persistRecord("minime.providers", providerState);

export function connectProvider(id: string) {
  providerState.update((s) => ({
    ...s,
    [id]: { connected: true, active: s[id]?.active ?? true },
  }));
}

export function toggleProvider(id: string, active: boolean) {
  providerState.update((s) => ({
    ...s,
    [id]: { connected: s[id]?.connected ?? false, active },
  }));
}

/* ---------- RouterKu gateway (mock, local only) ----------
   Real connection = Phase 2. The key never leaves this device. */
export const routerkuKey = writable<string>(loadString("minime.routerku.key", ""));
export const routerkuBaseUrl = writable<string>(loadString("minime.routerku.baseurl", ""));
routerkuKey.subscribe((v) => {
  try { localStorage.setItem("minime.routerku.key", v); } catch { /* ignore */ }
});
routerkuBaseUrl.subscribe((v) => {
  try { localStorage.setItem("minime.routerku.baseurl", v); } catch { /* ignore */ }
});
/** Save RouterKu credentials; mark connected only when a key is present. */
export function saveRouterku(key: string, baseUrl: string) {
  routerkuKey.set(key);
  routerkuBaseUrl.set(baseUrl);
  const ok = key.trim().length > 0;
  providerState.update((s) => ({
    ...s,
    routerku: { connected: ok, active: s["routerku"]?.active ?? true },
  }));
}

/* ---------- RuangAPI gateway (mock, local only) ----------
   Real connection = Phase 3. The key never leaves this device. */
export const ruangapiKey = writable<string>(loadString("minime.ruangapi.key", ""));
export const ruangapiMcpUrl = writable<string>(
  loadString("minime.ruangapi.mcp-url", "https://mcp.ruangapi.id/mcp"),
);
export const ruangapiConnected = writable<boolean>(
  loadString("minime.ruangapi.connected", "false") === "true",
);
ruangapiKey.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.key", v); } catch { /* ignore */ }
});
ruangapiMcpUrl.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.mcp-url", v); } catch { /* ignore */ }
});
ruangapiConnected.subscribe((v) => {
  try { localStorage.setItem("minime.ruangapi.connected", String(v)); } catch { /* ignore */ }
});
/** Mock-connect RuangAPI; mark connected only when a key is present. */
export function connectRuangapi(key: string) {
  ruangapiKey.set(key);
  ruangapiConnected.set(key.trim().length > 0);
}

function loadString(key: string, fallback: string): string {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    return fallback;
  }
}

/** Default provider id (radio selection). */
export const defaultProviderId = writable<string>(
  loadString("minime.default-provider", "ollama"),
);
defaultProviderId.subscribe((v) => {
  try {
    localStorage.setItem("minime.default-provider", v);
  } catch {
    /* ignore */
  }
});

function loadBool(key: string, fallback: boolean): boolean {
  try {
    const v = localStorage.getItem(key);
    return v === null ? fallback : v === "1";
  } catch {
    return fallback;
  }
}

/** Auto-discover local models & agents (mock). */
export const autoScan = writable<boolean>(loadBool("minime.auto-scan", true));
autoScan.subscribe((v) => {
  try {
    localStorage.setItem("minime.auto-scan", v ? "1" : "0");
  } catch {
    /* ignore */
  }
});

export interface CustomProvider {
  id: number;
  name: string;
  endpoint: string;
  active: boolean;
}

/** Custom providers added by the user (mock). */
let customProvSeq = 1;
export const customProviders = writable<CustomProvider[]>([]);

export function addCustomProvider(
  id: string,
  name: string,
  endpoint: string,
  apiKey: string,
  models: string[],
) {
  const pid = id.trim().toLowerCase().replace(/[^a-z0-9-_]/g, "-");
  const n = name.trim();
  const e = endpoint.trim().replace(/\/+$/, "");
  if (!pid || !n || !e || models.length === 0) return;
  const provId = `custom-${pid}`;
  customProviders.update((list) => [
    ...list,
    { id: customProvSeq++, name: n, endpoint: e, active: true },
  ]);
  // Daftarkan model-model yang dipilih ke section paid (custom provider = paid)
  customModels.update((m) => ({
    ...m,
    paid: [...m.paid, ...models.map((mid) => `${n}: ${mid}`)],
  }));
}

export function toggleCustomProvider(id: number, active: boolean) {
  customProviders.update((list) =>
    list.map((p) => (p.id === id ? { ...p, active } : p)),
  );
}

/* ---------- App settings (Settings view, two-column layout) ---------- */

export interface SettingsPrefs {
  langMode: "system" | "id" | "en" | "zh";
  logoColor: string;
  sendShortcut: "enter" | "ctrlEnter";
  steerAgents: boolean;
  memoriesPerAgent: number;
  externalLinks: "browser" | "inapp";
  launchAtLogin: boolean;
  fastConnect: boolean;
  keepBackground: boolean;
  restoreWorkspace: boolean;
  shareAnalytics: boolean;
  notifApproval: boolean;
  notifDone: boolean;
  notifSound: boolean;
  voiceEnabled: boolean;
  voiceLang: string;
  dictationModel: "parakeet" | "whisper" | "elevenlabs";
  dictationLang: string;
  dictationDownloads: string[];
  elevenlabsKeySaved: boolean;
  /** Permanent Mobile Connect link token ("" = no link generated). */
  mobileLinkToken: string;
  islandNotch: boolean;
  islandIdle: boolean;
  islandExtra: boolean;
  islandHaptics: boolean;
  islandPosition: "top" | "bottom";
  islandWidth: number;
  islandHeight: number;
  /** Migration flag 23 Okt 2026 (island default aktif). */
  islandMigrated23Okt?: boolean;
  computerBrowser: boolean;
  computerHidden: boolean;
  computerMirror: boolean;
  autoUpdate: boolean;
  updateTrack: "stable" | "beta";
  updateAutoInstall: boolean;
  updateMembers: boolean;
}

const SETTINGS_DEFAULTS: SettingsPrefs = {
  langMode: "system",
  logoColor: "#B18CFF",
  sendShortcut: "enter",
  steerAgents: false,
  memoriesPerAgent: 64,
  externalLinks: "browser",
  launchAtLogin: true,
  fastConnect: true,
  keepBackground: false,
  restoreWorkspace: true,
  shareAnalytics: true,
  notifApproval: true,
  notifDone: true,
  notifSound: false,
  voiceEnabled: true,
  voiceLang: "id",
  dictationModel: "parakeet",
  dictationLang: "auto",
  dictationDownloads: [],
  elevenlabsKeySaved: false,
  mobileLinkToken: "",
  islandNotch: false,
  islandIdle: false,
  islandExtra: true,
  islandHaptics: true,
  islandPosition: "top",
  islandWidth: 100,
  islandHeight: 100,
  computerBrowser: true,
  computerHidden: false,
  computerMirror: true,
  autoUpdate: true,
  updateTrack: "stable",
  updateAutoInstall: false,
  updateMembers: true,
};

export const settingsPrefs = writable<SettingsPrefs>({
  ...SETTINGS_DEFAULTS,
  ...loadRecord<unknown>("minime.settings-prefs"),
} as SettingsPrefs);
settingsPrefs.subscribe((v) => {
  try {
    localStorage.setItem("minime.settings-prefs", JSON.stringify(v));
  } catch {
    /* ignore */
  }
});

export function setPref<K extends keyof SettingsPrefs>(key: K, value: SettingsPrefs[K]): void {
  settingsPrefs.update((p) => ({ ...p, [key]: value }));
}

/** User profile (Settings > Profile). `photo` is a small data-URL
    (downscaled at upload); `signedInAt` stamps the local session start. */
export interface ProfileState {
  name: string;
  email: string;
  photo: string;
  signedInAt: number;
}

export const profileStore = writable<ProfileState>({
  name: "",
  email: "",
  photo: "",
  signedInAt: 0,
  ...(loadRecord<string>("minime.profile") as Partial<ProfileState>),
} as ProfileState);
profileStore.subscribe((v) => {
  try {
    localStorage.setItem("minime.profile", JSON.stringify(v));
  } catch {
    /* ignore */
  }
});
// Stamp the local session the first time the app runs with a profile.
if (get(profileStore).signedInAt === 0) {
  profileStore.update((p) => ({ ...p, signedInAt: Date.now() }));
}

/* ---------- Theme (keputusan Patria 19 Okt): system = follow OS,
   or force dark/light. Applied as data-theme on <html> by App.svelte. ---------- */

export type Theme = "system" | "dark" | "light";

export const theme = writable<Theme>(
  loadString("minime.theme", "system") as Theme,
);
theme.subscribe((v) => {
  try {
    localStorage.setItem("minime.theme", v);
  } catch {
    /* ignore */
  }
});

/** Folder open/collapse UI state per project (the Agents-view folder IS the
    project — Patria 20 Okt). Not persisted; defaults to open. */
export const folderOpen = writable<Record<string, boolean>>({});

export function toggleFolder(id: string) {
  folderOpen.update((m) => ({ ...m, [id]: !(m[id] ?? true) }));
}

export function isFolderOpen(id: string, openMap: Record<string, boolean>): boolean {
  return openMap[id] ?? true;
}

let customAgentSeq = 1;
const CUSTOM_AGENT_COLORS = ["#F4A7C3", "#A8D8FF", "#B5E8C0", "#FFD6A5", "#D3BFF0", "#FFF3B0"];

/** Agents created in-app via the "+" menu or the Marketplace.
 *  Persisted so template agents survive restarts. */
export interface RealModelInfo {
  id: string;
  label: string;
}

export interface RealProviderStatus {
  id: string;
  label: string;
  kind: "free" | "paid";
  /** "connected" | "no_key" | "unreachable" | "unknown" */
  state: string;
  detail: string;
  models: RealModelInfo[];
}

export interface ChatWireMessage {
  role: string;
  content: string;
}

export interface ChatWireResult {
  text: string;
  served_by: string;
}

/** True bila berjalan di Tauri (backend Rust tersedia). Di-cache per sesi. */
let tauriAvailable: boolean | null = null;
export async function isTauri(): Promise<boolean> {
  if (tauriAvailable !== null) return tauriAvailable;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("get_app_info");
    tauriAvailable = true;
  } catch {
    tauriAvailable = false;
  }
  return tauriAvailable;
}

async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | null> {
  if (!(await isTauri())) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return await invoke<T>(cmd, args ?? {});
}

function mirrorToMock(list: RealProviderStatus[]) {
  providerState.update((s) => {
    const next = { ...s };
    for (const p of list) {
      next[p.id] = {
        connected: p.state === "connected",
        active: next[p.id]?.active ?? true,
      };
    }
    return next;
  });
}

/** Pindai ulang semua provider via backend. Dipanggil saat layar
    Penyedia Model dibuka dan setelah connect/disconnect. */
export async function refreshProviders(): Promise<void> {
  try {
    const res = await tauriInvoke<RealProviderStatus[]>("provider_probe");
    if (res) {
      usingRealBackend.set(true);
      realProviders.set(res);
      mirrorToMock(res);
      await migrateRouterkuKey();
    }
  } catch {
    /* backend gagal — biarkan mock */
  }
}

/** Simpan API key provider berbayar ke OS Keychain (atau mock di browser). */
export async function connectProviderKey(
  id: string,
  apiKey: string,
): Promise<boolean> {
  const key = apiKey.trim();
  if (!key) return false;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: id,
      apiKey: key,
    });
    if (await isTauri()) {
      await refreshProviders();
      return true;
    }
  } catch {
    return false;
  }
  connectProvider(id); // fallback browser: mock
  return true;
}

/** Hapus API key provider dari OS Keychain. */
export async function disconnectProviderKey(id: string): Promise<void> {
  try {
    await tauriInvoke<void>("provider_disconnect", { providerId: id });
  } catch {
    /* abaikan */
  }
  providerState.update((s) => ({
    ...s,
    [id]: { connected: false, active: s[id]?.active ?? true },
  }));
  await refreshProviders();
}

/** Simpan API key ElevenLabs Scribe (Pengaturan > Suara) ke OS Keychain.
    Hanya status tersimpan yang dicatat di preferensi — kuncinya tidak. */
export async function saveElevenlabsKey(key: string): Promise<boolean> {
  const k = key.trim();
  if (!k) return false;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: "elevenlabs",
      apiKey: k,
    });
  } catch {
    return false;
  }
  setPref("elevenlabsKeySaved", true);
  return true;
}

/** Hapus API key ElevenLabs dari OS Keychain. */
export async function removeElevenlabsKey(): Promise<void> {
  try {
    await tauriInvoke<void>("provider_disconnect", {
      providerId: "elevenlabs",
    });
  } catch {
    /* abaikan */
  }
  setPref("elevenlabsKeySaved", false);
}

/** Migrasi sekali: pindahkan API key RouterKu yang dulu tersimpan di
    localStorage (mock Tahap 1) ke OS Keychain, lalu hapus dari sana. */
let routerkuMigrated = false;
async function migrateRouterkuKey(): Promise<void> {
  if (routerkuMigrated) return;
  routerkuMigrated = true;
  let raw = "";
  try {
    raw = localStorage.getItem("minime.routerku.key") ?? "";
  } catch {
    return;
  }
  if (!raw.trim()) return;
  try {
    await tauriInvoke<void>("provider_connect", {
      providerId: "routerku",
      apiKey: raw.trim(),
    });
    if (await isTauri()) {
      try {
        localStorage.removeItem("minime.routerku.key");
      } catch {
        /* ignore */
      }
      routerkuKey.set("");
    }
  } catch {
    /* abaikan */
  }
}

/** Simpan kredensial RouterKu: key → Keychain, base URL → settings.
/// Tidak lagi menyimpan raw key di localStorage. */
export async function saveRouterkuSecure(
  key: string,
  baseUrl: string,
): Promise<boolean> {
  const k = key.trim();
  const u = baseUrl.trim();
  try {
    if (u) {
      await tauriInvoke<void>("routerku_set_base_url", { baseUrl: u });
      routerkuBaseUrl.set(u);
    }
    if (!k) return false;
    await tauriInvoke<void>("provider_connect", {
      providerId: "routerku",
      apiKey: k,
    });
    if (await isTauri()) {
      await refreshProviders();
      return true;
    }
  } catch {
    return false;
  }
  saveRouterku(k, u); // fallback browser: mock lama
  return true;
}

/* ---------- System prompt builder (Tahap 2) ----------
   Persona agen = peran + nama + instruksi kustom + personalisasi.
   Playbook skill disuntikkan di backend Rust (skills.rs). */

/** Susun system prompt untuk agen. `roleLabel` = peran terterjemah dari UI. */

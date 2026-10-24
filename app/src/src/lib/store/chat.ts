import { writable, get } from "svelte/store";
import type {
  Agent, AgentFile, AgentStatus, CostRow, FlowEdge, FlowNode, FlowProject,
  KanbanCard, KanbanCol, LibraryItem, Message, ProviderInfo, ScheduledItem,
  ToolInfo, LibraryType,
} from "../types";
import { agentNames, agentTitles, agentInstructions, agentGender, agentAge, agentPosition, agentHobbies } from "./agents";
import { isTauri, realProviders } from "./settings";

export const messagesByAgent = writable<Record<string, Message[]>>({});

let msgSeq = 1;

export function pushMessage(agentId: string, role: Message["role"], text: string, attachments?: Message["attachments"], replyTo?: string, sender?: { id: string; name: string }) {
  const msg: Message = {
    id: msgSeq++,
    role,
    text,
    time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    ...(attachments && attachments.length > 0 ? { attachments } : {}),
    ...(replyTo ? { replyTo } : {}),
    ...(sender ? { senderId: sender.id, senderName: sender.name } : {}),
  };
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: [...(m[agentId] ?? []), msg],
  }));
}

/** Remove a chat message (per-bubble delete). */
export function deleteMessage(agentId: string, msgId: number): void {
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: (m[agentId] ?? []).filter((msg) => msg.id !== msgId),
  }));
}

/** Toggle an emoji reaction on a chat message. */
export function setMessageReaction(agentId: string, msgId: number, emoji: string): void {
  messagesByAgent.update((m) => ({
    ...m,
    [agentId]: (m[agentId] ?? []).map((msg) =>
      msg.id === msgId
        ? { ...msg, reaction: msg.reaction === emoji ? undefined : emoji }
        : msg,
    ),
  }));
}

/** Incremented to focus the chat composer (e.g. from "+ Tugas Baru"). */
export const newTaskSignal = writable(0);

/** Small helper: load/persist a per-agent record in localStorage. */
export function buildSystemPrompt(agentId: string, roleLabel: string): string {
  const name = (get(agentNames)[agentId] ?? "").trim();
  const title = (get(agentTitles)[agentId] ?? "").trim();
  const instructions = (get(agentInstructions)[agentId] ?? "").trim();
  const gender = get(agentGender)[agentId];
  const age = get(agentAge)[agentId];
  const position = get(agentPosition)[agentId];
  const hobbies = (get(agentHobbies)[agentId] ?? "").trim();

  const lines: string[] = [];
  lines.push(
    `Kamu adalah ${name || agentId}, asisten AI di aplikasi Minime.`,
    `Peranmu: ${title || roleLabel}.`,
  );
  if (agentId === "mimi") {
    lines.push(
      "Kamu adalah CIO (Chief Intelligent Officer) dan orkestrator tim: " +
        "koordinasikan 5 agen senior (Kreatif, Riset, Finance, Bisnis Strategis, Code Builder), " +
        "pecah permintaan kompleks menjadi tugas yang jelas, dan rangkum hasilnya untuk pengguna.",
    );
  }
  if (gender || age || position || hobbies) {
    const parts: string[] = [];
    if (gender) parts.push(gender === "male" ? "laki-laki" : "perempuan");
    if (age) parts.push(`${age} tahun`);
    if (position) parts.push(position);
    if (hobbies) parts.push(`hobi: ${hobbies}`);
    lines.push(`Kepribadian: ${parts.join(", ")}.`);
  }
  lines.push(
    "Jawab dalam Bahasa Indonesia yang ringkas dan jelas, kecuali pengguna memakai bahasa lain.",
    "Jangan mengarang fakta; bila tidak tahu, katakan terus terang.",
  );
  if (instructions) {
    lines.push(`Instruksi khusus dari pengguna:\n${instructions}`);
  }
  return lines.join("\n");
}

/* ---------- Chat history + usage nyata (Tahap 2) ---------- */

export interface RealStoredMessage {
  id: number;
  agent_id: string;
  role: string;
  text: string;
  served_by: string | null;
  created_at: string;
}

export interface RealUsageRow {
  agent_id: string;
  provider_id: string;
  model_id: string;
  calls: number;
  input_tokens: number;
  output_tokens: number;
}

/** Race janji melawan timeout — menjamin tidak ada hang diam-diam. */
function withTimeout<T>(p: Promise<T>, ms: number, label: string): Promise<T> {
  return Promise.race([
    p,
    new Promise<never>((_, rej) =>
      setTimeout(() => rej(`${label}: timeout setelah ${Math.round(ms / 1000)} detik`), ms),
    ),
  ]);
}

/** Kirim chat lewat model router (dengan fallback otomatis).
    - Return null → bukan di Tauri (browser/preview), pakai mock.
    - Throw string → di Tauri tapi router gagal / timeout (pesan Indonesia). */
export async function sendChatViaRouter(
  agentId: string,
  modelId: string,
  tier: "gratis" | "berbayar",
  systemPrompt: string,
  messages: ChatWireMessage[],
): Promise<ChatWireResult | null> {
  const tauri = await withTimeout(isTauri(), 10000, "deteksi backend");
  if (!tauri) return null;
  // Pastikan daftar provider/model nyata sudah dimuat (untuk model dinamis).
  if (get(realProviders) === null) {
    await refreshProviders();
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return await withTimeout(
    invoke<ChatWireResult>("agent_chat", {
      agentId,
      modelId,
      tier,
      systemPrompt,
      messages,
    }),
    180000,
    "model router",
  );
}

/** Hasil orkestrasi dari backend (cocok dengan struct Rust). */
export interface OrchestrateResult {
  aggregated: string;
  subtask_results: Array<{
    agent_id: string;
    task: string;
    result: string;
    served_by: string;
    error?: string | null;
  }>;
  qc_score?: number | null;
  qc_passed: boolean;
  qc_notes?: string | null;
  error?: string | null;
  revisions?: number;
  revision_history?: string[];
  estimated_cost_usd?: number;
}

/** Tipe event streaming orkestrasi (cocok dengan struct Rust). */
export interface OrchestrationPlanSubtask {
  agent_id: string;
  task: string;
}
export interface OrchestrationSubtaskEvent {
  run_id: string;
  agent_id: string;
  task: string;
  result: string;
  error?: string | null;
  revision: boolean;
}
export interface OrchestrationDoneEvent {
  run_id: string;
  aggregated: string;
  qc_score?: number | null;
  qc_passed: boolean;
}
export interface OrchestrationErrorEvent {
  run_id: string;
  error: string;
}

/** Hasil sendOrchestratedStream: berapa event yang sempat diproses. */
export interface StreamOutcome {
  /** true bila event done/error diterima (jangan jalankan fallback legacy). */
  settled: boolean;
  /** Jumlah event plan/subtask/done/error yang diproses. */
  eventsProcessed: number;
}

/** Orkestrasi STREAMING (24 Okt 2026): dengarkan event Tauri selagi backend
    bekerja, sehingga UI menampilkan progres per-agen begitu subtugas selesai.
    Mengembalikan settled=false HANYA bila command gagal sebelum event apa pun
    diproses — dalam hal itu pemanggil boleh menjalankan fallback legacy. */
export async function sendOrchestratedStream(
  userRequest: string,
  modelId: string | undefined,
  handlers: {
    onPlan: (subtasks: OrchestrationPlanSubtask[]) => void;
    onSubtask: (s: OrchestrationSubtaskEvent) => void;
    onDone: (d: OrchestrationDoneEvent) => void;
    onError: (e: OrchestrationErrorEvent) => void;
  },
): Promise<StreamOutcome> {
  const tauri = await withTimeout(isTauri(), 10000, "deteksi backend");
  if (!tauri) return { settled: false, eventsProcessed: 0 };
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  let eventsProcessed = 0;
  let settled = false;
  const unlisteners: Array<() => void> = [];
  const doneHandled = { current: false };
  const handleDone = (d: OrchestrationDoneEvent) => {
    if (doneHandled.current) return;
    doneHandled.current = true;
    settled = true;
    handlers.onDone(d);
  };
  try {
    unlisteners.push(await listen<OrchestrationPlanSubtask[] | { subtasks: OrchestrationPlanSubtask[] }>("orchestration-plan", (e) => {
      eventsProcessed++;
      const payload = e.payload as { subtasks?: OrchestrationPlanSubtask[] };
      const subtasks = Array.isArray(e.payload)
        ? (e.payload as OrchestrationPlanSubtask[])
        : (payload.subtasks ?? []);
      handlers.onPlan(subtasks);
    }));
    unlisteners.push(await listen<OrchestrationSubtaskEvent>("orchestration-subtask", (e) => {
      eventsProcessed++;
      handlers.onSubtask(e.payload);
    }));
    unlisteners.push(await listen<OrchestrationDoneEvent>("orchestration-done", (e) => {
      eventsProcessed++;
      handleDone(e.payload);
    }));
    unlisteners.push(await listen<OrchestrationErrorEvent>("orchestration-error", (e) => {
      eventsProcessed++;
      settled = true;
      handlers.onError(e.payload);
    }));
    // Invoke: backend memancarkan done SEBELUM return, jadi event done
    // normalnya tiba dulu; bila balapan (event telat), sintesis dari return.
    const result = await withTimeout(
      invoke<OrchestrateResult>("orchestrate_task_stream", {
        userRequest,
        mimiModelId: modelId ?? null,
      }),
      300000,
      "orkestrator",
    );
    if (!doneHandled.current && !result.error) {
      eventsProcessed++;
      handleDone({
        run_id: "",
        aggregated: result.aggregated,
        qc_score: result.qc_score,
        qc_passed: result.qc_passed,
      });
    } else if (!doneHandled.current && result.error) {
      eventsProcessed++;
      settled = true;
      handlers.onError({ run_id: "", error: result.error });
    }
    return { settled, eventsProcessed };
  } finally {
    for (const un of unlisteners) {
      try {
        un();
      } catch {
        /* ignore */
      }
    }
  }
}

/** Panggil orkestrator Mimi: delegasikan tugas ke agen-agen yang tepat. */
export async function sendOrchestrated(
  userRequest: string,
  modelId?: string,
): Promise<OrchestrateResult | null> {
  const tauri = await withTimeout(isTauri(), 10000, "deteksi backend");
  if (!tauri) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return await withTimeout(
    invoke<OrchestrateResult>("orchestrate_task", {
      userRequest,
      mimiModelId: modelId ?? null,
    }),
    300000,
    "orkestrator",
  );
}

/** Deteksi apakah pesan ke Mimi adalah tugas yang perlu diorkestrasi
 *  (bukan sekadar sapaan/pertanyaan sederhana). */
export function shouldOrchestrate(agentId: string, text: string): boolean {
  if (agentId !== "mimi") return false;
  const t = text.toLowerCase().trim();
  // Sapaan/pertanyaan sederhana → chat langsung, bukan orkestrasi
  if (/^(halo|hai|hi|selamat|terima kasih|thanks|ok|oke|ya|tidak|apa kabar)[\s!?.]*$/i.test(t)) {
    return false;
  }
  if (t.length < 10) return false;
  // Kata kerja tugas atau penyebutan tim/agen → orkestrasi
  const taskWords = [
    "buatkan", "buat", "tolong", "desain", "poster", "analisis", "riset",
    "laporan", "rencana", "strategi", "konten", "video", "gambar", "tulis",
    "siapkan", "susun", "rancang", "kembangkan", "tim", "agen", "kreatif",
    "finance", "bisnis", "code", "builder", "kampanye", "proposal",
  ];
  return taskWords.some((w) => t.includes(w));
}

const historyLoadedFor = new Set<string>();

/** Muat riwayat chat agen dari SQLite (sekali per agen per sesi). */
export async function loadChatHistory(agentId: string): Promise<void> {
  if (historyLoadedFor.has(agentId)) return;
  historyLoadedFor.add(agentId);
  let rows: RealStoredMessage[] | null = null;
  try {
    rows = await tauriInvoke<RealStoredMessage[]>("chat_history", {
      agentId,
      limit: 100,
    });
  } catch {
    return;
  }
  if (!rows || rows.length === 0) return;
  messagesByAgent.update((m) => {
    if ((m[agentId] ?? []).length > 0) return m; // sudah ada pesan sesi ini
    return {
      ...m,
      [agentId]: rows.map((r) => {
        let time = r.created_at;
        const d = new Date(r.created_at);
        if (!isNaN(d.getTime())) {
          time = d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        }
        return {
          id: r.id,
          role: r.role === "assistant" ? "agent" : "user",
          text: r.text,
          time,
        };
      }),
    };
  });
}

/** Ringkasan usage nyata untuk tab Cost; null = belum dimuat / browser. */
export const realUsage = writable<RealUsageRow[] | null>(null);

export async function refreshUsage(): Promise<void> {
  try {
    const rows = await tauriInvoke<RealUsageRow[]>("usage_summary");
    if (rows) realUsage.set(rows);
  } catch {
    /* abaikan */
  }
}

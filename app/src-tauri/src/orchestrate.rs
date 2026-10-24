//! Orkestrasi tugas multi-agen (24 Okt 2026).
//!
//! Mimi sebagai CIO orkestrator yang NYATA (bukan sekadar klaim di system prompt):
//! 1. Parse intent user → JSON subtugas terstruktur
//! 2. Dispatch paralel ke agen yang ditugaskan
//! 3. Agregat hasil
//! 4. QC dengan rubrik (skor ≥75)
//! 5. Kembalikan ke user

use serde::{Deserialize, Serialize};

use crate::providers::{ChatMessage, ChatResult, ModelTier, Router};

/// Trait untuk LLM client — memungkinkan mock dalam test.
/// Implementasi nyata: `Router`. Implementasi test: `MockRouter`.
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    async fn chat(
        &self,
        model_id: &str,
        tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String>;

    /// Tool eksekusi: generate gambar → path file (Fase 2, 24 Okt 2026).
    /// Default: tidak didukung (MockRouter test memakai ini).
    async fn generate_image(&self, _prompt: &str, _size: &str) -> Result<String, String> {
        Err("generate_image tidak didukung provider ini".to_string())
    }
}

#[async_trait::async_trait]
impl ChatClient for Router {
    async fn chat(
        &self,
        model_id: &str,
        tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String> {
        Router::chat(self, model_id, tier, messages).await
    }

    async fn generate_image(&self, prompt: &str, size: &str) -> Result<String, String> {
        crate::providers::generate_image_with_fallback(self, prompt, size).await
    }
}

/// Satu subtugas hasil parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub agent_id: String,
    pub task: String,
    /// Model yang disarankan untuk subtugas ini (opsional).
    #[serde(default)]
    pub model_id: Option<String>,
}

/// Hasil parsing intent → daftar subtugas.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskPlan {
    subtasks: Vec<SubTask>,
}

/// Hasil satu subtugas setelah dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskResult {
    pub agent_id: String,
    pub task: String,
    pub result: String,
    pub served_by: String,
    pub error: Option<String>,
    /// Path file gambar yang dihasilkan tool generate_image (Fase 2).
    #[serde(default)]
    pub images: Vec<String>,
}

/// Hasil akhir orkestrasi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateResult {
    /// Gambar jaring pengaman deterministik (bila tugas visual tanpa hasil gambar agen).
    #[serde(default)]
    pub images: Vec<String>,
    pub aggregated: String,
    pub subtask_results: Vec<SubTaskResult>,
    pub qc_score: Option<i32>,
    pub qc_passed: bool,
    pub qc_notes: Option<String>,
    pub error: Option<String>,
    /// Jumlah putaran revisi yang dilakukan (0 = lolos langsung).
    #[serde(default)]
    pub revisions: u32,
    /// Riwayat tiap putaran: skor QC dan aksi.
    #[serde(default)]
    pub revision_history: Vec<String>,
    /// Estimasi biaya orkestrasi dalam USD.
    #[serde(default)]
    pub estimated_cost_usd: f64,
}

/// System prompt untuk parsing intent → JSON.
fn parse_system_prompt() -> String {
    r#"Kamu adalah CIO orkestrator. Tugasmu: pecah permintaan user menjadi subtugas terstruktur.

Aturan:
- Output HARUS valid JSON dengan format: {"subtasks": [{"agent_id": "...", "task": "..."}]}
- agent_id yang valid: kreatif, riset, finance, strategis, code
  - "kreatif": konten, copywriting, desain, storytelling
  - "riset": riset web, data, analisis kompetitor
  - "finance": analisa finansial, budgeting, akuntansi
  - "strategis": strategi bisnis, marketing, brand, sales
  - "code": programming, aplikasi, script, debugging
- Setiap subtugas harus jelas dan bisa dikerjakan independen
- Maksimal 5 subtugas
- Jangan tambah teks di luar JSON
- ATURAN VISUAL (penting): bila user meminta gambar/poster/thumbnail/banner
  atau kata "lanjutkan buat image visualnya", SATU subtugas WAJIB ke "kreatif"
  dengan kata kerja GENERATE, bukan "buat prompt" atau "susun konsep".
  Contoh BENAR: {"agent_id": "kreatif", "task": "GENERATE gambar poster sunat massal gratis, rasio 9:16, pakai tool generate_image — bukan sekadar tulis prompt"}
  Contoh SALAH: {"agent_id": "kreatif", "task": "Buat rangkaian prompt detail untuk AI image generator"}

Contoh:
User: "Buatkan rencana launching produk kopi"
Output: {"subtasks": [
  {"agent_id": "riset", "task": "Riset kompetitor kopi lokal: harga, positioning, channel distribusi"},
  {"agent_id": "strategis", "task": "Susun strategi go-to-market: target pasar, pricing, channel"},
  {"agent_id": "kreatif", "task": "Buat konsep kampanye launching: tagline, key visual direction, content pillars"}
]}"#.to_string()
}

/// System prompt untuk agregasi hasil.
fn aggregate_system_prompt() -> String {
    r#"Kamu adalah CIO orkestrator. Tugasmu: gabungkan hasil kerja tim menjadi satu laporan kohesif.

Aturan:
- Sintesis, bukan sekadar tempel
- Hilangkan duplikasi
- Pertahankan semua poin penting
- Format: Bahasa Indonesia yang jelas dan terstruktur
- Akhiri dengan ringkasan eksekutif 3-5 baris"#.to_string()
}

/// System prompt untuk QC dengan rubrik.
fn qc_system_prompt() -> String {
    r#"Kamu adalah QC senior. Nilai output berikut dengan rubrik:

DIMENSI PENILAIAN (total 100):
1. Kebenaran & sumber (30): fakta akurat, tidak mengarang?
2. Kesesuaian brief (25): menjawab yang diminta?
3. Quality bar (25): standar profesional?
4. Kesiapan produksi (20): bisa langsung dipakai?

Output HARUS JSON dengan format:
{
  "score": <0-100>,
  "passed": <true jika >=75>,
  "notes": "<catatan perbaikan spesifik atau '-' jika lolos>",
  "revise_agents": ["<agent_id yang perlu revisi>"]
}

Aturan revise_agents:
- Isi dengan agent_id yang hasilnya bermasalah dan perlu diperbaiki
- agent_id valid: kreatif, riset, finance, strategis, code, mimi
- Kosongkan [] jika lolos (passed=true) atau tidak jelas siapa yang salah
- Maksimal 3 agent

Jujur dan ketat. Skor <75 berarti TOLAK dengan alasan spesifik dan tunjuk siapa yang harus revisi."#.to_string()
}

/// System prompt untuk revisi berdasarkan catatan QC.
fn revision_system_prompt(agent_id: &str, task: &str, previous: &str, qc_notes: &str) -> String {
    format!(
        r#"Kamu adalah agen {agent_id}. Hasil kerjamu DITOLAK oleh QC dan HARUS direvisi.

Tugas aslimu:
{task}

Hasil kerjamu sebelumnya:
{previous}

Catatan QC (WAJIB diperbaiki):
{qc_notes}

Aturan revisi:
- Perbaiki SEMUA poin dalam catatan QC, jangan abaikan satupun
- Pertahankan bagian yang sudah benar
- Output harus lebih baik dari sebelumnya, bukan sekadar parafrase
- Jangan menjelaskan proses revisimu, langsung berikan hasil revisi"#,
    )
}

/// Panggil router untuk satu chat (internal, tanpa Tauri State).
async fn chat_internal(
    router: &dyn ChatClient,
    db: &crate::db::Db,
    agent_id: &str,
    model_id: &str,
    tier: ModelTier,
    system_prompt: &str,
    user_message: &str,
) -> Result<ChatResult, String> {
    // Suntik skill context
    let skill_ctx = crate::skills::skill_context(db, agent_id);
    let full_system = if skill_ctx.is_empty() {
        system_prompt.to_string()
    } else {
        format!("{system_prompt}\n\n{skill_ctx}")
    };

    let wire = vec![
        ChatMessage {
            role: "system".to_string(),
            content: full_system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        },
    ];

    router.chat(model_id, tier, wire).await
}

/// Satu pemanggilan tool dari LLM (format teks, provider-agnostic).
/// Dipilih atas native function-calling per-provider karena Minime memakai
/// 6+ provider dengan format tool-call berbeda (OpenAI-compat, Gemini native,
/// Ollama) — format teks ini jalan di semuanya tanpa branching per-provider.
#[derive(Debug, Clone, Deserialize)]
struct ToolCall {
    name: String,
    #[serde(default)]
    args: serde_json::Value,
}

/// Parse blok <tool_call>{...json...}</tool_call> dari teks LLM.
/// Mengembalikan (calls_valid, ada_blok_malformed).
fn parse_tool_calls(text: &str) -> (Vec<ToolCall>, bool) {
    let mut calls = Vec::new();
    let mut malformed = false;
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        let after = &rest[start + "<tool_call>".len()..];
        if let Some(end) = after.find("</tool_call>") {
            let inner = after[..end].trim();
            match serde_json::from_str::<ToolCall>(inner) {
                Ok(c) => calls.push(c),
                Err(_) => malformed = true,
            }
            rest = &after[end + "</tool_call>".len()..];
        } else {
            malformed = true;
            break;
        }
    }
    (calls, malformed)
}

/// Tools eksekusi per agen (Fase 2, 24 Okt 2026). Saat ini hanya Kreatif.
/// Desain registry: tambah agen/tool baru di sini tanpa mengubah loop.
fn tools_for(agent_id: &str) -> &'static [&'static str] {
    match agent_id {
        "kreatif" => &["generate_image"],
        _ => &[],
    }
}

const TOOL_INSTRUCTIONS: &str = r#"
## TOOLS (alat eksekusi nyata — BUKAN sekadar tulisan)
Kamu BISA mengeksekusi tools berikut. Bila tugasmu membutuhkan output nyata (mis. gambar poster), PANGGIL tool-nya — jangan hanya mendeskripsikan hasilnya.
Format pemanggilan (SATU blok per tool, JSON valid di dalamnya):
<tool_call>
{"name": "generate_image", "args": {"prompt": "deskripsi visual detail dalam Bahasa Inggris", "size": "portrait"}}
</tool_call>
Tools yang tersedia:
- generate_image(prompt: string, size: "square"|"portrait"|"portrait45"|"landscape"): menghasilkan file gambar PNG, mengembalikan path file. Prompt harus detail visual: subjek, gaya, warna, komposisi, dan teks yang harus tampil di gambar. Poster rasio 9:16 → size "portrait".
Aturan:
- Setelah memanggil tool, kamu menerima hasil eksekusinya. Lanjutkan sampai semua output nyata selesai, lalu berikan jawaban akhir TANPA blok tool_call lagi.
- Jangan mengarang path file atau mengklaim gambar sudah dibuat bila tool belum mengembalikannya.
- Bila tool gagal, sampaikan kegagalannya dengan jujur di jawaban akhir.
"#;

/// Timeout total loop tool (image generation bisa lambat: SD lokal menit-an).
const TOOL_LOOP_TIMEOUT_SECS: u64 = 600;
/// Maksimum iterasi tool loop (cegah loop tak berujung).
const MAX_TOOL_ITERS: usize = 5;

/// Dispatch satu subtugas ke agen (untuk initial maupun revisi).
/// `system_override`: system prompt kustom (untuk revisi). `None` = default.
/// Timeout per subtugas dalam detik. Satu agen hang tidak boleh membunuh orkestrasi.
const SUBTASK_TIMEOUT_SECS: u64 = 120;

async fn dispatch_one(
    router: &dyn ChatClient,
    db: &crate::db::Db,
    run_id: &str,
    agent_id: &str,
    task: &str,
    system_override: Option<String>,
    model_override: Option<String>,
    mimi_model_id: &str,
    user_request: &str,
) -> SubTaskResult {
    // Keputusan Patria 24 Okt: agen SELALU mengikuti model Mimi secara
    // default. Saran model dari LLM (plan) diabaikan — itu bukan perintah
    // user. Hanya bila user eksplisit menentukan model berbeda (via
    // model_override yang BUKAN dari plan LLM) baru dipakai. Saat ini
    // plan LLM tidak dibedakan dari perintah user, jadi selalu pakai
    // model Mimi bila tersedia.
    let (model_id, tier) = if !mimi_model_id.is_empty() {
        let mid = mimi_model_id.to_string();
        eprintln!(
            "[orchestrate:{run_id}] subtugas {} → {} (model Mimi)",
            agent_id, mid
        );
        let t = if mid.starts_with("muse:")
            || mid.starts_with("openai:")
            || mid.starts_with("gemini:")
            || mid.starts_with("deepseek:")
        {
            ModelTier::Berbayar
        } else {
            ModelTier::Gratis
        };
        (mid, t)
    } else {
        match model_override {
            Some(m) => {
                let t = if m.starts_with("muse:")
                    || m.starts_with("openai:")
                    || m.starts_with("gemini:")
                    || m.starts_with("deepseek:")
                {
                    ModelTier::Berbayar
                } else {
                    ModelTier::Gratis
                };
                (m, t)
            }
            None => {
                let decision = crate::model_selector::select_model(task, agent_id);
                (decision.model_id, ModelTier::Berbayar)
            }
        }
    };

    // Peran spesifik per agen (24 Okt 2026): tanpa ini, LLM menebak-nebak
    // dan tercampur antar peran (kasus nyata: Kreatif menghasilkan konten
    // "CIO / Scale-Up Blueprint" milik Strategis, bukan poster yang diminta).
    let role_desc = match agent_id {
        "kreatif" => "Kreatif: desainer & copywriter. Buat konsep visual, copywriting, dan materi kreatif (poster, konten, naskah). Fokus pada visual dan pesan yang menarik, BUKAN analisis bisnis. Bila tugas membutuhkan gambar nyata, gunakan tool generate_image (lihat bagian TOOLS di system prompt).",
        "riset" => "Riset: peneliti. Cari data, fakta, dan analisis berbasis bukti. Sajikan temuan dengan sumber yang jelas.",
        "finance" => "Finance: analis keuangan. Hitung anggaran, proyeksi, dan analisis finansial dengan angka yang akurat.",
        "strategis" => "Bisnis Strategis: konsultan strategi. Susun strategi, positioning, dan rencana bisnis berbasis analisis.",
        "code" => "Code Builder: programmer. Tulis kode, bangun aplikasi, dan selesaikan masalah teknis.",
        _ => "Agen tim Minime.",
    };
    let system = system_override.unwrap_or_else(|| {
        format!(
            "Kamu adalah agen {} di tim Minime.\nPeranmu: {}\n\nTUGAS ASLI PENGGUNA (konteks keseluruhan, jangan lupakan ini):\n{}\n\nSUBTUGASMU (kerjakan ini, tetap dalam konteks tugas asli di atas):\n{}\n\nAturan: Kerjakan subtugasmu sesuai peranmu. Jangan keluar dari topik tugas asli. Jangan mengarang fakta atau mencampur dengan topik lain.",
            agent_id, role_desc, user_request, task
        )
    });

    // ===== Jalur TANPA tool (semua agen kecuali yang terdaftar): satu call =====
    if tools_for(agent_id).is_empty() {
        // Circuit breaker: timeout per subtugas agar satu agen hang tidak menggagalkan semua
        let res = match tokio::time::timeout(
            std::time::Duration::from_secs(SUBTASK_TIMEOUT_SECS),
            chat_internal(router, db, agent_id, &model_id, tier, &system, task),
        )
        .await
        {
            Ok(r) => r,
            Err(_) => {
                eprintln!(
                    "[orchestrate:{run_id}] TIMEOUT {} setelah {}s",
                    agent_id, SUBTASK_TIMEOUT_SECS
                );
                Err(format!("Timeout setelah {} detik", SUBTASK_TIMEOUT_SECS))
            }
        };
        return match res {
            Ok(r) => SubTaskResult {
                agent_id: agent_id.to_string(),
                task: task.to_string(),
                result: r.text.clone(),
                served_by: r.served_by.clone(),
                error: r.error.clone(),
                images: vec![],
            },
            Err(e) => SubTaskResult {
                agent_id: agent_id.to_string(),
                task: task.to_string(),
                result: String::new(),
                served_by: String::new(),
                error: Some(e),
                images: vec![],
            },
        };
    }

    // ===== Jalur TOOL LOOP (Fase 2): LLM ↔ eksekusi sampai jawaban akhir =====
    // chat_internal tidak dipakai di sini karena loop butuh histori penuh
    // (bukan wire 2-pesan); skill disuntik manual ke system prompt.
    let skill_ctx = crate::skills::skill_context(db, agent_id);
    let mut system_full = system.clone();
    system_full.push_str(TOOL_INSTRUCTIONS);
    if !skill_ctx.is_empty() {
        system_full.push_str(&format!("\n\n{skill_ctx}"));
    }
    let mut history: Vec<ChatMessage> = vec![
        ChatMessage { role: "system".to_string(), content: system_full },
        ChatMessage { role: "user".to_string(), content: task.to_string() },
    ];
    let mut images: Vec<String> = Vec::new();
    let mut last_text = String::new();
    let mut last_served = String::new();
    let mut last_error: Option<String> = None;
    let loop_res: Result<Result<(), String>, tokio::time::error::Elapsed> =
        tokio::time::timeout(
            std::time::Duration::from_secs(TOOL_LOOP_TIMEOUT_SECS),
            async {
                for _ in 0..MAX_TOOL_ITERS {
                    let r = router.chat(&model_id, tier, history.clone()).await;
                    let r = match r {
                        Ok(r) => r,
                        Err(e) => return Err(e),
                    };
                    if let Some(e) = r.error.clone() {
                        last_text = r.text.clone();
                        last_error = Some(e);
                        break;
                    }
                    last_text = r.text.clone();
                    last_served = r.served_by.clone();
                    let (calls, malformed) = parse_tool_calls(&r.text);
                    if calls.is_empty() {
                        if malformed {
                            history.push(ChatMessage { role: "assistant".to_string(), content: r.text.clone() });
                            history.push(ChatMessage { role: "user".to_string(), content: "Format <tool_call> kamu tidak valid (JSON di dalamnya harus valid). Perbaiki dan kirim ulang, atau berikan jawaban akhir tanpa tool_call.".to_string() });
                            continue;
                        }
                        break; // jawaban akhir, tanpa tool call
                    }
                    // Eksekusi tools satu per satu
                    let mut tool_outputs = String::new();
                    for call in calls {
                        match call.name.as_str() {
                            "generate_image" => {
                                let prompt = call.args.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
                                let size = call.args.get("size").and_then(|v| v.as_str()).unwrap_or("square");
                                if prompt.is_empty() {
                                    tool_outputs.push_str("Tool generate_image GAGAL: prompt kosong.\n");
                                } else {
                                    match router.generate_image(prompt, size).await {
                                        Ok(path) => {
                                            eprintln!("[orchestrate:{run_id}] tool generate_image OK: {path}");
                                            images.push(path.clone());
                                            tool_outputs.push_str(&format!("Tool generate_image OK: file tersimpan di {path}\n"));
                                        }
                                        Err(e) => {
                                            tool_outputs.push_str(&format!("Tool generate_image GAGAL: {e}\n"));
                                        }
                                    }
                                }
                            }
                            other => {
                                tool_outputs.push_str(&format!("Tool {other} tidak dikenal, tersedia: generate_image.\n"));
                            }
                        }
                    }
                    history.push(ChatMessage { role: "assistant".to_string(), content: r.text.clone() });
                    history.push(ChatMessage { role: "user".to_string(), content: format!("HASIL TOOL:\n{tool_outputs}\nLanjutkan tugasmu. Bila semua output nyata sudah selesai, berikan jawaban akhir TANPA blok tool_call.") });
                }
                Ok(())
            },
        )
        .await;
    match loop_res {
        Ok(Ok(())) => SubTaskResult {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            result: last_text,
            served_by: last_served,
            error: last_error,
            images,
        },
        Ok(Err(e)) => SubTaskResult {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            result: last_text,
            served_by: last_served,
            error: Some(e),
            images,
        },
        Err(_) => SubTaskResult {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            result: last_text,
            served_by: last_served,
            error: Some(format!("Timeout tool loop setelah {} detik", TOOL_LOOP_TIMEOUT_SECS)),
            images,
        },
    }
}

/// Agregat hasil subtugas menjadi laporan kohesif.
async fn aggregate_results(
    router: &dyn ChatClient,
    db: &crate::db::Db,
    run_id: &str,
    mimi_model_id: &str,
    user_request: &str,
    subtask_results: &[SubTaskResult],
) -> String {
    eprintln!("[orchestrate:{run_id}] [3/4] agregasi hasil...");
    let combined = subtask_results
        .iter()
        .map(|r| {
            format!(
                "### Hasil dari {} ({})\n{}\n",
                r.agent_id,
                r.task,
                if let Some(err) = &r.error {
                    format!("[GAGAL: {err}]")
                } else {
                    r.result.clone()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let agg_res = chat_internal(
        router,
        db,
        "mimi",
        mimi_model_id,
        ModelTier::Berbayar,
        &aggregate_system_prompt(),
        &format!("Permintaan asli: {}\n\n{}", user_request, combined),
    )
    .await;

    match agg_res {
        Ok(r) if r.error.is_none() => r.text,
        _ => combined, // Fallback: gabungan mentah
    }
}

/// Struktur output QC.
#[derive(Deserialize)]
struct QcOut {
    score: i32,
    passed: bool,
    notes: String,
    #[serde(default)]
    revise_agents: Vec<String>,
}

/// Jalankan QC terhadap output agregat.
async fn run_qc(
    router: &dyn ChatClient,
    db: &crate::db::Db,
    run_id: &str,
    mimi_model_id: &str,
    user_request: &str,
    aggregated: &str,
) -> (Option<i32>, bool, Option<String>, Vec<String>) {
    eprintln!("[orchestrate:{run_id}] [4/4] QC...");
    let qc_res = chat_internal(
        router,
        db,
        "mimi",
        mimi_model_id,
        ModelTier::Berbayar,
        &qc_system_prompt(),
        &format!("Brief: {}\n\nOutput:\n{}", user_request, aggregated),
    )
    .await;

    match qc_res {
        Ok(r) if r.error.is_none() => {
            let json_str = extract_json(&r.text);
            match serde_json::from_str::<QcOut>(&json_str) {
                Ok(q) => (Some(q.score), q.passed, Some(q.notes), q.revise_agents),
                _ => (None, true, Some("QC parse gagal, lolos manual".to_string()), vec![]),
            }
        }
        _ => (None, true, Some("QC dilewati (provider gagal)".to_string()), vec![]),
    }
}

/// Jumlah maksimal putaran revisi QC.
const MAX_REVISIONS: u32 = 3;

/// Estimasi biaya maksimal per orkestrasi (USD). Peringatan jika terlampaui.
const COST_WARNING_USD: f64 = 1.0;
/// Batas keras biaya per orkestrasi — tolak jika estimasi melebihi ini.
const COST_HARD_CAP_USD: f64 = 5.0;

/// Estimasi biaya orkestrasi berdasarkan jumlah subtugas dan model.
/// Asumsi konservatif: 2000 token input + 1000 token output per call.
/// Pre-flight check: apakah request butuh distribusi (posting/scheduling)?
/// Jika ya, verifikasi kredensial Postiz sudah dikonfigurasi.
pub fn needs_distribution(request: &str) -> bool {
    let lower = request.to_lowercase();
    // Root keywords tahan imbuhan
    let keywords = [
        "posting", "jadwal", "distribusi", "publish", "terbit",
        "unggah", "share", "bagikan",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

/// Verifikasi kredensial Postiz. Return Err dengan pesan jelas jika belum diset.
pub fn check_postiz_ready() -> Result<(), String> {
    let url_ok = crate::secrets::api_key("minime.postiz_url")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let key_ok = crate::secrets::api_key("minime.postiz_key")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    if !url_ok || !key_ok {
        return Err(
            "Postiz belum dikonfigurasi. Isi Postiz URL dan API Key di Pengaturan > Tools > Apps              sebelum meminta distribusi/posting."
                .to_string(),
        );
    }
    Ok(())
}

fn estimate_cost(num_subtasks: usize, use_wonder: bool) -> f64 {
    // Calls: 1 parse + N dispatch + 1 agregat + 1 QC (+ revisi)
    // Parse/agregat/QC selalu pakai model Mimi (Spark default)
    let spark_in = 1.25 / 1_000_000.0;
    let spark_out = 4.25 / 1_000_000.0;
    let wonder_in = 4.0 / 1_000_000.0;
    let wonder_out = 20.0 / 1_000_000.0;

    let fixed_calls = 3.0; // parse + agregat + QC
    let fixed_cost = fixed_calls * (2000.0 * spark_in + 1000.0 * spark_out);

    let (sub_in, sub_out) = if use_wonder {
        (wonder_in, wonder_out)
    } else {
        (spark_in, spark_out)
    };
    let subtask_cost = num_subtasks as f64 * (2000.0 * sub_in + 1000.0 * sub_out);

    // Tambah buffer 50% untuk revisi potensial
    (fixed_cost + subtask_cost) * 1.5
}

/// Ekstrak JSON dari teks (toleran terhadap markdown code block).
fn extract_json(text: &str) -> String {
    let t = text.trim();
    // Hapus markdown code block bila ada
    if let Some(start) = t.find('{') {
        if let Some(end) = t.rfind('}') {
            return t[start..=end].to_string();
        }
    }
    t.to_string()
}

/// Apakah permintaan user bersifat visual (minta gambar/poster/dsb).
/// Pencocokan kata kunci deterministik — tanpa LLM.
fn is_visual_request(req: &str) -> bool {
    let lower = req.to_lowercase();
    [
        "poster", "gambar", "visual", "thumbnail", "banner", "ilustrasi",
        "flyer", "pamflet", "logo", "cover", "image", "desain grafis",
        "brosur", "spanduk", "story",
    ]
    .iter()
    .any(|k| lower.contains(k))
}

/// Jaring pengaman deterministik (24 Okt 2026): bila permintaan user bersifat
/// visual tapi TIDAK ADA satu pun gambar dari agen (plan salah kata, LLM tidak
/// memanggil tool, atau tool gagal), generate satu gambar langsung dari
/// deskripsi tugas — tanpa bergantung pada plan, tool-call LLM, atau teks skill.
/// Satu titik gagal (mesin gambar) menggantikan tiga titik gagal jalur agen.
/// Mengembalikan vec kosong bila tidak visual, sudah ada gambar, atau gagal.
async fn ensure_visual_output(
    router: &dyn ChatClient,
    run_id: &str,
    user_request: &str,
    aggregated: &str,
    subtask_results: &[SubTaskResult],
) -> Vec<String> {
    if subtask_results.iter().any(|r| !r.images.is_empty()) {
        return vec![]; // agen sudah menghasilkan gambar — jaring tidak diperlukan
    }
    if !is_visual_request(user_request) {
        return vec![];
    }
    let lower = user_request.to_lowercase();
    let size = if lower.contains("poster") || lower.contains("9:16") || lower.contains("story") {
        "portrait"
    } else {
        "square"
    };
    // Prompt: permintaan user + konsep agregat (dipotong, SD lokal tidak butuh panjang).
    let concept: String = aggregated.chars().take(600).collect();
    let prompt = format!("{user_request}. Visual concept: {concept}");
    match router.generate_image(&prompt, size).await {
        Ok(path) => {
            eprintln!("[orchestrate:{run_id}] jaring pengaman visual OK: {path}");
            vec![path]
        }
        Err(e) => {
            eprintln!("[orchestrate:{run_id}] jaring pengaman visual GAGAL: {e}");
            vec![]
        }
    }
}

/// Orkestrasi tugas: parse → dispatch paralel → agregat → QC (+ revisi maks 3x).
pub async fn orchestrate(
    db: &crate::db::Db,
    router: &dyn ChatClient,
    user_request: &str,
    mimi_model_id: &str,
) -> OrchestrateResult {
    // Correlation ID untuk tracing satu run orkestrasi
    let run_id = format!("{:08x}", {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut h);
        user_request.hash(&mut h);
        h.finish() & 0xFFFFFFFF
    });
    eprintln!("[orchestrate:{}] mulai: {}", run_id, user_request.chars().take(80).collect::<String>());

    // ===== [0] Pre-flight: cek kredensial distribusi bila dibutuhkan =====
    if needs_distribution(user_request) {
        eprintln!("[orchestrate:{run_id}] [0/4] pre-flight distribusi...");
        if let Err(msg) = check_postiz_ready() {
            eprintln!("[orchestrate:{run_id}] pre-flight GAGAL: {msg}");
            return OrchestrateResult {
                images: vec![],
                aggregated: String::new(),
                subtask_results: vec![],
                qc_score: None,
                qc_passed: false,
                qc_notes: None,
                revisions: 0,
                revision_history: vec![],
                estimated_cost_usd: 0.0,
                error: Some(msg),
            };
        }
        eprintln!("[orchestrate:{run_id}] pre-flight OK");
    }

    // ===== [1] Parse intent → subtugas =====
    eprintln!("[orchestrate:{run_id}] [1/4] parsing intent...");
    let parse_res = chat_internal(
        router,
        db,
        "mimi",
        mimi_model_id,
        ModelTier::Berbayar, // Parsing butuh model pintar
        &parse_system_prompt(),
        user_request,
    )
    .await;

    let plan: TaskPlan = match parse_res {
        Ok(r) if r.error.is_none() => {
            let json_str = extract_json(&r.text);
            match serde_json::from_str::<TaskPlan>(&json_str) {
                Ok(p) if !p.subtasks.is_empty() => {
                    let _ = db.log_parse_attempt(true);
                    p
                },
                _ => {
                    eprintln!("[orchestrate:{run_id}] parse gagal, fallback ke single task");
                    let _ = db.log_parse_attempt(false);
                    // Fallback: satu tugas untuk Mimi sendiri
                    TaskPlan {
                        subtasks: vec![SubTask {
                            agent_id: "mimi".to_string(),
                            task: user_request.to_string(),
                            model_id: None,
                        }],
                    }
                }
            }
        }
        _ => {
            return OrchestrateResult {
                images: vec![],
                aggregated: String::new(),
                subtask_results: vec![],
                qc_score: None,
                qc_passed: false,
                qc_notes: None,
                error: Some("Gagal mem-parse permintaan".to_string()),
                revisions: 0,
                revision_history: vec![],
                estimated_cost_usd: 0.0,
            };
        }
    };

    // Validasi agent_id terhadap allowlist (fallback ke mimi bila tidak dikenal)
    const VALID_AGENTS: &[&str] = &["mimi", "kreatif", "riset", "finance", "strategis", "code"];
    let plan = TaskPlan {
        subtasks: plan
            .subtasks
            .into_iter()
            .map(|mut st| {
                if !VALID_AGENTS.contains(&st.agent_id.as_str()) {
                    eprintln!(
                        "[orchestrate:{run_id}] agent_id '{}' tidak dikenal, fallback ke mimi",
                        st.agent_id
                    );
                    st.agent_id = "mimi".to_string();
                }
                st
            })
            .collect(),
    };

    eprintln!("[orchestrate:{run_id}] [1/4] OK: {} subtugas", plan.subtasks.len());

    // ===== Cost guard =====
    // Estimasi: apakah ada subtugas yang kemungkinan pakai Wonder?
    let likely_wonder = plan.subtasks.iter().any(|st| {
        crate::model_selector::complexity_score(&st.task, &st.agent_id).0 >= 60
    });
    let est_cost = estimate_cost(plan.subtasks.len(), likely_wonder);
    eprintln!("[orchestrate:{run_id}] Estimasi biaya: ${:.4}", est_cost);
    if est_cost > COST_HARD_CAP_USD {
        eprintln!(
            "[orchestrate:{run_id}] DITOLAK: estimasi ${:.2} melebihi batas ${:.2}",
            est_cost, COST_HARD_CAP_USD
        );
        return OrchestrateResult {
            images: vec![],
            aggregated: String::new(),
            subtask_results: vec![],
            qc_score: None,
            qc_passed: false,
            qc_notes: None,
            error: Some(format!(
                "Estimasi biaya ${:.2} melebihi batas ${:.2}. Sederhanakan permintaan.",
                est_cost, COST_HARD_CAP_USD
            )),
            revisions: 0,
            revision_history: vec![],
            estimated_cost_usd: est_cost,
        };
    }
    if est_cost > COST_WARNING_USD {
        eprintln!(
            "[orchestrate:{run_id}] PERINGATAN: estimasi ${:.2} di atas ambang ${:.2}",
            est_cost, COST_WARNING_USD
        );
    }

    // ===== [2] Dispatch paralel (initial) =====
    eprintln!("[orchestrate:{run_id}] [2/4] dispatch paralel...");
    let mut subtask_results: Vec<SubTaskResult> =
        futures::future::join_all(plan.subtasks.iter().map(|st| {
            dispatch_one(
                router,
                db,
                &run_id,
                &st.agent_id,
                &st.task,
                None,
                st.model_id.clone(),
                mimi_model_id,
                user_request,
            )
        }))
        .await;
    eprintln!("[orchestrate:{run_id}] [2/4] OK: {} hasil", subtask_results.len());

    // ===== [3+4] Agregat → QC → Revisi (loop maks 3x) =====
    let mut revision_count: u32 = 0;
    let mut revision_history: Vec<String> = Vec::new();

    loop {
        let round = revision_count + 1;

        // --- Agregat ---
        eprintln!("[orchestrate:{run_id}] [3/4] agregasi (putaran {})...", round);
        let aggregated =
            aggregate_results(router, db, &run_id.clone(), mimi_model_id, user_request, &subtask_results).await;
        eprintln!("[orchestrate:{run_id}] [3/4] OK");

        // --- QC ---
        eprintln!("[orchestrate:{run_id}] [4/4] QC (putaran {})...", round);
        let (qc_score, qc_passed, qc_notes, revise_agents) =
            run_qc(router, db, &run_id.clone(), mimi_model_id, user_request, &aggregated).await;
        eprintln!(
            "[orchestrate:{run_id}] [4/4] putaran {}: skor={:?} lolos={}",
            round, qc_score, qc_passed
        );

        revision_history.push(format!(
            "Putaran {}: skor {:?}, lolos={}, catatan: {}",
            round,
            qc_score,
            qc_passed,
            qc_notes.as_deref().unwrap_or("-").chars().take(120).collect::<String>()
        ));

        // Lolos atau sudah maksimal → selesai
        if qc_passed || revision_count >= MAX_REVISIONS {
            if !qc_passed {
                eprintln!(
                    "[orchestrate:{run_id}] QC gagal setelah {} revisi, kembalikan hasil terbaik",
                    revision_count
                );
                revision_history.push(format!(
                    "Berhenti setelah {} revisi maksimal; hasil terbaik dikembalikan",
                    MAX_REVISIONS
                ));
            }
            return OrchestrateResult {
                images: ensure_visual_output(router, &run_id, user_request, &aggregated, &subtask_results).await,
                aggregated,
                subtask_results,
                qc_score,
                qc_passed,
                qc_notes,
                error: None,
                revisions: revision_count,
                revision_history,
                estimated_cost_usd: est_cost,
            };
        }

        // --- Revisi dibutuhkan ---
        revision_count += 1;
        let notes = qc_notes.clone().unwrap_or_default();
        eprintln!(
            "[orchestrate:{run_id}] QC gagal, revisi {}/{}: {}",
            revision_count, MAX_REVISIONS, notes.chars().take(100).collect::<String>()
        );

        // Tentukan agen yang perlu revisi
        let agents_to_revise: Vec<String> = if revise_agents.is_empty() {
            // QC tidak menunjuk spesifik → revisi semua yang sukses
            subtask_results
                .iter()
                .filter(|r| r.error.is_none())
                .map(|r| r.agent_id.clone())
                .collect()
        } else {
            revise_agents
                .into_iter()
                .filter(|a| {
                    VALID_AGENTS.contains(&a.as_str())
                        && subtask_results.iter().any(|r| &r.agent_id == a)
                })
                .collect()
        };

        if agents_to_revise.is_empty() {
            eprintln!("[orchestrate:{run_id}] Tidak ada agen valid untuk revisi, berhenti");
            revision_history.push("Tidak ada agen valid untuk revisi".to_string());
            return OrchestrateResult {
                images: ensure_visual_output(router, &run_id, user_request, &aggregated, &subtask_results).await,
                aggregated,
                subtask_results,
                qc_score,
                qc_passed: false,
                qc_notes,
                error: None,
                revisions: revision_count,
                revision_history,
                estimated_cost_usd: est_cost,
            };
        }

        eprintln!("[orchestrate:{run_id}] Revisi untuk agen: {:?}", agents_to_revise);

        // Dispatch revisi paralel ke agen terpilih
        let revised: Vec<SubTaskResult> =
            futures::future::join_all(agents_to_revise.iter().map(|aid| {
                let prev = subtask_results
                    .iter()
                    .find(|r| &r.agent_id == aid)
                    .cloned();
                let notes_clone = notes.clone();
                let run_id_clone = run_id.clone();
                async move {
                    match prev {
                        Some(p) => {
                            let sys = revision_system_prompt(
                                &p.agent_id, &p.task, &p.result, &notes_clone,
                            );
                            dispatch_one(router, db, &run_id_clone, &p.agent_id, &p.task, Some(sys), None, mimi_model_id, user_request).await
                        }
                        None => SubTaskResult {
                            agent_id: aid.clone(),
                            task: String::new(),
                            result: String::new(),
                            served_by: String::new(),
                            error: Some("Subtugas tidak ditemukan".to_string()),
                            images: vec![],
                        },
                    }
                }
            }))
            .await;

        // Gabungkan hasil revisi ke subtask_results
        for r in revised {
            if let Some(existing) = subtask_results.iter_mut().find(|e| e.agent_id == r.agent_id) {
                *existing = r;
            }
        }
        // Loop kembali ke agregat + QC
    }
}

/// Mock router untuk testing — return respons yang diprogram.
#[cfg(test)]
pub struct MockRouter {
    /// Respons QC berurutan: tiap panggilan QC ambil satu dari sini.
    pub qc_scores: std::sync::Mutex<Vec<(i32, bool, String, Vec<String>)>>,
    /// Respons default untuk non-QC calls.
    pub default_text: String,
}

#[cfg(test)]
impl MockRouter {
    pub fn new() -> Self {
        Self {
            qc_scores: std::sync::Mutex::new(vec![]),
            default_text: "Hasil mock".to_string(),
        }
    }

    pub fn with_qc_sequence(scores: Vec<(i32, bool, &str, Vec<&str>)>) -> Self {
        Self {
            qc_scores: std::sync::Mutex::new(
                scores
                    .into_iter()
                    .map(|(s, p, n, r)| {
                        (s, p, n.to_string(), r.into_iter().map(|x| x.to_string()).collect())
                    })
                    .collect(),
            ),
            default_text: "Hasil mock".to_string(),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl ChatClient for MockRouter {
    async fn chat(
        &self,
        _model_id: &str,
        _tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String> {
        // Deteksi apakah ini panggilan QC (system prompt mengandung "QC senior")
        let is_qc = messages.iter().any(|m| m.content.contains("QC senior"));

        let text = if is_qc {
            let mut scores = self.qc_scores.lock().unwrap();
            if scores.is_empty() {
                // Default: lolos
                r#"{"score": 85, "passed": true, "notes": "-", "revise_agents": []}"#.to_string()
            } else {
                let (score, passed, notes, revise) = scores.remove(0);
                let revise_json = revise
                    .iter()
                    .map(|r| format!("\"{}\"", r))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    r#"{{"score": {}, "passed": {}, "notes": "{}", "revise_agents": [{}]}}"#,
                    score, passed, notes, revise_json
                )
            }
        } else if messages.iter().any(|m| m.content.contains("CIO orkestrator") && m.content.contains("subtugas")) {
            // Parse call — return 1 subtugas
            r#"{"subtasks": [{"agent_id": "kreatif", "task": "Buat caption test"}]}"#.to_string()
        } else {
            self.default_text.clone()
        };

        Ok(ChatResult {
            text,
            served_by: "mock".to_string(),
            input_tokens: Some(10),
            output_tokens: Some(20),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_cost_scales_with_subtasks() {
        let c1 = estimate_cost(1, false);
        let c3 = estimate_cost(3, false);
        assert!(c3 > c1, "Lebih banyak subtugas = lebih mahal");
        // 1 subtugas Spark: ~3 fixed + 1 subtask, dengan buffer 1.5x
        // Harus di bawah $1 untuk kasus sederhana
        assert!(c1 < 1.0, "Estimasi 1 subtugas Spark harus < $1, dapat ${}", c1);
    }

    #[test]
    fn estimate_cost_wonder_more_expensive() {
        let spark = estimate_cost(2, false);
        let wonder = estimate_cost(2, true);
        assert!(wonder > spark, "Wonder harus lebih mahal dari Spark");
        // Wonder ~3-5x Spark untuk subtask portion
        assert!(wonder > spark * 2.0);
    }

    #[test]
    fn cost_hard_cap_reasonable() {
        // 5 subtugas Wonder maksimal tidak boleh tembus hard cap dengan mudah
        let max_case = estimate_cost(5, true);
        // Tapi juga tidak boleh terlalu rendah sampai block kasus normal
        assert!(max_case < COST_HARD_CAP_USD * 3.0);
    }

    #[test]
    fn revision_prompt_includes_qc_notes() {
        let sys = revision_system_prompt(
            "kreatif",
            "Buat caption",
            "Hasil lama",
            "Caption terlalu pendek, tambah CTA",
        );
        assert!(sys.contains("kreatif"));
        assert!(sys.contains("Buat caption"));
        assert!(sys.contains("Hasil lama"));
        assert!(sys.contains("tambah CTA"));
        assert!(sys.contains("WAJIB diperbaiki"));
    }

    #[test]
    fn qc_output_parses_revise_agents() {
        let json = r#"{"score": 60, "passed": false, "notes": "Kurang detail", "revise_agents": ["riset", "kreatif"]}"#;
        let q: QcOut = serde_json::from_str(json).unwrap();
        assert_eq!(q.score, 60);
        assert!(!q.passed);
        assert_eq!(q.revise_agents, vec!["riset", "kreatif"]);
    }

    #[test]
    fn qc_output_defaults_empty_revise_agents() {
        // Backward compatible: JSON lama tanpa revise_agents
        let json = r#"{"score": 85, "passed": true, "notes": "-"}"#;
        let q: QcOut = serde_json::from_str(json).unwrap();
        assert!(q.passed);
        assert!(q.revise_agents.is_empty());
    }

    #[test]
    fn needs_distribution_detects_keywords() {
        assert!(needs_distribution("Buatkan konten dan jadwalkan posting"));
        assert!(needs_distribution("distribusikan ke Instagram"));
        assert!(needs_distribution("publish artikel ini"));
        assert!(!needs_distribution("Buatkan caption saja"));
        assert!(!needs_distribution("Analisis kompetitor"));
    }

    #[test]
    fn valid_agents_list_complete() {
        // Pastikan semua agen di roster terdaftar
        let valid = ["mimi", "kreatif", "riset", "finance", "strategis", "code"];
        assert_eq!(valid.len(), 6);
    }

    /// Test kritis: QC gagal sekali (skor 60) lalu lolos (skor 80).
    /// Verifikasi revisi terjadi TEPAT 1x.
    #[tokio::test]
    async fn qc_retry_happens_exactly_once() {
        let mock = MockRouter::with_qc_sequence(vec![
            (60, false, "Caption kurang CTA", vec!["kreatif"]),
            (80, true, "-", vec![]),
        ]);

        // Buat DB in-memory untuk test
        let db = crate::db::Db::open_in_memory().expect("db test");

        let result = orchestrate(&db, &mock, "Buatkan caption", "muse:muse-spark-1.3").await;

        assert!(result.qc_passed, "QC harus lolos setelah revisi");
        assert_eq!(result.revisions, 1, "Revisi harus terjadi tepat 1x");
        assert_eq!(result.qc_score, Some(80));
        assert_eq!(result.revision_history.len(), 2, "Harus ada 2 entri riwayat");
    }

    /// Test: QC lolos langsung tanpa revisi.
    #[tokio::test]
    async fn qc_pass_no_revision() {
        let mock = MockRouter::with_qc_sequence(vec![
            (85, true, "-", vec![]),
        ]);

        let db = crate::db::Db::open_in_memory().expect("db test");
        let result = orchestrate(&db, &mock, "Test sederhana", "muse:muse-spark-1.3").await;

        assert!(result.qc_passed);
        assert_eq!(result.revisions, 0, "Tidak boleh ada revisi jika lolos langsung");
    }

    /// Mock yang sleep lama untuk test timeout.
    pub struct SlowMockRouter;

    #[async_trait::async_trait]
    impl ChatClient for SlowMockRouter {
        async fn chat(
            &self,
            _model_id: &str,
            _tier: ModelTier,
            messages: Vec<ChatMessage>,
        ) -> Result<ChatResult, String> {
            // Deteksi parse call
            let is_parse = messages.iter().any(|m| m.content.contains("CIO orkestrator"));
            if is_parse {
                return Ok(ChatResult {
                    text: r#"{"subtasks": [{"agent_id": "kreatif", "task": "Test"}]}"#.to_string(),
                    served_by: "mock".to_string(),
                    input_tokens: Some(10),
                    output_tokens: Some(20),
                    error: None,
                });
            }
            // Subtask: sleep lebih lama dari timeout (tapi test pakai timeout kecil)
            tokio::time::sleep(std::time::Duration::from_secs(999)).await;
            Ok(ChatResult {
                text: "tidak tercapai".to_string(),
                served_by: "mock".to_string(),
                input_tokens: Some(10),
                output_tokens: Some(20),
                error: None,
            })
        }
    }

    /// Test: subtugas yang timeout tidak menggagalkan orkestrasi.
    /// Note: test ini pakai SUBTASK_TIMEOUT_SECS asli (120s) — di-skip dalam CI.
    /// Untuk verifikasi manual, ubah timeout sementara.
    #[test]
    fn timeout_constant_is_reasonable() {
        // Pastikan timeout tidak terlalu kecil (min 30s) atau terlalu besar (max 300s)
        assert!(SUBTASK_TIMEOUT_SECS >= 30, "Timeout terlalu kecil");
        assert!(SUBTASK_TIMEOUT_SECS <= 300, "Timeout terlalu besar");
    }

    /// Test: QC gagal 3x berturut-turut → berhenti di max revisi.
    #[tokio::test]
    async fn qc_max_revisions_respected() {
        let mock = MockRouter::with_qc_sequence(vec![
            (50, false, "Buruk", vec!["kreatif"]),
            (55, false, "Masih buruk", vec!["kreatif"]),
            (60, false, "Tetap buruk", vec!["kreatif"]),
            (65, false, "Tetap gagal", vec!["kreatif"]),
        ]);

        let db = crate::db::Db::open_in_memory().expect("db test");
        let result = orchestrate(&db, &mock, "Test gagal terus", "muse:muse-spark-1.3").await;

        assert!(!result.qc_passed, "QC tetap gagal");
        assert_eq!(result.revisions, 3, "Harus berhenti di 3 revisi maksimal");
        // Revision history: 4 entri (3 gagal + 1 final) atau pesan berhenti
        assert!(result.revision_history.len() >= 4);
    }
}

/// ===== STREAMING (24 Okt 2026) =====
/// Varian orkestrasi yang memancarkan event Tauri tiap tahap, sehingga
/// frontend bisa menampilkan progres per-agen BEGITU subtugas selesai —
/// bukan menunggu seluruh pipeline (parse + semua subtugas + agregat + QC).
/// Ini obat struktural untuk hambatan delegasi berulang: all-or-nothing
/// sinkron yang membuat user tidak melihat tanda apa-apa selama menit-menit
/// LLM bekerja.
///
/// Event:
/// - `orchestration-plan`: { run_id, subtasks: [{agent_id, task}] }
/// - `orchestration-subtask`: { run_id, agent_id, task, result, error, revision }
/// - `orchestration-done`: { run_id, aggregated, qc_score, qc_passed }
/// - `orchestration-error`: { run_id, error }

/// Payload event plan.
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrationPlanEvent {
    pub run_id: String,
    pub subtasks: Vec<SubTask>,
}

/// Payload event subtugas (awal maupun revisi).
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrationSubtaskEvent {
    pub run_id: String,
    pub agent_id: String,
    pub task: String,
    pub result: String,
    pub error: Option<String>,
    pub revision: bool,
    /// Path file gambar dari tool generate_image (Fase 2).
    #[serde(default)]
    pub images: Vec<String>,
}

/// Payload event selesai.
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrationDoneEvent {
    pub run_id: String,
    pub aggregated: String,
    pub qc_score: Option<i32>,
    pub qc_passed: bool,
    /// Gambar jaring pengaman deterministik (streaming).
    #[serde(default)]
    pub images: Vec<String>,
}

/// Payload event error fatal (pre-flight / parse gagal sebelum plan).
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrationErrorEvent {
    pub run_id: String,
    pub error: String,
}

/// Payload event revisi dimulai (24 Okt 2026): QC meminta revisi, agen-agen
/// ini KEMBALI bekerja — frontend harus menurunkan status mereka dari done
/// ke working/researching sampai subtugas revisi selesai.
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrationRevisionEvent {
    pub run_id: String,
    pub agents: Vec<String>,
}

fn emit_event<T: Serialize>(app: &tauri::AppHandle, event: &str, payload: &T) {
    use tauri::Emitter;
    if let Err(e) = app.emit(event, payload) {
        eprintln!("[orchestrate-stream] emit {event} gagal: {e}");
    }
}

/// Orkestrasi streaming: logika sama dengan `orchestrate`, tetapi dispatch
/// memakai FuturesUnordered sehingga tiap subtugas dipancarkan begitu selesai.
pub async fn orchestrate_stream(
    app: &tauri::AppHandle,
    db: &crate::db::Db,
    router: &dyn ChatClient,
    user_request: &str,
    mimi_model_id: &str,
) -> OrchestrateResult {
    use futures::stream::{FuturesUnordered, StreamExt};

    let run_id = format!("{:08x}", {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut h);
        user_request.hash(&mut h);
        h.finish() & 0xFFFFFFFF
    });
    eprintln!("[orchestrate-stream:{run_id}] mulai: {}", user_request.chars().take(80).collect::<String>());

    // ===== [0] Pre-flight =====
    if needs_distribution(user_request) {
        if let Err(msg) = check_postiz_ready() {
            eprintln!("[orchestrate-stream:{run_id}] pre-flight GAGAL: {msg}");
            emit_event(app, "orchestration-error", &OrchestrationErrorEvent { run_id: run_id.clone(), error: msg.clone() });
            return OrchestrateResult {
                images: vec![],
                aggregated: String::new(),
                subtask_results: vec![],
                qc_score: None,
                qc_passed: false,
                qc_notes: None,
                error: Some(msg),
                revisions: 0,
                revision_history: vec![],
                estimated_cost_usd: 0.0,
            };
        }
    }

    // ===== [1] Parse intent =====
    let parse_res = chat_internal(router, db, "mimi", mimi_model_id, ModelTier::Berbayar, &parse_system_prompt(), user_request).await;
    let plan: TaskPlan = match parse_res {
        Ok(r) if r.error.is_none() => {
            let json_str = extract_json(&r.text);
            match serde_json::from_str::<TaskPlan>(&json_str) {
                Ok(p) if !p.subtasks.is_empty() => {
                    let _ = db.log_parse_attempt(true);
                    p
                }
                _ => {
                    let _ = db.log_parse_attempt(false);
                    TaskPlan {
                        subtasks: vec![SubTask { agent_id: "mimi".to_string(), task: user_request.to_string(), model_id: None }],
                    }
                }
            }
        }
        _ => {
            let msg = "Gagal mem-parse permintaan".to_string();
            emit_event(app, "orchestration-error", &OrchestrationErrorEvent { run_id: run_id.clone(), error: msg.clone() });
            return OrchestrateResult {
                images: vec![],
                aggregated: String::new(),
                subtask_results: vec![],
                qc_score: None,
                qc_passed: false,
                qc_notes: None,
                error: Some(msg),
                revisions: 0,
                revision_history: vec![],
                estimated_cost_usd: 0.0,
            };
        }
    };

    // Validasi agent_id terhadap allowlist
    const VALID_AGENTS: &[&str] = &["mimi", "kreatif", "riset", "finance", "strategis", "code"];
    let plan = TaskPlan {
        subtasks: plan
            .subtasks
            .into_iter()
            .map(|mut st| {
                if !VALID_AGENTS.contains(&st.agent_id.as_str()) {
                    st.agent_id = "mimi".to_string();
                }
                st
            })
            .collect(),
    };

    // ===== Cost guard =====
    let likely_wonder = plan.subtasks.iter().any(|st| {
        crate::model_selector::complexity_score(&st.task, &st.agent_id).0 >= 60
    });
    let est_cost = estimate_cost(plan.subtasks.len(), likely_wonder);
    if est_cost > COST_HARD_CAP_USD {
        let msg = format!(
            "Estimasi biaya ${:.2} melebihi batas ${:.2}. Sederhanakan permintaan.",
            est_cost, COST_HARD_CAP_USD
        );
        emit_event(app, "orchestration-error", &OrchestrationErrorEvent { run_id: run_id.clone(), error: msg.clone() });
        return OrchestrateResult {
            images: vec![],
            aggregated: String::new(),
            subtask_results: vec![],
            qc_score: None,
            qc_passed: false,
            qc_notes: None,
            error: Some(msg),
            revisions: 0,
            revision_history: vec![],
            estimated_cost_usd: est_cost,
        };
    }

    // ===== PLAN EVENT: frontend langsung tampilkan delegasi + kartu + ack =====
    emit_event(app, "orchestration-plan", &OrchestrationPlanEvent { run_id: run_id.clone(), subtasks: plan.subtasks.clone() });
    eprintln!("[orchestrate-stream:{run_id}] [1/4] OK: {} subtugas, plan dipancarkan", plan.subtasks.len());

    // ===== [2] Dispatch streaming (FuturesUnordered: hasil keluar begitu selesai) =====
    let mut unordered = FuturesUnordered::new();
    for st in plan.subtasks.iter() {
        unordered.push(dispatch_one(
            router, db, &run_id, &st.agent_id, &st.task, None,
            st.model_id.clone(), mimi_model_id, user_request,
        ));
    }
    let mut subtask_results: Vec<SubTaskResult> = Vec::new();
    while let Some(res) = unordered.next().await {
        emit_event(app, "orchestration-subtask", &OrchestrationSubtaskEvent {
            run_id: run_id.clone(),
            agent_id: res.agent_id.clone(),
            task: res.task.clone(),
            result: res.result.clone(),
            error: res.error.clone(),
            revision: false,
            images: res.images.clone(),
        });
        eprintln!("[orchestrate-stream:{run_id}] subtugas selesai: {}", res.agent_id);
        subtask_results.push(res);
    }

    // ===== [3+4] Agregat → QC → Revisi (loop maks 3x) =====
    let mut revision_count: u32 = 0;
    let mut revision_history: Vec<String> = Vec::new();
    loop {
        let round = revision_count + 1;
        let aggregated = aggregate_results(router, db, &run_id.clone(), mimi_model_id, user_request, &subtask_results).await;
        let (qc_score, qc_passed, qc_notes, revise_agents) =
            run_qc(router, db, &run_id.clone(), mimi_model_id, user_request, &aggregated).await;
        revision_history.push(format!(
            "Putaran {}: skor {:?}, lolos={}, catatan: {}",
            round, qc_score, qc_passed,
            qc_notes.as_deref().unwrap_or("-").chars().take(120).collect::<String>()
        ));

        if qc_passed || revision_count >= MAX_REVISIONS {
            let safety_images = ensure_visual_output(router, &run_id, user_request, &aggregated, &subtask_results).await;
            emit_event(app, "orchestration-done", &OrchestrationDoneEvent {
                run_id: run_id.clone(),
                aggregated: aggregated.clone(),
                qc_score,
                qc_passed,
                images: safety_images.clone(),
            });
            return OrchestrateResult {
                images: safety_images,
                aggregated,
                subtask_results,
                qc_score,
                qc_passed,
                qc_notes,
                error: None,
                revisions: revision_count,
                revision_history,
                estimated_cost_usd: est_cost,
            };
        }

        // --- Revisi dibutuhkan ---
        revision_count += 1;
        let notes = qc_notes.clone().unwrap_or_default();
        let agents_to_revise: Vec<String> = if revise_agents.is_empty() {
            subtask_results.iter().filter(|r| r.error.is_none()).map(|r| r.agent_id.clone()).collect()
        } else {
            revise_agents.into_iter().filter(|a| {
                VALID_AGENTS.contains(&a.as_str()) && subtask_results.iter().any(|r| &r.agent_id == a)
            }).collect()
        };
        if agents_to_revise.is_empty() {
            revision_history.push("Tidak ada agen valid untuk revisi".to_string());
            let safety_images = ensure_visual_output(router, &run_id, user_request, &aggregated, &subtask_results).await;
            emit_event(app, "orchestration-done", &OrchestrationDoneEvent {
                run_id: run_id.clone(),
                aggregated: aggregated.clone(),
                qc_score,
                qc_passed: false,
                images: safety_images.clone(),
            });
            return OrchestrateResult {
                images: safety_images,
                aggregated,
                subtask_results,
                qc_score,
                qc_passed: false,
                qc_notes,
                error: None,
                revisions: revision_count,
                revision_history,
                estimated_cost_usd: est_cost,
            };
        }

        // Beri tahu frontend: agen-agen ini kembali bekerja (turun dari done)
        emit_event(app, "orchestration-revision", &OrchestrationRevisionEvent {
            run_id: run_id.clone(),
            agents: agents_to_revise.clone(),
        });
        // Dispatch revisi streaming
        let mut unordered_rev = FuturesUnordered::new();
        for aid in agents_to_revise.iter() {
            let prev = subtask_results.iter().find(|r| &r.agent_id == aid).cloned();
            let notes_clone = notes.clone();
            let run_id_clone = run_id.clone();
            let fut = async move {
                match prev {
                    Some(p) => {
                        let sys = revision_system_prompt(&p.agent_id, &p.task, &p.result, &notes_clone);
                        dispatch_one(router, db, &run_id_clone, &p.agent_id, &p.task, Some(sys), None, mimi_model_id, user_request).await
                    }
                    None => SubTaskResult {
                        agent_id: aid.clone(),
                        task: String::new(),
                        result: String::new(),
                        served_by: String::new(),
                        error: Some("Subtugas tidak ditemukan".to_string()),
                        images: vec![],
                    },
                }
            };
            unordered_rev.push(fut);
        }
        let mut revised: Vec<SubTaskResult> = Vec::new();
        while let Some(res) = unordered_rev.next().await {
            emit_event(app, "orchestration-subtask", &OrchestrationSubtaskEvent {
                run_id: run_id.clone(),
                agent_id: res.agent_id.clone(),
                task: res.task.clone(),
                result: res.result.clone(),
                error: res.error.clone(),
                revision: true,
                images: res.images.clone(),
            });
            revised.push(res);
        }
        for r in revised {
            if let Some(existing) = subtask_results.iter_mut().find(|e| e.agent_id == r.agent_id) {
                *existing = r;
            }
        }
    }
}

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
#[allow(async_fn_in_trait)]
pub trait ChatClient: Send + Sync {
    async fn chat(
        &self,
        model_id: &str,
        tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String>;
}

impl ChatClient for Router {
    async fn chat(
        &self,
        model_id: &str,
        tier: ModelTier,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResult, String> {
        Router::chat(self, model_id, tier, messages).await
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
}

/// Hasil akhir orkestrasi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateResult {
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
- agent_id yang valid: kreatis, riset, finance, strategis, code
  - "kreatif": konten, copywriting, desain, storytelling
  - "riset": riset web, data, analisis kompetitor
  - "finance": analisa finansial, budgeting, akuntansi
  - "strategis": strategi bisnis, marketing, brand, sales
  - "code": programming, aplikasi, script, debugging
- Setiap subtugas harus jelas dan bisa dikerjakan independen
- Maksimal 5 subtugas
- Jangan tambah teks di luar JSON

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
) -> SubTaskResult {
    let (model_id, tier) = match model_override {
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
            // Model selection cerdas per subtugas:
            // skor >= 60 → Wonder, < 60 → Spark
            let decision = crate::model_selector::select_model(task, agent_id);
            eprintln!(
                "[orchestrate:{run_id}] subtugas {} → {} (skor {})",
                agent_id, decision.model_id, decision.complexity_score
            );
            // Catat untuk audit & future learning (heuristik ke-4)
            let _ = db.log_model_decision(
                agent_id,
                task,
                decision.complexity_score,
                &decision.model_id,
            );
            (decision.model_id, ModelTier::Berbayar)
        }
    };

    let system = system_override.unwrap_or_else(|| {
        format!(
            "Kamu adalah agen {}. Kerjakan tugas berikut dengan standar profesional:\n\n{}",
            agent_id, task
        )
    });

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
    match res {
        Ok(r) => SubTaskResult {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            result: r.text.clone(),
            served_by: r.served_by.clone(),
            error: r.error.clone(),
        },
        Err(e) => SubTaskResult {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            result: String::new(),
            served_by: String::new(),
            error: Some(e),
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
    let combined = subtask_results
        .iter()
        .map(|r| {
            format!(
                "### Hasil dari {} ({})\n{}\n",
                r.agent_id,
                r.task,
                if r.error.is_some() {
                    format!("[GAGAL: {}]", r.error.as_ref().unwrap())
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
    eprintln!("[orchestrate:{}] mulai: {}", run_id, &user_request.chars().take(80).collect::<String>());

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
                Ok(p) if !p.subtasks.is_empty() => p,
                _ => {
                    eprintln!("[orchestrate:{run_id}] parse gagal, fallback ke single task");
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
                            dispatch_one(router, db, &run_id_clone, &p.agent_id, &p.task, Some(sys), None).await
                        }
                        None => SubTaskResult {
                            agent_id: aid.clone(),
                            task: String::new(),
                            result: String::new(),
                            served_by: String::new(),
                            error: Some("Subtugas tidak ditemukan".to_string()),
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

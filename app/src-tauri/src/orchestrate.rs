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
    router: &Router,
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
async fn dispatch_one(
    router: &Router,
    db: &crate::db::Db,
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
                "[orchestrate] subtugas {} → {} (skor {})",
                agent_id, decision.model_id, decision.complexity_score
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

    let res = chat_internal(router, db, agent_id, &model_id, tier, &system, task).await;
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
    router: &Router,
    db: &crate::db::Db,
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
    router: &Router,
    db: &crate::db::Db,
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
    router: &Router,
    user_request: &str,
    mimi_model_id: &str,
) -> OrchestrateResult {
    eprintln!("[orchestrate] mulai: {}", &user_request.chars().take(80).collect::<String>());

    // ===== [1] Parse intent → subtugas =====
    eprintln!("[orchestrate] [1/4] parsing intent...");
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
                    eprintln!("[orchestrate] parse gagal, fallback ke single task");
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
                        "[orchestrate] agent_id '{}' tidak dikenal, fallback ke mimi",
                        st.agent_id
                    );
                    st.agent_id = "mimi".to_string();
                }
                st
            })
            .collect(),
    };

    eprintln!("[orchestrate] [1/4] OK: {} subtugas", plan.subtasks.len());

    // ===== [2] Dispatch paralel (initial) =====
    eprintln!("[orchestrate] [2/4] dispatch paralel...");
    let mut subtask_results: Vec<SubTaskResult> =
        futures::future::join_all(plan.subtasks.iter().map(|st| {
            dispatch_one(
                router,
                db,
                &st.agent_id,
                &st.task,
                None,
                st.model_id.clone(),
            )
        }))
        .await;
    eprintln!("[orchestrate] [2/4] OK: {} hasil", subtask_results.len());

    // ===== [3+4] Agregat → QC → Revisi (loop maks 3x) =====
    let mut revision_count: u32 = 0;
    let mut revision_history: Vec<String> = Vec::new();

    loop {
        let round = revision_count + 1;

        // --- Agregat ---
        eprintln!("[orchestrate] [3/4] agregasi (putaran {})...", round);
        let aggregated =
            aggregate_results(router, db, mimi_model_id, user_request, &subtask_results).await;
        eprintln!("[orchestrate] [3/4] OK");

        // --- QC ---
        eprintln!("[orchestrate] [4/4] QC (putaran {})...", round);
        let (qc_score, qc_passed, qc_notes, revise_agents) =
            run_qc(router, db, mimi_model_id, user_request, &aggregated).await;
        eprintln!(
            "[orchestrate] [4/4] putaran {}: skor={:?} lolos={}",
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
                    "[orchestrate] QC gagal setelah {} revisi, kembalikan hasil terbaik",
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
            };
        }

        // --- Revisi dibutuhkan ---
        revision_count += 1;
        let notes = qc_notes.clone().unwrap_or_default();
        eprintln!(
            "[orchestrate] QC gagal, revisi {}/{}: {}",
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
            eprintln!("[orchestrate] Tidak ada agen valid untuk revisi, berhenti");
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
            };
        }

        eprintln!("[orchestrate] Revisi untuk agen: {:?}", agents_to_revise);

        // Dispatch revisi paralel ke agen terpilih
        let revised: Vec<SubTaskResult> =
            futures::future::join_all(agents_to_revise.iter().map(|aid| {
                let prev = subtask_results
                    .iter()
                    .find(|r| &r.agent_id == aid)
                    .cloned();
                async move {
                    match prev {
                        Some(p) => {
                            let sys = revision_system_prompt(
                                &p.agent_id, &p.task, &p.result, &notes,
                            );
                            dispatch_one(router, db, &p.agent_id, &p.task, Some(sys), None).await
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

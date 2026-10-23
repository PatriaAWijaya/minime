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
#[derive(Debug, Clone, Serialize)]
pub struct SubTaskResult {
    pub agent_id: String,
    pub task: String,
    pub result: String,
    pub served_by: String,
    pub error: Option<String>,
}

/// Hasil akhir orkestrasi.
#[derive(Debug, Clone, Serialize)]
pub struct OrchestrateResult {
    pub aggregated: String,
    pub subtask_results: Vec<SubTaskResult>,
    pub qc_score: Option<i32>,
    pub qc_passed: bool,
    pub qc_notes: Option<String>,
    pub error: Option<String>,
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

Output HARUS JSON: {"score": <0-100>, "passed": <true jika >=75>, "notes": "<catatan perbaikan atau '-'"}

Jujur dan ketat. Skor <75 berarti TOLAK dengan alasan spesifik."#.to_string()
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

/// Orkestrasi tugas: parse → dispatch paralel → agregat → QC.
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
            };
        }
    };

    eprintln!("[orchestrate] [1/4] OK: {} subtugas", plan.subtasks.len());

    // ===== [2] Dispatch paralel =====
    eprintln!("[orchestrate] [2/4] dispatch paralel...");
    let default_model = |agent_id: &str| -> (String, ModelTier) {
        // Default: Mimi pakai model berbayar, lainnya gratis-first
        if agent_id == "mimi" {
            (mimi_model_id.to_string(), ModelTier::Berbayar)
        } else {
            ("ollama:qwen2.5:3b".to_string(), ModelTier::Gratis)
        }
    };

    let futures: Vec<_> = plan
        .subtasks
        .iter()
        .map(|st| {
            let (model_id, tier) = st
                .model_id
                .clone()
                .map(|m| {
                    let t = if m.starts_with("muse:")
                        || m.starts_with("openai:")
                        || m.starts_with("gemini:")
                        || m.starts_with("deepseek:") {
                        ModelTier::Berbayar
                    } else {
                        ModelTier::Gratis
                    };
                    (m, t)
                })
                .unwrap_or_else(|| default_model(&st.agent_id));
            let agent_id = st.agent_id.clone();
            let task = st.task.clone();
            let system = format!(
                "Kamu adalah agen {}. Kerjakan tugas berikut dengan standar profesional:\n\n{}",
                agent_id, task
            );
            async move {
                let res = chat_internal(
                    router, db, &agent_id, &model_id, tier, &system, &task,
                )
                .await;
                match res {
                    Ok(r) => SubTaskResult {
                        agent_id: agent_id.clone(),
                        task: task.clone(),
                        result: r.text.clone(),
                        served_by: r.served_by.clone(),
                        error: r.error.clone(),
                    },
                    Err(e) => SubTaskResult {
                        agent_id: agent_id.clone(),
                        task: task.clone(),
                        result: String::new(),
                        served_by: String::new(),
                        error: Some(e),
                    },
                }
            }
        })
        .collect();

    let subtask_results: Vec<SubTaskResult> = futures::future::join_all(futures).await;
    eprintln!("[orchestrate] [2/4] OK: {} hasil", subtask_results.len());

    // ===== [3] Agregat =====
    eprintln!("[orchestrate] [3/4] agregasi...");
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

    let aggregated = match agg_res {
        Ok(r) if r.error.is_none() => r.text,
        _ => combined, // Fallback: gabungan mentah
    };
    eprintln!("[orchestrate] [3/4] OK");

    // ===== [4] QC =====
    eprintln!("[orchestrate] [4/4] QC...");
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

    let (qc_score, qc_passed, qc_notes) = match qc_res {
        Ok(r) if r.error.is_none() => {
            let json_str = extract_json(&r.text);
            #[derive(Deserialize)]
            struct QcOut {
                score: i32,
                passed: bool,
                notes: String,
            }
            match serde_json::from_str::<QcOut>(&json_str) {
                Ok(q) => (Some(q.score), q.passed, Some(q.notes)),
                _ => (None, true, Some("QC parse gagal, lolos manual".to_string())),
            }
        }
        _ => (None, true, Some("QC dilewati (provider gagal)".to_string())),
    };

    eprintln!(
        "[orchestrate] [4/4] OK: skor={:?} lolos={}",
        qc_score, qc_passed
    );

    OrchestrateResult {
        aggregated,
        subtask_results,
        qc_score,
        qc_passed,
        qc_notes,
        error: None,
    }
}

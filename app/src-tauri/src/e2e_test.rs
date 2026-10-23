//! E2E test sementara — workflow Ramadan (24 Okt 2026).
//! Skenario: riset + konten kalender 30 hari menjelang Ramadan
//!           + draft poster + caption + jadwal posting
//! Alur: Mimi → Kreatif → QC → draft Postiz
//!
//! HAPUS setelah test selesai.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2EResult {
    pub scenario: String,
    pub orchestrate_result: crate::orchestrate::OrchestrateResult,
    pub postiz_draft_id: Option<String>,
    pub postiz_error: Option<String>,
    pub started_at: String,
    pub finished_at: String,
    pub success: bool,
}

/// Jalankan E2E workflow Ramadan.
pub async fn run_ramadan_e2e(
    db: &crate::db::Db,
    router: &crate::providers::Router,
) -> E2EResult {
    let started = chrono::Utc::now().to_rfc3339();
    eprintln!("[e2e] ===== MULAI E2E RAMADAN =====");

    let scenario = "Riset dan buatkan konten kalender 30 hari menjelang Ramadan \
        untuk akun dakwah. Sertakan: tema harian, draft caption Instagram \
        untuk 5 hari pertama, konsep poster (deskripsi visual), dan jadwal \
        posting yang optimal.";

    // [1-4] Orkestrasi via Mimi
    eprintln!("[e2e] [1] Orkestrasi via Mimi...");
    let orch = crate::orchestrate::orchestrate(
        db,
        router,
        scenario,
        "muse:muse-spark-1.3",
    )
    .await;

    eprintln!(
        "[e2e] [2] Orkestrasi selesai. QC skor: {:?}, lolos: {}",
        orch.qc_score, orch.qc_passed
    );

    // [5] Buat draft Postiz (hanya bila QC lolos dan ada hasil)
    let (draft_id, draft_err) = if orch.qc_passed && !orch.aggregated.is_empty() {
        eprintln!("[e2e] [3] Buat draft Postiz...");
        // Ambil 500 karakter pertama sebagai konten draft
        let content: String = orch.aggregated.chars().take(500).collect();
        match crate::social::create_postiz_draft(
            format!("[E2E TEST] Kalender Konten Ramadan\n\n{}", content),
            vec!["instagram".to_string()],
            None, // tidak dijadwalkan, hanya draft
            vec![],
        )
        .await
        {
            Ok(r) => {
                eprintln!("[e2e] [3] Draft Postiz OK: {}", r.draft_id);
                (Some(r.draft_id), None)
            }
            Err(e) => {
                eprintln!("[e2e] [3] Draft Postiz GAGAL: {}", e);
                (None, Some(e))
            }
        }
    } else {
        eprintln!("[e2e] [3] Skip Postiz (QC tidak lolos atau hasil kosong)");
        (
            None,
            Some("Dilewati: QC tidak lolos atau hasil kosong".to_string()),
        )
    };

    let finished = chrono::Utc::now().to_rfc3339();
    let success = orch.qc_passed && draft_id.is_some();

    eprintln!(
        "[e2e] ===== SELESAI: success={} =====",
        success
    );

    // Tulis hasil ke file untuk monitoring via SSH
    let result = E2EResult {
        scenario: scenario.to_string(),
        orchestrate_result: orch,
        postiz_draft_id: draft_id,
        postiz_error: draft_err,
        started_at: started,
        finished_at: finished,
        success,
    };

    if let Ok(json) = serde_json::to_string_pretty(&result) {
        let path = std::env::temp_dir().join("minime_e2e_ramadan.json");
        let _ = std::fs::write(&path, json);
        eprintln!("[e2e] Hasil ditulis ke {:?}", path);
    }

    result
}

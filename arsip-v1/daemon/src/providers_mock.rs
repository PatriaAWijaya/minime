//! Mock LLM provider — ONLY for integration testing.
//! Registered when `MINIME_MOCK_LLM=1`. Returns deterministic canned
//! responses per agent so the full pipeline (routing → fallback → handoff →
//! QC scoring → approval → deliverable) can be verified end-to-end without
//! external LLM availability. NEVER used in production.

use crate::provider::{
    ChatChunk, ChatRequest, ModelInfo, ModelProvider, ProviderError, ProviderKind,
};
use tokio::sync::mpsc::{self, Receiver};

pub struct MockProvider {
    /// If true, the first call fails with Failed to prove fallback works.
    pub fail_first: bool,
    calls: std::sync::atomic::AtomicUsize,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            fail_first: std::env::var("MINIME_MOCK_FAIL_FIRST").is_ok(),
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl ModelProvider for MockProvider {
    fn id(&self) -> &str {
        "mock"
    }
    fn kind(&self) -> ProviderKind {
        ProviderKind::Free
    }
    fn list_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "mock-echo".to_string(),
            label: "Mock Echo (test only)".to_string(),
            usd_per_1m_in: 0.0,
            usd_per_1m_out: 0.0,
            context_window: 8192,
        }]
    }

    fn complete_stream(&self, req: ChatRequest) -> Receiver<Result<ChatChunk, ProviderError>> {
        let (tx, rx) = mpsc::channel(64);
        let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let fail_first = self.fail_first;
        tokio::spawn(async move {
            if fail_first && n == 0 {
                let _ = tx
                    .send(Err(ProviderError::Failed(
                        "mock: simulated first-call failure (fallback test)".to_string(),
                    )))
                    .await;
                return;
            }
            // Canned response per agent role, derived from the system prompt.
            let sp = req.system_prompt.to_lowercase();
            // Fase 3: echo playbook yang di-inject executor — bukti wiring skill.
            // Dihitung dulu agar muncul di SEMUA cabang output mock.
            let mut playbooks = Vec::new();
            for line in req.system_prompt.lines() {
                if let Some(rest) = line.strip_prefix("=== PLAYBOOK: ") {
                    if let Some(name) = rest.strip_suffix(" ===") {
                        playbooks.push(name.to_string());
                    }
                }
            }
            let pb_line = if playbooks.is_empty() {
                "PLAYBOOK AKTIF: (tidak ada playbook stage-level yang aktif)".to_string()
            } else {
                format!("PLAYBOOK AKTIF: {}", playbooks.join(", "))
            };
            let text = if sp.contains("juri senior") || sp.contains("qc") {
                format!(
                    "Hasil penilaian QC terhadap output tahap produksi:\n\
                     - Kebenaran & sumber: cukup (20/30)\n\
                     - Kesesuaian brief: baik (22/25)\n\
                     - Quality bar konten: baik (20/25)\n\
                     - Kesiapan produksi: baik (18/20)\n\
                     Total: 80/100 → LOLOS ke Approval dengan catatan ringan.\n\
                     SKOR: 80\n\
                     ALASAN: Output sesuai brief dan memenuhi quality bar dengan catatan minor.\n\
                     {pb_line}"
                )
            } else {
                let task_line = req
                    .messages
                    .to_string()
                    .chars()
                    .take(120)
                    .collect::<String>();
                format!(
                    "[MOCK OUTPUT — model: {}]\nRingkasan eksekusi mock untuk task ini. \
                     Konteks diterima: {}…\n\
                     {pb_line}\n\
                     (Ini adalah output deterministik dari mock provider untuk verifikasi pipeline.)",
                    req.model, task_line
                )
            };
            let in_tok = (req.system_prompt.len() as u64) / 4;
            let out_tok = (text.len() as u64) / 4;
            // Stream in two chunks to exercise the streaming path.
            let mid = text.len() / 2;
            for part in [&text[..mid], &text[mid..]] {
                let _ = tx
                    .send(Ok(ChatChunk {
                        delta_text: part.to_string(),
                        input_tokens: 0,
                        output_tokens: 0,
                        done: false,
                    }))
                    .await;
            }
            let _ = tx
                .send(Ok(ChatChunk {
                    delta_text: String::new(),
                    input_tokens: in_tok,
                    output_tokens: out_tok,
                    done: true,
                }))
                .await;
        });
        rx
    }
}

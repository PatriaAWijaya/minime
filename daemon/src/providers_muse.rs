//! Muse provider — Meta Model API (`https://api.meta.ai/v1`), OpenAI-compatible
//! Chat Completions (verified from Meta's public cookbook, Oct 2026).
//! Key dari secrets::api_key("muse") (Keychain, fallback env MINIME_MUSE_KEY).
//! Region check: HTTP 403/451 (atau pesan region) → ProviderError::RegionBlocked
//! agar executor otomatis fallback ke Gemini (sesuai arsitektur §2.2).

use crate::provider::{
    ChatChunk, ChatRequest, ModelInfo, ModelProvider, ProviderError, ProviderKind,
};
use crate::secrets;
use serde_json::json;
use tokio::sync::mpsc;

const BASE_URL: &str = "https://api.meta.ai/v1";
pub const DEFAULT_MODEL: &str = "muse-spark-1.3";

pub struct MuseProvider {
    client: reqwest::Client,
}

impl MuseProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(180))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

impl Default for MuseProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelProvider for MuseProvider {
    fn id(&self) -> &str {
        "muse"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::PaidApiKey
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: DEFAULT_MODEL.to_string(),
            label: "Muse Spark 1.3".to_string(),
            context_window: 1_048_576,
            usd_per_1m_in: 1.25,
            usd_per_1m_out: 4.25,
        }]
    }

    fn complete_stream(
        &self,
        req: ChatRequest,
    ) -> mpsc::Receiver<Result<ChatChunk, ProviderError>> {
        let (tx, rx) = mpsc::channel(64);
        let client = self.client.clone();
        tokio::spawn(async move {
            let res = call_muse(&client, &req, &tx).await;
            if let Err(e) = res {
                let _ = tx.send(Err(e)).await;
            }
        });
        rx
    }
}

async fn call_muse(
    client: &reqwest::Client,
    req: &ChatRequest,
    tx: &mpsc::Sender<Result<ChatChunk, ProviderError>>,
) -> Result<(), ProviderError> {
    let key = secrets::api_key("muse").ok_or_else(|| ProviderError::NoKey("muse".to_string()))?;
    let model = if req.model.is_empty() {
        DEFAULT_MODEL
    } else {
        req.model.as_str()
    };

    let mut messages = Vec::new();
    if !req.system_prompt.trim().is_empty() {
        messages.push(json!({"role": "system", "content": req.system_prompt}));
    }
    match &req.messages {
        serde_json::Value::Array(arr) => {
            for m in arr {
                let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if role == "system" || content.trim().is_empty() {
                    continue;
                }
                messages.push(json!({"role": role, "content": content}));
            }
        }
        serde_json::Value::String(s) => {
            messages.push(json!({"role": "user", "content": s}));
        }
        _ => {}
    }
    if messages.is_empty() {
        messages.push(json!({"role": "user", "content": "(pesan kosong)"}));
    }

    let mut body = json!({ "model": model, "messages": messages, "stream": false });
    if let Some(max) = req.max_tokens {
        body["max_tokens"] = json!(max);
    }

    let resp = client
        .post(format!("{BASE_URL}/chat/completions"))
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| ProviderError::Failed(format!("muse http: {e}")))?;

    let status = resp.status();
    let val: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ProviderError::Failed(format!("muse bad json: {e}")))?;

    if !status.is_success() {
        let msg = val
            .get("error")
            .and_then(|e| {
                e.get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("HTTP {status}"));
        // Region gate: Meta may refuse by geography — fall back, don't fail the task.
        if status.as_u16() == 403 || status.as_u16() == 451 || is_region_msg(&msg) {
            return Err(ProviderError::RegionBlocked(format!("muse: {msg}")));
        }
        if status.as_u16() == 429 {
            return Err(ProviderError::Failed(format!("muse rate-limited: {msg}")));
        }
        return Err(ProviderError::Failed(format!("muse {status}: {msg}")));
    }

    let text = val
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    let usage = val.get("usage");
    let in_tok = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| (text.len() as u64).div_ceil(4));
    let out_tok = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| (text.len() as u64).div_ceil(4));

    for piece in text.chars().collect::<Vec<_>>().chunks(400) {
        let s: String = piece.iter().collect();
        let _ = tx
            .send(Ok(ChatChunk {
                delta_text: s,
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
    Ok(())
}

fn is_region_msg(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("region")
        || m.contains("geograph")
        || m.contains("location")
        || m.contains("country")
}

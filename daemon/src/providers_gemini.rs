//! Gemini provider — native Google AI REST (`generateContent`).
//! Key dari secrets::api_key("gemini") (Keychain, fallback env MINIME_GEMINI_KEY).
//! Tanpa key → ProviderError::NoKey agar routing lanjut ke fallback.

use crate::provider::{
    ChatChunk, ChatRequest, ModelInfo, ModelProvider, ProviderError, ProviderKind,
};
use crate::secrets;
use serde_json::json;
use tokio::sync::mpsc;

pub struct GeminiProvider {
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: crate::provider::http_client(),
        }
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelProvider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::PaidApiKey
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.5-pro".to_string(),
                label: "Gemini 2.5 Pro".to_string(),
                context_window: 1_048_576,
                usd_per_1m_in: 1.25,
                usd_per_1m_out: 10.0,
            },
            ModelInfo {
                id: "gemini-2.5-flash".to_string(),
                label: "Gemini 2.5 Flash".to_string(),
                context_window: 1_048_576,
                usd_per_1m_in: 0.30,
                usd_per_1m_out: 2.50,
            },
        ]
    }

    fn complete_stream(
        &self,
        req: ChatRequest,
    ) -> mpsc::Receiver<Result<ChatChunk, ProviderError>> {
        let (tx, rx) = mpsc::channel(64);
        let client = self.client.clone();
        tokio::spawn(async move {
            let res = call_gemini(&client, &req, &tx).await;
            if let Err(e) = res {
                let _ = tx.send(Err(e)).await;
            }
        });
        rx
    }
}

async fn call_gemini(
    client: &reqwest::Client,
    req: &ChatRequest,
    tx: &mpsc::Sender<Result<ChatChunk, ProviderError>>,
) -> Result<(), ProviderError> {
    let key =
        secrets::api_key("gemini").ok_or_else(|| ProviderError::NoKey("gemini".to_string()))?;
    let model = if req.model.is_empty() {
        "gemini-2.5-flash"
    } else {
        req.model.as_str()
    };
    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

    let messages = openai_to_gemini_contents(&req.messages);
    let mut body = json!({ "contents": messages });
    if !req.system_prompt.trim().is_empty() {
        body["systemInstruction"] = json!({"parts":[{"text": req.system_prompt}]});
    }
    if let Some(max) = req.max_tokens {
        body["generationConfig"] = json!({"maxOutputTokens": max});
    }

    let resp = client
        .post(&url)
        .header("x-goog-api-key", key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ProviderError::Failed(format!("gemini http: {e}")))?;

    let status = resp.status();
    let val: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ProviderError::Failed(format!("gemini bad json: {e}")))?;
    if !status.is_success() {
        let msg = val
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        if status.as_u16() == 429 {
            return Err(ProviderError::Failed(format!("gemini rate-limited: {msg}")));
        }
        return Err(ProviderError::Failed(format!("gemini {status}: {msg}")));
    }

    let mut text = String::new();
    if let Some(cands) = val.get("candidates").and_then(|c| c.as_array()) {
        if let Some(parts) = cands
            .first()
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
        {
            for p in parts {
                if let Some(t) = p.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            }
        }
    }
    let usage = val.get("usageMetadata");
    let in_tok = usage
        .and_then(|u| u.get("promptTokenCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| (req.system_prompt.len() as u64 + text.len() as u64).div_ceil(4));
    let out_tok = usage
        .and_then(|u| u.get("candidatesTokenCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| (text.len() as u64).div_ceil(4));

    // Kirim sebagai beberapa chunk agar UI tetap hidup.
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

/// Convert OpenAI-style messages to Gemini `contents` (roles user/model).
fn openai_to_gemini_contents(messages: &serde_json::Value) -> serde_json::Value {
    let mut out = Vec::new();
    if let Some(arr) = messages.as_array() {
        for m in arr {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            if role == "system" {
                continue; // system goes to systemInstruction
            }
            let gem_role = if role == "assistant" { "model" } else { "user" };
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if content.trim().is_empty() {
                continue;
            }
            out.push(json!({"role": gem_role, "parts":[{"text": content}]}));
        }
    }
    if out.is_empty() {
        out.push(json!({"role":"user","parts":[{"text":"(pesan kosong)"}]}));
    }
    serde_json::Value::Array(out)
}

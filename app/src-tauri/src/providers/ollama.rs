use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderStatus, ProviderKind};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        let base_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self {
            base_url,
            client: http_client(),
        }
    }

    pub fn id() -> &'static str {
        "ollama"
    }

    pub async fn tags(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_| ProviderError::NotReachable("ollama".to_string()))?;
        if !resp.status().is_success() {
            return Err(ProviderError::NotReachable("ollama".to_string()));
        }
        #[derive(Deserialize)]
        struct Tags {
            models: Vec<TagModel>,
        }
        #[derive(Deserialize)]
        struct TagModel {
            name: String,
        }
        let tags: Tags = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("ollama tags: {e}")))?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: format!("ollama:{}", m.name),
                label: m.name,
            })
            .collect())
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let url = format!("{}/api/chat", self.base_url);
        let model = model.strip_prefix("ollama:").unwrap_or(model);
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|_| ProviderError::NotReachable("ollama".to_string()))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "ollama chat HTTP {}",
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ChatResp {
            message: RespMsg,
            prompt_eval_count: Option<u64>,
            eval_count: Option<u64>,
        }
        #[derive(Deserialize)]
        struct RespMsg {
            content: String,
        }
        let out: ChatResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("ollama chat: {e}")))?;
        Ok(ChatResult {
            text: out.message.content,
            served_by: Self::id().to_string(),
            input_tokens: out.prompt_eval_count,
            output_tokens: out.eval_count,
            error: None,
        })
    }

    pub async fn status(&self) -> ProviderStatus {
        match self.tags().await {
            Ok(models) => ProviderStatus {
                id: Self::id().to_string(),
                label: "Ollama".to_string(),
                kind: ProviderKind::Free,
                state: "connected".to_string(),
                detail: format!("{} model lokal", models.len()),
                models,
            },
            Err(_) => ProviderStatus {
                id: Self::id().to_string(),
                label: "Ollama".to_string(),
                kind: ProviderKind::Free,
                state: "unreachable".to_string(),
                detail: "Ollama tidak berjalan di localhost:11434".to_string(),
                models: vec![],
            },
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Klien OpenAI-compatible generik (Muse, Gemini, DeepSeek, RouterKu)
// ---------------------------------------------------------------------------

/// Satu klien untuk semua provider HTTP yang bicara skema
/// `POST {base_url}/chat/completions` ala OpenAI.

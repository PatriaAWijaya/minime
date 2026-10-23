use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub struct OpenAiCompatProvider {
    id: &'static str,
    label: &'static str,
    base_url: String,
    key_account: &'static str,
    models: Vec<ModelInfo>,
    client: reqwest::Client,
}

impl OpenAiCompatProvider {
    pub fn muse() -> Self {
        Self {
            id: "muse",
            label: "Muse",
            base_url: "https://api.meta.ai/v1".to_string(),
            key_account: "muse",
            models: vec![
                ModelInfo {
                    id: "muse:muse-spark-1.3".to_string(),
                    label: "Muse Spark 1.3".to_string(),
                },
                ModelInfo {
                    id: "muse:muse-wonder-1.0".to_string(),
                    label: "Muse Wonder 1.0 (power mode)".to_string(),
                },
                ModelInfo {
                    id: "muse:muse-image-1.0".to_string(),
                    label: "Muse Image 1.0".to_string(),
                },
                ModelInfo {
                    id: "muse:muse-voice-transcribe-1.0".to_string(),
                    label: "Muse Voice Transcribe 1.0".to_string(),
                },
                ModelInfo {
                    id: "muse:sam-3.1".to_string(),
                    label: "SAM 3.1".to_string(),
                },
            ],
            client: http_client(),
        }
    }

    pub fn gemini() -> Self {
        Self {
            id: "gemini",
            label: "Gemini",
            // Endpoint OpenAI-compatible resmi Google.
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            key_account: "gemini",
            models: vec![ModelInfo {
                id: "gemini:gemini-3.8-flash".to_string(),
                label: "Gemini 3.8 Flash".to_string(),
            }],
            client: http_client(),
        }
    }

    pub fn deepseek() -> Self {
        Self {
            id: "deepseek",
            label: "DeepSeek",
            base_url: "https://api.deepseek.com/v1".to_string(),
            key_account: "deepseek",
            models: vec![ModelInfo {
                id: "deepseek:deepseek-chat".to_string(),
                label: "DeepSeek Chat".to_string(),
            }],
            client: http_client(),
        }
    }

    pub fn routerku(base_url: Option<String>) -> Self {
        Self {
            id: "routerku",
            label: "RouterKu",
            base_url: base_url
                .filter(|u| !u.trim().is_empty())
                .unwrap_or_else(|| "https://organization.api-github.com/v1".to_string()),
            key_account: "routerku",
            // Katalog diisi dari GET {base_url}/models saat terhubung.
            models: vec![],
            client: http_client(),
        }
    }

    fn key(&self) -> Result<String, ProviderError> {
        secrets::api_key(self.key_account)
            .ok_or_else(|| ProviderError::NoKey(self.id.to_string()))
    }

    pub async fn status(&self) -> ProviderStatus {
        let has_key = self.key().is_ok();
        // Coba daftar model bila ada key (GET /models, skema OpenAI).
        let mut models = self.models.clone();
        let mut detail = String::new();
        let state = if !has_key {
            "no_key".to_string()
        } else {
            match self.fetch_models().await {
                Ok(fetched) => {
                    if !fetched.is_empty() {
                        models = fetched;
                    }
                    detail = format!("{} model", models.len());
                    "connected".to_string()
                }
                Err(e) => {
                    detail = e.to_string();
                    "unreachable".to_string()
                }
            }
        };
        if detail.is_empty() && !has_key {
            detail = "API key belum dikonfigurasi".to_string();
        }
        ProviderStatus {
            id: self.id.to_string(),
            label: self.label.to_string(),
            kind: ProviderKind::PaidApiKey,
            state,
            detail,
            models,
        }
    }

    pub async fn fetch_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let key = self.key()?;
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&key)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} models HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ModelsResp {
            data: Vec<ModelEntry>,
        }
        #[derive(Deserialize)]
        struct ModelEntry {
            id: String,
        }
        let out: ModelsResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        Ok(out
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: format!("{}:{}", self.id, m.id),
                label: m.id,
            })
            .collect())
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let key = self.key()?;
        let model = model
            .strip_prefix(&format!("{}:", self.id))
            .unwrap_or(model);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} chat HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ChatResp {
            choices: Vec<Choice>,
            usage: Option<Usage>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: RespMsg,
        }
        #[derive(Deserialize)]
        struct RespMsg {
            content: Option<String>,
        }
        #[derive(Deserialize)]
        struct Usage {
            prompt_tokens: Option<u64>,
            completion_tokens: Option<u64>,
        }
        let out: ChatResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        let text = out
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        Ok(ChatResult {
            text,
            served_by: self.id.to_string(),
            input_tokens: out.usage.as_ref().and_then(|u| u.prompt_tokens),
            output_tokens: out.usage.as_ref().and_then(|u| u.completion_tokens),
            error: None,
        })
    }
    /// Generate gambar via muse-image-1.0 (OpenAI-compatible /v1/images/generations).
    /// Mengembalikan bytes PNG/JPEG dari API.
    pub async fn image(
        &self,
        prompt: &str,
        size: &str,
    ) -> Result<Vec<u8>, ProviderError> {
        let key = self.key()?;
        let url = format!("{}/images/generations", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": "muse-image-1.0",
            "prompt": prompt,
            "size": size,
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} image HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct ImageResp {
            data: Vec<ImageData>,
        }
        #[derive(Deserialize)]
        struct ImageData {
            #[serde(default)]
            url: Option<String>,
            #[serde(default)]
            b64_json: Option<String>,
        }
        let out: ImageResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        let item = out.data.into_iter().next().ok_or_else(|| {
            ProviderError::Failed(format!("{}: respons image kosong", self.id))
        })?;
        if let Some(b64) = item.b64_json {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))
        } else if let Some(url) = item.url {
            let bytes = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?
                .bytes()
                .await
                .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
            Ok(bytes.to_vec())
        } else {
            Err(ProviderError::Failed(format!(
                "{}: respons image tanpa url/b64_json",
                self.id
            )))
        }
    }

    /// Transkripsi audio via muse-voice-transcribe-1.0.
    /// Mengembalikan teks hasil transkripsi.
    pub async fn transcribe(
        &self,
        audio_path: &str,
    ) -> Result<String, ProviderError> {
        let key = self.key()?;
        let url = format!("{}/audio/transcriptions", self.base_url.trim_end_matches('/'));
        let file_bytes = std::fs::read(audio_path)
            .map_err(|e| ProviderError::Failed(format!("baca audio gagal: {e}")))?;
        let file_name = std::path::Path::new(audio_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();
        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name);
        let form = reqwest::multipart::Form::new()
            .text("model", "muse-voice-transcribe-1.0")
            .part("file", part);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} transcribe HTTP {}",
                self.id,
                resp.status()
            )));
        }
        #[derive(Deserialize)]
        struct TranscribeResp {
            text: String,
        }
        let out: TranscribeResp = resp
            .json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        Ok(out.text)
    }

    /// Segmentasi via sam-3.1: kirim gambar + prompt teks, terima mask/box JSON.
    pub async fn segment(
        &self,
        image_b64: &str,
        prompt: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let key = self.key()?;
        let url = format!("{}/segmentations", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": "sam-3.1",
            "image": image_b64,
            "prompt": prompt,
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))?;
        if !resp.status().is_success() {
            return Err(ProviderError::Failed(format!(
                "{} segment HTTP {}",
                self.id,
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| ProviderError::Failed(format!("{}: {e}", self.id)))
    }
}

// ---------------------------------------------------------------------------
// OpenCode (gratis, via CLI)
// ---------------------------------------------------------------------------

/// Free tier via CLI `opencode` yang terpasang (pola yang sama dipakai
/// OpenBot: tanpa akun untuk tier gratis). Perintah:
/// `opencode run -m <model> <prompt>`.

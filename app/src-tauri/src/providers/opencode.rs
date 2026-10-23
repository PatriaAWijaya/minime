use super::types::{ChatMessage, ChatResult, ModelInfo, ProviderError, ProviderKind, ProviderStatus};
use crate::secrets;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub struct OpenCodeProvider;

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn id() -> &'static str {
        "opencode"
    }

    fn find_cli(&self) -> Option<std::path::PathBuf> {
        // 1. Env override explisit.
        if let Ok(p) = std::env::var("OPENCODE_BIN") {
            let p = std::path::PathBuf::from(p);
            if p.is_file() {
                return Some(p);
            }
        }
        // 2. PATH.
        if let Ok(paths) = std::env::var("PATH") {
            for dir in std::env::split_paths(&paths) {
                let p = dir.join("opencode");
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        // 3. Lokasi instalasi standar opencode (lihat README anomalyco/opencode
        //    "Installation Directory"): $HOME/bin lalu $HOME/.opencode/bin.
        if let Ok(home) = std::env::var("HOME") {
            for sub in ["bin/opencode", ".opencode/bin/opencode"] {
                let p = std::path::PathBuf::from(&home).join(sub);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn free_models() -> Vec<ModelInfo> {
        // Katalog free tier (label = daftar yang disetujui Patria).
        const MODELS: &[(&str, &str)] = &[
            ("opencode:glm-5.3-flash-free", "GLM-5.3-Flash Free"),
            ("opencode:deepseek-v4.1-flash-free", "DeepSeek V4.1 Flash Free"),
            ("opencode:nemotron-3.5-lightning-free", "Nemotron 3.5 Lightning Free"),
            ("opencode:nemotron-3.5-super-vl-free", "Nemotron 3.5 Super VL Free"),
            ("opencode:nemotron-3-ultra-free", "Nemotron 3 Ultra Free"),
            ("opencode:mimo-v2.6-flash-free", "MiMo-V2.6-Flash Free"),
            ("opencode:longcat-2.5-preview-free", "LongCat 2.5 Preview Free"),
            ("opencode:beam-free", "Beam Free"),
            ("opencode:big-pickle", "Big Pickle"),
            ("opencode:space-bunny-free", "Space Bunny Free"),
        ];
        MODELS
            .iter()
            .map(|(id, label)| ModelInfo {
                id: id.to_string(),
                label: label.to_string(),
            })
            .collect()
    }

    pub async fn status(&self) -> ProviderStatus {
        match self.find_cli() {
            Some(_) => ProviderStatus {
                id: Self::id().to_string(),
                label: "OpenCode".to_string(),
                kind: ProviderKind::Free,
                state: "connected".to_string(),
                detail: "CLI terpasang".to_string(),
                models: Self::free_models(),
            },
            None => ProviderStatus {
                id: Self::id().to_string(),
                label: "OpenCode".to_string(),
                kind: ProviderKind::Free,
                state: "unreachable".to_string(),
                detail: "CLI opencode tidak ditemukan di PATH".to_string(),
                models: Self::free_models(),
            },
        }
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatResult, ProviderError> {
        let bin = self
            .find_cli()
            .ok_or_else(|| ProviderError::NotReachable("opencode".to_string()))?;
        let model = model.strip_prefix("opencode:").unwrap_or(model);
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let out = tokio::process::Command::new(&bin)
            .args(["run", "-m", model])
            .arg(prompt)
            .output()
            .await
            .map_err(|e| ProviderError::Failed(format!("opencode: {e}")))?;
        if !out.status.success() {
            return Err(ProviderError::Failed(format!(
                "opencode exit {}: {}",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(ChatResult::text_only(text, Self::id().to_string()))
    }
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Tier model yang dipilih user di UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Gratis,
    Berbayar,
}

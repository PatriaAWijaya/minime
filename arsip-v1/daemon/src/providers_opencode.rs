//! OpenCode free-models provider — clean-room integration via the pinned
//! OpenCode CLI (same pattern OpenBot uses: download & pin the CLI, no account
//! needed for the free tier). The daemon shells out to:
//!   `opencode run -m <provider/model> --dir <workdir> <prompt>`
//! Free-model catalog below mirrors the IDs advertised by the CLI
//! (`opencode models`), labels match Patria's approved picker list.

use crate::provider::{
    ChatChunk, ChatRequest, ModelInfo, ModelProvider, ProviderError, ProviderKind,
};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Pinned CLI version — bump deliberately, never silently.
pub const OPENCODE_PINNED_VERSION: &str = "v1.19.3";

/// Free catalog: (cli_model_id, display label). Labels = approved free plan.
const FREE_MODELS: &[(&str, &str)] = &[
    (
        "opencode/muse-spark-1.3-contributor-free",
        "Muse Spark 1.3 Free",
    ),
    ("opencode/glm-5.3-flash-free", "GLM-5.3-Flash Free"),
    (
        "opencode/deepseek-v4.1-flash-free",
        "DeepSeek V4.1 Flash Free",
    ),
    (
        "opencode/nemotron-3.5-lightning-free",
        "Nemotron 3.5 Lightning Free",
    ),
    (
        "opencode/nemotron-3.5-super-vl-free",
        "Nemotron 3.5 Super VL Free",
    ),
    ("opencode/nemotron-3-ultra-free", "Nemotron 3 Ultra Free"),
    ("opencode/mimo-v2.6-flash-free", "MiMo-V2.6-Flash Free"),
    (
        "opencode/longcat-2.5-preview-free",
        "LongCat 2.5 Preview Free",
    ),
    ("opencode/big-pickle", "Big Pickle Free"),
    ("opencode/cat-jam-free", "Cat Jam Free"),
];

pub const DEFAULT_FREE_MODEL: &str = "opencode/muse-spark-1.3-contributor-free";

pub struct OpenCodeProvider {
    repo_dir: PathBuf,
}

impl OpenCodeProvider {
    pub fn new(repo_dir: PathBuf) -> Self {
        Self { repo_dir }
    }

    /// Locate the pinned CLI: repo tools/bin → PATH. Returns the binary path.
    fn find_cli(&self) -> Option<PathBuf> {
        let pinned = self.repo_dir.join("tools/bin/opencode");
        if pinned.is_file() {
            return Some(pinned);
        }
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|d| d.join("opencode"))
                .find(|p| p.is_file())
        })
    }

    /// Best-effort download of the pinned CLI into repo tools/bin.
    /// Called lazily on first use, never at daemon startup.
    fn ensure_cli(&self) -> Result<PathBuf, ProviderError> {
        if let Some(bin) = self.find_cli() {
            return Ok(bin);
        }
        let (os, arch) = Self::platform()?;
        let url = format!(
            "https://github.com/sst/opencode/releases/download/{ver}/opencode-{os}-{arch}.tar.gz",
            ver = OPENCODE_PINNED_VERSION
        );
        eprintln!("[opencode] CLI tidak ditemukan — mengunduh {OPENCODE_PINNED_VERSION} ...");
        let bin_dir = self.repo_dir.join("tools/bin");
        std::fs::create_dir_all(&bin_dir)
            .map_err(|e| ProviderError::Failed(format!("cannot create tools dir: {e}")))?;
        let tgz = bin_dir.join("opencode-dl.tar.gz");
        let status = std::process::Command::new("curl")
            .args([
                "-fsSL",
                "--max-time",
                "180",
                "-o",
                &tgz.to_string_lossy(),
                &url,
            ])
            .status()
            .map_err(|e| ProviderError::Failed(format!("curl gagal: {e}")))?;
        if !status.success() {
            return Err(ProviderError::Failed(format!(
                "unduh OpenCode CLI gagal (curl exit {status}); install manual: https://opencode.ai"
            )));
        }
        let status = std::process::Command::new("tar")
            .args([
                "xzf",
                &tgz.to_string_lossy(),
                "-C",
                &bin_dir.to_string_lossy(),
            ])
            .status()
            .map_err(|e| ProviderError::Failed(format!("tar gagal: {e}")))?;
        std::fs::remove_file(&tgz).ok();
        if !status.success() {
            return Err(ProviderError::Failed(
                "ekstrak OpenCode CLI gagal".to_string(),
            ));
        }
        let bin = bin_dir.join("opencode");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&bin)
                .map_err(|e| ProviderError::Failed(format!("baca metadata opencode: {e}")))?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&bin, perms).ok();
        }
        if bin.is_file() {
            eprintln!("[opencode] CLI terinstal di {}", bin.display());
            Ok(bin)
        } else {
            Err(ProviderError::Failed(
                "binary opencode tidak ditemukan setelah unduh".to_string(),
            ))
        }
    }

    fn platform() -> Result<(&'static str, &'static str), ProviderError> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(("linux", "x64"));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(("darwin", "arm64"));
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Ok(("darwin", "x64"));
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(("windows", "x64"));
        #[allow(unreachable_code)]
        Err(ProviderError::Failed(
            "platform tidak didukung untuk auto-download OpenCode CLI".to_string(),
        ))
    }
}

impl ModelProvider for OpenCodeProvider {
    fn id(&self) -> &str {
        "opencode"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Free
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        FREE_MODELS
            .iter()
            .map(|(id, label)| ModelInfo {
                id: id.to_string(),
                label: label.to_string(),
                context_window: 200_000,
                usd_per_1m_in: 0.0,
                usd_per_1m_out: 0.0,
            })
            .collect()
    }

    fn complete_stream(
        &self,
        req: ChatRequest,
    ) -> mpsc::Receiver<Result<ChatChunk, ProviderError>> {
        let (tx, rx) = mpsc::channel(64);
        let this = OpenCodeProvider {
            repo_dir: self.repo_dir.clone(),
        };
        tokio::spawn(async move {
            let res = this.run_cli(&req, &tx).await;
            if let Err(e) = res {
                let _ = tx.send(Err(e)).await;
            }
        });
        rx
    }
}

impl OpenCodeProvider {
    async fn run_cli(
        &self,
        req: &ChatRequest,
        tx: &mpsc::Sender<Result<ChatChunk, ProviderError>>,
    ) -> Result<(), ProviderError> {
        let bin = self.ensure_cli()?;
        let model = if req.model.is_empty() {
            DEFAULT_FREE_MODEL.to_string()
        } else {
            req.model.clone()
        };

        // Workdir: run inside the agent's workspace so any file activity
        // stays in the agent's sandbox. Fallback to a temp dir.
        let workdir = req
            .workdir
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var("MINIME_WORK_DIR").map(PathBuf::from).ok())
            .unwrap_or_else(|| std::env::temp_dir().join("minime-opencode"));

        // System prompt has no dedicated flag in `opencode run`; prefix it.
        let mut prompt = String::new();
        if !req.system_prompt.trim().is_empty() {
            prompt.push_str("INSTRUKSI SISTEM (patuhi selalu):\n");
            prompt.push_str(req.system_prompt.trim());
            prompt.push_str("\n\n---\n\n");
        }
        if let Some(max) = req.max_tokens {
            prompt.push_str(&format!(
                "(Jawaban maksimal ~{max} token; ringkas bila memungkinkan.)\n\n"
            ));
        }
        prompt.push_str(&messages_to_text(&req.messages));

        let timeout_ms: u64 = 600_000; // 10 menit per panggilan
        let mut child = tokio::process::Command::new(&bin)
            .args(["run", "-m", &model, "--dir", &workdir.to_string_lossy()])
            .arg(&prompt)
            .env("NO_COLOR", "1")
            .env("TERM", "dumb")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ProviderError::Failed(format!("gagal menjalankan opencode: {e}")))?;

        let stdout = child.stdout.take().ok_or_else(|| {
            ProviderError::Failed("tidak bisa membaca stdout opencode".to_string())
        })?;
        let child_pid = child.id();
        // Seluruh pembacaan stdout + tunggu proses dibatasi timeout yang sama.
        // (next_line() tanpa timeout akan menggantung selamanya bila CLI macet.)
        let run_result =
            tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), async {
                let mut lines = BufReader::new(stdout).lines();
                let mut full = String::new();
                // Stream stdout line-by-line as chunks.
                while let Ok(Some(line)) = lines.next_line().await {
                    if line.trim().is_empty() {
                        continue;
                    }
                    full.push_str(&line);
                    full.push('\n');
                    let _ = tx
                        .send(Ok(ChatChunk {
                            delta_text: line + "\n",
                            input_tokens: 0,
                            output_tokens: 0,
                            done: false,
                        }))
                        .await;
                }
                let output = child.wait_with_output().await;
                (full, output)
            })
            .await;
        let (full, output) = match run_result {
            Ok((full, Ok(output))) => (full, output),
            Ok((_, Err(e))) => {
                return Err(ProviderError::Failed(format!("opencode gagal: {e}")));
            }
            Err(_) => {
                // Timeout → bunuh proses via pid (handle sudah di-drop).
                if let Some(pid) = child_pid {
                    let _ = std::process::Command::new("kill")
                        .args(["-9", &pid.to_string()])
                        .output();
                }
                return Err(ProviderError::Failed(
                    "opencode timeout (10 menit)".to_string(),
                ));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail: String = stderr
                .lines()
                .rev()
                .take(8)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            return Err(ProviderError::Failed(format!(
                "opencode exit {}: {}",
                output.status,
                tail.chars().take(800).collect::<String>()
            )));
        }

        // Token counts are not reported by the CLI → estimate.
        let in_est = (prompt.len() as u64).div_ceil(4);
        let out_est = (full.len() as u64).div_ceil(4);
        let _ = tx
            .send(Ok(ChatChunk {
                delta_text: String::new(),
                input_tokens: in_est,
                output_tokens: out_est,
                done: true,
            }))
            .await;
        Ok(())
    }
}

/// Flatten OpenAI-style messages JSON into plain text.
fn messages_to_text(messages: &serde_json::Value) -> String {
    let mut out = String::new();
    if let Some(arr) = messages.as_array() {
        for m in arr {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if content.trim().is_empty() {
                continue;
            }
            out.push_str(&format!("[{role}]\n{content}\n\n"));
        }
    } else if let Some(s) = messages.as_str() {
        out.push_str(s);
    }
    out
}

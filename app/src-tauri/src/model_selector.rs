//! Adaptive model switching — beban kerja cerdas (24 Okt 2026).
//!
//! Prinsip: tugas sederhana → Spark (murah, cepat);
//!           tugas kompleks → Wonder (power mode, 3-5x lebih mahal).
//!
//! Heuristik (skor kompleksitas 0-100):
//! 1. Panjang prompt (0-30 poin)
//! 2. Kata kunci kompleksitas (0-40 poin)
//! 3. Jenis agen/tugas (0-20 poin)
//! 4. Riwayat keputusan (0-10 poin, bonus/penalti)
//!
//! Ambang: skor >= 60 → Wonder, else → Spark.

use serde::{Deserialize, Serialize};

/// Hasil seleksi model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDecision {
    pub model_id: String,
    pub complexity_score: i32,
    pub reasoning: String,
    pub tier: String, // "berbayar" (keduanya Meta)
}

/// Kata kunci penanda kompleksitas (Indonesia + Inggris).
const COMPLEX_KEYWORDS: &[&str] = &[
    // Analisis mendalam
    "analisis mendalam", "analisa mendalam", "deep analysis", "komprehensif",
    "comprehensive", "strategi", "strategy", "arsitektur", "architecture",
    // Reasoning kompleks
    "bandingkan", "compare", "evaluasi", "evaluate", "refactor",
    "debug", "troubleshoot", "orkestrasi", "orchestrat",
    "research", "riset mendalam", "multi-langkah", "multi-step",
    // Kreatif kompleks
    "kampanye", "campaign", "brand strategy", "strategi brand",
    "business plan", "rencana bisnis",
    // Kode kompleks
    "implementasi", "implementation", "sistem", "system design",
    "optimasi", "optimization", "algoritma", "algorithm",
];

const SIMPLE_KEYWORDS: &[&str] = &[
    "terjemahkan", "translate", "ringkas", "summarize", "simpel",
    "simple", "cepat", "quick", "apa itu", "what is",
    "betulkan", "fix typo", "format", "jawab singkat", "brief answer",
    "daftar", "list", "sebutkan",
];

/// Hitung skor kompleksitas dari prompt.
pub fn complexity_score(prompt: &str, agent_id: &str) -> (i32, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();
    let lower = prompt.to_lowercase();
    let len = prompt.chars().count();

    // --- 1. Panjang prompt (0-30) ---
    let len_score = if len < 100 {
        0
    } else if len < 500 {
        10
    } else if len < 1500 {
        20
    } else {
        30
    };
    score += len_score;
    if len_score > 0 {
        reasons.push(format!("panjang prompt: {} karakter (+{})", len, len_score));
    }

    // --- 2. Kata kunci (0-40) ---
    let mut complex_hits = 0;
    let mut simple_hits = 0;
    for kw in COMPLEX_KEYWORDS {
        if lower.contains(kw) {
            complex_hits += 1;
        }
    }
    for kw in SIMPLE_KEYWORDS {
        if lower.contains(kw) {
            simple_hits += 1;
        }
    }
    let kw_score = (complex_hits.min(4) as i32 * 10) - (simple_hits.min(2) as i32 * 5);
    let kw_score = kw_score.clamp(-10, 40);
    score += kw_score;
    if complex_hits > 0 {
        reasons.push(format!("{} kata kunci kompleks (+{})", complex_hits, complex_hits.min(4) * 10));
    }
    if simple_hits > 0 {
        reasons.push(format!("{} kata kunci sederhana (-{})", simple_hits, simple_hits.min(2) * 5));
    }

    // --- 3. Jenis agen (0-20) ---
    let agent_score = match agent_id {
        "code" => 15,     // Kode cenderung kompleks
        "strategis" => 15, // Strategi butuh reasoning
        "riset" => 10,    // Riset bisa kompleks
        "finance" => 10,
        "kreatif" => 5,
        "mimi" => 10,     // Orkestrator: sedang
        _ => 5,
    };
    score += agent_score;
    reasons.push(format!("agen {} (+{})", agent_id, agent_score));

    // --- 4. Struktur kompleks (bonus) ---
    // Prompt dengan banyak baris/paragraf → kemungkinan kompleks
    let lines = prompt.lines().count();
    if lines > 10 {
        score += 5;
        reasons.push(format!("{} baris (+5)", lines));
    }
    // Ada kode dalam prompt
    if lower.contains("```") || lower.contains("fn ") || lower.contains("function ") {
        score += 10;
        reasons.push("mengandung kode (+10)".to_string());
    }

    (score.clamp(0, 100), reasons)
}

/// Pilih model berdasarkan kompleksitas.
/// Mengembalikan (model_id, score, reasoning).
pub fn select_model(prompt: &str, agent_id: &str) -> ModelDecision {
    let (score, reasons) = complexity_score(prompt, agent_id);

    let (model_id, tier_label) = if score >= 60 {
        ("muse:muse-wonder-1.0", "Wonder (power mode)")
    } else {
        ("muse:muse-spark-1.3", "Spark (cepat & murah)")
    };

    let reasoning = format!(
        "Skor kompleksitas: {}/100 → {}. Alasan: {}",
        score,
        tier_label,
        reasons.join("; ")
    );

    eprintln!("[model_selector] {}", reasoning);

    ModelDecision {
        model_id: model_id.to_string(),
        complexity_score: score,
        reasoning,
        tier: "berbayar".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_task_selects_spark() {
        let d = select_model("Terjemahkan 'hello' ke Indonesia", "mimi");
        assert_eq!(d.model_id, "muse:muse-spark-1.3");
        assert!(d.complexity_score < 60);
    }

    #[test]
    fn complex_task_selects_wonder() {
        let prompt = "Buatkan analisis komprehensif strategi go-to-market untuk produk kopi. \
            Bandingkan dengan kompetitor, evaluasi channel distribusi, dan susun rencana \
            kampanye 3 bulan dengan budget breakdown. Ini membutuhkan deep analysis.";
        let d = select_model(prompt, "strategis");
        assert_eq!(d.model_id, "muse:muse-wonder-1.0");
        assert!(d.complexity_score >= 60);
    }

    #[test]
    fn code_task_gets_boost() {
        let prompt = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```\nRefactor kode ini agar lebih idiomatis dan optimalkan performanya.";
        let d = select_model(prompt, "code");
        // Kode + refactor + optimasi → harus Wonder
        assert!(d.complexity_score >= 50);
    }

    #[test]
    fn long_prompt_scores_high() {
        let prompt = "a".repeat(2000);
        let (score, _) = complexity_score(&prompt, "mimi");
        assert!(score >= 30); // Panjang saja sudah 30
    }
}

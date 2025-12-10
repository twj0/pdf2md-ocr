use crate::ocr_engine::OcrBlock;

const MATH_SYMBOLS: &[char] = &[
    '+', '-', '=', '/', '*', '^', '_', '%', '∞', '∑', '∫', '≈', '≠', '∂', '√', 'π', 'λ', 'θ',
    'β', 'α', 'γ', '\\', '{', '}', '[', ']', '≤', '≥',
];

/// Heuristic to flag blocks that look like formulas.
pub fn detect_formula_candidates(blocks: &[OcrBlock]) -> Vec<usize> {
    blocks
        .iter()
        .enumerate()
        .filter_map(|(idx, block)| {
            let text = block.text.trim();
            if text.is_empty() {
                return None;
            }
            let math_count = text.chars().filter(|c| MATH_SYMBOLS.contains(c)).count();
            let len = text.chars().count().max(1);
            let ratio = math_count as f32 / len as f32;

            let looks_like_inline_latex =
                text.contains("\\frac") || text.contains("\\sum") || text.contains("\\int");

            if ratio > 0.25 || looks_like_inline_latex {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

pub fn wrap_formula(text: &str) -> String {
    format!("$$\n{}\n$$", text.trim())
}

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    Tesseract,
    Paddle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Toggle on/off disk cache
    pub enabled: bool,
    /// Directory for cached artifacts
    pub dir: PathBuf,
    /// Cache preprocessed images
    pub preprocess: bool,
    /// Cache OCR results to skip re-run
    pub ocr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// DPI for rendering PDF pages
    pub dpi: u32,
    /// OCR languages (e.g., "eng+chi_sim+equ")
    pub languages: String,
    /// Enable adaptive language detection
    pub detect_language: bool,
    /// Enable image preprocessing
    pub preprocess: bool,
    /// Number of parallel threads
    pub threads: usize,
    /// OCR backend
    pub engine: EngineKind,
    /// Run layout analysis (PP-Structure style grouping)
    pub layout: bool,
    /// Enable math OCR pipeline
    pub math_ocr: bool,
    /// Optional PaddleOCR model directory (det/cls/rec ONNX files)
    pub paddle_model_dir: Option<PathBuf>,
    /// Optional LaTeX-OCR/Math model directory
    pub math_model_dir: Option<PathBuf>,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Prefer GPU acceleration when available
    pub use_gpu: bool,
    /// Auto-tune config based on document heuristics
    pub auto_config: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dpi: 300,
            languages: "eng+chi_sim+equ".to_string(),
            detect_language: true,
            preprocess: true,
            threads: num_cpus::get(),
            engine: EngineKind::Paddle,
            layout: true,
            math_ocr: true,
            paddle_model_dir: None,
            math_model_dir: None,
            cache: CacheConfig {
                enabled: true,
                dir: PathBuf::from(".cache/rust-ocr2md"),
                preprocess: true,
                ocr: true,
            },
            use_gpu: false,
            auto_config: true,
        }
    }
}

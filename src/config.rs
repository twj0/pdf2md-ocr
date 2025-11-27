use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// DPI for rendering PDF pages
    pub dpi: u32,
    
    /// OCR languages (e.g., "eng+chi_sim+equ")
    pub languages: String,
    
    /// Enable image preprocessing
    pub preprocess: bool,
    
    /// Number of parallel threads
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dpi: 300,
            languages: "eng+chi_sim+equ".to_string(),
            preprocess: true,
            threads: num_cpus::get(),
        }
    }
}

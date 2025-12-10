use whatlang::{Detector, Lang};

pub struct LanguageDetector {
    detector: Detector,
}

impl LanguageDetector {
    pub fn new() -> Self {
        Self {
            detector: Detector::new(),
        }
    }

    /// Detect likely language code compatible with Tesseract/Paddle configs.
    pub fn detect(&self, text: &str) -> Option<&'static str> {
        let info = self.detector.detect(text)?;
        let lang = info.lang();
        match lang {
            Lang::Eng => Some("eng"),
            Lang::Cmn => Some("chi_sim"),
            Lang::Jpn => Some("jpn"),
            Lang::Kor => Some("kor"),
            Lang::Spa => Some("spa"),
            Lang::Fra => Some("fra"),
            _ => None,
        }
    }

    /// Merge detected language with existing hint string like "eng+chi_sim".
    pub fn merge_with_hint(&self, existing: &str, detected: &str) -> String {
        if existing.split('+').any(|l| l == detected) {
            existing.to_string()
        } else {
            format!("{}+{}", detected, existing)
        }
    }
}

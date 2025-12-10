use anyhow::{Context, Result};
use image::DynamicImage;
use paddle_ocr_rs::ocr_lite::OcrLite;
use paddle_ocr_rs::ocr_result::TextBlock as PaddleTextBlock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tesseract_rs::TesseractAPI;

use crate::config::{Config, EngineKind};
use crate::language::LanguageDetector;
use crate::{layout, math};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockType {
    Text,
    Formula,
    Table,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OcrBlock {
    pub text: String,
    pub confidence: f32,
    pub bbox: Option<BoundingBox>,
    pub block_type: BlockType,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OcrPage {
    pub blocks: Vec<OcrBlock>,
    pub detected_language: Option<String>,
}

pub struct OcrEngine {
    primary: EngineKind,
    languages: String,
    tessdata_dir: PathBuf,
    paddle: Option<Arc<Mutex<PaddleWrapper>>>,
    language_detector: LanguageDetector,
}

impl OcrEngine {
    pub fn new(config: &Config) -> Result<Self> {
        let tessdata_dir = Self::get_tessdata_dir();
        // Verify Tesseract is present if needed for primary/math/lang detection
        if matches!(config.engine, EngineKind::Tesseract) || config.math_ocr || config.detect_language {
            Self::verify_tesseract(&tessdata_dir)?;
        }

        let paddle = if matches!(config.engine, EngineKind::Paddle) {
            Some(Arc::new(Mutex::new(PaddleWrapper::new(
                config.paddle_model_dir.clone(),
                config.threads,
            )?)))
        } else {
            None
        };

        Ok(Self {
            primary: config.engine,
            languages: config.languages.clone(),
            tessdata_dir,
            paddle,
            language_detector: LanguageDetector::new(),
        })
    }

    fn get_tessdata_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("TESSDATA_PREFIX") {
            return PathBuf::from(dir);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata).join("tesseract-rs").join("tessdata");
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(".tesseract-rs").join("tessdata");
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("tesseract-rs")
                    .join("tessdata");
            }
        }

        PathBuf::from("./tessdata")
    }

    pub fn recognize(&self, image: &DynamicImage, config: &Config) -> Result<OcrPage> {
        let mut detected_language = None;
        let mut language_hint = self.languages.clone();

        if config.detect_language {
            if let Some(sample) = self.sample_language_text(image) {
                if let Some(lang) = self.language_detector.detect(&sample) {
                    detected_language = Some(lang.to_string());
                    language_hint = self.language_detector.merge_with_hint(&language_hint, lang);
                }
            }
        }

        let mut blocks = match self.primary {
            EngineKind::Paddle => self.recognize_with_paddle(image)?,
            EngineKind::Tesseract => {
                let text = self.recognize_with_tesseract(image, &language_hint)?;
                vec![OcrBlock {
                    text,
                    confidence: 1.0,
                    bbox: None,
                    block_type: BlockType::Text,
                    language: detected_language.clone().or_else(|| Some(language_hint.clone())),
                }]
            }
        };

        if config.layout {
            layout::sort_by_reading_order(&mut blocks);
        }

        if config.math_ocr {
            let candidates = math::detect_formula_candidates(&blocks);
            for idx in candidates {
                if let Some(bbox) = blocks[idx].bbox.clone() {
                    let crop = crop_image(image, &bbox);
                    let formula_text = self.recognize_formula(&crop)?;
                    blocks[idx].text = math::wrap_formula(&formula_text);
                    blocks[idx].block_type = BlockType::Formula;
                } else {
                    blocks[idx].block_type = BlockType::Formula;
                }
            }
        }

        Ok(OcrPage {
            blocks,
            detected_language,
        })
    }

    fn recognize_formula(&self, image: &DynamicImage) -> Result<String> {
        // Prefer math language pack if available
        let formula_langs = "equ+eng+chi_sim";
        self.recognize_with_tesseract(image, formula_langs)
    }

    fn recognize_with_tesseract(&self, image: &DynamicImage, languages: &str) -> Result<String> {
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();
        let image_data = gray.into_raw();

        let api = TesseractAPI::new();
        api.init(
            self.tessdata_dir
                .to_str()
                .unwrap_or("."),
            languages,
        )
        .map_err(|e| anyhow::anyhow!("Tesseract initialization failed: {:?}", e))?;

        api.set_image(
            &image_data,
            width as i32,
            height as i32,
            1,
            width as i32,
        )
        .map_err(|e| anyhow::anyhow!("Failed to set image: {:?}", e))?;

        let text = api
            .get_utf8_text()
            .map_err(|e| anyhow::anyhow!("OCR recognition failed: {:?}", e))?;

        Ok(text)
    }

    fn sample_language_text(&self, image: &DynamicImage) -> Option<String> {
        let thumbnail = image.thumbnail(640, 640);
        self.recognize_with_tesseract(&thumbnail, "eng+chi_sim")
            .ok()
            .map(|t| t.chars().take(400).collect())
    }

    fn recognize_with_paddle(&self, image: &DynamicImage) -> Result<Vec<OcrBlock>> {
        let engine = self
            .paddle
            .as_ref()
            .context("Paddle OCR not initialized. Set --engine paddle and ensure models exist.")?;
        let mut guard = engine.lock().unwrap();
        let rgb = image.to_rgb8();
        let padding = guard.padding;
        let max_side_len = guard.max_side_len;
        let box_score_thresh = guard.box_score_thresh;
        let box_thresh = guard.box_thresh;
        let un_clip_ratio = guard.un_clip_ratio;
        let res = guard.detect(
            &rgb,
            padding,
            max_side_len,
            box_score_thresh,
            box_thresh,
            un_clip_ratio,
            true,
            false,
        )?;

        let mut blocks = Vec::with_capacity(res.text_blocks.len());
        for tb in res.text_blocks {
            let bbox = points_to_bbox(&tb);
            blocks.push(OcrBlock {
                text: tb.text,
                confidence: tb.text_score,
                bbox,
                block_type: BlockType::Text,
                language: None,
            });
        }

        Ok(blocks)
    }

    fn verify_tesseract(tessdata_dir: &PathBuf) -> Result<()> {
        let api = TesseractAPI::new();
        api.init(tessdata_dir.to_str().unwrap_or("."), "eng")
            .map_err(|_| {
                anyhow::anyhow!(
                    "Tesseract initialization failed. Ensure tessdata exists at: {}\nSet TESSDATA_PREFIX or run download_tessdata.ps1",
                    tessdata_dir.display()
                )
            })?;
        Ok(())
    }
}

fn points_to_bbox(tb: &PaddleTextBlock) -> Option<BoundingBox> {
    if tb.box_points.is_empty() {
        return None;
    }
    let min_x = tb.box_points.iter().map(|p| p.x).min()?;
    let max_x = tb.box_points.iter().map(|p| p.x).max()?;
    let min_y = tb.box_points.iter().map(|p| p.y).min()?;
    let max_y = tb.box_points.iter().map(|p| p.y).max()?;

    Some(BoundingBox {
        x: min_x,
        y: min_y,
        width: max_x.saturating_sub(min_x),
        height: max_y.saturating_sub(min_y),
    })
}

fn crop_image(image: &DynamicImage, bbox: &BoundingBox) -> DynamicImage {
    let x = bbox.x.min(image.width().saturating_sub(1));
    let y = bbox.y.min(image.height().saturating_sub(1));
    let w = bbox.width.min(image.width().saturating_sub(x));
    let h = bbox.height.min(image.height().saturating_sub(y));
    image.crop_imm(x, y, w, h)
}

struct PaddleWrapper {
    inner: OcrLite,
    padding: u32,
    max_side_len: u32,
    box_score_thresh: f32,
    box_thresh: f32,
    un_clip_ratio: f32,
}

impl PaddleWrapper {
    fn new(model_dir: Option<PathBuf>, threads: usize) -> Result<Self> {
        let base_dir = model_dir.unwrap_or_else(|| PathBuf::from("./models/paddle"));
        let det = base_dir.join("ch_PP-OCRv4_det_infer.onnx");
        let cls = base_dir.join("ch_ppocr_mobile_v2.0_cls_infer.onnx");
        let rec = base_dir.join("ch_PP-OCRv4_rec_infer.onnx");

        if !det.exists() || !cls.exists() || !rec.exists() {
            anyhow::bail!(
                "PaddleOCR models not found under {}. Expected det/cls/rec ONNX files.\n\
                 Download PP-OCR models (det/cls/rec) into the directory or set --paddle-model-dir to the model folder.",
                base_dir.display()
            );
        }

        let mut inner = OcrLite::new();
        inner
            .init_models(
                det.to_str().unwrap_or_default(),
                cls.to_str().unwrap_or_default(),
                rec.to_str().unwrap_or_default(),
                threads,
            )
            .context("Failed to initialize PaddleOCR models")?;

        Ok(Self {
            inner,
            padding: 30,
            max_side_len: 1920,
            box_score_thresh: 0.3,
            box_thresh: 0.6,
            un_clip_ratio: 1.6,
        })
    }

    fn detect(
        &mut self,
        image: &image::RgbImage,
        padding: u32,
        max_side_len: u32,
        box_score_thresh: f32,
        box_thresh: f32,
        un_clip_ratio: f32,
        do_angle: bool,
        most_angle: bool,
    ) -> Result<paddle_ocr_rs::ocr_result::OcrResult, paddle_ocr_rs::ocr_error::OcrError> {
        self.inner.detect(
            image,
            padding,
            max_side_len,
            box_score_thresh,
            box_thresh,
            un_clip_ratio,
            do_angle,
            most_angle,
        )
    }
}

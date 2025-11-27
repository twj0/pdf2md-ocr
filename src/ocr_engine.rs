use anyhow::Result;
use image::DynamicImage;
use std::path::PathBuf;
use tesseract_rs::TesseractAPI;

use crate::config::Config;

pub struct OcrEngine {
    languages: String,
    tessdata_dir: PathBuf,
}

impl OcrEngine {
    pub fn new(config: &Config) -> Result<Self> {
        // Get tessdata directory
        let tessdata_dir = Self::get_tessdata_dir();
        
        // Verify Tesseract is available
        Self::verify_tesseract(&tessdata_dir)?;

        Ok(Self {
            languages: config.languages.clone(),
            tessdata_dir,
        })
    }

    fn get_tessdata_dir() -> PathBuf {
        // Check TESSDATA_PREFIX environment variable first
        if let Ok(dir) = std::env::var("TESSDATA_PREFIX") {
            return PathBuf::from(dir);
        }
        
        // Use default location based on OS
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata)
                    .join("tesseract-rs")
                    .join("tessdata");
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join(".tesseract-rs")
                    .join("tessdata");
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
        
        // Fallback to current directory
        PathBuf::from("./tessdata")
    }

    pub fn recognize(&self, image: &DynamicImage) -> Result<String> {
        // Convert to grayscale for better OCR
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();
        let image_data = gray.into_raw();

        // Initialize Tesseract API
        let api = TesseractAPI::new();
        
        // Initialize with tessdata directory and language
        api.init(
            self.tessdata_dir.to_str().unwrap_or("."),
            &self.languages,
        ).map_err(|e| anyhow::anyhow!("Tesseract initialization failed: {:?}", e))?;

        // Set the image data
        api.set_image(
            &image_data,
            width as i32,
            height as i32,
            1, // bytes per pixel (grayscale)
            width as i32, // bytes per line
        ).map_err(|e| anyhow::anyhow!("Failed to set image: {:?}", e))?;

        // Get the recognized text
        let text = api.get_utf8_text()
            .map_err(|e| anyhow::anyhow!("OCR recognition failed: {:?}", e))?;

        Ok(text)
    }

    fn verify_tesseract(tessdata_dir: &PathBuf) -> Result<()> {
        // Try to initialize Tesseract to verify it's working
        let api = TesseractAPI::new();
        
        api.init(
            tessdata_dir.to_str().unwrap_or("."),
            "eng",
        ).map_err(|_| {
            anyhow::anyhow!(
                "Tesseract initialization failed. Please ensure:\n\
                1. tessdata directory exists at: {}\n\
                2. Language data files (eng.traineddata, etc.) are in the tessdata directory\n\
                3. Or set TESSDATA_PREFIX environment variable to point to the tessdata directory",
                tessdata_dir.display()
            )
        })?;

        Ok(())
    }
}

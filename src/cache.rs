use crate::config::CacheConfig;
use crate::pdf_processor::PageResult;
use anyhow::Result;
use image::DynamicImage;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct CacheManager {
    config: CacheConfig,
    ocr_dir: PathBuf,
    preprocess_dir: PathBuf,
}

impl CacheManager {
    pub fn new(config: &CacheConfig) -> Result<Self> {
        let ocr_dir = config.dir.join("ocr");
        let preprocess_dir = config.dir.join("preprocess");

        if config.enabled {
            fs::create_dir_all(&ocr_dir)?;
            fs::create_dir_all(&preprocess_dir)?;
        }

        Ok(Self {
            config: config.clone(),
            ocr_dir,
            preprocess_dir,
        })
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn make_hash(
        &self,
        pdf_path: &Path,
        page_num: usize,
        dpi: u32,
        config_fingerprint: &str,
        image_bytes: &[u8],
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pdf_path.to_string_lossy().as_bytes());
        hasher.update(page_num.to_le_bytes());
        hasher.update(dpi.to_le_bytes());
        hasher.update(config_fingerprint.as_bytes());
        hasher.update(image_bytes);
        let digest = hasher.finalize();
        hex::encode(digest)
    }

    pub fn load_page(&self, hash: &str) -> Result<Option<PageResult>> {
        if !self.config.enabled || !self.config.ocr {
            return Ok(None);
        }
        let path = self.ocr_dir.join(format!("{hash}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)?;
        let page: PageResult = serde_json::from_str(&data)?;
        Ok(Some(page))
    }

    pub fn store_page(&self, hash: &str, page: &PageResult) -> Result<()> {
        if !self.config.enabled || !self.config.ocr {
            return Ok(());
        }
        let path = self.ocr_dir.join(format!("{hash}.json"));
        let data = serde_json::to_string(page)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_preprocessed(&self, hash: &str) -> Result<Option<DynamicImage>> {
        if !self.config.enabled || !self.config.preprocess {
            return Ok(None);
        }
        let path = self.preprocess_dir.join(format!("{hash}.png"));
        if !path.exists() {
            return Ok(None);
        }
        let img = image::open(path)?;
        Ok(Some(img))
    }

    pub fn store_preprocessed(&self, hash: &str, image: &DynamicImage) -> Result<()> {
        if !self.config.enabled || !self.config.preprocess {
            return Ok(());
        }
        let path = self.preprocess_dir.join(format!("{hash}.png"));
        image.save(path)?;
        Ok(())
    }
}

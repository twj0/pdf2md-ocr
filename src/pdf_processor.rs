use anyhow::{Context, Result};
use image::{DynamicImage, RgbaImage};
use indicatif::ProgressBar;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::ocr_engine::OcrEngine;

#[derive(Clone, Debug)]
pub struct PageResult {
    pub page_num: usize,
    pub text: String,
    pub image_width: u32,
    pub image_height: u32,
}

pub struct PdfProcessor {
    pdf_path: PathBuf,
    dpi: u32,
    page_count: usize,
}

impl PdfProcessor {
    pub fn new<P: AsRef<Path>>(pdf_path: P, dpi: u32) -> Result<Self> {
        let pdf_path = pdf_path.as_ref().to_path_buf();
        
        // Load PDF temporarily to get page count
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library("pdfium"))
                .context("Failed to load PDFium library. Please ensure PDFium is installed.")?
        );

        let document = pdfium
            .load_pdf_from_file(&pdf_path, None)
            .context("Failed to load PDF file")?;

        let page_count = document.pages().len() as usize;

        Ok(Self {
            pdf_path,
            dpi,
            page_count,
        })
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn process_pages(
        &self,
        page_range: &[usize],
        ocr_engine: &OcrEngine,
        config: &Config,
        progress_bar: &ProgressBar,
    ) -> Result<Vec<PageResult>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let pdf_path = self.pdf_path.clone();
        let dpi = self.dpi;

        // Process pages sequentially since pdfium is not thread-safe by default
        // and we need to reload the document for each thread anyway
        for &page_num in page_range {
            match process_single_page(&pdf_path, page_num, dpi, ocr_engine, config) {
                Ok(result) => {
                    results.lock().unwrap().push(result);
                }
                Err(e) => {
                    errors.lock().unwrap().push((page_num, e.to_string()));
                }
            }
            progress_bar.inc(1);
        }

        let mut final_results = results.lock().unwrap().clone();
        final_results.sort_by_key(|r| r.page_num);

        let errors = errors.lock().unwrap();
        if !errors.is_empty() {
            eprintln!("⚠️  Errors occurred on {} pages:", errors.len());
            for (page, err) in errors.iter() {
                eprintln!("  Page {}: {}", page, err);
            }
        }

        Ok(final_results)
    }
}

fn process_single_page(
    pdf_path: &Path,
    page_num: usize,
    dpi: u32,
    ocr_engine: &OcrEngine,
    config: &Config,
) -> Result<PageResult> {
    // Load PDF for this page
    let pdfium = Pdfium::new(
        Pdfium::bind_to_system_library()
            .or_else(|_| Pdfium::bind_to_library("pdfium"))
            .context("Failed to load PDFium library")?
    );

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .context("Failed to load PDF file")?;

    // Render PDF page to image
    let page = document
        .pages()
        .get((page_num - 1) as u16)
        .context(format!("Failed to get page {}", page_num))?;

    let render_config = PdfRenderConfig::new()
        .set_target_width((page.width().value * dpi as f32 / 72.0) as i32)
        .set_maximum_height((page.height().value * dpi as f32 / 72.0) as i32);

    let bitmap = page
        .render_with_config(&render_config)
        .context(format!("Failed to render page {}", page_num))?;

    // Convert to DynamicImage
    let image = bitmap_to_image(&bitmap)?;

    // Preprocess if enabled
    let processed_image = if config.preprocess {
        crate::image_processor::preprocess_image(image)?
    } else {
        image
    };

    // Perform OCR
    let text = ocr_engine.recognize(&processed_image)
        .context(format!("OCR failed on page {}", page_num))?;

    Ok(PageResult {
        page_num,
        text,
        image_width: processed_image.width(),
        image_height: processed_image.height(),
    })
}

fn bitmap_to_image(bitmap: &PdfBitmap) -> Result<DynamicImage> {
    let width = bitmap.width() as u32;
    let height = bitmap.height() as u32;
    let buffer = bitmap.as_raw_bytes();

    // PDFium returns BGRA format
    let mut rgba_buffer = Vec::with_capacity(buffer.len());
    for chunk in buffer.chunks_exact(4) {
        rgba_buffer.push(chunk[2]); // R
        rgba_buffer.push(chunk[1]); // G
        rgba_buffer.push(chunk[0]); // B
        rgba_buffer.push(chunk[3]); // A
    }

    let image = RgbaImage::from_raw(width, height, rgba_buffer)
        .context("Failed to create image from bitmap")?;

    Ok(DynamicImage::ImageRgba8(image))
}

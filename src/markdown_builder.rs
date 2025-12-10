use anyhow::Result;
use chrono::Local;
use std::path::Path;

use crate::ocr_engine::BlockType;
use crate::pdf_processor::PageResult;

pub struct MarkdownBuilder {
    include_metadata: bool,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self {
            include_metadata: true,
        }
    }

    pub fn build(&self, results: &[PageResult], source_path: &Path) -> Result<String> {
        let mut markdown = String::new();

        // Add metadata header
        if self.include_metadata {
            markdown.push_str(&self.generate_metadata(source_path, results));
            markdown.push_str("\n---\n\n");
        }

        // Add content from each page
        for result in results {
            markdown.push_str(&format!("## Page {}\n\n", result.page_num));

            for block in &result.blocks {
                let cleaned_text = self.clean_ocr_text(&block.text);
                match block.block_type {
                    BlockType::Formula => {
                        markdown.push_str(&cleaned_text);
                        markdown.push_str("\n\n");
                    }
                    _ => {
                        markdown.push_str(&cleaned_text);
                        markdown.push_str("\n\n");
                    }
                }
            }
        }

        Ok(markdown)
    }

    fn generate_metadata(&self, source_path: &Path, results: &[PageResult]) -> String {
        format!(
            "# Document OCR Result\n\n\
            - **Source**: {}\n\
            - **Processed**: {}\n\
            - **Total Pages**: {}\n\
            - **Tool**: RustOCR2md\n",
            source_path.display(),
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            results.len()
        )
    }

    fn clean_ocr_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for MarkdownBuilder {
    fn default() -> Self {
        Self::new()
    }
}

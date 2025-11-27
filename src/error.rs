use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("PDF processing error: {0}")]
    PdfError(String),

    #[error("OCR engine error: {0}")]
    OcrEngineError(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, OcrError>;

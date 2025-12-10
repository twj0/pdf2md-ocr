mod pdf_processor;
mod ocr_engine;
mod image_processor;
mod markdown_builder;
mod config;
mod error;
mod cache;
mod language;
mod layout;
mod math;

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;
use std::io::{self, Write};

use crate::config::{CacheConfig, Config, EngineKind};
use crate::pdf_processor::PdfProcessor;
use crate::ocr_engine::OcrEngine;
use crate::markdown_builder::MarkdownBuilder;

#[derive(Parser)]
#[command(name = "RustOCR2md")]
#[command(about = "High-performance PDF to Markdown converter with OCR\n\nUsage: Drag and drop a PDF file onto this executable, or run from command line.", long_about = None)]
struct Cli {
    /// Input PDF file path (can be provided by dragging file onto exe)
    #[arg(index = 1)]
    input: Option<PathBuf>,

    /// Output markdown file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number of threads (default: CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// DPI for PDF rendering (default: 300)
    #[arg(short, long, default_value = "300")]
    dpi: u32,

    /// Languages for OCR (default: eng+chi_sim+equ)
    #[arg(short, long, default_value = "eng+chi_sim+equ")]
    languages: String,

    /// OCR backend: tesseract or paddle
    #[arg(long, value_enum, default_value = "paddle")]
    engine: EngineKind,

    /// Enable PP-Structure style layout analysis
    #[arg(long, default_value = "true")]
    layout: bool,

    /// Enable language detection and dynamic model switch
    #[arg(long, default_value = "true")]
    detect_language: bool,

    /// Enable math OCR for formulas
    #[arg(long, default_value = "true")]
    math_ocr: bool,

    /// PaddleOCR model directory (det/cls/rec ONNX files)
    #[arg(long)]
    paddle_model_dir: Option<PathBuf>,

    /// Optional math model directory (LaTeX-OCR)
    #[arg(long)]
    math_model_dir: Option<PathBuf>,

    /// Enable on-disk cache (preprocess + OCR)
    #[arg(long, default_value = "true")]
    cache: bool,

    /// Cache preprocessed images
    #[arg(long, default_value = "true")]
    cache_preprocess: bool,

    /// Cache OCR results
    #[arg(long, default_value = "true")]
    cache_ocr: bool,

    /// Override cache directory
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    /// Prefer GPU acceleration (if supported by ONNX Runtime)
    #[arg(long, default_value = "false")]
    use_gpu: bool,

    /// Auto-tune pipeline based on document type
    #[arg(long, default_value = "true")]
    auto_config: bool,

    /// Enable image preprocessing for better accuracy
    #[arg(long, default_value = "true")]
    preprocess: bool,

    /// Page range (e.g., 1-10, or "all")
    #[arg(long, default_value = "all")]
    pages: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ Error: {}", e);
        if let Some(source) = e.source() {
            eprintln!("   Caused by: {}", source);
        }
        wait_for_enter();
        std::process::exit(1);
    }
}

fn wait_for_enter() {
    println!("\nPress Enter to exit...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();
    
    // Get input path - either from argument or prompt user
    let input_path = match cli.input {
        Some(path) => path,
        None => {
            println!("🚀 RustOCR2md - High-performance PDF OCR Tool");
            println!("================================================");
            println!("\nNo input file provided.");
            println!("Usage: Drag a PDF file onto this executable, or:");
            println!("       rust-ocr2md.exe <pdf_file>");
            println!("\nEnter PDF file path (or drag file here):");
            print!("> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            // Remove quotes and whitespace that Windows adds when dragging
            let path = input.trim().trim_matches('"').trim();
            if path.is_empty() {
                anyhow::bail!("No file path provided");
            }
            PathBuf::from(path)
        }
    };
    
    // Validate input file exists
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {}", input_path.display());
    }
    
    if input_path.extension().map(|e| e.to_ascii_lowercase()) != Some("pdf".into()) {
        anyhow::bail!("Input file must be a PDF: {}", input_path.display());
    }

    println!("🚀 RustOCR2md - High-performance PDF OCR Tool");
    println!("================================================");

    // Setup configuration
    let default_cache_dir = Config::default().cache.dir;
    let config = Config {
        dpi: cli.dpi,
        languages: cli.languages.clone(),
        detect_language: cli.detect_language,
        preprocess: cli.preprocess,
        threads: cli.threads.unwrap_or_else(num_cpus::get),
        engine: cli.engine,
        layout: cli.layout,
        math_ocr: cli.math_ocr,
        paddle_model_dir: cli.paddle_model_dir.clone(),
        math_model_dir: cli.math_model_dir.clone(),
        cache: CacheConfig {
            enabled: cli.cache,
            dir: cli.cache_dir.unwrap_or(default_cache_dir),
            preprocess: cli.cache_preprocess,
            ocr: cli.cache_ocr,
        },
        use_gpu: cli.use_gpu,
        auto_config: cli.auto_config,
    };

    println!("📄 Input: {}", input_path.display());
    println!("⚙️  Config: {} DPI, {} threads, Languages: {}, Engine: {:?}", 
             config.dpi, config.threads, config.languages, config.engine);

    // Initialize components
    let pdf_processor = PdfProcessor::new(&input_path, config.dpi)?;
    let total_pages = pdf_processor.page_count();
    
    println!("📖 Total pages: {}", total_pages);

    // Parse page range
    let page_range = parse_page_range(&cli.pages, total_pages)?;
    println!("🎯 Processing pages: {:?}", page_range);

    // Setup progress bar
    let pb = ProgressBar::new(page_range.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("█▓▒░ "),
    );

    // Process pages with OCR
    let ocr_engine = OcrEngine::new(&config)?;
    let results = pdf_processor.process_pages(&page_range, &ocr_engine, &config, &pb)?;

    pb.finish_with_message("✅ OCR completed!");

    // Build markdown
    println!("📝 Building Markdown...");
    let markdown = MarkdownBuilder::new().build(&results, &input_path)?;

    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = input_path.clone();
        path.set_extension("md");
        path
    });

    // Write output
    std::fs::write(&output_path, markdown)?;

    let duration = start.elapsed();
    println!("\n✨ Success!");
    println!("📄 Output: {}", output_path.display());
    println!("⏱️  Time: {:.2}s", duration.as_secs_f64());
    println!("🚀 Speed: {:.2} pages/sec", page_range.len() as f64 / duration.as_secs_f64());

    wait_for_enter();
    Ok(())
}

fn parse_page_range(range_str: &str, total_pages: usize) -> Result<Vec<usize>> {
    if range_str == "all" {
        return Ok((1..=total_pages).collect());
    }

    let parts: Vec<&str> = range_str.split('-').collect();
    match parts.len() {
        1 => {
            let page: usize = parts[0].parse()?;
            if page < 1 || page > total_pages {
                anyhow::bail!("Page {} out of range (1-{})", page, total_pages);
            }
            Ok(vec![page])
        }
        2 => {
            let start: usize = parts[0].parse()?;
            let end: usize = parts[1].parse()?;
            if start < 1 || end > total_pages || start > end {
                anyhow::bail!("Invalid page range: {}", range_str);
            }
            Ok((start..=end).collect())
        }
        _ => anyhow::bail!("Invalid page range format: {}", range_str),
    }
}

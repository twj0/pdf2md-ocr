

🚀 高性能 PDF OCR 转 Markdown 工具，使用 Rust 编写，专为处理大体积学术 PDF 文件设计。

## ✨ 特性

- 🚀 **极速处理**: 多线程并行 OCR，充分利用 CPU 性能
- 🎯 **高准确率**: 支持英语、简体中文和数学公式识别
- 🔧 **图像预处理**: 自适应阈值 (Otsu) 和中值滤波降噪，提升 OCR 质量
- 📊 **进度显示**: 实时显示处理进度和速度
- 💾 **Markdown输出**: 直接生成 Markdown 格式文档
- 🖱️ **拖放支持**: 直接将 PDF 拖到 exe 上即可运行
- 📦 **独立运行**: 编译后单文件 + pdfium.dll 即可使用

## 📋 系统要求

### 必需软件

1. **Rust** (已安装 ✓)
2. **Tesseract OCR** (需要安装)
3. **PDFium库** (可选，推荐安装)

### 安装Tesseract OCR

#### Windows
1. 下载安装包: https://github.com/UB-Mannheim/tesseract/wiki
2. 安装时选择以下语言包:
   - English (eng)
   - Simplified Chinese (chi_sim)
   - Math/Equations (equ)
3. 添加到系统PATH (安装程序会询问)

验证安装:
```bash
tesseract --version
tesseract --list-langs
```

### 安装PDFium (可选但推荐)

下载预编译库:
- Windows: https://github.com/bblanchon/pdfium-binaries/releases
- 将 `pdfium.dll` 放在系统PATH或项目目录

## 🚀 快速开始

### 方法一：使用预编译版本（推荐）

1. 下载 `release_package` 文件夹
2. 运行 `download_tessdata.ps1` 下载 OCR 语言数据
3. 将 PDF 文件拖到 `rust-ocr2md.exe` 上即可

### 方法二：从源码编译

```bash
# 克隆项目
git clone https://github.com/yourusername/RustOCR2md.git
cd RustOCR2md

# 下载依赖库
powershell -ExecutionPolicy Bypass -File download_pdfium.ps1
powershell -ExecutionPolicy Bypass -File download_tessdata.ps1

# 编译
cargo build --release

# 运行
.\target\release\rust-ocr2md.exe input.pdf
```

### 基本使用

```bash
# 方式1: 直接拖放 PDF 到 exe 上

# 方式2: 命令行
rust-ocr2md.exe input.pdf
rust-ocr2md.exe input.pdf -o output.md

# 指定页面范围
rust-ocr2md.exe input.pdf --pages 1-10

# 自定义线程数和DPI
rust-ocr2md.exe input.pdf --threads 8 --dpi 400

# 禁用图像预处理(更快但可能降低准确度)
rust-ocr2md.exe input.pdf --preprocess false
```

### 命令行参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `<PDF_FILE>` | 输入PDF文件路径 (位置参数) | 必需 |
| `-o, --output` | 输出Markdown文件路径 | 与输入同名.md |
| `-t, --threads` | 并行线程数 | CPU核心数 |
| `-d, --dpi` | PDF渲染DPI | 300 |
| `-l, --languages` | OCR语言 | eng+chi_sim+equ |
| `--preprocess` | 启用图像预处理 | true |
| `--pages` | 页面范围 (如: 1-10) | all |

## 📖 使用示例

### 示例1: 处理学术论文

```bash
rust-ocr2md.exe paper.pdf -o paper.md --dpi 350 --threads 12
```

### 示例2: 快速预览前几页

```bash
rust-ocr2md.exe book.pdf --pages 1-5 --preprocess false
```

### 示例3: 高质量OCR

```bash
rust-ocr2md.exe document.pdf --dpi 400 --languages eng+chi_sim+equ --threads 16
```

## ⚡ 性能优化建议

1. **线程数**: 设置为CPU核心数的1-1.5倍
2. **DPI**: 
   - 低质量PDF: 400+
   - 标准PDF: 300
   - 高清PDF: 200-250
3. **图像预处理**: 
   - 扫描文档: 启用
   - 电子文档: 可禁用以提速
4. **页面范围**: 处理部分页面测试最佳配置

## 🏗️ 项目架构

```
src/
├── main.rs              # 入口和CLI
├── config.rs            # 配置管理
├── error.rs             # 错误类型
├── pdf_processor.rs     # PDF处理和页面渲染
├── ocr_engine.rs        # Tesseract OCR封装
├── image_processor.rs   # 图像预处理
└── markdown_builder.rs  # Markdown生成
```

## 🔧 技术栈

- **PDF处理**: pdfium-render
- **OCR引擎**: tesseract-rs
- **图像处理**: image, imageproc
- **并行处理**: rayon
- **CLI**: clap
- **进度条**: indicatif

## 📝 输出格式

生成的Markdown包含:
- 文档元数据 (源文件、处理时间、页数)
- 按页分隔的OCR文本
- 自动清理的文本格式

## ⚠️ 常见问题

### 错误: "Failed to load PDFium library"
- 安装PDFium或使用系统库

### 错误: "Tesseract initialization failed"
- 确认Tesseract已安装且在PATH中
- 检查语言包是否安装: `tesseract --list-langs`

### OCR准确率低
- 提高DPI值 (300 -> 400)
- 启用图像预处理
- 检查PDF质量

### 处理速度慢
- 减少线程数避免过度竞争
- 降低DPI值
- 禁用预处理
- 分批处理页面



🚀 高性能 PDF OCR 转 Markdown 工具，使用 Rust 编写，专为处理大体积学术 PDF 文件设计。

## ✨ 特性

- 🚀 **双引擎**: PaddleOCR（ONNX Runtime）+ Tesseract 可切换，支持按页自动语言检测
- 📐 **布局分析**: PP-Structure 风格的文本框排序，保留阅读顺序
- 🧠 **数学公式**: 公式区域启发式检测，使用 Tesseract `equ` 语言包生成公式片段（可封装外部 LaTeX-OCR）
- 🔧 **预处理**: 自适应阈值 + 中值滤波，接口预留 GPU 加速开关
- 💾 **智能缓存**: 页面哈希、预处理与 OCR 结果落盘，重复运行秒级命中
- 📊 **进度显示**: 实时显示处理进度和速度
- 🖱️ **拖放支持**: 直接将 PDF 拖到 exe 上即可运行
- 📦 **独立运行**: 编译后单文件 + pdfium.dll 即可使用

## 📋 系统要求

### 必需软件

1. **Rust** (已安装 ✓)
2. **Tesseract OCR** (需要安装，用于语言检测与公式识别)
3. **PaddleOCR 模型** (ONNX 版，默认路径 `./models/paddle`)
4. **PDFium库** (可选，推荐安装)

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

### 准备 PaddleOCR 模型

放置 ONNX 模型到 `./models/paddle`（默认路径），或通过 `--paddle-model-dir` 指定：

- `ch_PP-OCRv4_det_infer.onnx`
- `ch_ppocr_mobile_v2.0_cls_infer.onnx`
- `ch_PP-OCRv4_rec_infer.onnx`

> 可从 PaddleOCR 官方发布、社区镜像(比如huggingface)或者直接谷歌搜索获取 PP-OCRv4/v5 模型。

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
| `--engine` | OCR 引擎: `paddle` 或 `tesseract` | paddle |
| `--layout` | 启用布局分析 | true |
| `--detect-language` | 自动语言检测并切换 | true |
| `--math-ocr` | 启用公式检测+识别 | true |
| `--paddle-model-dir` | PaddleOCR 模型目录 | ./models/paddle |
| `--math-model-dir` | 外部 LaTeX-OCR 模型目录（预留） | - |
| `--cache` | 启用缓存 | true |
| `--cache-preprocess` | 缓存预处理图像 | true |
| `--cache-ocr` | 缓存 OCR 结果 | true |
| `--cache-dir` | 缓存目录 | .cache/rust-ocr2md |
| `--use-gpu` | 预处理尝试 GPU 加速（预留） | false |
| `--auto-config` | 根据文档类型自适应参数 | true |
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

### 示例4: PaddleOCR 布局+缓存

```bash
rust-ocr2md.exe book.pdf --engine paddle --paddle-model-dir models/paddle --detect-language true --cache true
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
5. **缓存**: 默认开启，重复运行同一 PDF 可大幅提速（哈希命中预处理与 OCR 结果）

## 🧮 公式 / 布局流水线

- PaddleOCR 负责文本检测与识别；启用 `--layout` 会按阅读顺序排序文本框。
- 公式检测采用符号比率 + LaTeX 关键字启发式；落在公式框内会调用 Tesseract `equ` 语言包识别，再包裹为 `$$...$$`。
- 可通过 `--math-model-dir` 预留的目录接入外部 LaTeX-OCR 模型（需要自带 ONNX/外部推理逻辑后接入 `math.rs`）。

## 🏗️ 项目架构

```
src/
├── main.rs              # 入口和CLI
├── config.rs            # 配置管理
├── error.rs             # 错误类型
├── pdf_processor.rs     # PDF处理和页面渲染
├── ocr_engine.rs        # OCR 引擎编排（Paddle/Tesseract、语言检测、公式识别）
├── image_processor.rs   # 图像预处理
├── cache.rs             # 页面/预处理缓存
├── language.rs          # 语言检测
├── layout.rs            # 布局排序
├── math.rs              # 公式检测/封装
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

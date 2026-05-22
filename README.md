# kreuzberg-txtai

<div align="center" style="display: flex; flex-wrap: wrap; gap: 8px; justify-content: center; margin: 20px 0;">
  <a href="https://pypi.org/project/kreuzberg-txtai/">
    <img src="https://img.shields.io/pypi/v/kreuzberg-txtai?label=PyPI&color=007ec6" alt="PyPI">
  </a>
  <a href="https://pypi.org/project/kreuzberg-txtai/">
    <img src="https://img.shields.io/pypi/pyversions/kreuzberg-txtai?color=007ec6" alt="Python versions">
  </a>
  <a href="https://pypi.org/project/kreuzberg-txtai/">
    <img src="https://img.shields.io/pypi/dm/kreuzberg-txtai" alt="Downloads">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg-txtai/actions/workflows/ci.yaml">
    <img src="https://github.com/kreuzberg-dev/kreuzberg-txtai/actions/workflows/ci.yaml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg-txtai/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg">
    <img src="https://img.shields.io/github/stars/kreuzberg-dev/kreuzberg?style=flat&label=Kreuzberg&color=007ec6" alt="Kreuzberg">
  </a>
  <a href="https://docs.kreuzberg.dev">
    <img src="https://img.shields.io/badge/docs-kreuzberg.dev-blue" alt="Documentation">
  </a>
</div>

<img width="3384" height="573" alt="Kreuzberg Banner" src="https://github.com/user-attachments/assets/1b6c6ad7-3b6d-4171-b1c9-f2026cc9deb8" />

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/xt9WY3GnKR">
    <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>

A [Kreuzberg](https://github.com/kreuzberg-dev/kreuzberg)-backed document extraction pipeline for [txtai](https://github.com/neuml/txtai) and any Python framework built around the `__call__` convention.

`KreuzbergPipeline` replaces txtai's built-in `Textractor` (Apache Tika-based) with Kreuzberg's Rust-powered extraction stack, turning document paths into a `list[dict]` with `content` and `metadata` fields — surfacing title, MIME type, and page count that Tika flattens away.

## Installation

```bash
pip install kreuzberg-txtai
```

For the txtai integration examples below:

```bash
pip install "kreuzberg-txtai[txtai]"
```

Requires Python 3.10+.

## Quick Start

```python
from kreuzberg_txtai import KreuzbergPipeline

pipeline = KreuzbergPipeline()
docs = pipeline(["doc1.pdf", "doc2.docx", "doc3.html"])

for doc in docs:
    print(doc["metadata"]["source"], "->", len(doc["content"]), "chars")
```

Each element in `docs` looks like:

```python
{
    "content": "# Sample Document\n\nExtracted text...",
    "metadata": {
        "source": "doc1.pdf",
        "mime_type": "application/pdf",
        "title": "Sample Document",
        "page_count": 5,
    },
}
```

## Features

- **90+ file formats** — PDF, DOCX, PPTX, XLSX, images, HTML, Markdown, plain text, and more via Kreuzberg
- **Stable dict contract** — every extraction returns `content` + `metadata` with the same four keys, regardless of source format
- **Rich metadata** — source path, MIME type, title, and page count surface directly
- **Batch support** — pass a single path or a `list[str]`; output is always `list[dict]` in input order
- **Full Kreuzberg control** — pass an `ExtractionConfig` to drive output format, OCR backend/language, `force_ocr`, and every other Kreuzberg knob
- **Framework-agnostic** — txtai is an optional extra, not a hard dependency; the pipeline works in any framework that accepts a callable
- **Typed** — ships with a `py.typed` marker; full mypy strict compatibility

## Usage Examples

### RAG ingestion with `txtai.Embeddings`

The dominant real-world pattern — extract, index, search:

```python
from kreuzberg_txtai import KreuzbergPipeline
from txtai import Embeddings

pipeline = KreuzbergPipeline()
docs = pipeline(["doc1.pdf", "doc2.docx", "doc3.html"])

embeddings = Embeddings({
    "path": "sentence-transformers/all-MiniLM-L6-v2",
    "content": True,
})
embeddings.index([(i, doc["content"], None) for i, doc in enumerate(docs)])

results = embeddings.search("query", limit=5)
```

### Inside a `txtai.workflow.Task`

`Task` accepts any callable, so `KreuzbergPipeline` drops in without wrappers. Because the pipeline returns `list[dict]`, downstream tasks that expect strings need a one-line adapter:

```python
from txtai.workflow import Task, Workflow
from kreuzberg_txtai import KreuzbergPipeline

extract = KreuzbergPipeline()

wf = Workflow([
    Task(extract),
    Task(lambda docs: [d["content"] for d in docs]),  # flatten dicts -> strings
])

list(wf(["doc1.pdf", "doc2.pdf"]))
```

### Framework-free loop

```python
from kreuzberg import ExtractionConfig
from kreuzberg_txtai import KreuzbergPipeline

pipeline = KreuzbergPipeline(config=ExtractionConfig(output_format="plain"))
for doc in pipeline(["scan1.pdf", "scan2.pdf"]):
    print(doc["metadata"]["source"], "->", len(doc["content"]), "chars")
```

No txtai needed — the class works on just the core `kreuzberg` dependency.

### Tuning extraction with `ExtractionConfig`

Every Kreuzberg knob — output format, OCR backend and language, `force_ocr`, chunking, custom mime handling — lives on `ExtractionConfig`. Build one and hand it to the pipeline:

```python
from kreuzberg import ExtractionConfig, OcrConfig
from kreuzberg_txtai import KreuzbergPipeline

custom = ExtractionConfig(
    output_format="markdown",
    ocr=OcrConfig(backend="tesseract", language="eng+deu"),
    force_ocr=True,
)

pipeline = KreuzbergPipeline(config=custom)
docs = pipeline("scanned_report.pdf")
```

See the [Kreuzberg docs](https://docs.kreuzberg.dev) for the full list of `ExtractionConfig` and `OcrConfig` fields.

## Constructor

| Parameter | Type | Default | Notes |
|---|---|---|---|
| `config` | `ExtractionConfig \| None` | `None` | Drives output format, OCR settings, `force_ocr`, and every other Kreuzberg option. `None` falls back to Kreuzberg's defaults. |

## Return Shape

`__call__` always returns `list[dict]` — a single-path input still returns a length-1 list. Each dict has exactly two top-level keys:

- `content` — the extracted text in the format set by `config.output_format` (Kreuzberg's default when no config is passed)
- `metadata` — a dict with exactly four keys: `source`, `mime_type`, `title`, `page_count`

Missing metadata fields are `None` (rather than omitted) to keep the dict shape stable across document types.

## Related Projects

- **[kreuzberg](https://github.com/kreuzberg-dev/kreuzberg)** — the extraction engine powering this package
- **[langchain-kreuzberg](https://github.com/kreuzberg-dev/langchain-kreuzberg)** — Kreuzberg document loader for LangChain
- **[llama-index-kreuzberg](https://github.com/kreuzberg-dev/llama-index-kreuzberg)** — LlamaIndex reader and node parser
- **[kreuzberg-crewai](https://github.com/kreuzberg-dev/kreuzberg-crewai)** — CrewAI agent tool
- **[kreuzberg-surrealdb](https://github.com/kreuzberg-dev/kreuzberg-surrealdb)** — SurrealDB ingestion connector

## License

MIT — see [LICENSE](./LICENSE).

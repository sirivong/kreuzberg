---
status: accepted
date: 2026-05-07
decision-makers: ["Na'aman Hirschfeld"]
---

# Pure-Rust native PDF engine by default

## Context and Problem Statement

PDF extraction ran on Pdfium, a native library each build and package had to ship. Its build and release cost appears in the history:
composite actions to download, cache and stage the library (`download-pdfium`, `cache-pdfium`,
`stage-pdfium-runtime`), download mirrors and 503 fallbacks (32ff58ff79, 11a47ba330), and a bundled `pdfium-render`
fork. From 2026-03-15, `pdf_oxide`, a pure-Rust engine, was integrated as an alternative backend (b12d1fa980,
d409616f03).

## Considered Options

* Pdfium as the PDF engine
* The pure-Rust engine (`pdf_oxide`, vendored since 2026-08-22 as `xberg-native-pdf`) as the default, with Pdfium
  only as an explicit opt-in

## Decision Outcome

Chosen option: "The pure-Rust engine (`pdf_oxide`, vendored since 2026-08-22 as `xberg-native-pdf`) as the default,
with Pdfium only as an explicit opt-in", because it removes the native Pdfium dependency from every build: the 1.0.0
CHANGELOG records "pdfium is gone; `pdf_oxide`, a pure-Rust engine, is now the sole PDF backend — no native pdfium
dependency".

* 124ba41808 (2026-05-07) removed Pdfium and made `pdf_oxide` the sole backend, deleting the three Pdfium CI actions.
* 149a9130e6 (2026-08-19) added `PdfConfig.backend`, defaulting to the pure-Rust engine. fcfb6620b3 (2026-08-21)
  brought Pdfium back as an opt-in engine behind the `pdf-pdfium` feature, with a narrower surface: text, page
  count and Info-dictionary metadata.
* c8ed4a98cb (2026-08-22) vendored the engine as `crates/xberg-native-pdf`, and 9ffbdcae55 renamed the backend to
  `native`.
* Selecting `pdfium` in a build without `pdf-pdfium` is an error, never a silent fallback to the native engine.

### Consequences

* Good, because the default build, every binding and the WASM target extract PDFs without a native library to ship
  or discover.
* Good, because xberg owns the PDF engine's source and can fix extraction defects in this repository.
* Bad, because PDF parsing quality is now this project's own maintenance burden, not an upstream one.
* Bad, because a caller who selects Pdfium must supply `libpdfium` and point `PDFIUM_DYNAMIC_LIB_PATH` at it.

### Confirmation

* `crates/xberg/Cargo.toml:133` sets `pdf = ["pdf-native"]`, and `:153` defines the opt-in
  `pdf-pdfium = ["dep:pdfium-render", "tokio-runtime"]`.
* `crates/xberg/src/core/config/pdf.rs:20` marks `PdfBackend::Native` as `#[default]`.
* The pdf-backends skill (`.ai-rulez/skills/pdf-backends/SKILL.md`) records the native default and the no-fallback
  rule.

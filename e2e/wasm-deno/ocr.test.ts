// Auto-generated tests for ocr fixtures.
// Run with: deno test --allow-read

import type { ExtractionResult } from "./helpers.ts";
import { assertions, buildConfig, extractBytes, initWasm, resolveDocument, shouldSkipFixture } from "./helpers.ts";

// Initialize WASM module once at module load time
await initWasm();

Deno.test("ocr_image_hello_world", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/test_hello_world.png");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/png", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_image_hello_world",
				["tesseract"],
				"Requires Tesseract OCR for image text extraction.",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/png"]);
	assertions.assertMinContentLength(result, 5);
	assertions.assertContentContainsAny(result, ["hello", "world"]);
});

Deno.test("ocr_image_no_text", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/flower_no_text.jpg");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/jpeg", config);
	} catch (error) {
		if (shouldSkipFixture(error, "ocr_image_no_text", ["tesseract"], "Skip when Tesseract is unavailable.")) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/jpeg"]);
	assertions.assertMaxContentLength(result, 200);
});

Deno.test("ocr_paddle_confidence_filter", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/ocr_image.jpg");
	const config = buildConfig({
		force_ocr: true,
		ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { min_confidence: 80.0 } },
	});
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/jpeg", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_confidence_filter",
				["onnxruntime", "paddle-ocr"],
				"Tests confidence threshold filtering with PaddleOCR",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/jpeg"]);
	assertions.assertMinContentLength(result, 1);
});

Deno.test("ocr_paddle_image_chinese", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/chi_sim_image.jpeg");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "ch" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/jpeg", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_image_chinese",
				["onnxruntime", "paddle-ocr"],
				"Requires PaddleOCR with Chinese models",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/jpeg"]);
	assertions.assertMinContentLength(result, 1);
});

Deno.test("ocr_paddle_image_english", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/test_hello_world.png");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "en" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/png", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_image_english",
				["onnxruntime", "paddle-ocr"],
				"Requires PaddleOCR with ONNX Runtime",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/png"]);
	assertions.assertMinContentLength(result, 5);
	assertions.assertContentContainsAny(result, ["hello", "Hello", "world", "World"]);
});

Deno.test("ocr_paddle_markdown", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/test_hello_world.png");
	const config = buildConfig({
		force_ocr: true,
		ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { output_format: "markdown" } },
	});
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/png", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_markdown",
				["onnxruntime", "paddle-ocr"],
				"Tests markdown output format parity with Tesseract",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/png"]);
	assertions.assertMinContentLength(result, 5);
	assertions.assertContentContainsAny(result, ["hello", "Hello", "world", "World"]);
});

Deno.test("ocr_paddle_pdf_scanned", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("pdfs/ocr_test.pdf");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "en" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "application/pdf", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_pdf_scanned",
				["onnxruntime", "paddle-ocr"],
				"Requires PaddleOCR with ONNX Runtime",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["application/pdf"]);
	assertions.assertMinContentLength(result, 20);
	assertions.assertContentContainsAny(result, ["Docling", "Markdown", "JSON"]);
});

Deno.test("ocr_paddle_structured", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/test_hello_world.png");
	const config = buildConfig({
		force_ocr: true,
		ocr: { backend: "paddle-ocr", element_config: { include_elements: true }, language: "en" },
	});
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/png", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_structured",
				["onnxruntime", "paddle-ocr"],
				"Tests structured output with bbox/confidence preservation",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/png"]);
	assertions.assertMinContentLength(result, 5);
	assertions.assertOcrElements(result, true, true, true, null);
});

Deno.test("ocr_paddle_table_detection", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("images/simple_table.png");
	const config = buildConfig({
		force_ocr: true,
		ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { enable_table_detection: true } },
	});
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "image/png", config);
	} catch (error) {
		if (
			shouldSkipFixture(
				error,
				"ocr_paddle_table_detection",
				["onnxruntime", "paddle-ocr"],
				"Tests table detection capability with PaddleOCR",
			)
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["image/png"]);
	assertions.assertMinContentLength(result, 10);
	assertions.assertTableCount(result, 1, null);
});

Deno.test("ocr_pdf_image_only_german", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("pdf/image_only_german_pdf.pdf");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "application/pdf", config);
	} catch (error) {
		if (shouldSkipFixture(error, "ocr_pdf_image_only_german", ["tesseract"], "Skip if OCR backend unavailable.")) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["application/pdf"]);
	assertions.assertMinContentLength(result, 20);
	assertions.assertMetadataExpectation(result, "format_type", { eq: "pdf" });
});

Deno.test("ocr_pdf_rotated_90", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("pdf/ocr_test_rotated_90.pdf");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "application/pdf", config);
	} catch (error) {
		if (
			shouldSkipFixture(error, "ocr_pdf_rotated_90", ["tesseract"], "Skip automatically when OCR backend is missing.")
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["application/pdf"]);
	assertions.assertMinContentLength(result, 10);
});

Deno.test("ocr_pdf_tesseract", { permissions: { read: true } }, async () => {
	const documentBytes = await resolveDocument("pdf/ocr_test.pdf");
	const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
	let result: ExtractionResult | null = null;
	try {
		// Sync file extraction - WASM uses extractBytes with pre-read bytes
		result = await extractBytes(documentBytes, "application/pdf", config);
	} catch (error) {
		if (
			shouldSkipFixture(error, "ocr_pdf_tesseract", ["tesseract"], "Skip automatically if OCR backend is unavailable.")
		) {
			return;
		}
		throw error;
	}
	if (result === null) {
		return;
	}
	assertions.assertExpectedMime(result, ["application/pdf"]);
	assertions.assertMinContentLength(result, 20);
	assertions.assertContentContainsAny(result, ["Docling", "Markdown", "JSON"]);
});

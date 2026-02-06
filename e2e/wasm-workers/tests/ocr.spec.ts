// Auto-generated tests for ocr fixtures.
// Designed for Cloudflare Workers with Vitest + Miniflare

import type { ExtractionResult } from "@kreuzberg/wasm";
import { extractBytes } from "@kreuzberg/wasm";
import { describe, expect, it } from "vitest";
import { assertions, buildConfig, getFixture, shouldSkipFixture } from "./helpers.js";

describe("ocr", () => {
	it("ocr_image_hello_world", async () => {
		const documentBytes = getFixture("images/test_hello_world.png");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_image_no_text", async () => {
		const documentBytes = getFixture("images/flower_no_text.jpg");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_confidence_filter", async () => {
		const documentBytes = getFixture("images/ocr_image.jpg");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({
			force_ocr: true,
			ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { min_confidence: 80.0 } },
		});
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_image_chinese", async () => {
		const documentBytes = getFixture("images/chi_sim_image.jpeg");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "ch" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_image_english", async () => {
		const documentBytes = getFixture("images/test_hello_world.png");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "en" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_markdown", async () => {
		const documentBytes = getFixture("images/test_hello_world.png");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({
			force_ocr: true,
			ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { output_format: "markdown" } },
		});
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_pdf_scanned", async () => {
		const documentBytes = getFixture("pdfs/ocr_test.pdf");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "paddle-ocr", language: "en" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_structured", async () => {
		const documentBytes = getFixture("images/test_hello_world.png");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({
			force_ocr: true,
			ocr: { backend: "paddle-ocr", element_config: { include_elements: true }, language: "en" },
		});
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_paddle_table_detection", async () => {
		const documentBytes = getFixture("images/simple_table.png");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({
			force_ocr: true,
			ocr: { backend: "paddle-ocr", language: "en", paddle_ocr_config: { enable_table_detection: true } },
		});
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_pdf_image_only_german", async () => {
		const documentBytes = getFixture("pdf/image_only_german_pdf.pdf");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_pdf_rotated_90", async () => {
		const documentBytes = getFixture("pdf/ocr_test_rotated_90.pdf");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
		let result: ExtractionResult | null = null;
		try {
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

	it("ocr_pdf_tesseract", async () => {
		const documentBytes = getFixture("pdf/ocr_test.pdf");
		if (documentBytes === null) {
			console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
			return;
		}

		const config = buildConfig({ force_ocr: true, ocr: { backend: "tesseract", language: "eng" } });
		let result: ExtractionResult | null = null;
		try {
			result = await extractBytes(documentBytes, "application/pdf", config);
		} catch (error) {
			if (
				shouldSkipFixture(
					error,
					"ocr_pdf_tesseract",
					["tesseract"],
					"Skip automatically if OCR backend is unavailable.",
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
});

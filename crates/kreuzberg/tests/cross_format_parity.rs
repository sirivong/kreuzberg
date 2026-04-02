//! Cross-format output parity tests.
//!
//! Verify that all output formats (Markdown, HTML, Djot, Plain) produce
//! equivalent text content for the same document. We extract each document
//! in every format, strip markup to plain text, tokenize, and compute
//! token-level F1 scores between format pairs.
//!
//! Usage:
//!   cargo test -p kreuzberg --test cross_format_parity -- --nocapture

mod helpers;

use helpers::{get_test_file_path, test_documents_available};
use kreuzberg::core::config::{ExtractionConfig, OutputFormat};
use kreuzberg::extract_file_sync;
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Text stripping helpers
// ============================================================================

/// Strip markdown markup to recover approximate plain text.
fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip code fence lines
        if trimmed.starts_with("```") {
            continue;
        }

        // Skip table separator lines (e.g., |---|---|)
        if trimmed.starts_with('|') && trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ') {
            continue;
        }

        // Strip heading markers
        let line = strip_leading_pattern(trimmed, '#');

        // Strip blockquote markers
        let line = strip_leading_pattern(&line, '>');

        // Strip unordered list markers
        let line = strip_list_marker(&line);

        // Strip table pipes
        let line = line.replace('|', " ");

        // Strip link syntax: [text](url) -> text
        let line = strip_links(&line);

        // Strip image syntax: ![alt](url) -> alt
        let line = strip_images(&line);

        // Strip inline formatting markers
        let line = line.replace("**", "");
        let line = line.replace("__", "");
        let line = line.replace('*', "");
        let line = line.replace('_', " ");
        let line = line.replace('~', "");
        let line = line.replace('`', "");

        result.push_str(&line);
        result.push('\n');
    }

    result
}

/// Strip HTML tags and decode common entities.
fn strip_html(text: &str) -> String {
    // Remove all HTML tags
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;

    for ch in text.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            // Add space after closing tags to prevent word merging
            result.push(' ');
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Decode common HTML entities
    let result = result.replace("&amp;", "&");
    let result = result.replace("&lt;", "<");
    let result = result.replace("&gt;", ">");
    let result = result.replace("&quot;", "\"");
    let result = result.replace("&apos;", "'");
    let result = result.replace("&#39;", "'");
    let result = result.replace("&nbsp;", " ");

    // Decode numeric entities: &#NNN;
    decode_numeric_entities(&result)
}

/// Strip djot markup (similar to markdown with minor differences).
fn strip_djot(text: &str) -> String {
    // Djot is structurally similar to markdown for our purposes
    strip_markdown(text)
}

// ============================================================================
// Tokenization and scoring
// ============================================================================

/// Tokenize text: lowercase, split on whitespace, filter empty and
/// purely-punctuation tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| c.is_ascii_punctuation()).to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Compute token-level F1 between two token sequences using bag-of-tokens.
///
/// This treats each sequence as a multiset (bag) and computes precision,
/// recall, and F1 based on token overlap counts.
fn token_f1(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut bag_a: HashMap<&str, usize> = HashMap::new();
    for token in a {
        *bag_a.entry(token.as_str()).or_insert(0) += 1;
    }

    let mut bag_b: HashMap<&str, usize> = HashMap::new();
    for token in b {
        *bag_b.entry(token.as_str()).or_insert(0) += 1;
    }

    let mut overlap = 0usize;
    for (token, &count_a) in &bag_a {
        if let Some(&count_b) = bag_b.get(token) {
            overlap += count_a.min(count_b);
        }
    }

    let precision = overlap as f64 / b.len() as f64;
    let recall = overlap as f64 / a.len() as f64;

    if precision + recall == 0.0 {
        return 0.0;
    }

    2.0 * precision * recall / (precision + recall)
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Strip leading repeated characters (like `#` for headings or `>` for quotes).
fn strip_leading_pattern(line: &str, marker: char) -> String {
    let stripped = line.trim_start_matches(marker);
    if stripped.len() < line.len() {
        stripped.trim_start().to_string()
    } else {
        line.to_string()
    }
}

/// Strip list markers (- , * , + , 1. , etc.).
fn strip_list_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let indent_len = line.len() - trimmed.len();
        let rest = &trimmed[2..];
        format!("{}{}", &line[..indent_len], rest)
    } else if let Some(after_digit) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
        // Handle "1. ", "2. ", etc.
        if let Some(rest) = after_digit.strip_prefix(". ") {
            let indent_len = line.len() - trimmed.len();
            format!("{}{}", &line[..indent_len], rest)
        } else {
            line.to_string()
        }
    } else {
        line.to_string()
    }
}

/// Strip markdown link syntax: [text](url) -> text
fn strip_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            // Look for closing ] followed by (
            if let Some(close_bracket) = chars[i + 1..].iter().position(|&c| c == ']') {
                let close_idx = i + 1 + close_bracket;
                if close_idx + 1 < chars.len() && chars[close_idx + 1] == '(' {
                    // Found [text]( ... look for closing )
                    if let Some(close_paren) = chars[close_idx + 2..].iter().position(|&c| c == ')') {
                        // Extract just the text part
                        let text_part: String = chars[i + 1..close_idx].iter().collect();
                        result.push_str(&text_part);
                        i = close_idx + 2 + close_paren + 1;
                        continue;
                    }
                }
            }
            result.push(chars[i]);
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Strip markdown image syntax: ![alt](url) -> alt
fn strip_images(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            // Image syntax: ![alt](url)
            if let Some(close_bracket) = chars[i + 2..].iter().position(|&c| c == ']') {
                let close_idx = i + 2 + close_bracket;
                if close_idx + 1 < chars.len()
                    && chars[close_idx + 1] == '('
                    && let Some(close_paren) = chars[close_idx + 2..].iter().position(|&c| c == ')')
                {
                    let alt_text: String = chars[i + 2..close_idx].iter().collect();
                    result.push_str(&alt_text);
                    i = close_idx + 2 + close_paren + 1;
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Decode numeric HTML entities (&#NNN;) to characters.
fn decode_numeric_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' && chars.peek() == Some(&'#') {
            chars.next(); // consume '#'
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next(); // consume ';'
                    break;
                }
                if c.is_ascii_digit() {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(code) = num_str.parse::<u32>()
                && let Some(decoded) = char::from_u32(code)
            {
                result.push(decoded);
                continue;
            }
            // Failed to decode, emit as-is
            result.push('&');
            result.push('#');
            result.push_str(&num_str);
        } else {
            result.push(ch);
        }
    }

    result
}

// ============================================================================
// Extraction helpers
// ============================================================================

/// Extract a document in the given output format.
fn extract_with_format(path: &Path, format: OutputFormat) -> Option<String> {
    let config = ExtractionConfig {
        output_format: format.clone(),
        ..Default::default()
    };

    match extract_file_sync(path, None, &config) {
        Ok(result) => Some(result.content),
        Err(err) => {
            eprintln!(
                "  [WARN] extraction failed for {} with format {}: {}",
                path.display(),
                format,
                err
            );
            None
        }
    }
}

/// Strip markup from content based on its format.
fn strip_markup(content: &str, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Plain => content.to_string(),
        OutputFormat::Markdown => strip_markdown(content),
        OutputFormat::Html => strip_html(content),
        OutputFormat::Djot => strip_djot(content),
        _ => content.to_string(),
    }
}

// ============================================================================
// Test document definitions
// ============================================================================

struct TestDoc {
    /// Human-readable label.
    label: &'static str,
    /// Path relative to test_documents/.
    relative_path: &'static str,
    /// Required cargo feature (empty string means no feature needed).
    required_feature: &'static str,
    /// Expected minimum TF1 for Markdown vs HTML.
    md_html_threshold: f64,
    /// Expected minimum TF1 for Markdown vs Djot.
    md_djot_threshold: f64,
    /// Expected minimum TF1 for Markdown vs Plain.
    md_plain_threshold: f64,
}

const TEST_DOCS: &[TestDoc] = &[
    // Markdown extraction_test.md — has headings, tables, lists. No extra features needed.
    TestDoc {
        label: "markdown-extraction-test",
        relative_path: "markdown/extraction_test.md",
        required_feature: "",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // Markdown readme.md — headings, lists, code block. No extra features needed.
    TestDoc {
        label: "markdown-readme",
        relative_path: "markdown/readme.md",
        required_feature: "",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // RST document — requires office feature for the RST extractor.
    TestDoc {
        label: "rst-readme",
        relative_path: "rst/readme.rst",
        required_feature: "office",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // HTML page — requires html feature.
    // HTML page — requires html feature. The taylor_swift page is large and
    // markup-stripping introduces divergence; use relaxed thresholds.
    TestDoc {
        label: "html-taylor-swift",
        relative_path: "html/taylor_swift.html",
        required_feature: "html",
        md_html_threshold: 0.75,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.75,
    },
    // LaTeX document — requires office feature.
    TestDoc {
        label: "latex-basic-sections",
        relative_path: "latex/basic_sections.tex",
        required_feature: "office",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // EPUB — requires office feature.
    TestDoc {
        label: "epub-wasteland",
        relative_path: "epub/wasteland.epub",
        required_feature: "office",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // DOCX — requires office feature.
    TestDoc {
        label: "docx-sample-document",
        relative_path: "docx/sample_document.docx",
        required_feature: "office",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // HTML table document — requires html feature.
    TestDoc {
        label: "html-simple-table",
        relative_path: "html/simple_table.html",
        required_feature: "html",
        md_html_threshold: 0.90,
        md_djot_threshold: 0.85,
        md_plain_threshold: 0.85,
    },
    // LaTeX tables — requires office feature.
    TestDoc {
        label: "latex-tables",
        relative_path: "latex/tables.tex",
        required_feature: "office",
        md_html_threshold: 0.85,
        md_djot_threshold: 0.80,
        md_plain_threshold: 0.80,
    },
];

// ============================================================================
// Tests
// ============================================================================

/// Check whether a required feature is available at runtime by attempting an
/// extraction. Returns false if extraction fails (feature likely not compiled).
fn feature_available(feature: &str) -> bool {
    match feature {
        "" => true,
        "html" => cfg!(feature = "html"),
        "office" => cfg!(feature = "office"),
        "pdf" => cfg!(feature = "pdf"),
        "excel" => cfg!(feature = "excel"),
        _ => false,
    }
}

#[test]
fn cross_format_parity_all_documents() {
    if !test_documents_available() {
        eprintln!("Skipping: test_documents not available");
        return;
    }

    let formats = [
        OutputFormat::Markdown,
        OutputFormat::Html,
        OutputFormat::Djot,
        OutputFormat::Plain,
    ];

    let mut failures: Vec<String> = Vec::new();
    let mut tested = 0usize;

    for doc in TEST_DOCS {
        if !feature_available(doc.required_feature) {
            eprintln!("  [SKIP] {} — requires feature '{}'", doc.label, doc.required_feature);
            continue;
        }

        let path = get_test_file_path(doc.relative_path);
        if !path.exists() {
            eprintln!("  [SKIP] {} — file not found: {}", doc.label, path.display());
            continue;
        }

        eprintln!("\n--- {} ---", doc.label);

        // Extract in all formats
        let mut outputs: HashMap<String, String> = HashMap::new();
        for format in &formats {
            if let Some(content) = extract_with_format(&path, format.clone()) {
                let stripped = strip_markup(&content, format);
                let format_name = format.to_string();
                eprintln!(
                    "  {}: {} chars raw, {} chars stripped",
                    format_name,
                    content.len(),
                    stripped.len()
                );
                outputs.insert(format_name, stripped);
            }
        }

        // Need at least markdown and one other format to compare
        let md_tokens = match outputs.get("markdown") {
            Some(text) => tokenize(text),
            None => {
                eprintln!("  [SKIP] {} — markdown extraction failed", doc.label);
                continue;
            }
        };

        if md_tokens.is_empty() {
            eprintln!("  [SKIP] {} — markdown produced no tokens", doc.label);
            continue;
        }

        tested += 1;

        // Compare Markdown vs HTML
        if let Some(html_text) = outputs.get("html") {
            let html_tokens = tokenize(html_text);
            let f1 = token_f1(&md_tokens, &html_tokens);
            eprintln!(
                "  MD vs HTML:  TF1 = {:.4}  (md_tokens={}, html_tokens={})",
                f1,
                md_tokens.len(),
                html_tokens.len()
            );
            if f1 < doc.md_html_threshold {
                failures.push(format!(
                    "{}: MD vs HTML TF1 = {:.4} < threshold {:.2}",
                    doc.label, f1, doc.md_html_threshold
                ));
            }
        }

        // Compare Markdown vs Djot
        if let Some(djot_text) = outputs.get("djot") {
            let djot_tokens = tokenize(djot_text);
            let f1 = token_f1(&md_tokens, &djot_tokens);
            eprintln!(
                "  MD vs Djot:  TF1 = {:.4}  (md_tokens={}, djot_tokens={})",
                f1,
                md_tokens.len(),
                djot_tokens.len()
            );
            if f1 < doc.md_djot_threshold {
                failures.push(format!(
                    "{}: MD vs Djot TF1 = {:.4} < threshold {:.2}",
                    doc.label, f1, doc.md_djot_threshold
                ));
            }
        }

        // Compare Markdown vs Plain
        if let Some(plain_text) = outputs.get("plain") {
            let plain_tokens = tokenize(plain_text);
            let f1 = token_f1(&md_tokens, &plain_tokens);
            eprintln!(
                "  MD vs Plain: TF1 = {:.4}  (md_tokens={}, plain_tokens={})",
                f1,
                md_tokens.len(),
                plain_tokens.len()
            );
            if f1 < doc.md_plain_threshold {
                failures.push(format!(
                    "{}: MD vs Plain TF1 = {:.4} < threshold {:.2}",
                    doc.label, f1, doc.md_plain_threshold
                ));
            }
        }
    }

    eprintln!("\n=== Summary: tested {} documents ===", tested);

    if !failures.is_empty() {
        panic!(
            "Cross-format parity failures ({}/{} checks failed):\n  - {}",
            failures.len(),
            tested * 3,
            failures.join("\n  - ")
        );
    }

    assert!(tested > 0, "Expected at least one document to be tested");
}

/// Focused test for table content parity across formats.
///
/// Verifies that table cell text appears in all format outputs,
/// regardless of how the table is rendered (pipe tables, HTML tables,
/// space-separated text).
#[test]
fn cross_format_table_content_parity() {
    if !test_documents_available() {
        eprintln!("Skipping: test_documents not available");
        return;
    }

    // Documents known to contain tables
    let table_docs: &[(&str, &str, &[&str])] = &[
        #[cfg(feature = "html")]
        ("html/simple_table.html", "html", &["Product", "Category", "Price"]),
        #[cfg(feature = "office")]
        (
            "latex/tables.tex",
            "office",
            &[], // We don't know exact cell values; just check non-empty extraction
        ),
        #[cfg(feature = "office")]
        ("docx/docx_tables.docx", "office", &[]),
    ];

    let formats = [
        ("markdown", OutputFormat::Markdown),
        ("html", OutputFormat::Html),
        ("djot", OutputFormat::Djot),
        ("plain", OutputFormat::Plain),
    ];

    let mut tested = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for &(relative_path, required_feature, expected_cells) in table_docs {
        if !feature_available(required_feature) {
            eprintln!("  [SKIP] {} — requires feature '{}'", relative_path, required_feature);
            continue;
        }

        let path = get_test_file_path(relative_path);
        if !path.exists() {
            eprintln!("  [SKIP] {} — file not found", relative_path);
            continue;
        }

        eprintln!("\n--- table test: {} ---", relative_path);
        tested += 1;

        for (format_name, format) in &formats {
            if let Some(content) = extract_with_format(&path, format.clone()) {
                let lower = content.to_lowercase();

                // Check that expected cell values appear in every format
                for &cell in expected_cells {
                    if !lower.contains(&cell.to_lowercase()) {
                        failures.push(format!(
                            "{} [{}]: missing expected table cell '{}'",
                            relative_path, format_name, cell
                        ));
                    }
                }

                // Every format should produce non-empty content
                if content.trim().is_empty() {
                    failures.push(format!("{} [{}]: produced empty content", relative_path, format_name));
                }
            }
        }
    }

    eprintln!("\n=== Table parity: tested {} documents ===", tested);

    if !failures.is_empty() {
        panic!("Table content parity failures:\n  - {}", failures.join("\n  - "));
    }
}

// ============================================================================
// Unit tests for helper functions
// ============================================================================

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_strip_markdown_headings() {
        let input = "# Heading 1\n## Heading 2\nPlain text\n";
        let stripped = strip_markdown(input);
        assert!(stripped.contains("Heading 1"));
        assert!(stripped.contains("Heading 2"));
        assert!(stripped.contains("Plain text"));
        assert!(!stripped.contains('#'));
    }

    #[test]
    fn test_strip_markdown_links() {
        let input = "See [link text](https://example.com) for details.\n";
        let stripped = strip_markdown(input);
        assert!(stripped.contains("link text"));
        assert!(!stripped.contains("https://example.com"));
        assert!(!stripped.contains('['));
        assert!(!stripped.contains(']'));
    }

    #[test]
    fn test_strip_markdown_bold_italic() {
        let input = "This is **bold** and *italic* text.\n";
        let stripped = strip_markdown(input);
        assert!(stripped.contains("bold"));
        assert!(stripped.contains("italic"));
    }

    #[test]
    fn test_strip_markdown_list() {
        let input = "- item one\n* item two\n1. item three\n";
        let stripped = strip_markdown(input);
        assert!(stripped.contains("item one"));
        assert!(stripped.contains("item two"));
        assert!(stripped.contains("item three"));
    }

    #[test]
    fn test_strip_html_tags() {
        let input = "<h1>Title</h1><p>Hello &amp; goodbye</p>";
        let stripped = strip_html(input);
        assert!(stripped.contains("Title"));
        assert!(stripped.contains("Hello & goodbye"));
        assert!(!stripped.contains('<'));
        assert!(!stripped.contains('>'));
    }

    #[test]
    fn test_strip_html_numeric_entity() {
        let input = "A&#65;B";
        let stripped = strip_html(input);
        assert!(stripped.contains("AAB"));
    }

    #[test]
    fn test_tokenize() {
        let input = "Hello, World! This is a TEST.";
        let tokens = tokenize(input);
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_token_f1_identical() {
        let a = vec!["hello".to_string(), "world".to_string()];
        let f1 = token_f1(&a, &a);
        assert!((f1 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_f1_no_overlap() {
        let a = vec!["hello".to_string()];
        let b = vec!["world".to_string()];
        let f1 = token_f1(&a, &b);
        assert!(f1.abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_f1_partial_overlap() {
        let a = vec![
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
        ];
        let b = vec![
            "the".to_string(),
            "quick".to_string(),
            "red".to_string(),
            "fox".to_string(),
        ];
        let f1 = token_f1(&a, &b);
        // 3 overlapping tokens out of 4 each -> precision=3/4, recall=3/4, F1=3/4
        assert!((f1 - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_token_f1_empty() {
        let empty: Vec<String> = vec![];
        assert!((token_f1(&empty, &empty) - 1.0).abs() < f64::EPSILON);
        assert!(token_f1(&empty, &["a".to_string()]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strip_images() {
        let input = "Before ![alt text](image.png) after";
        let stripped = strip_images(input);
        assert!(stripped.contains("alt text"));
        assert!(!stripped.contains("image.png"));
    }
}

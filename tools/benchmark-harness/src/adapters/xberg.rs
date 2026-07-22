//! Xberg adapter for Wave 2 benchmark harness.
//!
//! Provides subprocess-based extraction via xberg with support for:
//! - Three pipelines: baseline, layout, paddle-ocr
//! - Single-file and batch extraction modes
//! - JSON envelope parsing (ExtractEnvelope and BatchEnvelope)

use crate::{
    adapters::subprocess::SubprocessAdapter,
    error::Result,
    types::{BatchCapability, BatchEntryPoint, BatchTimingScope, OutputFormat, XbergPipeline},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use which::which;

const XBERG_CLI_BINARY_ENV_VAR: &str = "XBERG_CLI_BINARY";

/// Environment variable that requests per-stage cold-start timing from the xberg CLI (must
/// match `crates/xberg-cli/src/commands/extract.rs::STAGE_TIMING_ENV_VAR`).
///
/// Passed unconditionally to every xberg subprocess invocation this adapter spawns. This is
/// cheap for the CLI to check (a single `std::env::var` read gated behind an `if`) and lets
/// `xberg extract --format json` include a `stage_timings` object that the harness can parse
/// out of the subprocess's stdout for cold-start attribution (see
/// `tools/benchmark-harness/src/types.rs::StageTimings`).
///
/// Note: parsing `stage_timings` out of the subprocess's JSON stdout happens in
/// `SubprocessAdapter::parse_output` (`adapters/subprocess.rs`), which is out of scope for this
/// change — see the module-level TODO below.
const STAGE_TIMING_ENV_VAR: &str = "XBERG_EMIT_STAGE_TIMING";
const BENCHMARK_CONFIG_JSON: &str = r#"{"extraction_timeout_secs":1740,"use_cache":false}"#;

fn benchmark_base_args(batch: bool, content_format: &str) -> Vec<String> {
    let subcommand = if batch { "batch" } else { "extract" };
    vec![
        subcommand.to_string(),
        "--no-config-discovery".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--content-format".to_string(),
        content_format.to_string(),
        "--config-json".to_string(),
        BENCHMARK_CONFIG_JSON.to_string(),
    ]
}

fn append_tesseract_ocr_args(args: &mut Vec<String>, ocr_enabled: bool) {
    if ocr_enabled {
        args.extend([
            "--ocr".to_string(),
            "true".to_string(),
            "--ocr-backend".to_string(),
            "tesseract".to_string(),
        ]);
    }
}

/// Creates a Xberg adapter for the given pipeline and configuration.
///
/// # Arguments
/// * `pipeline` - The pipeline variant (baseline, layout, paddle-ocr)
/// * `output_format` - Output format for extraction (markdown or plaintext)
/// * `batch` - Whether to use batch extraction mode
///
/// # Returns
/// * `Ok(SubprocessAdapter)` - Configured adapter ready for extraction
/// * `Err(Error)` - If xberg cannot be located
pub fn create_xberg_adapter(
    pipeline: XbergPipeline,
    output_format: OutputFormat,
    batch: bool,
    ocr_enabled: bool,
) -> Result<SubprocessAdapter> {
    if !ocr_enabled
        && matches!(
            pipeline,
            XbergPipeline::PaddleOcr
                | XbergPipeline::CandleTrocr
                | XbergPipeline::CandlePaddleocrVl
                | XbergPipeline::CandleGlmOcr
                | XbergPipeline::CandleDeepseekOcr
                | XbergPipeline::CandlePaddleocrVl15
        )
    {
        return Err(crate::Error::Config(format!(
            "xberg pipeline '{}' requires OCR, but OCR is disabled",
            pipeline.as_str()
        )));
    }

    let cli_path = locate_xberg_cli()?;

    let content_format = match output_format {
        OutputFormat::Markdown => "markdown",
        OutputFormat::Plaintext => "plain",
    };

    let mut args = benchmark_base_args(batch, content_format);

    match pipeline {
        XbergPipeline::Baseline => {
            append_tesseract_ocr_args(&mut args, ocr_enabled);
        }
        XbergPipeline::Layout => {
            args.push("--layout".to_string());
            args.push("true".to_string());
            args.push("--use-layout-for-markdown".to_string());
            append_tesseract_ocr_args(&mut args, ocr_enabled);
        }
        XbergPipeline::PaddleOcr => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("paddle-ocr".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
        XbergPipeline::CandleTrocr => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("candle-trocr".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
        XbergPipeline::CandlePaddleocrVl => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("candle-paddleocr-vl".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
        XbergPipeline::CandleGlmOcr => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("candle-glm-ocr".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
        XbergPipeline::CandleDeepseekOcr => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("candle-deepseek-ocr".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
        XbergPipeline::CandlePaddleocrVl15 => {
            args.push("--ocr".to_string());
            args.push("true".to_string());
            args.push("--ocr-backend".to_string());
            args.push("candle-paddleocr-vl".to_string());
            args.push("--force-ocr".to_string());
            args.push("true".to_string());
        }
    }

    args.push("--pdf-backend".to_string());
    args.push("pdf-oxide".to_string());

    let format_slug = match output_format {
        OutputFormat::Markdown => "markdown",
        OutputFormat::Plaintext => "plaintext",
    };
    let framework_name = if batch {
        format!("xberg-{}-{}-batch", format_slug, pipeline.as_str())
    } else {
        format!("xberg-{}-{}", format_slug, pipeline.as_str())
    };
    let supported_formats = vec![
        "pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "txt", "md", "html", "xml", "json", "odt", "ods", "odp",
        "epub", "rtf", "csv", "json", "yaml", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp", "zip", "tar",
        "gz", "7z",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    let env = vec![(STAGE_TIMING_ENV_VAR.to_string(), "1".to_string())];

    let single_file_args = batch.then(|| {
        let mut single_args = args.clone();
        single_args[0] = "extract".to_string();
        single_args
    });

    let mut adapter = if batch {
        SubprocessAdapter::with_batch_capability(
            &framework_name,
            cli_path,
            args,
            env,
            supported_formats,
            BatchCapability {
                entry_point: BatchEntryPoint::XbergCliExtractBatch,
                timing_scope: BatchTimingScope::ColdEndToEndSubprocess,
                per_item_timing: true,
            },
        )
    } else {
        SubprocessAdapter::new(&framework_name, cli_path, args, env, supported_formats)
    }
    .with_supported_output_formats(vec![output_format]);
    if let Some(single_args) = single_file_args {
        adapter = adapter.with_single_file_args(single_args);
    }

    Ok(adapter)
}

/// Locates the xberg executable.
///
/// Searches in priority order:
/// 1. The executable file specified by `XBERG_CLI_BINARY`
/// 2. `target/release/xberg`
/// 3. `target/debug/xberg`
/// 4. `which xberg`
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the executable
/// * `Err(Error)` - If the override is invalid or xberg cannot be found
fn locate_xberg_cli() -> Result<PathBuf> {
    let binary_override = std::env::var_os(XBERG_CLI_BINARY_ENV_VAR);
    locate_xberg_cli_with_override(binary_override.as_deref())
}

fn locate_xberg_cli_with_override(binary_override: Option<&OsStr>) -> Result<PathBuf> {
    if let Some(binary_override) = binary_override {
        let path = PathBuf::from(binary_override);
        validate_xberg_cli_override(&path)?;
        return Ok(path);
    }

    let release_path = PathBuf::from("target/release/xberg");
    if release_path.exists() {
        return Ok(release_path);
    }

    let debug_path = PathBuf::from("target/debug/xberg");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    if let Ok(path) = which("xberg") {
        return Ok(path);
    }

    Err(crate::Error::Benchmark(
        "xberg binary not found. Build with: cargo build --release -p xberg-cli --features all".to_string(),
    ))
}

fn validate_xberg_cli_override(path: &Path) -> Result<()> {
    let metadata = path.metadata().map_err(|error| {
        crate::Error::Benchmark(format!(
            "{XBERG_CLI_BINARY_ENV_VAR} points to `{}` but its metadata could not be read: {error}",
            path.display()
        ))
    })?;

    if !metadata.is_file() {
        return Err(crate::Error::Benchmark(format!(
            "{XBERG_CLI_BINARY_ENV_VAR} must point to an executable file, but `{}` is not a file",
            path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(crate::Error::Benchmark(format!(
                "{XBERG_CLI_BINARY_ENV_VAR} must point to an executable file, but `{}` has no execute permission",
                path.display()
            )));
        }
    }

    #[cfg(windows)]
    {
        let extension = path.extension().and_then(OsStr::to_str).map(str::to_ascii_lowercase);
        let is_executable = matches!(extension.as_deref(), Some("exe" | "com" | "cmd" | "bat"));
        if !is_executable {
            return Err(crate::Error::Benchmark(format!(
                "{XBERG_CLI_BINARY_ENV_VAR} must point to an executable file, but `{}` has no executable extension",
                path.display()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_executable_file(directory: &Path) -> PathBuf {
        let path = directory.join(if cfg!(windows) { "xberg.exe" } else { "xberg" });
        std::fs::write(&path, b"test executable").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = path.metadata().unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).unwrap();
        }

        path
    }

    #[test]
    fn test_pipeline_baseline_str() {
        assert_eq!(XbergPipeline::Baseline.as_str(), "baseline");
    }

    #[test]
    fn test_pipeline_layout_str() {
        assert_eq!(XbergPipeline::Layout.as_str(), "layout");
    }

    #[test]
    fn test_pipeline_paddle_ocr_str() {
        assert_eq!(XbergPipeline::PaddleOcr.as_str(), "paddle-ocr");
    }

    #[test]
    fn test_output_format_markdown() {
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
    }

    #[test]
    fn test_output_format_plaintext() {
        assert_eq!(OutputFormat::Plaintext.to_string(), "plaintext");
    }

    #[test]
    fn benchmark_config_disables_extraction_cache() {
        let config: serde_json::Value = serde_json::from_str(BENCHMARK_CONFIG_JSON).unwrap();
        assert_eq!(config["use_cache"], false);
    }

    #[test]
    fn benchmark_invocations_disable_user_config_discovery() {
        for batch in [false, true] {
            let args = benchmark_base_args(batch, "markdown");
            assert!(args.iter().any(|arg| arg == "--no-config-discovery"));
        }
    }

    #[test]
    fn disabled_ocr_does_not_emit_feature_gated_cli_flags() {
        let mut args = Vec::new();
        append_tesseract_ocr_args(&mut args, false);
        assert!(args.is_empty());

        append_tesseract_ocr_args(&mut args, true);
        assert_eq!(args, ["--ocr", "true", "--ocr-backend", "tesseract"]);
    }

    #[test]
    fn ocr_only_pipeline_is_rejected_when_ocr_is_disabled() {
        let error = match create_xberg_adapter(XbergPipeline::PaddleOcr, OutputFormat::Markdown, false, false) {
            Ok(_) => panic!("OCR-only adapter should not be created when OCR is disabled"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("requires OCR"));
    }

    #[test]
    fn explicit_binary_override_takes_precedence() {
        let directory = tempfile::tempdir().unwrap();
        let path = create_executable_file(directory.path());

        let located = locate_xberg_cli_with_override(Some(path.as_os_str())).unwrap();

        assert_eq!(located, path);
    }

    #[test]
    fn explicit_binary_override_rejects_missing_path_with_context() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing-xberg");

        let error = locate_xberg_cli_with_override(Some(path.as_os_str())).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(XBERG_CLI_BINARY_ENV_VAR));
        assert!(message.contains(path.to_string_lossy().as_ref()));
        assert!(message.contains("metadata could not be read"));
    }

    #[test]
    fn explicit_binary_override_rejects_directory() {
        let directory = tempfile::tempdir().unwrap();

        let error = locate_xberg_cli_with_override(Some(directory.path().as_os_str())).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(XBERG_CLI_BINARY_ENV_VAR));
        assert!(message.contains(directory.path().to_string_lossy().as_ref()));
        assert!(message.contains("is not a file"));
    }

    #[cfg(unix)]
    #[test]
    fn explicit_binary_override_rejects_file_without_execute_permission() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("xberg");
        std::fs::write(&path, b"not executable").unwrap();

        let error = locate_xberg_cli_with_override(Some(path.as_os_str())).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(XBERG_CLI_BINARY_ENV_VAR));
        assert!(message.contains(path.to_string_lossy().as_ref()));
        assert!(message.contains("has no execute permission"));
    }

    #[cfg(windows)]
    #[test]
    fn explicit_binary_override_rejects_file_without_executable_extension() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("xberg.txt");
        std::fs::write(&path, b"not executable").unwrap();

        let error = locate_xberg_cli_with_override(Some(path.as_os_str())).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(XBERG_CLI_BINARY_ENV_VAR));
        assert!(message.contains(path.to_string_lossy().as_ref()));
        assert!(message.contains("has no executable extension"));
    }
}

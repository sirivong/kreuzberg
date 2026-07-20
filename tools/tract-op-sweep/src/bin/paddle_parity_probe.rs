//! Throwaway probe for issue #1275 Phase 5: does tract give numeric parity with ONNX
//! Runtime on the exact ONNX artifacts `xberg-paddle-ocr` loads by default, and can a
//! tract plan pinned to one concrete NCHW shape at load time still run a *different*
//! concrete shape at inference time (DBNet/CRNN see a new shape per document)?
//!
//! Not part of the crate — a scratch binary, deleted before the final commit unless
//! judged worth keeping. Run with cached models already in the HF hub cache:
//!
//! ```sh
//! cargo run -p tract-op-sweep --bin paddle-parity-probe
//! ```

use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor as OrtTensor;
// Glob import: `f32::fact` below resolves through the `DatumExt` extension trait, which
// (like `Framework`/`InferenceModelExt`/`IntoRunnable`) is only reachable via the prelude
// glob, not by qualifying individual items.
use tract_onnx::prelude::*;

fn default_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{home}/.cache/huggingface/hub"))
}

/// Find the newest cached file whose path ends with `suffix` inside `root`.
fn find_cached(root: &Path, suffix: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.to_string_lossy().ends_with(suffix) {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if best.as_ref().is_none_or(|(t, _)| modified > *t) {
                    best = Some((modified, path));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Deterministic pseudo-image data in `[0, 1)`, independent of engine, for `shape`.
fn synthetic_input(shape: &[usize]) -> Vec<f32> {
    let count: usize = shape.iter().product();
    (0..count).map(|i| ((i % 255) as f32) / 255.0).collect()
}

/// Run `path` through ONNX Runtime at `shape`. `ort::Error<R>` embeds non-Send/Sync raw
/// pointers via its resource-typed variant, so every fallible ORT call is stringified
/// immediately rather than propagated with `?` (which would need `anyhow::Error: From<..>`).
fn ort_run(path: &Path, shape: &[usize], data: &[f32]) -> Result<Vec<f32>, String> {
    let builder = Session::builder().map_err(|e| e.to_string())?;
    let builder = builder
        .with_optimization_level(GraphOptimizationLevel::All)
        .map_err(|e| e.to_string())?;
    let builder = builder.with_intra_threads(1).map_err(|e| e.to_string())?;
    let mut builder = builder.with_inter_threads(1).map_err(|e| e.to_string())?;
    let mut session = builder.commit_from_file(path).map_err(|e| e.to_string())?;
    let input_name = session.inputs()[0].name().to_string();
    let array = ndarray::ArrayD::from_shape_vec(shape.to_vec(), data.to_vec()).map_err(|e| e.to_string())?;
    let tensor = OrtTensor::from_array(array).map_err(|e| e.to_string())?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => tensor])
        .map_err(|e| e.to_string())?;
    let (_, first) = outputs.iter().next().ok_or_else(|| "no ORT output".to_string())?;
    let (_, flat) = first.try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
    Ok(flat.to_vec())
}

/// Load `path`, pin input 0 to `shape`, optimize, and build a runnable plan.
fn tract_load_pinned(path: &Path, shape: &[usize]) -> TractResult<std::sync::Arc<TypedRunnableModel>> {
    tract_onnx::onnx()
        .model_for_path(path)?
        .with_input_fact(0, f32::fact(shape).into())?
        .into_optimized()?
        .into_runnable()
}

fn tract_run(plan: &std::sync::Arc<TypedRunnableModel>, shape: &[usize], data: &[f32]) -> Result<Vec<f32>, String> {
    let tensor = Tensor::from_shape(shape, data).map_err(|e| e.to_string())?;
    let inputs: Vec<TValue> = vec![TValue::from(tensor)];
    let outputs = plan.run(inputs.into_iter().collect()).map_err(|e| e.to_string())?;
    let first = outputs.first().ok_or_else(|| "no tract output".to_string())?;
    let view = first.to_plain_array_view::<f32>().map_err(|e| e.to_string())?;
    Ok(view.iter().copied().collect())
}

fn max_abs_diff(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    Some(a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0f32, f32::max))
}

fn probe(name: &str, path: &Path, primary_shape: &[usize], secondary_shape: Option<&[usize]>) {
    println!("\n=== {name} ({}) ===", path.display());

    // ORT at the primary shape (always dynamic-shape-capable, no pinning needed).
    let primary_data = synthetic_input(primary_shape);
    let ort_primary = match ort_run(path, primary_shape, &primary_data) {
        Ok(v) => v,
        Err(e) => {
            println!("  ORT @ {primary_shape:?}: ERROR {e}");
            return;
        }
    };
    println!("  ORT   @ {primary_shape:?}: {} outputs", ort_primary.len());

    // tract: load, pin to the primary shape, optimize, build runnable plan.
    let load_start = std::time::Instant::now();
    let plan = match tract_load_pinned(path, primary_shape) {
        Ok(p) => p,
        Err(e) => {
            println!("  tract load/optimize/runnable @ {primary_shape:?}: ERROR {e:#}");
            return;
        }
    };
    let load_elapsed = load_start.elapsed();

    let run_start = std::time::Instant::now();
    let tract_primary = match tract_run(&plan, primary_shape, &primary_data) {
        Ok(v) => v,
        Err(e) => {
            println!("  tract run @ {primary_shape:?}: ERROR {e}");
            return;
        }
    };
    let run_elapsed = run_start.elapsed();
    println!(
        "  tract @ {primary_shape:?}: {} outputs (load+pin+optimize {load_elapsed:?}, run {run_elapsed:?})",
        tract_primary.len()
    );

    match max_abs_diff(&ort_primary, &tract_primary) {
        Some(diff) => println!("  PARITY @ primary shape: max |Δ| = {diff}"),
        None => println!(
            "  PARITY @ primary shape: output length mismatch (ORT={}, tract={})",
            ort_primary.len(),
            tract_primary.len()
        ),
    }

    // Does the SAME plan (pinned+optimized at primary_shape) accept a genuinely
    // different concrete shape at run() time? This is the real production question
    // for DBNet/CRNN, whose per-document input size varies.
    if let Some(secondary_shape) = secondary_shape {
        let secondary_data = synthetic_input(secondary_shape);

        print!("  tract SAME plan @ different shape {secondary_shape:?}: ");
        match tract_run(&plan, secondary_shape, &secondary_data) {
            Ok(v) => println!("ran, {} outputs (needs separate ORT-at-shape2 check below)", v.len()),
            Err(e) => println!("ERROR {e}"),
        }

        // A freshly re-pinned tract plan at the secondary shape, for comparison. Timed
        // separately: this is the per-shape reload+optimize cost DBNet/CRNN would pay on
        // every new document geometry if the seam builds one plan per exact input shape.
        let repin_start = std::time::Instant::now();
        let plan2_result = tract_load_pinned(path, secondary_shape);
        println!(
            "  tract re-pin+optimize+runnable @ {secondary_shape:?}: {:?}",
            repin_start.elapsed()
        );
        match plan2_result {
            Ok(plan2) => match tract_run(&plan2, secondary_shape, &secondary_data) {
                Ok(tract_secondary) => match ort_run(path, secondary_shape, &secondary_data) {
                    Ok(ort_secondary) => match max_abs_diff(&ort_secondary, &tract_secondary) {
                        Some(diff) => println!("  PARITY @ re-pinned secondary shape: max |Δ| = {diff}"),
                        None => println!("  PARITY @ re-pinned secondary shape: output length mismatch"),
                    },
                    Err(e) => println!("  ORT @ secondary shape: ERROR {e}"),
                },
                Err(e) => println!("  tract run (re-pinned) @ {secondary_shape:?}: ERROR {e}"),
            },
            Err(e) => println!("  tract load/optimize/runnable (re-pinned) @ {secondary_shape:?}: ERROR {e:#}"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cache = default_cache_dir();

    if let Some(path) = find_cached(&cache, "v2/classifiers/PP-LCNet_x1_0_textline_ori.onnx") {
        // AngleNet's actual production cls model (NOT the legacy ch_ppocr_mobile_v2.0_cls_infer
        // used in the Phase-0 sweep's "angle_cls" row): angle_net.rs resizes crops to
        // ANGLE_DST_WIDTH=160 x ANGLE_DST_HEIGHT=80, i.e. NCHW [1,3,80,160].
        probe(
            "textline_ori (AngleNet cls, production model)",
            &path,
            &[1, 3, 80, 160],
            None,
        );
    } else {
        println!("SKIP textline_ori: not cached");
    }

    if let Some(path) = find_cached(&cache, "v6/det/medium/model.onnx") {
        // DBNet: ScaleParam rounds dst_width/dst_height to multiples of 32 per document,
        // so the real deployment sees a different concrete shape almost every time.
        probe(
            "det_v6_medium (DbNet, production model)",
            &path,
            &[1, 3, 640, 640],
            Some(&[1, 3, 320, 480]),
        );
    } else {
        println!("SKIP det_v6_medium: not cached");
    }

    if let Some(path) = find_cached(&cache, "v6/rec/medium/model.onnx") {
        // CRNN: height fixed at 48, width varies with the padded batch's widest crop.
        probe(
            "rec_v6_medium (CrnnNet, production model)",
            &path,
            &[1, 3, 48, 320],
            Some(&[1, 3, 48, 192]),
        );
    } else {
        println!("SKIP rec_v6_medium: not cached");
    }

    Ok(())
}

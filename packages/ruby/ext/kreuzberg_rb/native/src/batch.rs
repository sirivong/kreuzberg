//! Batch extraction functions
//!
//! Handles batch extraction of multiple files or byte arrays.

use crate::config::parse_extraction_config;
use crate::error_handling::{kreuzberg_error, runtime_error};
use crate::result::extraction_result_to_ruby;

use magnus::{Error, RArray, RHash, RString, Ruby, Value, scan_args::scan_args, TryConvert};

/// Batch extract content from multiple files (synchronous)
pub fn batch_extract_files_sync(args: &[Value]) -> Result<RArray, Error> {
    let ruby = Ruby::get().expect("Ruby not initialized");
    let args = scan_args::<(RArray,), (), (), (), RHash, ()>(args)?;
    let (paths_array,) = args.required;
    let opts = Some(args.keywords);

    let config = parse_extraction_config(&ruby, opts)?;

    let paths: Vec<String> = paths_array.to_vec::<String>()?;

    let results = kreuzberg::batch_extract_file_sync(paths, &config).map_err(kreuzberg_error)?;

    let results_array = ruby.ary_new();
    for result in results {
        results_array.push(extraction_result_to_ruby(&ruby, result)?)?;
    }

    Ok(results_array)
}

/// Batch extract content from multiple files (asynchronous)
pub fn batch_extract_files(args: &[Value]) -> Result<RArray, Error> {
    let ruby = Ruby::get().expect("Ruby not initialized");
    let args = scan_args::<(RArray,), (), (), (), RHash, ()>(args)?;
    let (paths_array,) = args.required;
    let opts = Some(args.keywords);

    let config = parse_extraction_config(&ruby, opts)?;

    let paths: Vec<String> = paths_array.to_vec::<String>()?;

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| runtime_error(format!("Failed to create Tokio runtime: {}", e)))?;

    let results = runtime
        .block_on(async { kreuzberg::batch_extract_file(paths, &config).await })
        .map_err(kreuzberg_error)?;

    let results_array = ruby.ary_new();
    for result in results {
        results_array.push(extraction_result_to_ruby(&ruby, result)?)?;
    }

    Ok(results_array)
}

/// Batch extract content from multiple byte arrays (synchronous)
pub fn batch_extract_bytes_sync(args: &[Value]) -> Result<RArray, Error> {
    let ruby = Ruby::get().expect("Ruby not initialized");
    let args = scan_args::<(RArray, RArray), (), (), (), RHash, ()>(args)?;
    let (bytes_array, mime_types_array) = args.required;
    let opts = Some(args.keywords);

    let config = parse_extraction_config(&ruby, opts)?;

    let bytes_vec: Vec<RString> = bytes_array
        .into_iter()
        .map(RString::try_convert)
        .collect::<Result<_, _>>()?;
    let mime_types: Vec<String> = mime_types_array.to_vec::<String>()?;

    if bytes_vec.len() != mime_types.len() {
        return Err(runtime_error(format!(
            "bytes_array and mime_types must have the same length: {} vs {}",
            bytes_vec.len(),
            mime_types.len()
        )));
    }

    let contents: Vec<(Vec<u8>, String)> = bytes_vec
        .iter()
        .zip(mime_types.iter())
        .map(|(bytes, mime): (&RString, &String)| (unsafe { bytes.as_slice() }.to_vec(), mime.clone()))
        .collect();

    let results = kreuzberg::batch_extract_bytes_sync(contents, &config).map_err(kreuzberg_error)?;

    let results_array = ruby.ary_new();
    for result in results {
        results_array.push(extraction_result_to_ruby(&ruby, result)?)?;
    }

    Ok(results_array)
}

/// Batch extract content from multiple byte arrays (asynchronous)
pub fn batch_extract_bytes(args: &[Value]) -> Result<RArray, Error> {
    let ruby = Ruby::get().expect("Ruby not initialized");
    let args = scan_args::<(RArray, RArray), (), (), (), RHash, ()>(args)?;
    let (bytes_array, mime_types_array) = args.required;
    let opts = Some(args.keywords);

    let config = parse_extraction_config(&ruby, opts)?;

    let bytes_vec: Vec<RString> = bytes_array
        .into_iter()
        .map(RString::try_convert)
        .collect::<Result<_, _>>()?;
    let mime_types: Vec<String> = mime_types_array.to_vec::<String>()?;

    if bytes_vec.len() != mime_types.len() {
        return Err(runtime_error(format!(
            "bytes_array and mime_types must have the same length: {} vs {}",
            bytes_vec.len(),
            mime_types.len()
        )));
    }

    let contents: Vec<(Vec<u8>, String)> = bytes_vec
        .iter()
        .zip(mime_types.iter())
        .map(|(bytes, mime): (&RString, &String)| (unsafe { bytes.as_slice() }.to_vec(), mime.clone()))
        .collect();

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| runtime_error(format!("Failed to create Tokio runtime: {}", e)))?;

    let results = runtime
        .block_on(async { kreuzberg::batch_extract_bytes(contents, &config).await })
        .map_err(kreuzberg_error)?;

    let results_array = ruby.ary_new();
    for result in results {
        results_array.push(extraction_result_to_ruby(&ruby, result)?)?;
    }

    Ok(results_array)
}

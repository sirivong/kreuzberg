//! CORD (Receipt OCR) dataset loader.

use super::{DatasetError, Result, Split, StructuredFixture};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// Load CORD dataset fixtures from the given root directory.
///
/// The CORD dataset contains receipt documents with OCR ground truth.
/// Fixtures are organized by split (train/val/test).
///
/// # Arguments
///
/// * `root` - Root directory containing the CORD dataset
/// * `split` - Dataset split to load
///
/// # Returns
///
/// A vector of [`StructuredFixture`] items for the requested split.
pub fn load(root: &Path, split: Split) -> Result<Vec<StructuredFixture>> {
    let split_name = match split {
        Split::Train => "train",
        Split::Val => "val",
        Split::Test => "test",
    };

    let dataset_root = root.join("CORD").join(split_name);
    if !dataset_root.exists() {
        return Err(DatasetError::NotFound(dataset_root.display().to_string()));
    }

    let manifest_path = root.join("manifests").join("cord.toml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| DatasetError::Other(format!("Failed to read CORD manifest: {}", e)))?;

    let schema = load_cord_schema()?;
    let mut fixtures = Vec::new();

    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            continue;
        }

        let doc_file = parts[0];
        let gt_file = parts[1];

        let doc_path = dataset_root.join(doc_file);
        let gt_path = dataset_root.join(gt_file);

        if !doc_path.exists() || !gt_path.exists() {
            continue;
        }

        let mut gt_json: Value = serde_json::from_str(&fs::read_to_string(&gt_path)?)?;

        normalize_cord_menu(&mut gt_json);

        fixtures.push(StructuredFixture {
            document_path: doc_path,
            schema: schema.clone(),
            ground_truth: gt_json,
            dataset: "cord",
            split,
        });
    }

    Ok(fixtures)
}

/// Wrap a single `menu` object into a one-element array so leaf paths align
/// across fixtures regardless of whether the receipt had one item or many.
fn normalize_cord_menu(value: &mut Value) {
    if let Value::Object(map) = value
        && let Some(menu) = map.get_mut("menu")
        && menu.is_object()
    {
        let single = std::mem::replace(menu, Value::Null);
        *menu = Value::Array(vec![single]);
    }
}

/// Load the CORD JSON schema.
///
/// Reflects the actual CORD v2 structure (menu/sub_total/total). All values are
/// kept as `string` to match the GT format (which serialises numerics as
/// quoted strings like `"60.000"`). The schema is permissive
/// (`additionalProperties: true`) so receipts with extra fields don't fail
/// validation, but the listed fields cover the bulk of the dataset's coverage.
fn load_cord_schema() -> Result<Value> {
    Ok(json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "additionalProperties": true,
        "properties": {
            "menu": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": true,
                    "properties": {
                        "nm": { "type": "string", "description": "Item name as printed" },
                        "num": { "type": "string", "description": "Item / SKU number" },
                        "cnt": { "type": "string", "description": "Item quantity / count" },
                        "price": { "type": "string", "description": "Unit price as printed" },
                        "itemsubtotal": { "type": "string", "description": "Line subtotal as printed" }
                    }
                }
            },
            "sub_total": {
                "type": "object",
                "additionalProperties": true,
                "properties": {
                    "subtotal_price": { "type": "string" },
                    "discount_price": { "type": "string" },
                    "tax_price": { "type": "string" },
                    "service_price": { "type": "string" }
                }
            },
            "total": {
                "type": "object",
                "additionalProperties": true,
                "properties": {
                    "total_price": { "type": "string", "description": "Grand total" },
                    "cashprice": { "type": "string" },
                    "creditcardprice": { "type": "string" },
                    "changeprice": { "type": "string" },
                    "menuqty_cnt": { "type": "string", "description": "Total items count" }
                }
            }
        }
    }))
}

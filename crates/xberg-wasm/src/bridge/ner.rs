//! NER (Named Entity Recognition) bridge with injected dispatch.
//!
//! The WASM engine calls an externally-injected JavaScript object that
//! implements a `ner(text, categories)` async method, called positionally to
//! match [`call_injected_ner`]. The host returns a promise resolving to an
//! array of entities (`{ category, text, start, end, confidence? }`).
//!
//! There is no in-binary fallback: when no backend is injected, NER is
//! unavailable and calls return an error saying so.

use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;

use xberg::types::entity::{Entity, EntityCategory};

use crate::bridge::js_from_any;

/// Resolve NER through the injected backend, with a configurable bridge timeout.
pub async fn resolve_ner_with_timeout(
    injected: Option<js_sys::Object>,
    text: &str,
    categories: &[EntityCategory],
    timeout_ms: u32,
) -> Result<Vec<Entity>, JsValue> {
    match injected {
        Some(obj) => call_injected_ner(obj, text, categories, timeout_ms).await,
        None => Err(js_from_any(
            "NER unavailable: no NER backend injected; pass a `ner` object in the engine injection",
        )),
    }
}

/// The wire form of an [`EntityCategory`]: the serde snake_case name for the
/// built-in variants, the raw label for `Custom`. `serde_json::to_value`
/// alone would render `Custom("x")` as an object and lose the label.
fn category_wire_name(category: &EntityCategory) -> String {
    match category {
        EntityCategory::Custom(label) => label.clone(),
        other => serde_json::to_value(other)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default(),
    }
}

/// Call the injected JS `ner(text, categories)` method and deserialize the
/// returned promise into a `Vec<Entity>`.
async fn call_injected_ner(
    obj: Object,
    text: &str,
    categories: &[EntityCategory],
    timeout_ms: u32,
) -> Result<Vec<Entity>, JsValue> {
    let fn_val = Reflect::get(&obj, &JsValue::from_str("ner"))
        .map_err(|e| js_from_any(format!("failed to read 'ner' property: {e:?}")))?;
    let func: Function = fn_val
        .dyn_into()
        .map_err(|_| js_from_any("injected NER object has no 'ner' function"))?;

    let js_text = JsValue::from_str(text);
    let js_cats = js_sys::Array::new();
    for c in categories {
        js_cats.push(&JsValue::from_str(&category_wire_name(c)));
    }
    let args = js_sys::Array::of2(&js_text, &js_cats);

    let result = func.apply(&obj, &args)?;
    let promise = Promise::resolve(&result);
    let js_val = crate::bridge::timed_js_future_with_timeout(promise, timeout_ms).await?;

    serde_wasm_bindgen::from_value(js_val).map_err(|e| js_from_any(format!("failed to deserialize NER result: {e}")))
}

//! NER (Named Entity Recognition) bridge with injected dispatch.
//!
//! The WASM engine calls an externally-injected JavaScript object that
//! implements a `ner(text, categories)` async method, called positionally to
//! match this file's own `call_injected_ner`. When no injection is provided,
//! NER is unavailable and calls return an error saying so.

use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;

use async_trait::async_trait;
use xberg::text::ner::NerBackend;
use xberg::types::entity::{Entity, EntityCategory};

/// Resolve the best available NER backend for the current request.
pub async fn resolve_ner(
    injected: Option<js_sys::Object>,
    text: &str,
    categories: &[EntityCategory],
) -> Result<Vec<Entity>, JsValue> {
    resolve_ner_with_timeout(injected, text, categories, crate::bridge::BRIDGE_TIMEOUT_MS).await
}

/// Like [`resolve_ner`] but with a configurable bridge timeout.
pub async fn resolve_ner_with_timeout(
    injected: Option<js_sys::Object>,
    text: &str,
    categories: &[EntityCategory],
    timeout_ms: u32,
) -> Result<Vec<Entity>, JsValue> {
    match injected {
        Some(obj) => call_injected_ner(obj, text, categories, timeout_ms).await,
        None => Err(js_from_any(
            "NER unavailable: no injected backend",
        )),
    }
}

/// Call the injected JS `ner(text, categories)` method and deserialize the
/// returned promise into a Vec<Entity>.
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
        let cat_str = serde_json::to_value(c)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        js_cats.push(&JsValue::from_str(&cat_str));
    }
    let args = js_sys::Array::of2(&js_text, &js_cats);

    let result = func.apply(&obj, &args)?;
    let promise = Promise::from(result);
    let js_val = crate::bridge::timed_js_future_with_timeout(promise, timeout_ms).await?;

    let entities: Vec<Entity> = serde_wasm_bindgen::from_value(js_val)
        .map_err(|e| js_from_any(format!("failed to deserialize NER result: {e}")))?;
    Ok(entities)
}

/// Adapter that wraps an injected JS NER object as a [`NerBackend`].
pub(crate) struct JsNerBridge {
    obj: Object,
    timeout_ms: u32,
}

impl JsNerBridge {
    /// Wrap an injected JS object that exposes `ner(text, categories)`.
    pub fn new(obj: Object, timeout_ms: u32) -> Self {
        Self { obj, timeout_ms }
    }
}

#[async_trait(?Send)]
impl NerBackend for JsNerBridge {
    async fn detect(&self, text: &str, categories: &[EntityCategory]) -> xberg::Result<Vec<Entity>> {
        call_injected_ner(self.obj.clone(), text, categories, self.timeout_ms)
            .await
            .map_err(|e| xberg::XbergError::Plugin {
                message: format!("JS NER bridge: {e:?}"),
                plugin_name: "js-ner-bridge".to_string(),
            })
    }
}

/// Convert a Display error into a JsValue suitable for propagation.
fn js_from_any(v: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&v.to_string())
}

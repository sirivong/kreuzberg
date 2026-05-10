//! Renderer plugin trait.
//!
//! This module defines the trait for implementing custom document renderers
//! that convert [`InternalDocument`] to output format strings.

use std::sync::Arc;

use crate::Result;
use crate::plugins::Plugin;
use crate::types::internal::InternalDocument;

/// Trait for document renderers that convert [`InternalDocument`] to output strings.
///
/// Renderers are typically stateless converters that transform the internal
/// document representation into a specific output format (Markdown, HTML,
/// Djot, plain text, etc.). They participate in the standard [`Plugin`]
/// lifecycle so custom renderers can be registered from any supported binding
/// language.
///
/// The format name is exposed via [`Plugin::name`]. For stateless renderers
/// the [`Plugin`] lifecycle methods (`version`, `initialize`, `shutdown`) all
/// take no-op defaults and need not be overridden.
///
/// # Thread Safety
///
/// Renderers must be `Send + Sync` (inherited from [`Plugin`]).
///
/// # Example
///
/// ```rust
/// use kreuzberg::plugins::{Plugin, Renderer};
/// use kreuzberg::types::internal::InternalDocument;
/// use kreuzberg::Result;
///
/// struct CustomRenderer;
///
/// impl Plugin for CustomRenderer {
///     fn name(&self) -> &str { "custom" }
/// }
///
/// impl Renderer for CustomRenderer {
///     fn render(&self, doc: &InternalDocument) -> Result<String> {
///         Ok(format!("Custom output with {} elements", doc.elements.len()))
///     }
/// }
/// ```
pub trait Renderer: Plugin {
    /// Render an [`InternalDocument`] to the output format.
    ///
    /// # Arguments
    ///
    /// * `doc` - The internal document to render
    ///
    /// # Returns
    ///
    /// The rendered output as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    fn render(&self, doc: &InternalDocument) -> Result<String>;
}

/// Register a renderer plugin with the global registry.
///
/// The renderer's format name is taken from [`Plugin::name`]. Registering a
/// renderer with a name that already exists replaces the previous renderer
/// for that format.
pub fn register_renderer(renderer: Arc<dyn Renderer>) -> Result<()> {
    use crate::plugins::registry::get_renderer_registry;

    let registry = get_renderer_registry();
    let mut registry = registry.write();
    registry.register(renderer)
}

/// Unregister a renderer by format name.
pub fn unregister_renderer(name: &str) {
    use crate::plugins::registry::get_renderer_registry;

    let registry = get_renderer_registry();
    let mut registry = registry.write();
    registry.remove(name);
}

/// List names of all registered renderers.
pub fn list_renderers() -> Vec<String> {
    use crate::plugins::registry::get_renderer_registry;

    let registry = get_renderer_registry();
    let registry = registry.read();
    registry.list()
}

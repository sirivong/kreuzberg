//! Table-related types for document extraction.

use serde::{Deserialize, Serialize};

/// Extracted table structure.
///
/// Represents a table detected and extracted from a document (PDF, image, etc.).
/// Tables are converted to both structured cell data and Markdown format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct Table {
    /// Table cells as a 2D vector (rows × columns)
    pub cells: Vec<Vec<String>>,
    /// Markdown representation of the table
    pub markdown: String,
    /// Page number where the table was found (1-indexed)
    pub page_number: usize,
}

/// Individual table cell with content and optional styling.
///
/// Future extension point for rich table support with cell-level metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct TableCell {
    /// Cell content as text
    pub content: String,
    /// Row span (number of rows this cell spans)
    #[serde(default = "default_span")]
    pub row_span: usize,
    /// Column span (number of columns this cell spans)
    #[serde(default = "default_span")]
    pub col_span: usize,
    /// Whether this is a header cell
    #[serde(default)]
    pub is_header: bool,
}

fn default_span() -> usize {
    1
}

//! Per-page bounding-box aggregation for chunk `page_spans` (#1295).
//!
//! [`crate::chunking::boundaries::calculate_page_spans`] derives the *page numbers*
//! a chunk covers from byte-range overlap against [`PageBoundary`](crate::types::PageBoundary)
//! markers — the same mechanism already used for `first_page`/`last_page`. This module adds the
//! *bounding box* half: once a document's structured node tree
//! ([`DocumentStructure`]) is available, [`populate_page_span_bboxes`] fills each
//! [`PageSpan`](crate::types::PageSpan)'s `bbox` with the union of the bounding boxes of the
//! body-layer nodes on that page whose text appears in the chunk.
//!
//! There is currently no byte-offset mapping from rendered output back to individual
//! [`DocumentNode`](crate::types::document_structure::DocumentNode)s (tracked under #1294/#1296;
//! see [`DocumentStructure::node_rendered_offset`](crate::types::document_structure::DocumentStructure::node_rendered_offset)).
//! In its absence, node-to-chunk membership on a given page is determined by a textual
//! containment check — the same substring-matching approach
//! [`locate_page_boundaries`](crate::core::pipeline::features) already uses to align raw page
//! text with rendered content — rather than a byte-exact intersection.

use crate::types::document_structure::{ContentLayer, DocumentNode, DocumentStructure, NodeContent};
use crate::types::{BoundingBox, Chunk};

/// Minimum trimmed node-text length considered for containment matching.
///
/// Very short text (e.g. a single character or short label) matches too many chunks by
/// coincidence to be a meaningful signal of chunk membership, so nodes with shorter text are
/// skipped for bbox aggregation (their page still appears in `page_spans`, just without a bbox
/// contribution from that node).
const MIN_NODE_TEXT_MATCH_LEN: usize = 4;

/// Fill in `bbox` on each chunk's `page_spans` using the document's structured node tree.
///
/// For every chunk and every [`PageSpan`](crate::types::PageSpan) already present on it (as
/// produced by `calculate_page_spans`), this unions the bounding boxes of all body-layer nodes
/// that:
/// - fall on that page (`node.page..=node.page_end` covers the span's page), and
/// - carry a `bbox`, and
/// - carry matching text (see [`node_text_for_matching`]) found verbatim within the chunk's
///   `content`.
///
/// Chunks with empty `page_spans` (no page-boundary provenance) and nodes without a usable text
/// or bbox are skipped. A span's `bbox` stays `None` when no node in `structure` matches.
pub(crate) fn populate_page_span_bboxes(chunks: &mut [Chunk], structure: &DocumentStructure) {
    for chunk in chunks.iter_mut() {
        if chunk.metadata.page_spans.is_empty() {
            continue;
        }

        for span in chunk.metadata.page_spans.iter_mut() {
            span.bbox = union_matching_node_bboxes(&chunk.content, span.page, &structure.nodes);
        }
    }
}

/// Union the bounding boxes of all nodes on `page` whose text is found in `chunk_content`.
fn union_matching_node_bboxes(chunk_content: &str, page: u32, nodes: &[DocumentNode]) -> Option<BoundingBox> {
    nodes
        .iter()
        .filter(|node| node.content_layer == ContentLayer::Body)
        .filter(|node| node_covers_page(node, page))
        .filter_map(|node| node.bbox.map(|bbox| (node, bbox)))
        .filter(|(node, _)| node_text_matches_chunk(node, chunk_content))
        .map(|(_, bbox)| bbox)
        .fold(None, |acc, bbox| Some(union_bbox(acc, bbox)))
}

/// Whether `node`'s page range (`page..=page_end`, falling back to just `page`) includes `page`.
fn node_covers_page(node: &DocumentNode, page: u32) -> bool {
    match (node.page, node.page_end) {
        (Some(start), Some(end)) => (start..=end).contains(&page),
        (Some(start), None) => start == page,
        (None, _) => false,
    }
}

/// Check whether `node`'s matchable text (see [`node_text_for_matching`]) appears verbatim in
/// `chunk_content`, after trimming and enforcing [`MIN_NODE_TEXT_MATCH_LEN`].
fn node_text_matches_chunk(node: &DocumentNode, chunk_content: &str) -> bool {
    let Some(text) = node_text_for_matching(&node.content) else {
        return false;
    };
    let trimmed = text.trim();
    trimmed.len() >= MIN_NODE_TEXT_MATCH_LEN && chunk_content.contains(trimmed)
}

/// Extract the primary text content of a node for chunk-containment matching, if any.
///
/// Only variants that carry a single, directly comparable text string are handled — container
/// nodes (`List`, `Group`, `Quote`, …), tables, and images have no single text span to match
/// against rendered chunk content.
fn node_text_for_matching(content: &NodeContent) -> Option<&str> {
    match content {
        NodeContent::Title { text }
        | NodeContent::Heading { text, .. }
        | NodeContent::Paragraph { text }
        | NodeContent::ListItem { text }
        | NodeContent::Code { text, .. }
        | NodeContent::Formula { text }
        | NodeContent::Footnote { text }
        | NodeContent::Citation { text, .. } => Some(text.as_str()),
        _ => None,
    }
}

/// Union two bounding boxes into the smallest box that encloses both, or just `next` if `acc` is
/// absent.
fn union_bbox(acc: Option<BoundingBox>, next: BoundingBox) -> BoundingBox {
    match acc {
        None => next,
        Some(acc) => BoundingBox {
            x0: acc.x0.min(next.x0),
            y0: acc.y0.min(next.y0),
            x1: acc.x1.max(next.x1),
            y1: acc.y1.max(next.y1),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::document_structure::NodeId;
    use crate::types::{ChunkMetadata, ChunkType, PageSpan};

    fn body_node(text: &str, page: u32, bbox: Option<BoundingBox>) -> DocumentNode {
        DocumentNode {
            id: NodeId::default(),
            content: NodeContent::Paragraph { text: text.to_string() },
            parent: None,
            children: Vec::new(),
            content_layer: ContentLayer::Body,
            page: Some(page),
            page_end: None,
            bbox,
            annotations: Vec::new(),
            attributes: None,
        }
    }

    fn bbox(x0: f64, y0: f64, x1: f64, y1: f64) -> BoundingBox {
        BoundingBox { x0, y0, x1, y1 }
    }

    fn chunk_with_spans(content: &str, spans: Vec<PageSpan>) -> Chunk {
        Chunk {
            content: content.to_string(),
            chunk_type: ChunkType::default(),
            embedding: None,
            metadata: ChunkMetadata {
                byte_start: 0,
                byte_end: content.len(),
                token_count: None,
                chunk_index: 0,
                total_chunks: 1,
                first_page: spans.first().map(|s| s.page),
                last_page: spans.last().map(|s| s.page),
                heading_context: None,
                heading_path: Vec::new(),
                image_indices: Vec::new(),
                node_ids: Vec::new(),
                page_spans: spans,
            },
        }
    }

    #[test]
    fn should_populate_bbox_when_single_page_node_text_found_in_chunk() {
        let structure = DocumentStructure {
            nodes: vec![body_node(
                "Hello world, this is page one.",
                1,
                Some(bbox(10.0, 20.0, 100.0, 200.0)),
            )],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Hello world, this is page one.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(chunks[0].metadata.page_spans.len(), 1);
        assert_eq!(chunks[0].metadata.page_spans[0].page, 1);
        assert_eq!(
            chunks[0].metadata.page_spans[0].bbox,
            Some(bbox(10.0, 20.0, 100.0, 200.0))
        );
    }

    #[test]
    fn should_union_bboxes_when_multi_page_chunk_has_a_node_on_each_page() {
        let structure = DocumentStructure {
            nodes: vec![
                body_node("Content from page three.", 3, Some(bbox(72.2, 255.8, 400.0, 400.0))),
                body_node("Content from page four.", 4, Some(bbox(56.6, 610.1, 530.3, 766.1))),
            ],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Content from page three.\n\nContent from page four.",
            vec![PageSpan { page: 3, bbox: None }, PageSpan { page: 4, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        let spans = &chunks[0].metadata.page_spans;
        assert_eq!(
            spans.len(),
            2,
            "chunk spanning pages 3-4 must keep one PageSpan per page"
        );
        assert_eq!(spans[0].page, 3);
        assert_eq!(spans[0].bbox, Some(bbox(72.2, 255.8, 400.0, 400.0)));
        assert_eq!(spans[1].page, 4);
        assert_eq!(spans[1].bbox, Some(bbox(56.6, 610.1, 530.3, 766.1)));
    }

    #[test]
    fn should_union_multiple_node_bboxes_on_the_same_page() {
        let structure = DocumentStructure {
            nodes: vec![
                body_node("First paragraph on the page.", 1, Some(bbox(10.0, 10.0, 50.0, 50.0))),
                body_node("Second paragraph on the page.", 1, Some(bbox(60.0, 60.0, 120.0, 120.0))),
            ],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "First paragraph on the page.\n\nSecond paragraph on the page.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(
            chunks[0].metadata.page_spans[0].bbox,
            Some(bbox(10.0, 10.0, 120.0, 120.0))
        );
    }

    #[test]
    fn should_leave_bbox_none_when_no_node_bbox_available() {
        let structure = DocumentStructure {
            nodes: vec![body_node("Hello world, this is page one.", 1, None)],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Hello world, this is page one.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(chunks[0].metadata.page_spans[0].bbox, None);
    }

    #[test]
    fn should_leave_bbox_none_when_node_text_not_found_in_chunk() {
        let structure = DocumentStructure {
            nodes: vec![body_node(
                "Unrelated content elsewhere.",
                1,
                Some(bbox(1.0, 1.0, 2.0, 2.0)),
            )],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Completely different chunk text.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(chunks[0].metadata.page_spans[0].bbox, None);
    }

    #[test]
    fn should_skip_non_body_layer_nodes() {
        let mut header = body_node("Running header text.", 1, Some(bbox(1.0, 1.0, 2.0, 2.0)));
        header.content_layer = ContentLayer::Header;
        let structure = DocumentStructure {
            nodes: vec![header],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Running header text.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(
            chunks[0].metadata.page_spans[0].bbox, None,
            "header/footer/footnote nodes must not contribute bboxes"
        );
    }

    #[test]
    fn should_noop_when_chunk_has_no_page_spans() {
        let structure = DocumentStructure {
            nodes: vec![body_node("Some text.", 1, Some(bbox(1.0, 1.0, 2.0, 2.0)))],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans("Some text.", Vec::new())];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert!(chunks[0].metadata.page_spans.is_empty());
    }

    #[test]
    fn should_match_node_that_spans_a_page_range() {
        let mut node = body_node("Table caption spanning pages.", 2, Some(bbox(5.0, 5.0, 15.0, 15.0)));
        node.page_end = Some(3);
        let structure = DocumentStructure {
            nodes: vec![node],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Table caption spanning pages.",
            vec![PageSpan { page: 3, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(
            chunks[0].metadata.page_spans[0].bbox,
            Some(bbox(5.0, 5.0, 15.0, 15.0)),
            "node with page_end covering the span page must match"
        );
    }

    #[test]
    fn should_not_match_short_text_below_min_length_threshold() {
        let structure = DocumentStructure {
            nodes: vec![body_node("Hi", 1, Some(bbox(1.0, 1.0, 2.0, 2.0)))],
            source_format: None,
            relationships: Vec::new(),
            node_types: Vec::new(),
        };
        let mut chunks = vec![chunk_with_spans(
            "Hi there, this chunk contains Hi as a substring.",
            vec![PageSpan { page: 1, bbox: None }],
        )];

        populate_page_span_bboxes(&mut chunks, &structure);

        assert_eq!(
            chunks[0].metadata.page_spans[0].bbox, None,
            "text shorter than MIN_NODE_TEXT_MATCH_LEN must not be treated as a membership signal"
        );
    }

    #[test]
    fn union_bbox_expands_to_enclose_both_boxes() {
        let result = union_bbox(Some(bbox(0.0, 0.0, 10.0, 10.0)), bbox(5.0, -5.0, 20.0, 8.0));
        assert_eq!(result, bbox(0.0, -5.0, 20.0, 10.0));
    }

    #[test]
    fn union_bbox_returns_next_when_acc_is_none() {
        let result = union_bbox(None, bbox(1.0, 2.0, 3.0, 4.0));
        assert_eq!(result, bbox(1.0, 2.0, 3.0, 4.0));
    }
}

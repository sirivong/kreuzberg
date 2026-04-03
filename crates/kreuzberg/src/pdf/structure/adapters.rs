//! Adapters that convert extraction-source-specific types into the unified
//! [`PageContent`] DTO for the shared markdown pipeline.

use pdfium_render::prelude::{ContentRole, ExtractedBlock};

use super::content::{ContentElement, ElementLevel, PageContent, SemanticRole};
use super::geometry::Rect;
// ── Structure tree adapter ──────────────────────────────────────────────

/// Convert structure-tree `ExtractedBlock`s into a [`PageContent`].
///
/// Flattens the block hierarchy into a flat list of `ContentElement`s,
/// mapping `ContentRole` to `SemanticRole` and extracting bounding boxes.
pub(super) fn from_structure_tree(blocks: &[ExtractedBlock]) -> PageContent {
    let mut elements = Vec::new();
    flatten_blocks(blocks, &mut elements);

    PageContent { elements }
}

/// Recursively flatten `ExtractedBlock` hierarchy into `ContentElement`s.
fn flatten_blocks(blocks: &[ExtractedBlock], elements: &mut Vec<ContentElement>) {
    for block in blocks {
        if !block.children.is_empty() {
            flatten_blocks(&block.children, elements);
            continue;
        }

        if block.text.trim().is_empty() {
            continue;
        }

        let bbox = block
            .bounds
            .as_ref()
            .map(|b| Rect::from_lbrt(b.left().value, b.bottom().value, b.right().value, b.top().value));

        let (semantic_role, list_label) = map_content_role(&block.role);

        elements.push(ContentElement {
            text: block.text.clone(),
            bbox,
            font_size: block.font_size,
            is_bold: block.is_bold,
            is_italic: block.is_italic,
            is_monospace: block.is_monospace,
            semantic_role: Some(semantic_role),
            level: ElementLevel::Block,
            list_label,
            layout_class: None,
        });
    }
}

/// Map `ContentRole` from pdfium to our `SemanticRole`.
fn map_content_role(role: &ContentRole) -> (SemanticRole, Option<String>) {
    match role {
        ContentRole::Heading { level } => (SemanticRole::Heading { level: *level }, None),
        ContentRole::Paragraph => (SemanticRole::Paragraph, None),
        ContentRole::ListItem { label } => (SemanticRole::ListItem, label.clone()),
        ContentRole::TableCell { .. } => (SemanticRole::TableCell, None),
        ContentRole::Figure { .. } => (SemanticRole::Figure, None),
        ContentRole::Caption => (SemanticRole::Caption, None),
        ContentRole::Code => (SemanticRole::Code, None),
        ContentRole::BlockQuote => (SemanticRole::BlockQuote, None),
        ContentRole::Link { .. } => (SemanticRole::Paragraph, None),
        ContentRole::Other(s) if s == "Formula" => (SemanticRole::Formula, None),
        ContentRole::Other(_) => (SemanticRole::Other, None),
    }
}

// ── OCR adapter ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pdfium_render::prelude::PdfPoints;
    use pdfium_render::prelude::PdfRect;

    fn make_block(role: ContentRole, text: &str) -> ExtractedBlock {
        ExtractedBlock {
            role,
            text: text.to_string(),
            bounds: None,
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }
    }

    fn make_block_with_bounds(role: ContentRole, text: &str) -> ExtractedBlock {
        ExtractedBlock {
            role,
            text: text.to_string(),
            bounds: Some(PdfRect::new(
                PdfPoints::new(100.0),
                PdfPoints::new(50.0),
                PdfPoints::new(200.0),
                PdfPoints::new(400.0),
            )),
            font_size: Some(12.0),
            is_bold: true,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_from_structure_tree_basic() {
        let blocks = vec![
            make_block(ContentRole::Heading { level: 1 }, "Title"),
            make_block(ContentRole::Paragraph, "Body text"),
        ];
        let page = from_structure_tree(&blocks);
        assert_eq!(page.elements.len(), 2);
        assert_eq!(page.elements[0].semantic_role, Some(SemanticRole::Heading { level: 1 }));
        assert_eq!(page.elements[1].semantic_role, Some(SemanticRole::Paragraph));
    }

    #[test]
    fn test_from_structure_tree_skips_empty() {
        let blocks = vec![
            make_block(ContentRole::Paragraph, ""),
            make_block(ContentRole::Paragraph, "   "),
            make_block(ContentRole::Paragraph, "Real text"),
        ];
        let page = from_structure_tree(&blocks);
        assert_eq!(page.elements.len(), 1);
        assert_eq!(page.elements[0].text, "Real text");
    }

    #[test]
    fn test_from_structure_tree_flattens_children() {
        let blocks = vec![ExtractedBlock {
            role: ContentRole::Other("Table".to_string()),
            text: String::new(),
            bounds: None,
            font_size: None,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: vec![
                make_block(ContentRole::Paragraph, "Cell 1"),
                make_block(ContentRole::Paragraph, "Cell 2"),
            ],
        }];
        let page = from_structure_tree(&blocks);
        assert_eq!(page.elements.len(), 2);
    }

    #[test]
    fn test_from_structure_tree_maps_bounds() {
        let blocks = vec![make_block_with_bounds(ContentRole::Paragraph, "With bounds")];
        let page = from_structure_tree(&blocks);
        let elem = &page.elements[0];
        assert!(elem.bbox.is_some());
        assert!(elem.is_bold);
    }

    #[test]
    fn test_from_structure_tree_list_item_label() {
        let blocks = vec![ExtractedBlock {
            role: ContentRole::ListItem {
                label: Some("1.".to_string()),
            },
            text: "First item".to_string(),
            bounds: None,
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }];
        let page = from_structure_tree(&blocks);
        assert_eq!(page.elements[0].semantic_role, Some(SemanticRole::ListItem));
        assert_eq!(page.elements[0].list_label, Some("1.".to_string()));
    }

    #[test]
    fn test_map_content_role_all_variants() {
        assert_eq!(
            map_content_role(&ContentRole::Heading { level: 3 }),
            (SemanticRole::Heading { level: 3 }, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::Paragraph),
            (SemanticRole::Paragraph, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::ListItem {
                label: Some("a.".to_string())
            }),
            (SemanticRole::ListItem, Some("a.".to_string()))
        );
        assert_eq!(
            map_content_role(&ContentRole::TableCell {
                row: 0,
                col: 0,
                is_header: false,
            }),
            (SemanticRole::TableCell, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::Figure { alt_text: None }),
            (SemanticRole::Figure, None)
        );
        assert_eq!(map_content_role(&ContentRole::Caption), (SemanticRole::Caption, None));
        assert_eq!(map_content_role(&ContentRole::Code), (SemanticRole::Code, None));
        assert_eq!(
            map_content_role(&ContentRole::BlockQuote),
            (SemanticRole::BlockQuote, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::Link { url: None }),
            (SemanticRole::Paragraph, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::Other("Formula".to_string())),
            (SemanticRole::Formula, None)
        );
        assert_eq!(
            map_content_role(&ContentRole::Other("Unknown".to_string())),
            (SemanticRole::Other, None)
        );
    }

    #[test]
    fn test_from_structure_tree_page_metadata() {
        let page = from_structure_tree(&[]);
        assert!(page.elements.is_empty());
    }
}

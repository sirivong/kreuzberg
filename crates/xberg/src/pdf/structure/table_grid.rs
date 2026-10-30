use crate::types::internal::{ElementKind, InternalDocument, NATIVE_TABLE_GRID_ATTRIBUTE};
use crate::types::{Table, TableGrid};

type TableIdentity<'a> = (u32, Option<[u64; 4]>, &'a [Vec<String>]);

pub(crate) struct TableGridSidecar {
    pub(crate) table: Table,
    pub(crate) grid: TableGrid,
}

#[cfg(feature = "layout-detection")]
#[derive(Default)]
pub(crate) struct RecognizedNativeTables {
    pub(crate) tables: Vec<Table>,
    pub(crate) grids: Vec<TableGridSidecar>,
}

#[cfg(feature = "layout-detection")]
impl From<Vec<Table>> for RecognizedNativeTables {
    fn from(tables: Vec<Table>) -> Self {
        Self { tables, grids: vec![] }
    }
}

fn identity(table: &Table) -> TableIdentity<'_> {
    let bbox = table.bounding_box.map(|bbox| {
        [
            bbox.x0.to_bits(),
            bbox.y0.to_bits(),
            bbox.x1.to_bits(),
            bbox.y1.to_bits(),
        ]
    });
    (table.page_number, bbox, &table.cells)
}

pub(crate) fn retained_grid(table: &Table, attributes: Option<&ahash::AHashMap<String, String>>) -> Option<TableGrid> {
    let encoded = attributes?.get(NATIVE_TABLE_GRID_ATTRIBUTE)?;
    match serde_json::from_str::<TableGrid>(encoded) {
        Ok(grid) => crate::extraction::derive::native_grid_matches_table(table, &grid).then_some(grid),
        Err(error) => {
            tracing::warn!(%error, "discarding invalid retained table geometry");
            None
        }
    }
}

pub(crate) fn attach_table_grids(
    document: &mut InternalDocument,
    sidecars: &[TableGridSidecar],
) -> serde_json::Result<()> {
    let mut candidates = ahash::AHashMap::new();
    for sidecar in sidecars {
        let grid = crate::extraction::derive::native_grid_matches_table(&sidecar.table, &sidecar.grid)
            .then_some(&sidecar.grid);
        // Multiple producers for one identity cannot prove which geometry survived deduplication. ~keep
        candidates
            .entry(identity(&sidecar.table))
            .and_modify(|value| *value = None)
            .or_insert(grid);
    }
    let mut final_counts = ahash::AHashMap::<TableIdentity<'_>, usize>::new();
    for table in &document.tables {
        *final_counts.entry(identity(table)).or_default() += 1;
    }
    for element in &mut document.elements {
        let ElementKind::Table { table_index } = element.kind else {
            continue;
        };
        let Some(table) = document.tables.get(table_index as usize) else {
            continue;
        };
        if final_counts.get(&identity(table)) != Some(&1)
            || element.page.is_some_and(|page| page != table.page_number)
            || retained_grid(table, element.attributes.as_ref()).is_some()
        {
            continue;
        }
        let Some(Some(grid)) = candidates.get(&identity(table)) else {
            continue;
        };
        element
            .attributes
            .get_or_insert_with(Default::default)
            .insert(NATIVE_TABLE_GRID_ATTRIBUTE.to_owned(), serde_json::to_string(grid)?);
    }
    Ok(())
}

#[cfg(test)]
#[path = "table_grid/tests.rs"]
mod tests;

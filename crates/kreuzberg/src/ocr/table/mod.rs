pub mod tsv_parser;

pub use html_to_markdown_rs::hocr::{HocrWord, reconstruct_table, table_to_markdown};
pub use tsv_parser::extract_words_from_tsv;

/// Post-process a raw table grid to validate structure and clean up.
///
/// Returns `None` if the table fails structural validation:
/// - All rows empty after stripping blank rows
/// - >66% of non-empty cells exceed 60 chars (prose, not table data)
/// - Fewer than 2 columns or 2 rows after processing
/// - Any column has <15% of total content AND >50% empty cells (text overflow)
///
/// On success, returns a cleaned table with:
/// - Merged header rows
/// - Removed header-only columns
/// - Normalized cell content
pub fn post_process_table(mut table: Vec<Vec<String>>) -> Option<Vec<Vec<String>>> {
    // Strip empty rows
    table.retain(|row| row.iter().any(|cell| !cell.trim().is_empty()));
    if table.is_empty() {
        return None;
    }

    // Reject prose: if >50% of non-empty cells exceed 60 chars, it's not a table.
    // Also reject if average cell length exceeds 40 chars — real table cells are
    // typically short values, not flowing sentences.
    let mut non_empty = 0usize;
    let mut long_cells = 0usize;
    let mut total_chars = 0usize;
    for row in &table {
        for cell in row {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            let char_count = trimmed.chars().count();
            non_empty += 1;
            total_chars += char_count;
            if char_count > 60 {
                long_cells += 1;
            }
        }
    }

    if non_empty > 0 {
        // >50% long cells → prose
        if long_cells * 2 > non_empty {
            return None;
        }
        // Average cell length >50 chars → likely prose paragraphs, not table data
        if total_chars / non_empty > 50 {
            return None;
        }
    }

    // Two-column rejection: tables with exactly 2 columns are almost always false
    // positives from two-column text layouts (academic papers, newsletters, etc.).
    // Real tables with only 2 meaningful columns are extremely rare in practice;
    // they typically have at least 3 columns (key-value tables have a label column).
    let col_count = table.first().map_or(0, Vec::len);
    if col_count <= 2 {
        return None;
    }

    // Find where data rows start (first row with ≥3 cells containing digits)
    let data_start = table
        .iter()
        .enumerate()
        .find_map(|(idx, row)| {
            let digit_cells = row
                .iter()
                .filter(|cell| cell.chars().any(|c| c.is_ascii_digit()))
                .count();
            if digit_cells >= 3 { Some(idx) } else { None }
        })
        .unwrap_or(0);

    let mut header_rows = if data_start > 0 {
        table[..data_start].to_vec()
    } else {
        Vec::new()
    };
    let mut data_rows = table[data_start..].to_vec();

    // Keep at most 2 header rows
    if header_rows.len() > 2 {
        header_rows = header_rows[header_rows.len() - 2..].to_vec();
    }

    // If no header detected, promote first data row
    if header_rows.is_empty() {
        if data_rows.len() < 2 {
            return None;
        }
        header_rows.push(data_rows[0].clone());
        data_rows = data_rows[1..].to_vec();
    }

    let column_count = header_rows.first().or_else(|| data_rows.first()).map_or(0, Vec::len);

    if column_count == 0 {
        return None;
    }

    // Merge multi-row headers into a single header row
    let mut header = vec![String::new(); column_count];
    for row in &header_rows {
        for (idx, cell) in row.iter().enumerate() {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !header[idx].is_empty() {
                header[idx].push(' ');
            }
            header[idx].push_str(trimmed);
        }
    }

    let mut processed = Vec::new();
    processed.push(header);
    processed.extend(data_rows);

    if processed.len() <= 1 {
        return None;
    }

    // Remove header-only columns (header text but no data)
    let mut col = 0;
    while col < processed[0].len() {
        let header_text = processed[0][col].trim().to_string();
        let data_empty = processed[1..]
            .iter()
            .all(|row| row.get(col).is_none_or(|cell| cell.trim().is_empty()));

        if data_empty {
            merge_header_only_column(&mut processed, col, header_text);
        } else {
            col += 1;
        }

        if processed.is_empty() || processed[0].is_empty() {
            return None;
        }
    }

    // Final dimension check: must have ≥2 columns and ≥2 rows
    if processed[0].len() < 2 || processed.len() <= 1 {
        return None;
    }

    // Column sparsity check: reject if any column has >75% empty data cells.
    // This catches false positives from regular text split into pseudo-columns.
    let data_row_count = processed.len() - 1;
    if data_row_count > 0 {
        for c in 0..processed[0].len() {
            let empty_count = processed[1..]
                .iter()
                .filter(|row| row.get(c).is_none_or(|cell| cell.trim().is_empty()))
                .count();
            if empty_count * 4 > data_row_count * 3 {
                return None;
            }
        }
    }

    // Overall density check: reject if <40% of data cells are non-empty.
    // Real tables have mostly filled cells; sparse grids are usually text misdetection.
    {
        let total_data_cells = data_row_count * processed[0].len();
        if total_data_cells > 0 {
            let filled = processed[1..]
                .iter()
                .flat_map(|row| row.iter())
                .filter(|cell| !cell.trim().is_empty())
                .count();
            if filled * 5 < total_data_cells * 2 {
                return None;
            }
        }
    }

    // Content asymmetry check: reject if any column carries <12% of total content
    // AND has >50% empty data cells. This catches regular prose that overflows into
    // a pseudo-column (e.g., paragraph text split at the right margin).
    {
        let num_cols = processed[0].len();
        let col_char_counts: Vec<usize> = (0..num_cols)
            .map(|c| {
                processed[1..]
                    .iter()
                    .map(|row| row.get(c).map_or(0, |cell| cell.trim().len()))
                    .sum()
            })
            .collect();
        let total_chars: usize = col_char_counts.iter().sum();

        if total_chars > 0 {
            for (c, &col_chars) in col_char_counts.iter().enumerate() {
                let char_share = col_chars as f64 / total_chars as f64;
                let empty_in_col = processed[1..]
                    .iter()
                    .filter(|row| row.get(c).is_none_or(|cell| cell.trim().is_empty()))
                    .count();
                let empty_ratio = empty_in_col as f64 / data_row_count as f64;

                // Column carries very little content and is mostly empty -> text overflow
                if char_share < 0.15 && empty_ratio > 0.5 {
                    return None;
                }
            }
        }
    }

    // Normalize cells
    for cell in &mut processed[0] {
        let text = cell.trim().replace("  ", " ");
        *cell = text;
    }

    for row in processed.iter_mut().skip(1) {
        for cell in row.iter_mut() {
            normalize_data_cell(cell);
        }
    }

    Some(processed)
}

fn merge_header_only_column(table: &mut [Vec<String>], col: usize, header_text: String) {
    if table.is_empty() || table[0].is_empty() {
        return;
    }

    let trimmed = header_text.trim();
    if trimmed.is_empty() && table.len() > 1 {
        for row in table.iter_mut() {
            row.remove(col);
        }
        return;
    }

    if !trimmed.is_empty() {
        if col > 0 {
            let mut target = col - 1;
            while target > 0 && table[0][target].trim().is_empty() {
                target -= 1;
            }
            if !table[0][target].trim().is_empty() || target == 0 {
                if !table[0][target].is_empty() {
                    table[0][target].push(' ');
                }
                table[0][target].push_str(trimmed);
                for row in table.iter_mut() {
                    row.remove(col);
                }
                return;
            }
        }

        if col + 1 < table[0].len() {
            if table[0][col + 1].trim().is_empty() {
                table[0][col + 1] = trimmed.to_string();
            } else {
                let mut updated = trimmed.to_string();
                updated.push(' ');
                updated.push_str(table[0][col + 1].trim());
                table[0][col + 1] = updated;
            }
            for row in table.iter_mut() {
                row.remove(col);
            }
            return;
        }
    }

    for row in table.iter_mut() {
        row.remove(col);
    }
}

fn normalize_data_cell(cell: &mut String) {
    let mut text = cell.trim().to_string();
    if text.is_empty() {
        cell.clear();
        return;
    }

    // Normalize dash variants to ASCII hyphen
    for ch in ['\u{2014}', '\u{2013}', '\u{2212}'] {
        text = text.replace(ch, "-");
    }

    if text.starts_with("- ") {
        text = format!("-{}", text[2..].trim_start());
    }

    text = text.replace("- ", "-");
    text = text.replace(" -", "-");
    text = text.replace("E-", "e-").replace("E+", "e+");

    if text == "-" {
        text.clear();
    }

    *cell = text;
}

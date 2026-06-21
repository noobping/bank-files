use super::text::{cell_text_width, wrap_print_cell};
use super::*;

pub(in crate::app) fn print_section_title_height(subtitle: &str, width: f64) -> f64 {
    if subtitle.is_empty() {
        return 30.0;
    }

    let max_chars =
        printable_chars_per_line(print_content_width(width), SECTION_SUBTITLE_FONT_SIZE).max(20);
    let lines = wrap_text_for_print(&tr(subtitle), max_chars).len().max(1);
    30.0 + lines as f64 * SECTION_SUBTITLE_LINE_HEIGHT
}

pub(in crate::app) fn print_table_row_height(
    columns: &[PrintColumn],
    cells: &[PrintCell],
    width: f64,
) -> f64 {
    let table_width = print_content_width(width);
    let total = print_columns_total(columns);
    let line_count = columns
        .iter()
        .zip(cells.iter())
        .map(|(column, cell)| {
            let cell_width = table_width * (column.width / total);
            wrap_print_cell(&cell.text, cell_text_width(cell_width))
                .len()
                .max(1)
        })
        .max()
        .unwrap_or(1);

    (TABLE_ROW_TOP_PADDING + TABLE_ROW_BOTTOM_PADDING + line_count as f64 * TABLE_TEXT_LINE_HEIGHT)
        .max(PRINT_TABLE_ROW_HEIGHT)
}

pub(super) fn print_columns_total(columns: &[PrintColumn]) -> f64 {
    columns
        .iter()
        .map(|column| column.width)
        .sum::<f64>()
        .max(1.0)
}

pub(in crate::app) fn metric_column_count(width: f64) -> usize {
    let content_width = print_content_width(width);
    if content_width >= 560.0 {
        4
    } else if content_width >= 440.0 {
        3
    } else {
        2
    }
}

pub(in crate::app) fn print_content_x() -> f64 {
    PRINT_CONTENT_SIDE_MARGIN
}

pub(in crate::app) fn print_content_width(width: f64) -> f64 {
    (width - PRINT_CONTENT_SIDE_MARGIN * 2.0).max(1.0)
}

pub(in crate::app) fn print_content_top() -> f64 {
    PRINT_CONTENT_TOP
}

pub(in crate::app) fn print_content_bottom(height: f64) -> f64 {
    (height - PRINT_CONTENT_BOTTOM_MARGIN).max(PRINT_CONTENT_TOP)
}

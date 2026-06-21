use super::*;

mod draw;
mod layout;
mod text;

const PRINT_CONTENT_SIDE_MARGIN: f64 = 12.0;
const PRINT_CONTENT_TOP: f64 = 66.0;
const PRINT_CONTENT_BOTTOM_MARGIN: f64 = 24.0;
pub(in crate::app) const PRINT_TABLE_ROW_HEIGHT: f64 = 21.0;

const TABLE_CELL_PADDING: f64 = 4.0;
const TABLE_TEXT_BASELINE_OFFSET: f64 = 13.8;
const TABLE_TEXT_FONT_SIZE: f64 = 8.0;
const TABLE_TEXT_LINE_HEIGHT: f64 = 9.8;
const TABLE_ROW_TOP_PADDING: f64 = 6.0;
const TABLE_ROW_BOTTOM_PADDING: f64 = 5.0;
const SECTION_SUBTITLE_FONT_SIZE: f64 = 9.0;
const SECTION_SUBTITLE_LINE_HEIGHT: f64 = 12.0;

pub(in crate::app) use draw::{
    draw_print_paragraph, draw_print_section_title, draw_print_table_header, draw_print_table_row,
};
pub(in crate::app) use layout::{
    metric_column_count, print_content_bottom, print_content_top, print_content_width,
    print_content_x, print_section_title_height, print_table_row_height,
};
pub(in crate::app) use text::{
    draw_print_text_fit, printable_chars_per_line, wrap_text_for_print, PrintTextBounds,
    PrintTextStyle,
};

#[cfg(test)]
mod tests;

use super::*;

mod page;
mod table;

pub(in crate::app) use page::{draw_print_report_page, print_element_height};
pub(in crate::app) use table::{
    draw_print_paragraph, draw_print_section_title, draw_print_table_header, draw_print_table_row,
    draw_print_text_fit, metric_column_count, print_content_bottom, print_content_top,
    print_content_width, print_content_x, print_section_title_height, print_table_row_height,
    printable_chars_per_line, PrintTextBounds, PrintTextStyle, PRINT_TABLE_ROW_HEIGHT,
};

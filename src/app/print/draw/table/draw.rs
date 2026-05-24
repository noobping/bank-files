use super::layout::print_columns_total;
use super::text::{cell_text_width, set_print_font, set_print_tone, wrap_print_cell};
use super::*;

pub(in crate::app) fn draw_print_section_title(
    cr: &gtk::cairo::Context,
    title: &str,
    subtitle: &str,
    width: f64,
    y: &mut f64,
) {
    let x = print_content_x();
    let content_width = print_content_width(width);

    cr.select_font_face(
        "Sans",
        gtk::cairo::FontSlant::Normal,
        gtk::cairo::FontWeight::Bold,
    );
    cr.set_font_size(12.0);
    set_print_tone(cr, PrintTone::Normal);
    cr.move_to(x, *y + 13.0);
    let _ = cr.show_text(&tr(title));
    if !subtitle.is_empty() {
        let max_chars = printable_chars_per_line(content_width, SECTION_SUBTITLE_FONT_SIZE).max(20);
        for (index, line) in wrap_text_for_print(&tr(subtitle), max_chars)
            .iter()
            .enumerate()
        {
            draw_print_text_fit(
                cr,
                line,
                PrintTextBounds::new(
                    x,
                    *y + 29.0 + index as f64 * SECTION_SUBTITLE_LINE_HEIGHT,
                    content_width,
                ),
                PrintTextStyle::new(
                    SECTION_SUBTITLE_FONT_SIZE,
                    gtk::cairo::FontWeight::Normal,
                    PrintTone::Muted,
                    PrintAlign::Left,
                ),
            );
        }
    }
    *y += print_section_title_height(subtitle, width);
}

pub(in crate::app) fn draw_print_paragraph(
    cr: &gtk::cairo::Context,
    body: &str,
    width: f64,
    y: &mut f64,
) {
    let max_chars = printable_chars_per_line(print_content_width(width), 10.0).max(20);
    set_print_font(cr, 10.0, gtk::cairo::FontWeight::Normal);
    set_print_tone(cr, PrintTone::Normal);
    let body = tr(body);
    for line in body
        .lines()
        .flat_map(|line| wrap_text_for_print(line, max_chars))
    {
        cr.move_to(print_content_x(), *y + 12.0);
        let _ = cr.show_text(&line);
        *y += 14.0;
    }
    *y += 10.0;
}

pub(in crate::app) fn draw_print_table_header(
    cr: &gtk::cairo::Context,
    columns: &[PrintColumn],
    width: f64,
    y: &mut f64,
) {
    let x = print_content_x();
    let table_width = print_content_width(width);
    cr.set_source_rgb(0.91, 0.91, 0.91);
    cr.rectangle(x, *y, table_width, PRINT_TABLE_ROW_HEIGHT);
    let _ = cr.fill();

    let mut cell_x = x;
    let total = print_columns_total(columns);
    for column in columns {
        let cell_width = table_width * (column.width / total);
        draw_print_text_fit(
            cr,
            &column.title,
            PrintTextBounds::new(
                cell_x + TABLE_CELL_PADDING,
                *y + TABLE_TEXT_BASELINE_OFFSET,
                cell_text_width(cell_width),
            ),
            PrintTextStyle::new(
                TABLE_TEXT_FONT_SIZE,
                gtk::cairo::FontWeight::Bold,
                PrintTone::Normal,
                column.align,
            ),
        );
        cell_x += cell_width;
    }
    *y += PRINT_TABLE_ROW_HEIGHT;
}

pub(in crate::app) fn draw_print_table_row(
    cr: &gtk::cairo::Context,
    columns: &[PrintColumn],
    cells: &[PrintCell],
    index: usize,
    width: f64,
    y: &mut f64,
) {
    let x = print_content_x();
    let table_width = print_content_width(width);
    let row_height = print_table_row_height(columns, cells, width);
    if index % 2 == 1 {
        cr.set_source_rgb(0.985, 0.985, 0.985);
        cr.rectangle(x, *y, table_width, row_height);
        let _ = cr.fill();
    }
    cr.set_source_rgb(0.88, 0.88, 0.88);
    cr.move_to(x, *y + row_height);
    cr.line_to(x + table_width, *y + row_height);
    let _ = cr.stroke();

    let mut cell_x = x;
    let total = print_columns_total(columns);
    for (column, cell) in columns.iter().zip(cells.iter()) {
        let cell_width = table_width * (column.width / total);
        let text_width = cell_text_width(cell_width);
        for (line_index, line) in wrap_print_cell(&cell.text, text_width).iter().enumerate() {
            draw_print_text_fit(
                cr,
                line,
                PrintTextBounds::new(
                    cell_x + TABLE_CELL_PADDING,
                    *y + TABLE_ROW_TOP_PADDING
                        + TABLE_TEXT_FONT_SIZE
                        + line_index as f64 * TABLE_TEXT_LINE_HEIGHT,
                    text_width,
                ),
                PrintTextStyle::new(
                    TABLE_TEXT_FONT_SIZE,
                    gtk::cairo::FontWeight::Normal,
                    cell.tone,
                    column.align,
                ),
            );
        }
        cell_x += cell_width;
    }
    *y += row_height;
}

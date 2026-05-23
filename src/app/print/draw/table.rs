use super::*;

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

#[derive(Clone, Copy)]
pub(in crate::app) struct PrintTextBounds {
    x: f64,
    baseline: f64,
    max_width: f64,
}

impl PrintTextBounds {
    pub(in crate::app) fn new(x: f64, baseline: f64, max_width: f64) -> Self {
        Self {
            x,
            baseline,
            max_width,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::app) struct PrintTextStyle {
    font_size: f64,
    weight: gtk::cairo::FontWeight,
    tone: PrintTone,
    align: PrintAlign,
}

impl PrintTextStyle {
    pub(in crate::app) fn new(
        font_size: f64,
        weight: gtk::cairo::FontWeight,
        tone: PrintTone,
        align: PrintAlign,
    ) -> Self {
        Self {
            font_size,
            weight,
            tone,
            align,
        }
    }
}

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

pub(in crate::app) fn draw_print_text_fit(
    cr: &gtk::cairo::Context,
    text: &str,
    bounds: PrintTextBounds,
    style: PrintTextStyle,
) {
    let max_width = bounds.max_width.max(1.0);
    set_print_font(cr, style.font_size, style.weight);
    set_print_tone(cr, style.tone);
    let text = truncate_for_width(text, max_width, style.font_size);
    let approximate_width = text.chars().count() as f64 * style.font_size * 0.54;
    let draw_x = match style.align {
        PrintAlign::Left => bounds.x,
        PrintAlign::Right => (bounds.x + max_width - approximate_width).max(bounds.x),
    };
    cr.move_to(draw_x, bounds.baseline);
    let _ = cr.show_text(&text);
}

pub(in crate::app) fn set_print_font(
    cr: &gtk::cairo::Context,
    size: f64,
    weight: gtk::cairo::FontWeight,
) {
    cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, weight);
    cr.set_font_size(size);
}

pub(in crate::app) fn set_print_tone(cr: &gtk::cairo::Context, tone: PrintTone) {
    let (r, g, b) = match tone {
        PrintTone::Normal => (0.10, 0.10, 0.10),
        PrintTone::Muted => (0.42, 0.42, 0.42),
        PrintTone::Positive => (0.00, 0.45, 0.22),
        PrintTone::Negative => (0.78, 0.12, 0.10),
        PrintTone::Warning => (0.72, 0.38, 0.00),
    };
    cr.set_source_rgb(r, g, b);
}

pub(in crate::app) fn print_columns_total(columns: &[PrintColumn]) -> f64 {
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

pub(in crate::app) fn truncate_for_width(text: &str, max_width: f64, font_size: f64) -> String {
    truncate(text, printable_chars_per_line(max_width, font_size).max(1))
}

pub(in crate::app) fn printable_chars_per_line(width: f64, font_size: f64) -> usize {
    (width.max(1.0) / (font_size * 0.54)).floor().max(1.0) as usize
}

pub(in crate::app) fn wrap_text_for_print(line: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    if line.trim().is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        for part in split_print_word(word, max_chars) {
            let next_len =
                current.chars().count() + usize::from(!current.is_empty()) + part.chars().count();
            if next_len > max_chars && !current.is_empty() {
                lines.push(current);
                current = String::new();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&part);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn wrap_print_cell(text: &str, max_width: f64) -> Vec<String> {
    let max_chars = printable_chars_per_line(max_width, TABLE_TEXT_FONT_SIZE).max(1);
    wrap_text_for_print(text, max_chars)
}

fn split_print_word(word: &str, max_chars: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for ch in word.chars() {
        if current.chars().count() >= max_chars {
            parts.push(current);
            current = String::new();
        }
        current.push(ch);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn cell_text_width(cell_width: f64) -> f64 {
    (cell_width - TABLE_CELL_PADDING * 2.0).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_wrapping_splits_long_words() {
        assert_eq!(
            wrap_text_for_print("abcdefghij", 4),
            vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
        );
    }

    #[test]
    fn table_row_height_grows_for_wrapped_cells() {
        let columns = vec![PrintColumn {
            title: "Message".to_string(),
            width: 1.0,
            align: PrintAlign::Left,
        }];
        let cells = vec![PrintCell {
            text: "This is a longer message that needs more than one printed line".to_string(),
            tone: PrintTone::Normal,
        }];

        assert!(print_table_row_height(&columns, &cells, 120.0) > PRINT_TABLE_ROW_HEIGHT);
    }
}

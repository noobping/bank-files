use super::*;

const PRINT_CONTENT_SIDE_MARGIN: f64 = 12.0;
const PRINT_CONTENT_TOP: f64 = 66.0;
const PRINT_CONTENT_BOTTOM_MARGIN: f64 = 24.0;
pub(in crate::app) const PRINT_TABLE_ROW_HEIGHT: f64 = 21.0;

const TABLE_CELL_PADDING: f64 = 4.0;
const TABLE_TEXT_BASELINE_OFFSET: f64 = 13.8;

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
        draw_print_text_fit(
            cr,
            &tr(subtitle),
            PrintTextBounds::new(x, *y + 29.0, content_width),
            PrintTextStyle::new(
                9.0,
                gtk::cairo::FontWeight::Normal,
                PrintTone::Muted,
                PrintAlign::Left,
            ),
        );
        *y += 42.0;
    } else {
        *y += 30.0;
    }
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
                8.0,
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
    if index % 2 == 1 {
        cr.set_source_rgb(0.985, 0.985, 0.985);
        cr.rectangle(x, *y, table_width, PRINT_TABLE_ROW_HEIGHT);
        let _ = cr.fill();
    }
    cr.set_source_rgb(0.88, 0.88, 0.88);
    cr.move_to(x, *y + PRINT_TABLE_ROW_HEIGHT);
    cr.line_to(x + table_width, *y + PRINT_TABLE_ROW_HEIGHT);
    let _ = cr.stroke();

    let mut cell_x = x;
    let total = print_columns_total(columns);
    for (column, cell) in columns.iter().zip(cells.iter()) {
        let cell_width = table_width * (column.width / total);
        draw_print_text_fit(
            cr,
            &cell.text,
            PrintTextBounds::new(
                cell_x + TABLE_CELL_PADDING,
                *y + TABLE_TEXT_BASELINE_OFFSET,
                cell_text_width(cell_width),
            ),
            PrintTextStyle::new(8.0, gtk::cairo::FontWeight::Normal, cell.tone, column.align),
        );
        cell_x += cell_width;
    }
    *y += PRINT_TABLE_ROW_HEIGHT;
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
    if line.trim().is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let next_len =
            current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
        if next_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn cell_text_width(cell_width: f64) -> f64 {
    (cell_width - TABLE_CELL_PADDING * 2.0).max(1.0)
}

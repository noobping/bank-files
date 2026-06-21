use super::*;

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

pub(super) fn wrap_print_cell(text: &str, max_width: f64) -> Vec<String> {
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

pub(super) fn cell_text_width(cell_width: f64) -> f64 {
    (cell_width - TABLE_CELL_PADDING * 2.0).max(1.0)
}

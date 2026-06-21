use super::*;

const PRINT_METRIC_CARD_HEIGHT: f64 = 58.0;
const PRINT_METRIC_GAP: f64 = 8.0;

pub(in crate::app) fn print_element_height(element: &PrintElement, width: f64) -> f64 {
    match element {
        PrintElement::Metrics(metrics) => {
            let columns = metric_column_count(width);
            let rows = metrics.len().div_ceil(columns).max(1);
            rows as f64 * PRINT_METRIC_CARD_HEIGHT
                + (rows.saturating_sub(1)) as f64 * PRINT_METRIC_GAP
                + 12.0
        }
        PrintElement::SectionTitle { subtitle, .. } => print_section_title_height(subtitle, width),
        PrintElement::Paragraph { body } => {
            let chars = printable_chars_per_line(print_content_width(width), 10.0).max(20);
            let lines = body
                .lines()
                .map(|line| line.chars().count().div_ceil(chars).max(1))
                .sum::<usize>();
            lines as f64 * 14.0 + 10.0
        }
        PrintElement::TableHeader { .. } => PRINT_TABLE_ROW_HEIGHT,
        PrintElement::TableRow { columns, cells, .. } => {
            print_table_row_height(columns, cells, width)
        }
    }
}

pub(in crate::app) fn draw_print_report_page(
    context: &gtk::PrintContext,
    report: &PrintReport,
    pages: &[PrintPage],
    page_nr: i32,
) {
    let cr = context.cairo_context();
    let width = context.width();
    let height = context.height();
    let page_index = page_nr.max(0) as usize;
    let page_count = pages.len().max(1);

    draw_report_chrome(&cr, report, width, height, page_index, page_count);
    let mut y = print_content_top();
    if let Some(page) = pages.get(page_index) {
        for element in &page.elements {
            match element {
                PrintElement::Metrics(metrics) => draw_print_metrics(&cr, metrics, width, &mut y),
                PrintElement::SectionTitle { title, subtitle } => {
                    draw_print_section_title(&cr, title, subtitle, width, &mut y)
                }
                PrintElement::Paragraph { body } => draw_print_paragraph(&cr, body, width, &mut y),
                PrintElement::TableHeader { columns } => {
                    draw_print_table_header(&cr, columns, width, &mut y)
                }
                PrintElement::TableRow {
                    columns,
                    cells,
                    index,
                } => draw_print_table_row(&cr, columns, cells, *index, width, &mut y),
            }
        }
    }
}

pub(in crate::app) fn draw_report_chrome(
    cr: &gtk::cairo::Context,
    report: &PrintReport,
    width: f64,
    height: f64,
    page_index: usize,
    page_count: usize,
) {
    let content_x = print_content_x();
    let content_width = print_content_width(width);
    let generated_width = content_width.min(170.0);
    let title_width = (content_width - generated_width - 12.0).max(1.0);
    let generated_x = content_x + content_width - generated_width;

    cr.set_source_rgb(1.0, 1.0, 1.0);
    let _ = cr.paint();
    draw_print_text_fit(
        cr,
        &tr(&report.title),
        PrintTextBounds::new(content_x, 32.0, title_width),
        PrintTextStyle::new(
            18.0,
            gtk::cairo::FontWeight::Bold,
            PrintTone::Normal,
            PrintAlign::Left,
        ),
    );

    draw_print_text_fit(
        cr,
        &report.generated,
        PrintTextBounds::new(generated_x, 32.0, generated_width),
        PrintTextStyle::new(
            9.0,
            gtk::cairo::FontWeight::Normal,
            PrintTone::Muted,
            PrintAlign::Right,
        ),
    );
    draw_print_text_fit(
        cr,
        &tr(&report.subtitle),
        PrintTextBounds::new(content_x, 48.0, content_width),
        PrintTextStyle::new(
            9.0,
            gtk::cairo::FontWeight::Normal,
            PrintTone::Muted,
            PrintAlign::Left,
        ),
    );

    cr.set_source_rgb(0.78, 0.78, 0.78);
    cr.set_line_width(0.8);
    cr.move_to(content_x, print_content_top() - 12.0);
    cr.line_to(content_x + content_width, print_content_top() - 12.0);
    let _ = cr.stroke();

    let footer = trf(
        "Page {page} of {pages}",
        &[
            ("page", (page_index + 1).to_string()),
            ("pages", page_count.to_string()),
        ],
    );
    draw_print_text_fit(
        cr,
        &footer,
        PrintTextBounds::new(content_x, height - 8.0, content_width),
        PrintTextStyle::new(
            8.0,
            gtk::cairo::FontWeight::Normal,
            PrintTone::Muted,
            PrintAlign::Right,
        ),
    );
}

pub(in crate::app) fn draw_print_metrics(
    cr: &gtk::cairo::Context,
    metrics: &[PrintMetric],
    width: f64,
    y: &mut f64,
) {
    let columns = metric_column_count(width);
    let content_width = print_content_width(width);
    let card_width = ((content_width - PRINT_METRIC_GAP * (columns.saturating_sub(1)) as f64)
        / columns as f64)
        .max(1.0);
    for (index, metric) in metrics.iter().enumerate() {
        let col = index % columns;
        let row = index / columns;
        let x = print_content_x() + col as f64 * (card_width + PRINT_METRIC_GAP);
        let top = *y + row as f64 * (PRINT_METRIC_CARD_HEIGHT + PRINT_METRIC_GAP);
        cr.set_source_rgb(0.96, 0.96, 0.96);
        cr.rectangle(x, top, card_width, PRINT_METRIC_CARD_HEIGHT);
        let _ = cr.fill();
        cr.set_source_rgb(0.82, 0.82, 0.82);
        cr.rectangle(x, top, card_width, PRINT_METRIC_CARD_HEIGHT);
        let _ = cr.stroke();
        draw_print_text_fit(
            cr,
            &metric.label,
            PrintTextBounds::new(x + 9.0, top + 16.0, card_width - 18.0),
            PrintTextStyle::new(
                8.0,
                gtk::cairo::FontWeight::Normal,
                PrintTone::Muted,
                PrintAlign::Left,
            ),
        );
        draw_print_text_fit(
            cr,
            &metric.value,
            PrintTextBounds::new(x + 9.0, top + 35.0, card_width - 18.0),
            PrintTextStyle::new(
                15.0,
                gtk::cairo::FontWeight::Bold,
                metric.tone,
                PrintAlign::Left,
            ),
        );
        draw_print_text_fit(
            cr,
            &metric.detail,
            PrintTextBounds::new(x + 9.0, top + 50.0, card_width - 18.0),
            PrintTextStyle::new(
                8.0,
                gtk::cairo::FontWeight::Normal,
                PrintTone::Muted,
                PrintAlign::Left,
            ),
        );
    }
    let rows = metrics.len().div_ceil(columns).max(1);
    *y += rows as f64 * PRINT_METRIC_CARD_HEIGHT
        + (rows.saturating_sub(1)) as f64 * PRINT_METRIC_GAP
        + 12.0;
}

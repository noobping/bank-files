use super::*;

#[derive(Debug, Clone)]
struct CashFlowHit {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    tooltip: String,
    hover_key: String,
    kind: CashFlowSegmentKind,
    label: String,
    budget_code: String,
}

impl CashFlowHit {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

struct CashFlowRow<'a> {
    label: &'static str,
    current_total: Decimal,
    current_segments: &'a [CashFlowSegment],
    previous_total: Option<Decimal>,
}

#[derive(Debug, Clone, Copy)]
struct CashFlowBarArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl CashFlowBarArea {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CashFlowBarScale {
    max_total: f64,
    palette: ChartPalette,
}

struct CashFlowBarInteraction<'a> {
    hovered_key: &'a Option<String>,
    hits: &'a mut Vec<CashFlowHit>,
}

pub fn year_cash_flow_chart<F>(breakdown: &CashFlowBreakdown, on_segment: F) -> gtk::DrawingArea
where
    F: Fn(CashFlowSegmentKind, String, String) + 'static,
{
    let area = gtk::DrawingArea::new();
    area.set_content_width(320);
    area.set_content_height(330);
    area.set_hexpand(true);
    area.set_vexpand(false);
    area.set_valign(gtk::Align::Start);
    area.add_css_class("card");

    let breakdown = breakdown.clone();
    let hits = Rc::new(RefCell::new(Vec::<CashFlowHit>::new()));
    let hovered = Rc::new(RefCell::new(None::<String>));
    let hits_for_draw = Rc::clone(&hits);
    let hovered_for_draw = Rc::clone(&hovered);
    area.set_draw_func(move |widget, cr, width, height| {
        let palette = chart_palette(widget);
        let width = width as f64;
        let height = height as f64;
        let narrow = width < 430.0;
        let margin_left = (if narrow { 104.0_f64 } else { 132.0_f64 }).min(width * 0.36);
        let margin_right = if narrow { 46.0 } else { 64.0 };
        let margin_top = 44.0;
        let axis_bottom = 34.0;
        let plot_width = (width - margin_left - margin_right).max(1.0);
        let available_rows = (height - margin_top - axis_bottom - 16.0).max(160.0);
        let row_gap = (available_rows / 4.0).clamp(42.0, 56.0);
        let bar_height = if narrow { 18.0 } else { 22.0 };
        let previous_offset = 6.0;

        cr.select_font_face(
            "Sans",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        cr.set_font_size(11.0);

        let mut hits = hits_for_draw.borrow_mut();
        hits.clear();

        let max_total = max_cash_flow_total(&breakdown).max(Decimal::ONE);
        let max_total_f = decimal_to_f64(max_total).max(1.0);
        let hovered_key = hovered_for_draw.borrow().clone();
        let bar_scale = CashFlowBarScale {
            max_total: max_total_f,
            palette,
        };

        set_rgb(cr, palette.dim);
        cr.move_to(margin_left, 24.0);
        let _ = cr.show_text(&format!(
            "{} {}",
            gettext("Scale"),
            compact_money_label(max_total_f)
        ));

        for (index, row) in cash_flow_rows(&breakdown).into_iter().enumerate() {
            let y = margin_top + index as f64 * row_gap;
            if let Some(previous_total) = row.previous_total {
                draw_previous_bar(
                    cr,
                    previous_total,
                    CashFlowBarArea::new(
                        margin_left,
                        y - previous_offset,
                        plot_width,
                        bar_height + previous_offset * 2.0,
                    ),
                    bar_scale,
                );
            }
            draw_current_bar(
                cr,
                &row,
                CashFlowBarArea::new(margin_left, y, plot_width, bar_height),
                bar_scale,
                CashFlowBarInteraction {
                    hovered_key: &hovered_key,
                    hits: &mut hits,
                },
            );
            draw_axis_label(
                cr,
                14.0,
                y + bar_height / 2.0 + 4.0,
                row.label,
                if narrow { 14 } else { 22 },
                palette,
            );
        }

        let axis_y = height - 34.0;
        set_rgb(cr, palette.grid);
        cr.set_line_width(1.0);
        cr.move_to(margin_left, axis_y);
        cr.line_to(margin_left + plot_width, axis_y);
        let _ = cr.stroke();
        set_rgb(cr, palette.dim);
        cr.move_to(margin_left, height - 16.0);
        let _ = cr.show_text("0");
        cr.move_to(
            (margin_left + plot_width - 56.0).max(margin_left + 8.0),
            height - 16.0,
        );
        let _ = cr.show_text(&compact_money_label(max_total_f));
    });

    let motion = gtk::EventControllerMotion::new();
    let area_for_motion = area.clone();
    let hits_for_motion = Rc::clone(&hits);
    let hovered_for_motion = Rc::clone(&hovered);
    motion.connect_motion(move |_, x, y| {
        let hit = hits_for_motion
            .borrow()
            .iter()
            .find(|hit| hit.contains(x, y))
            .cloned();
        area_for_motion.set_tooltip_text(hit.as_ref().map(|hit| hit.tooltip.as_str()));
        let next_key = hit.map(|hit| hit.hover_key);
        if *hovered_for_motion.borrow() != next_key {
            *hovered_for_motion.borrow_mut() = next_key;
            area_for_motion.queue_draw();
        }
    });
    let area_for_leave = area.clone();
    let hovered_for_leave = Rc::clone(&hovered);
    motion.connect_leave(move |_| {
        area_for_leave.set_tooltip_text(None);
        if hovered_for_leave.borrow().is_some() {
            *hovered_for_leave.borrow_mut() = None;
            area_for_leave.queue_draw();
        }
    });
    area.add_controller(motion);

    let click = gtk::GestureClick::new();
    click.set_button(0);
    let hits_for_click = Rc::clone(&hits);
    click.connect_released(move |_, _, x, y| {
        if let Some(hit) = hits_for_click
            .borrow()
            .iter()
            .find(|hit| hit.contains(x, y))
            .cloned()
        {
            on_segment(hit.kind, hit.label, hit.budget_code);
        }
    });
    area.add_controller(click);

    area
}

fn cash_flow_rows(breakdown: &CashFlowBreakdown) -> Vec<CashFlowRow<'_>> {
    vec![
        CashFlowRow {
            label: "Planned income",
            current_total: cash_flow_segment_total(&breakdown.current.planned_income),
            current_segments: &breakdown.current.planned_income,
            previous_total: breakdown
                .previous
                .as_ref()
                .map(|period| cash_flow_segment_total(&period.planned_income)),
        },
        CashFlowRow {
            label: "Real income",
            current_total: breakdown.current.totals.income,
            current_segments: &breakdown.current.actual_income,
            previous_total: breakdown
                .previous
                .as_ref()
                .map(|period| period.totals.income),
        },
        CashFlowRow {
            label: "Planned expenses",
            current_total: cash_flow_segment_total(&breakdown.current.planned_expenses),
            current_segments: &breakdown.current.planned_expenses,
            previous_total: breakdown
                .previous
                .as_ref()
                .map(|period| cash_flow_segment_total(&period.planned_expenses)),
        },
        CashFlowRow {
            label: "Real expenses",
            current_total: breakdown.current.totals.expenses,
            current_segments: &breakdown.current.actual_expenses,
            previous_total: breakdown
                .previous
                .as_ref()
                .map(|period| period.totals.expenses),
        },
    ]
}

fn draw_axis_label(
    cr: &gtk::cairo::Context,
    x: f64,
    y: f64,
    label: &str,
    max_chars: usize,
    palette: ChartPalette,
) {
    set_rgb(cr, palette.fg);
    cr.move_to(x, y);
    let translated = gettext(label);
    let _ = cr.show_text(&truncate_chart_label(&translated, max_chars));
}

fn draw_previous_bar(
    cr: &gtk::cairo::Context,
    total: Decimal,
    area: CashFlowBarArea,
    scale: CashFlowBarScale,
) {
    let total_width = area.width * (decimal_to_f64(total) / scale.max_total).clamp(0.0, 1.0);
    if total_width <= 0.0 {
        return;
    }
    set_rgb(cr, scale.palette.grid_strong);
    rounded_rect(cr, area.x, area.y, total_width, area.height, 6.0);
    let _ = cr.fill();
}

fn draw_current_bar(
    cr: &gtk::cairo::Context,
    row: &CashFlowRow<'_>,
    area: CashFlowBarArea,
    scale: CashFlowBarScale,
    interaction: CashFlowBarInteraction<'_>,
) {
    set_rgb(cr, scale.palette.grid);
    rounded_rect(cr, area.x, area.y, area.width * 0.004, area.height, 4.0);
    let _ = cr.fill();

    let mut cursor = area.x;
    for (index, segment) in row.current_segments.iter().enumerate() {
        let segment_width =
            area.width * (decimal_to_f64(segment.amount) / scale.max_total).clamp(0.0, 1.0);
        if segment_width <= 0.0 {
            continue;
        }
        let hover_key = segment_hover_key(segment);
        let color = slice_color(scale.palette, index);
        if interaction.hovered_key.as_deref() == Some(hover_key.as_str()) {
            set_rgb(cr, scale.palette.grid);
            rounded_rect(
                cr,
                cursor - 2.0,
                area.y - 3.0,
                segment_width + 4.0,
                area.height + 6.0,
                7.0,
            );
            let _ = cr.fill();
        }
        set_rgb(cr, color);
        rounded_rect(cr, cursor, area.y, segment_width.max(2.0), area.height, 5.0);
        let _ = cr.fill();

        interaction.hits.push(CashFlowHit {
            x: cursor,
            y: area.y,
            width: segment_width.max(2.0),
            height: area.height,
            tooltip: segment_tooltip(segment),
            hover_key,
            kind: segment.kind,
            label: segment.label.clone(),
            budget_code: segment.budget_code.clone(),
        });
        cursor += segment_width;
    }

    set_rgb(cr, scale.palette.fg);
    cr.move_to(
        (area.x + cursor - area.x + 8.0).min(area.x + area.width - 56.0),
        area.y + area.height / 2.0 + 4.0,
    );
    let _ = cr.show_text(&compact_money_label(decimal_to_f64(row.current_total)));
}

fn segment_tooltip(segment: &CashFlowSegment) -> String {
    trf(
        "{label} · {kind} · {amount}",
        &[
            ("label", segment.label.clone()),
            ("kind", gettext(cash_flow_kind_label(segment.kind))),
            (
                "amount",
                compact_money_label(decimal_to_f64(segment.amount)),
            ),
        ],
    )
}

fn cash_flow_kind_label(kind: CashFlowSegmentKind) -> &'static str {
    match kind {
        CashFlowSegmentKind::PlannedIncome => "planned income",
        CashFlowSegmentKind::ActualIncome => "real income",
        CashFlowSegmentKind::PlannedExpense => "planned expenses",
        CashFlowSegmentKind::ActualExpense => "real expenses",
    }
}

fn segment_hover_key(segment: &CashFlowSegment) -> String {
    format!(
        "{}:{}:{}",
        cash_flow_kind_label(segment.kind),
        segment.budget_code,
        segment.label
    )
}

fn max_cash_flow_total(breakdown: &CashFlowBreakdown) -> Decimal {
    cash_flow_rows(breakdown)
        .into_iter()
        .fold(Decimal::ZERO, |max_total, row| {
            let previous_total = row.previous_total.unwrap_or(Decimal::ZERO);
            max_total.max(row.current_total).max(previous_total)
        })
}

fn cash_flow_segment_total(segments: &[CashFlowSegment]) -> Decimal {
    segments
        .iter()
        .map(|segment| segment.amount)
        .sum::<Decimal>()
}

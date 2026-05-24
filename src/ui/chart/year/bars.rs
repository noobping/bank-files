use super::*;

#[derive(Debug, Clone)]
pub(super) struct CashFlowHit {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) tooltip: String,
    pub(super) hover_key: String,
    pub(super) kind: CashFlowSegmentKind,
    pub(super) label: String,
    pub(super) budget_code: String,
}

impl CashFlowHit {
    pub(super) fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

pub(super) struct CashFlowRow<'a> {
    pub(super) label: &'static str,
    current_total: Decimal,
    current_segments: &'a [CashFlowSegment],
    pub(super) previous_total: Option<Decimal>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CashFlowBarArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl CashFlowBarArea {
    pub(super) fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CashFlowBarScale {
    pub(super) max_total: f64,
    pub(super) palette: ChartPalette,
}

pub(super) struct CashFlowBarInteraction<'a> {
    pub(super) hovered_key: &'a Option<String>,
    pub(super) hits: &'a mut Vec<CashFlowHit>,
}

pub(super) fn cash_flow_rows(breakdown: &CashFlowBreakdown) -> Vec<CashFlowRow<'_>> {
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

pub(super) fn draw_axis_label(
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

pub(super) fn draw_previous_bar(
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

pub(super) fn draw_current_bar(
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

pub(super) fn max_cash_flow_total(breakdown: &CashFlowBreakdown) -> Decimal {
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

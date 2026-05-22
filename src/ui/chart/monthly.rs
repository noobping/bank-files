use super::*;

#[derive(Debug, Clone)]
struct MonthHit {
    month: MonthKey,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    tooltip: String,
}

impl MonthHit {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

pub fn monthly_graph<F>(months: &[MonthSummary], on_month: F) -> gtk::DrawingArea
where
    F: Fn(MonthKey) + 'static,
{
    let area = gtk::DrawingArea::new();
    area.set_content_width(320);
    area.set_content_height(280);
    area.set_hexpand(true);
    area.add_css_class("card");

    let months = months.to_vec();
    let hits = Rc::new(RefCell::new(Vec::<MonthHit>::new()));
    let hovered = Rc::new(Cell::new(None::<MonthKey>));
    let hits_for_draw = Rc::clone(&hits);
    let hovered_for_draw = Rc::clone(&hovered);
    area.set_draw_func(move |widget, cr, width, height| {
        let palette = chart_palette(widget);
        let width = width as f64;
        let height = height as f64;
        let margin_left = 58.0;
        let margin_right = 24.0;
        let margin_top = 30.0;
        let margin_bottom = 44.0;
        let plot_width = (width - margin_left - margin_right).max(1.0);
        let plot_height = (height - margin_top - margin_bottom).max(1.0);
        let baseline = margin_top + plot_height;

        cr.select_font_face(
            "Sans",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        cr.set_font_size(11.0);

        let mut hits = hits_for_draw.borrow_mut();
        hits.clear();

        set_rgb(cr, palette.grid);
        cr.set_line_width(1.0);
        cr.move_to(margin_left, baseline);
        cr.line_to(width - margin_right, baseline);
        let _ = cr.stroke();

        if months.is_empty() {
            set_rgb(cr, palette.fg);
            cr.move_to(margin_left, height / 2.0);
            let _ = cr.show_text(&gettext("No transactions to draw yet."));
            return;
        }

        let has_negative_balance = months
            .iter()
            .any(|month| month.totals.balance < Decimal::ZERO);
        let zero_y = if has_negative_balance {
            margin_top + plot_height * 0.64
        } else {
            baseline
        };
        let positive_space = (zero_y - margin_top).max(1.0);
        let negative_space = (baseline - zero_y).max(1.0);
        let max_positive = months
            .iter()
            .flat_map(|month| {
                [
                    decimal_to_f64(month.totals.income),
                    decimal_to_f64(month.totals.expenses),
                    decimal_to_f64(month.totals.balance.max(Decimal::ZERO)),
                ]
            })
            .fold(1.0_f64, f64::max);
        let max_negative = months
            .iter()
            .map(|month| decimal_to_f64((-month.totals.balance).max(Decimal::ZERO)))
            .fold(1.0_f64, f64::max);

        draw_horizontal_guide(
            cr,
            margin_left,
            width - margin_right,
            margin_top,
            palette,
            &compact_money_label(max_positive),
        );
        if has_negative_balance {
            draw_horizontal_guide(
                cr,
                margin_left,
                width - margin_right,
                baseline,
                palette,
                &format!("-{}", compact_money_label(max_negative)),
            );
        }

        set_rgb(cr, palette.grid_strong);
        cr.set_line_width(1.0);
        cr.move_to(margin_left, zero_y);
        cr.line_to(width - margin_right, zero_y);
        let _ = cr.stroke();
        set_rgb(cr, palette.dim);
        cr.move_to(12.0, zero_y + 4.0);
        let _ = cr.show_text("0");

        let group_width = plot_width / months.len() as f64;
        let bar_gap = if months.len() > 18 { 2.0 } else { 3.0 };
        let bar_width = (group_width / 5.5).clamp(3.0, 22.0);
        let hovered_month = hovered_for_draw.get();
        let mut balance_points = Vec::new();

        for (index, month) in months.iter().enumerate() {
            let group_start = margin_left + index as f64 * group_width;
            let x = group_start + group_width * 0.20;

            if hovered_month == Some(month.month) {
                set_rgb(cr, palette.grid);
                rounded_rect(
                    cr,
                    group_start + 2.0,
                    margin_top - 10.0,
                    (group_width - 4.0).max(1.0),
                    (baseline - margin_top + 28.0).max(1.0),
                    8.0,
                );
                let _ = cr.fill();
            }

            if index > 0 && month.month.month == 1 {
                set_rgb(cr, palette.grid);
                cr.set_line_width(1.0);
                cr.move_to(group_start, margin_top);
                cr.line_to(group_start, baseline);
                let _ = cr.stroke();
            }

            let income_height = positive_space * decimal_to_f64(month.totals.income) / max_positive;
            let expense_height =
                positive_space * decimal_to_f64(month.totals.expenses) / max_positive;

            set_rgb(cr, palette.positive);
            cr.rectangle(x, zero_y - income_height, bar_width, income_height);
            let _ = cr.fill();

            set_rgb(cr, palette.negative);
            cr.rectangle(
                x + bar_width + bar_gap,
                zero_y - expense_height,
                bar_width,
                expense_height,
            );
            let _ = cr.fill();

            let balance_x = x + bar_width + 1.5;
            let balance_y = if month.totals.balance >= Decimal::ZERO {
                zero_y - positive_space * decimal_to_f64(month.totals.balance) / max_positive
            } else {
                zero_y + negative_space * decimal_to_f64(-month.totals.balance) / max_negative
            };
            balance_points.push((balance_x, balance_y, month.totals.balance < Decimal::ZERO));

            if let Some(label) = month_axis_label(index, months.len(), month) {
                set_rgb(cr, palette.dim);
                cr.move_to(
                    (group_start + 2.0).min(width - margin_right - 28.0),
                    height - 14.0,
                );
                let _ = cr.show_text(&label);
            }

            hits.push(MonthHit {
                month: month.month,
                x: group_start,
                y: margin_top - 10.0,
                width: group_width,
                height: baseline - margin_top + 28.0,
                tooltip: monthly_tooltip(month),
            });
        }

        set_rgb(cr, palette.accent);
        cr.set_line_width(2.0);
        for (index, (x, y, _)) in balance_points.iter().enumerate() {
            if index == 0 {
                cr.move_to(*x, *y);
            } else {
                cr.line_to(*x, *y);
            }
        }
        let _ = cr.stroke();

        for (x, y, is_loss) in balance_points {
            if is_loss {
                set_rgb(cr, palette.negative);
            } else {
                set_rgb(cr, palette.accent);
            }
            cr.arc(x, y, 3.0, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();
        }

        draw_legend(cr, margin_left, margin_top, palette);
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
        let next_month = hit.as_ref().map(|hit| hit.month);
        area_for_motion.set_tooltip_text(hit.as_ref().map(|hit| hit.tooltip.as_str()));
        if hovered_for_motion.get() != next_month {
            hovered_for_motion.set(next_month);
            area_for_motion.queue_draw();
        }
    });
    let area_for_leave = area.clone();
    let hovered_for_leave = Rc::clone(&hovered);
    motion.connect_leave(move |_| {
        area_for_leave.set_tooltip_text(None);
        if hovered_for_leave.get().is_some() {
            hovered_for_leave.set(None);
            area_for_leave.queue_draw();
        }
    });
    area.add_controller(motion);

    let click = gtk::GestureClick::new();
    click.set_button(0);
    let hits_for_click = Rc::clone(&hits);
    click.connect_released(move |_, _, x, y| {
        if let Some(month) = hits_for_click
            .borrow()
            .iter()
            .find(|hit| hit.contains(x, y))
            .map(|hit| hit.month)
        {
            on_month(month);
        }
    });
    area.add_controller(click);

    area
}

fn monthly_tooltip(month: &MonthSummary) -> String {
    trf(
        "{month} · {count} transactions · {income} income · {expenses} expenses · {balance} balance",
        &[
            ("month", month_label(month.month)),
            ("count", month.totals.count.to_string()),
            ("income", compact_money_label(decimal_to_f64(month.totals.income))),
            ("expenses", compact_money_label(decimal_to_f64(month.totals.expenses))),
            ("balance", signed_chart_money(month.totals.balance)),
        ],
    )
}

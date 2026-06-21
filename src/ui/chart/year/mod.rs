use super::*;

mod bars;

use bars::*;

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

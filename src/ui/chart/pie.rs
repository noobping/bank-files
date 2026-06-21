use super::*;

#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: Decimal,
    pub detail: String,
}

impl PieSlice {
    pub fn new(label: String, value: Decimal, detail: String) -> Self {
        Self {
            label,
            value,
            detail,
        }
    }
}

pub fn sort_pie_slices_largest_first<T: Ord>(items: &mut [(T, PieSlice)]) {
    items.sort_by(|a, b| {
        b.1.value
            .cmp(&a.1.value)
            .then_with(|| a.1.label.cmp(&b.1.label))
            .then_with(|| a.0.cmp(&b.0))
    });
}

#[derive(Debug, Clone)]
struct PieHit {
    index: usize,
    center_x: f64,
    center_y: f64,
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
    tooltip: String,
}

impl PieHit {
    fn contains(&self, x: f64, y: f64) -> bool {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        let radius = (dx * dx + dy * dy).sqrt();
        if radius < self.inner_radius || radius > self.outer_radius + 8.0 {
            return false;
        }

        let mut angle = dy.atan2(dx);
        while angle < self.start_angle {
            angle += std::f64::consts::TAU;
        }
        angle >= self.start_angle && angle <= self.end_angle
    }
}

pub fn pie_chart_with_capacity<F>(
    title: &str,
    subtitle: &str,
    slices: &[PieSlice],
    center_label: &str,
    capacity: Option<Decimal>,
    on_slice: F,
) -> gtk::Box
where
    F: Fn(usize) + 'static,
{
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);
    card.set_width_request(320);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let title_label = gtk::Label::new(Some(&gettext(title)));
    title_label.add_css_class("title-4");
    title_label.set_xalign(0.0);
    title_label.set_wrap(true);
    title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    content.append(&title_label);

    let subtitle_label = gtk::Label::new(Some(&gettext(subtitle)));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.set_xalign(0.0);
    subtitle_label.set_wrap(true);
    subtitle_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    content.append(&subtitle_label);

    let area = gtk::DrawingArea::new();
    area.set_content_width(320);
    area.set_content_height(190);
    area.set_hexpand(true);
    area.set_can_target(true);
    content.append(&area);
    card.append(&content);

    let slices = slices.to_vec();
    let draw_slices = slices.clone();
    let center_label = center_label.to_string();
    let capacity_for_draw = capacity;
    let hits = Rc::new(RefCell::new(Vec::<PieHit>::new()));
    let hovered = Rc::new(Cell::new(None::<usize>));
    let hits_for_draw = Rc::clone(&hits);
    let hovered_for_draw = Rc::clone(&hovered);

    area.set_draw_func(move |widget, cr, width, height| {
        let palette = chart_palette(widget);
        let width = width as f64;
        let height = height as f64;
        let total = draw_slices
            .iter()
            .map(|slice| slice.value.max(Decimal::ZERO))
            .sum::<Decimal>();

        cr.select_font_face(
            "Sans",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        cr.set_font_size(11.0);

        let mut hits = hits_for_draw.borrow_mut();
        hits.clear();

        if draw_slices.is_empty() || total <= Decimal::ZERO {
            set_rgb(cr, palette.dim);
            cr.move_to(12.0, height / 2.0);
            let _ = cr.show_text(&gettext("No slices to draw yet."));
            return;
        }

        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let outer_radius = ((width.min(height) - 32.0) / 2.0).clamp(54.0, 82.0);
        let inner_radius = outer_radius * 0.58;
        let hovered = hovered_for_draw.get();
        let mut start_angle = -std::f64::consts::FRAC_PI_2;
        let denominator = pie_denominator(total, capacity_for_draw);
        let total_f64 = decimal_to_f64(denominator).max(1.0);

        set_rgb(cr, palette.grid);
        cr.new_path();
        cr.arc(
            center_x,
            center_y,
            outer_radius,
            -std::f64::consts::FRAC_PI_2,
            -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU,
        );
        cr.arc_negative(
            center_x,
            center_y,
            inner_radius,
            -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU,
            -std::f64::consts::FRAC_PI_2,
        );
        cr.close_path();
        let _ = cr.fill();

        for (index, slice) in draw_slices.iter().enumerate() {
            let value = slice.value.max(Decimal::ZERO);
            if value <= Decimal::ZERO {
                continue;
            }
            let fraction = decimal_to_f64(value) / total_f64;
            let end_angle = start_angle + std::f64::consts::TAU * fraction;
            let radius = if hovered == Some(index) {
                outer_radius + 6.0
            } else {
                outer_radius
            };

            set_rgb(cr, slice_color(palette, index));
            cr.new_path();
            cr.arc(center_x, center_y, radius, start_angle, end_angle);
            cr.arc_negative(center_x, center_y, inner_radius, end_angle, start_angle);
            cr.close_path();
            let _ = cr.fill();

            set_rgb(cr, palette.bg);
            cr.set_line_width(1.0);
            cr.arc(center_x, center_y, radius, start_angle, end_angle);
            let _ = cr.stroke();

            hits.push(PieHit {
                index,
                center_x,
                center_y,
                inner_radius,
                outer_radius: radius,
                start_angle,
                end_angle,
                tooltip: pie_tooltip(slice, value, fraction),
            });
            start_angle = end_angle;
        }

        set_rgb(cr, palette.bg);
        cr.arc(
            center_x,
            center_y,
            inner_radius - 1.0,
            0.0,
            std::f64::consts::TAU,
        );
        let _ = cr.fill();

        set_rgb(cr, palette.fg);
        cr.set_font_size(15.0);
        cr.move_to(
            center_x - (center_label.chars().count() as f64 * 4.2),
            center_y + 5.0,
        );
        let _ = cr.show_text(&center_label);
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
        let next_index = hit.as_ref().map(|hit| hit.index);
        area_for_motion.set_tooltip_text(hit.as_ref().map(|hit| hit.tooltip.as_str()));
        if hovered_for_motion.get() != next_index {
            hovered_for_motion.set(next_index);
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
        if let Some(index) = hits_for_click
            .borrow()
            .iter()
            .find(|hit| hit.contains(x, y))
            .map(|hit| hit.index)
        {
            on_slice(index);
        }
    });
    area.add_controller(click);

    card
}

fn pie_denominator(total: Decimal, capacity: Option<Decimal>) -> Decimal {
    let denominator = match capacity {
        Some(capacity) if capacity > total => capacity,
        _ => total,
    };
    denominator.max(Decimal::ONE)
}

fn pie_tooltip(slice: &PieSlice, value: Decimal, fraction: f64) -> String {
    trf(
        "{label} · {value} · {percent}% · {detail}",
        &[
            ("label", slice.label.clone()),
            ("value", compact_money_label(decimal_to_f64(value))),
            ("percent", format!("{:.1}", fraction * 100.0)),
            ("detail", slice.detail.clone()),
        ],
    )
}

#[cfg(test)]
#[path = "pie_tests.rs"]
mod tests;

use super::*;

pub(super) fn draw_horizontal_guide(
    cr: &gtk::cairo::Context,
    x1: f64,
    x2: f64,
    y: f64,
    palette: ChartPalette,
    label: &str,
) {
    set_rgb(cr, palette.grid);
    cr.set_line_width(1.0);
    cr.move_to(x1, y);
    cr.line_to(x2, y);
    let _ = cr.stroke();
    set_rgb(cr, palette.dim);
    cr.move_to(12.0, y + 4.0);
    let _ = cr.show_text(label);
}

pub(super) fn month_axis_label(index: usize, total: usize, month: &MonthSummary) -> Option<String> {
    if total <= 6 {
        return Some(format!("{:02}", month.month.month));
    }
    if total <= 12 {
        return (index == 0 || index == total - 1 || month.month.month % 2 == 1)
            .then(|| format!("{:02}", month.month.month));
    }
    if month.month.month == 1 {
        return Some(month.month.year.to_string());
    }
    if index == total - 1 || matches!(month.month.month, 4 | 7 | 10) {
        return Some(format!("{:02}", month.month.month));
    }
    None
}

pub(super) fn draw_legend(cr: &gtk::cairo::Context, x: f64, y: f64, palette: ChartPalette) {
    let items = [
        (palette.positive, "Income"),
        (palette.negative, "Expenses"),
        (palette.accent, "Monthly balance"),
    ];
    let mut cursor = x;
    for (color, label) in items {
        set_rgb(cr, color);
        cr.rectangle(cursor, y - 10.0, 10.0, 10.0);
        let _ = cr.fill();
        set_rgb(cr, palette.fg);
        cr.move_to(cursor + 14.0, y);
        let translated = gettext(label);
        let _ = cr.show_text(&translated);
        cursor += if label == "Monthly balance" {
            116.0
        } else {
            92.0
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ChartPalette {
    pub(super) bg: Rgb,
    pub(super) fg: Rgb,
    pub(super) dim: Rgb,
    pub(super) grid: Rgb,
    pub(super) grid_strong: Rgb,
    pub(super) positive: Rgb,
    pub(super) negative: Rgb,
    pub(super) accent: Rgb,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

pub(super) fn chart_palette(widget: &gtk::DrawingArea) -> ChartPalette {
    let context = widget.style_context();
    let fg = context
        .lookup_color("theme_fg_color")
        .map(rgb_from_rgba)
        .unwrap_or(Rgb {
            r: 0.92,
            g: 0.92,
            b: 0.90,
        });
    let bg = context
        .lookup_color("theme_bg_color")
        .map(rgb_from_rgba)
        .unwrap_or(Rgb {
            r: 0.15,
            g: 0.15,
            b: 0.16,
        });

    ChartPalette {
        bg,
        fg,
        dim: mix_rgb(bg, fg, 0.64),
        grid: mix_rgb(bg, fg, 0.22),
        grid_strong: mix_rgb(bg, fg, 0.38),
        positive: Rgb {
            r: 0.22,
            g: 0.63,
            b: 0.36,
        },
        negative: Rgb {
            r: 0.86,
            g: 0.30,
            b: 0.25,
        },
        accent: Rgb {
            r: 0.31,
            g: 0.55,
            b: 0.93,
        },
    }
}

fn rgb_from_rgba(color: gtk::gdk::RGBA) -> Rgb {
    Rgb {
        r: color.red() as f64,
        g: color.green() as f64,
        b: color.blue() as f64,
    }
}

fn mix_rgb(a: Rgb, b: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    Rgb {
        r: a.r + (b.r - a.r) * amount,
        g: a.g + (b.g - a.g) * amount,
        b: a.b + (b.b - a.b) * amount,
    }
}

pub(super) fn set_rgb(cr: &gtk::cairo::Context, color: Rgb) {
    cr.set_source_rgb(color.r, color.g, color.b);
}

pub(super) fn rounded_rect(
    cr: &gtk::cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let radius = radius.min(width / 2.0).min(height / 2.0);
    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    cr.close_path();
}

pub(super) fn signed_chart_money(value: Decimal) -> String {
    if value >= Decimal::ZERO {
        format!("+€ {}", value.round_dp(0))
    } else {
        format!("-€ {}", (-value).round_dp(0))
    }
}

pub(super) fn compact_money_label(value: f64) -> String {
    if value >= 1000.0 {
        format!("€ {:.1}k", value / 1000.0)
    } else {
        format!("€ {:.0}", value)
    }
}

pub(super) fn truncate_chart_label(label: &str, max_chars: usize) -> String {
    let mut chars = label.chars();
    let mut truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        truncated.push('…');
    }
    truncated
}

pub(super) fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

pub(super) fn slice_color(palette: ChartPalette, index: usize) -> Rgb {
    match index % 12 {
        0 => palette.accent,
        1 => palette.positive,
        2 => Rgb {
            r: 0.94,
            g: 0.62,
            b: 0.23,
        },
        3 => Rgb {
            r: 0.78,
            g: 0.45,
            b: 0.86,
        },
        4 => Rgb {
            r: 0.20,
            g: 0.70,
            b: 0.72,
        },
        5 => Rgb {
            r: 0.90,
            g: 0.48,
            b: 0.38,
        },
        6 => Rgb {
            r: 0.58,
            g: 0.64,
            b: 0.22,
        },
        7 => Rgb {
            r: 0.42,
            g: 0.56,
            b: 0.92,
        },
        8 => Rgb {
            r: 0.72,
            g: 0.52,
            b: 0.30,
        },
        9 => Rgb {
            r: 0.84,
            g: 0.38,
            b: 0.58,
        },
        10 => Rgb {
            r: 0.30,
            g: 0.66,
            b: 0.42,
        },
        _ => palette.negative,
    }
}

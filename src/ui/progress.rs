use super::*;

pub fn progress_row(title: &str, subtitle: &str, fraction: f64, detail: &str) -> gtk::Box {
    progress_row_with_state(title, subtitle, fraction, detail, ProgressState::Normal)
}

pub fn progress_row_with_state(
    title: &str,
    subtitle: &str,
    fraction: f64,
    detail: &str,
    state: ProgressState,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
    row.add_css_class("card");
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_margin_start(4);
    row.set_margin_end(4);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    if matches!(state, ProgressState::Error) {
        let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
        icon.add_css_class("error");
        icon.set_valign(gtk::Align::Start);
        heading.append(&icon);
    }
    let title_label = wrapped_label(title);
    title_label.set_hexpand(true);
    title_label.set_width_chars(1);
    title_label.set_max_width_chars(24);
    let detail_label = gtk::Label::new(Some(detail));
    detail_label.add_css_class("caption");
    if matches!(state, ProgressState::Error) {
        detail_label.add_css_class("error");
    }
    detail_label.set_xalign(1.0);
    detail_label.set_wrap(true);
    detail_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    detail_label.set_width_chars(1);
    detail_label.set_max_width_chars(18);
    heading.append(&title_label);
    heading.append(&detail_label);

    let subtitle_label = wrapped_label(subtitle);
    subtitle_label.add_css_class("dim-label");

    let progress = gtk::ProgressBar::new();
    progress.set_fraction(fraction.clamp(0.0, 1.0));
    if matches!(state, ProgressState::Error) {
        progress.add_css_class("error");
    }

    content.append(&heading);
    content.append(&subtitle_label);
    content.append(&progress);
    row.append(&content);
    row
}

pub fn activatable_progress_row_with_state<F>(
    title: &str,
    subtitle: &str,
    fraction: f64,
    detail: &str,
    state: ProgressState,
    on_activate: F,
) -> gtk::Box
where
    F: Fn() + 'static,
{
    activatable_card(
        progress_row_with_state(title, subtitle, fraction, detail, state),
        on_activate,
    )
}

#[derive(Debug, Clone)]
pub struct ComparisonMeasure {
    pub label: String,
    pub fraction: f64,
    pub state: ProgressState,
}

#[derive(Debug, Clone)]
pub struct ComparisonProgressRow {
    pub title: String,
    pub subtitle: String,
    pub current: ComparisonMeasure,
    pub previous: Option<ComparisonMeasure>,
    pub detail: String,
}

pub fn comparison_progress_row(config: ComparisonProgressRow) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
    row.add_css_class("card");
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_margin_start(4);
    row.set_margin_end(4);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    if matches!(config.current.state, ProgressState::Error) {
        let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
        icon.add_css_class("error");
        icon.set_valign(gtk::Align::Start);
        heading.append(&icon);
    }
    let title_label = wrapped_label(&config.title);
    title_label.set_hexpand(true);
    title_label.set_width_chars(1);
    title_label.set_max_width_chars(24);
    let detail_label = gtk::Label::new(Some(&config.detail));
    detail_label.add_css_class("caption");
    if matches!(config.current.state, ProgressState::Error) {
        detail_label.add_css_class("error");
    }
    detail_label.set_xalign(1.0);
    detail_label.set_wrap(true);
    detail_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    detail_label.set_width_chars(1);
    detail_label.set_max_width_chars(18);
    heading.append(&title_label);
    heading.append(&detail_label);

    let subtitle_label = wrapped_label(&config.subtitle);
    subtitle_label.add_css_class("dim-label");

    content.append(&heading);
    content.append(&subtitle_label);
    content.append(&labeled_overlay_progress_bar(
        config.current,
        config.previous,
    ));
    row.append(&content);
    row
}

pub fn activatable_comparison_progress_row<F>(
    config: ComparisonProgressRow,
    on_activate: F,
) -> gtk::Box
where
    F: Fn() + 'static,
{
    activatable_card(comparison_progress_row(config), on_activate)
}

fn labeled_overlay_progress_bar(
    current: ComparisonMeasure,
    previous: Option<ComparisonMeasure>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(&current.label));
    label.add_css_class("caption");
    if matches!(current.state, ProgressState::Error) {
        label.add_css_class("error");
    }
    label.set_width_chars(1);
    label.set_max_width_chars(18);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);

    let overlay = gtk::Overlay::new();
    overlay.set_hexpand(true);
    if let Some(previous) = previous {
        let previous_bar = gtk::ProgressBar::new();
        previous_bar.set_fraction(previous.fraction.clamp(0.0, 1.0));
        previous_bar.set_opacity(0.42);
        previous_bar.set_hexpand(true);
        previous_bar.set_tooltip_text(Some(&previous.label));
        if matches!(previous.state, ProgressState::Error) {
            previous_bar.add_css_class("error");
        }
        overlay.set_child(Some(&previous_bar));
    } else {
        let spacer = gtk::ProgressBar::new();
        spacer.set_fraction(0.0);
        spacer.set_opacity(0.0);
        spacer.set_hexpand(true);
        overlay.set_child(Some(&spacer));
    }

    let current_bar = gtk::ProgressBar::new();
    current_bar.set_fraction(current.fraction.clamp(0.0, 1.0));
    current_bar.set_hexpand(true);
    if matches!(current.state, ProgressState::Error) {
        current_bar.add_css_class("error");
    }
    overlay.add_overlay(&current_bar);

    row.append(&label);
    row.append(&overlay);
    row
}

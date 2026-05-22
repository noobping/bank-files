use super::*;

pub fn progress_row_with_action(
    title: &str,
    subtitle: &str,
    fraction: f64,
    detail: &str,
    action: Option<gtk::Widget>,
) -> gtk::Box {
    progress_row_with_state_and_action(
        title,
        subtitle,
        fraction,
        detail,
        ProgressState::Normal,
        action,
    )
}

pub fn progress_row_with_state_and_action(
    title: &str,
    subtitle: &str,
    fraction: f64,
    detail: &str,
    state: ProgressState,
    action: Option<gtk::Widget>,
) -> gtk::Box {
    let row = card_container();
    let content = card_content(gtk::Orientation::Vertical, 6);

    content.append(&progress_heading(title, state, action));

    let subtitle_label = progress_subtitle(subtitle, detail);

    let progress = gtk::ProgressBar::new();
    progress.set_fraction(fraction.clamp(0.0, 1.0));
    if matches!(state, ProgressState::Error) {
        progress.add_css_class("error");
    }

    content.append(&subtitle_label);
    content.append(&progress);
    row.append(&content);
    row
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

pub fn comparison_progress_row_with_action(
    config: ComparisonProgressRow,
    action: Option<gtk::Widget>,
) -> gtk::Box {
    let row = card_container();
    let content = card_content(gtk::Orientation::Vertical, 8);

    content.append(&progress_heading(
        &config.title,
        config.current.state,
        action,
    ));

    let subtitle_label = progress_subtitle_with_parts([
        config.subtitle.as_str(),
        config.current.label.as_str(),
        config.detail.as_str(),
    ]);

    content.append(&subtitle_label);
    content.append(&comparison_progress_bar(config.current, config.previous));
    row.append(&content);
    row
}

fn progress_heading(title: &str, state: ProgressState, action: Option<gtk::Widget>) -> gtk::Box {
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    heading.set_hexpand(true);

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
    heading.append(&title_label);

    if let Some(action) = action {
        action.set_valign(gtk::Align::Center);
        heading.append(&action);
    }

    heading
}

fn progress_subtitle(subtitle: &str, detail: &str) -> gtk::Label {
    progress_subtitle_with_parts([subtitle, detail])
}

fn progress_subtitle_with_parts<'a>(parts: impl IntoIterator<Item = &'a str>) -> gtk::Label {
    let text = parts
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let label = wrapped_label(&text);
    label.add_css_class("dim-label");
    label.set_vexpand(true);
    label.set_valign(gtk::Align::Start);
    label
}

fn comparison_progress_bar(
    current: ComparisonMeasure,
    previous: Option<ComparisonMeasure>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

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

    row.append(&overlay);
    row
}

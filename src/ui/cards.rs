use super::*;

pub fn card_container() -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);
    card
}

pub fn card_content(orientation: gtk::Orientation, spacing: i32) -> gtk::Box {
    let content = gtk::Box::new(orientation, spacing);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content
}

pub fn metric_card(title: &str, value: &str, subtitle: &str) -> gtk::Box {
    let card = card_container();
    let content = card_content(gtk::Orientation::Vertical, 4);

    let title_label = gtk::Label::new(Some(&gettext(title)));
    title_label.add_css_class("caption");
    title_label.set_xalign(0.0);
    title_label.set_width_chars(1);
    title_label.set_max_width_chars(28);
    title_label.set_wrap(true);
    title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);

    let value_label = gtk::Label::new(Some(value));
    value_label.add_css_class("title-2");
    value_label.set_xalign(0.0);
    value_label.set_width_chars(1);
    value_label.set_max_width_chars(18);
    value_label.set_wrap(true);
    value_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);

    let subtitle_label = wrapped_label(&gettext(subtitle));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.set_width_chars(1);
    subtitle_label.set_max_width_chars(32);

    content.append(&title_label);
    content.append(&value_label);
    content.append(&subtitle_label);
    card.append(&content);
    card
}

pub fn activatable_card<F>(card: gtk::Box, on_activate: F) -> gtk::Box
where
    F: Fn() + 'static,
{
    card.add_css_class("action-card");
    card.set_can_target(true);

    let card_widget = card.clone().upcast::<gtk::Widget>();
    let click = gtk::GestureClick::new();
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    click.connect_pressed(move |_, _, x, y| {
        if picked_widget_is_button(&card_widget, x, y) {
            return;
        }
        on_activate();
    });
    card.add_controller(click);

    card
}

fn picked_widget_is_button(widget: &gtk::Widget, x: f64, y: f64) -> bool {
    widget
        .pick(x, y, gtk::PickFlags::DEFAULT)
        .is_some_and(|picked| widget_or_ancestor_is_button(&picked))
}

fn widget_or_ancestor_is_button(widget: &gtk::Widget) -> bool {
    let mut current = Some(widget.clone());
    while let Some(widget) = current {
        if widget.is::<gtk::Button>() {
            return true;
        }
        current = widget.parent();
    }
    false
}

pub fn activatable_metric_card<F>(
    title: &str,
    value: &str,
    subtitle: &str,
    on_activate: F,
) -> gtk::Box
where
    F: Fn() + 'static,
{
    activatable_card(metric_card(title, value, subtitle), on_activate)
}

pub fn metric_grid(cards: Vec<gtk::Box>, max_children_per_line: u32) -> gtk::FlowBox {
    card_grid(cards, max_children_per_line)
}

pub fn card_grid(cards: Vec<gtk::Box>, max_children_per_line: u32) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .column_spacing(8)
        .row_spacing(8)
        .homogeneous(true)
        .selection_mode(gtk::SelectionMode::None)
        .min_children_per_line(1)
        .max_children_per_line(max_children_per_line)
        .hexpand(true)
        .build();

    for card in cards {
        append_card_to_grid(&flow, card);
    }

    flow
}

pub fn append_card_to_grid(flow: &gtk::FlowBox, card: gtk::Box) {
    let is_action_card = card.has_css_class("action-card");
    if !is_action_card {
        card.set_can_target(false);
        card.set_focusable(false);
    }
    let child = gtk::FlowBoxChild::builder()
        .child(&card)
        .can_target(is_action_card)
        .focusable(false)
        .build();
    flow.insert(&child, -1);
}

pub fn clear_card_grid(flow: &gtk::FlowBox) {
    while let Some(child) = flow.first_child() {
        flow.remove(&child);
    }
}

pub fn section_title(title: &str, subtitle: &str) -> gtk::Box {
    section_title_with_width(title, subtitle, 40, 64)
}

fn section_title_with_width(
    title: &str,
    subtitle: &str,
    title_max_width_chars: i32,
    subtitle_max_width_chars: i32,
) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 3);
    let title_label = gtk::Label::new(Some(&gettext(title)));
    title_label.add_css_class("title-3");
    title_label.set_xalign(0.0);
    title_label.set_width_chars(1);
    title_label.set_max_width_chars(title_max_width_chars);
    title_label.set_wrap(true);
    title_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    let subtitle_label = wrapped_label(&gettext(subtitle));
    subtitle_label.add_css_class("dim-label");
    subtitle_label.set_width_chars(1);
    subtitle_label.set_max_width_chars(subtitle_max_width_chars);
    box_.append(&title_label);
    box_.append(&subtitle_label);
    box_
}

pub fn section_title_with_action(
    title: &str,
    subtitle: &str,
    action: &impl IsA<gtk::Widget>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_hexpand(true);

    let title_box = section_title(title, subtitle);
    title_box.set_hexpand(true);

    action.set_valign(gtk::Align::Start);
    row.append(&title_box);
    row.append(action);
    row
}

pub fn section_group(title: &str, subtitle: &str) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 8);
    section.set_hexpand(true);
    section.set_width_request(320);
    section.append(&section_title_with_width(title, subtitle, 32, 44));
    section
}

pub fn loading_section_group(title: &str, subtitle: &str) -> gtk::Box {
    let section = section_group(title, subtitle);
    section.append(&loading_status_card("Loading data..."));
    section
}

fn loading_status_card(message: &str) -> gtk::Box {
    let card = card_container();
    let content = card_content(gtk::Orientation::Horizontal, 10);
    content.set_valign(gtk::Align::Center);

    let spinner = loading_spinner();
    spinner.set_size_request(20, 20);
    content.append(&spinner);

    let label = wrapped_label(&gettext(message));
    label.add_css_class("dim-label");
    label.set_hexpand(true);
    content.append(&label);

    card.append(&content);
    card
}

pub fn responsive_columns(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    responsive_columns_with_limits(sections, 1, 3)
}

pub fn responsive_columns_three_or_one(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    let max_columns = if sections.len() == 3 { 3 } else { 2 };
    responsive_columns_with_limits(sections, 1, max_columns)
}

pub fn responsive_chart_columns(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    responsive_columns_with_limits(sections, 1, 2)
}

fn responsive_columns_with_limits(
    sections: Vec<gtk::Box>,
    min_children_per_line: u32,
    max_children_per_line: u32,
) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .column_spacing(12)
        .row_spacing(12)
        .homogeneous(false)
        .selection_mode(gtk::SelectionMode::None)
        .min_children_per_line(min_children_per_line)
        .max_children_per_line(max_children_per_line)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .build();

    for section in sections {
        section.set_hexpand(true);
        section.set_halign(gtk::Align::Fill);
        section.set_valign(gtk::Align::Start);
        section.set_width_request(320);
        section.set_focusable(false);
        let child = gtk::FlowBoxChild::builder()
            .child(&section)
            .focusable(false)
            .hexpand(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .build();
        child.set_width_request(320);
        flow.insert(&child, -1);
    }

    flow
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressState {
    Normal,
    Error,
}

pub fn text_card(text: &str) -> gtk::Box {
    let card = card_container();

    let label = selectable_wrapped_label(text);
    label.set_margin_top(12);
    label.set_margin_bottom(12);
    label.set_margin_start(12);
    label.set_margin_end(12);
    card.append(&label);
    card
}

pub fn warning_card(title: &str, text: &str) -> gtk::Box {
    let card = card_container();
    card.add_css_class("warning-card");

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);

    let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
    icon.add_css_class("warning");
    icon.set_pixel_size(28);
    icon.set_valign(gtk::Align::Start);
    content.append(&icon);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title = gtk::Label::new(Some(&gettext(title)));
    title.add_css_class("title-4");
    title.add_css_class("warning-title");
    title.set_xalign(0.0);
    title.set_wrap(true);
    title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text_box.append(&title);

    let label = selectable_wrapped_label(text);
    label.set_width_chars(1);
    label.set_max_width_chars(78);
    text_box.append(&label);

    content.append(&text_box);
    card.append(&content);
    card
}

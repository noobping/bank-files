use super::*;

const ACTION_DIALOG_MAX_HEIGHT: i32 = 620;

pub fn month_label(month: MonthKey) -> String {
    format!("{} {}", month_name(month.month), month.year)
}

pub fn month_name(month: u32) -> String {
    let name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => return month.to_string(),
    };
    gettext(name)
}

pub fn loading_spinner() -> adw::Spinner {
    adw::Spinner::new()
}

pub fn action_dialog_header() -> adw::HeaderBar {
    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(true);
    header
}

pub fn cancelable_dialog_header(title: &str, subtitle: &str) -> adw::HeaderBar {
    let header = action_dialog_header();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        &gettext(title),
        &gettext(subtitle),
    )));
    header
}

pub fn dialog_toolbar_view(
    header: &impl IsA<gtk::Widget>,
    content: &impl IsA<gtk::Widget>,
) -> adw::ToolbarView {
    let view = adw::ToolbarView::new();
    view.add_top_bar(header);
    view.set_content(Some(content));
    view
}

pub fn wrapped_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label
}

pub fn selectable_wrapped_label(text: &str) -> gtk::Label {
    let label = wrapped_label(text);
    label.set_selectable(true);
    label
}

pub fn action_dialog_scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    action_dialog_scroll_with_min(child, 0)
}

pub fn action_dialog_scroll_with_min(
    child: &impl IsA<gtk::Widget>,
    min_content_height: i32,
) -> gtk::ScrolledWindow {
    action_dialog_scroll_with_limits(child, min_content_height, ACTION_DIALOG_MAX_HEIGHT)
}

pub fn action_dialog_scroll_with_limits(
    child: &impl IsA<gtk::Widget>,
    min_content_height: i32,
    max_content_height: i32,
) -> gtk::ScrolledWindow {
    let min_content_height = min_content_height.max(0);
    let max_content_height = max_content_height.max(min_content_height).max(1);
    let clamp = adw::Clamp::builder()
        .maximum_size(1080)
        .tightening_threshold(640)
        .child(child)
        .build();
    gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(false)
        .min_content_height(min_content_height)
        .max_content_height(max_content_height)
        .propagate_natural_height(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build()
}

pub fn linked_button_group() -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.add_css_class("linked");
    group
}

pub fn text_list_row(text: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(text).build();
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_title_lines(3);
    row
}

pub struct MoreListRow {
    pub container: adw::PreferencesGroup,
    pub row: adw::ActionRow,
}

pub fn more_list_row(title: &str, tooltip: &str) -> MoreListRow {
    let container = adw::PreferencesGroup::new();
    container.set_margin_top(4);
    container.set_margin_bottom(18);
    container.set_margin_start(4);
    container.set_margin_end(4);

    let row = adw::ActionRow::builder().title(gettext(title)).build();
    row.set_tooltip_text(Some(&gettext(tooltip)));
    row.set_activatable(true);
    row.add_prefix(&gtk::Image::from_icon_name("view-more-symbolic"));
    container.add(&row);

    MoreListRow { container, row }
}

pub fn page_box() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 18);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(18);
    page.set_margin_end(18);
    page
}

#[derive(Clone)]
struct ScrollPosition {
    adjustment: gtk::Adjustment,
    value: f64,
}

pub fn preserve_descendant_scroll_positions(anchor: &impl IsA<gtk::Widget>) {
    let root = top_widget(anchor.as_ref());
    let positions = descendant_scroll_positions(&root);
    if positions.is_empty() {
        return;
    }

    let second_pass_positions = positions.clone();
    gtk::glib::idle_add_local_once(move || {
        restore_scroll_positions(&positions);
        gtk::glib::idle_add_local_once(move || restore_scroll_positions(&second_pass_positions));
    });
}

fn top_widget(widget: &gtk::Widget) -> gtk::Widget {
    let mut current = widget.clone();
    while let Some(parent) = current.parent() {
        current = parent;
    }
    current
}

fn descendant_scroll_positions(root: &gtk::Widget) -> Vec<ScrollPosition> {
    let mut positions = Vec::new();
    collect_scroll_positions(root, &mut positions);
    positions
}

fn collect_scroll_positions(widget: &gtk::Widget, positions: &mut Vec<ScrollPosition>) {
    if let Some(scrolled_window) = widget.downcast_ref::<gtk::ScrolledWindow>() {
        let adjustment = scrolled_window.vadjustment();
        positions.push(ScrollPosition {
            value: adjustment.value(),
            adjustment,
        });
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        child = current.next_sibling();
        collect_scroll_positions(&current, positions);
    }
}

fn restore_scroll_positions(positions: &[ScrollPosition]) {
    for position in positions {
        position
            .adjustment
            .set_value(clamped_scroll_value(&position.adjustment, position.value));
    }
}

fn clamped_scroll_value(adjustment: &gtk::Adjustment, value: f64) -> f64 {
    let lower = adjustment.lower();
    let upper = (adjustment.upper() - adjustment.page_size()).max(lower);
    value.clamp(lower, upper)
}

pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn clear_list_box(container: &gtk::ListBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

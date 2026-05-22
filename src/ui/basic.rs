use super::*;

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

pub fn icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon_name)
        .tooltip_text(gettext(tooltip))
        .build()
}

pub fn overlay_icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let icon = gtk::Image::from_icon_name(icon_name);
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&icon));

    let button = gtk::Button::builder()
        .tooltip_text(gettext(tooltip))
        .build();
    button.set_child(Some(&overlay));
    button
}

pub fn set_button_icon(button: &gtk::Button, icon_name: &str) {
    if let Some(image) = button
        .child()
        .and_then(|child| child.downcast::<gtk::Overlay>().ok())
        .and_then(|overlay| overlay.child())
        .and_then(|child| child.downcast::<gtk::Image>().ok())
    {
        image.set_icon_name(Some(icon_name));
    } else {
        button.set_icon_name(icon_name);
    }
}

pub fn flat_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let button = plain_text_icon_button(icon_name, label, tooltip);
    button.add_css_class("flat");
    button
}

pub fn primary_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let button = plain_text_icon_button(icon_name, label, tooltip);
    button.add_css_class("suggested-action");
    button
}

pub fn plain_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    content.append(&gtk::Image::from_icon_name(icon_name));
    let label = gtk::Label::new(Some(&gettext(label)));
    label.set_width_chars(1);
    label.set_max_width_chars(22);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    content.append(&label);

    let button = gtk::Button::builder().tooltip_text(tooltip).build();
    button.set_tooltip_text(Some(&gettext(tooltip)));
    button.set_child(Some(&content));
    button
}

pub fn loading_spinner() -> adw::Spinner {
    adw::Spinner::new()
}

pub fn action_dialog_header() -> adw::HeaderBar {
    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
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

pub fn scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    let clamp = adw::Clamp::builder()
        .maximum_size(1080)
        .tightening_threshold(640)
        .child(child)
        .build();
    gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build()
}

pub fn compact_popover_scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(child)
        .max_content_height(360)
        .propagate_natural_height(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build()
}

pub fn linked_button_group() -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.add_css_class("linked");
    group
}

pub fn page_box() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 18);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(18);
    page.set_margin_end(18);
    page
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

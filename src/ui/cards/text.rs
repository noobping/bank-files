use super::*;

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
    title.add_css_class("warning");
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

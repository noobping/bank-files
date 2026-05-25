use super::*;

pub struct BadgeIconButton {
    pub button: gtk::Button,
    pub badge: gtk::Label,
}

pub fn icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon_name)
        .tooltip_text(gettext(tooltip))
        .build()
}

pub fn set_button_icon(button: &gtk::Button, icon_name: &str) {
    button.set_icon_name(icon_name);
}

pub fn flat_custom_button(tooltip: &str, child: &impl IsA<gtk::Widget>) -> gtk::Button {
    let button = gtk::Button::builder()
        .tooltip_text(gettext(tooltip))
        .child(child)
        .build();
    button.add_css_class("flat");
    button
}

pub fn flat_badge_icon_button(icon_name: &str, tooltip: &str) -> BadgeIconButton {
    let badge = gtk::Label::new(None);
    badge.add_css_class("caption");
    badge.set_visible(false);
    badge.set_halign(gtk::Align::Center);
    badge.set_valign(gtk::Align::Center);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    content.append(&badge);
    content.append(&gtk::Image::from_icon_name(icon_name));

    let button = flat_custom_button(tooltip, &content);
    button.set_focus_on_click(false);

    BadgeIconButton { button, badge }
}

pub fn primary_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let button = plain_text_icon_button(icon_name, label, tooltip);
    button.add_css_class("suggested-action");
    button
}

pub fn plain_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let content = adw::ButtonContent::builder()
        .icon_name(icon_name)
        .label(gettext(label))
        .build();

    gtk::Button::builder()
        .tooltip_text(gettext(tooltip))
        .child(&content)
        .build()
}

use super::*;

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

pub fn card_list_section_group(title: &str, subtitle: &str) -> gtk::Box {
    let section = section_group(title, subtitle);
    section.add_css_class("card-list-section");
    section
}

pub fn is_card_list_section(section: &gtk::Box) -> bool {
    section.has_css_class("card-list-section")
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

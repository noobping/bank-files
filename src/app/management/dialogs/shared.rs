use super::*;

const MANAGEMENT_FORM_DIALOG_WIDTH: i32 = 620;

pub(super) fn new_record_dialog(
    title: &str,
    subtitle: &str,
    add_label: &str,
) -> (adw::Dialog, gtk::Box, gtk::Button, gtk::Label) {
    let popup = build_action_form_dialog(
        title,
        subtitle,
        add_label,
        "list-add-symbolic",
        "Add to changes",
        "Search",
        MANAGEMENT_FORM_DIALOG_WIDTH,
    );

    (popup.dialog, popup.page, popup.submit_button, popup.status)
}

pub(super) fn scroll_to_bottom(scrolled_window: &gtk::ScrolledWindow) {
    let scrolled_window = scrolled_window.clone();
    adw::glib::idle_add_local_once(move || {
        let adjustment = scrolled_window.vadjustment();
        let bottom = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
        adjustment.set_value(bottom);
    });
}

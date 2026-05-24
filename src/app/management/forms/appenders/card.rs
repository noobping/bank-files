use super::super::*;

const EDIT_DETAILS_ICON: &str = "document-edit-symbolic";
const COLLAPSE_DETAILS_ICON: &str = "go-up-symbolic";

pub(super) struct CollapsibleFormCard {
    pub(super) form_box: gtk::Box,
    pub(super) drag_handle: gtk::Button,
    pub(super) title: gtk::Label,
    pub(super) subtitle: gtk::Label,
    pub(super) details: gtk::Box,
    pub(super) revert_button: gtk::Button,
    pub(super) delete_button: gtk::Button,
}

pub(super) fn collapsible_form_card(
    title: &str,
    subtitle: &str,
    delete_tooltip: &str,
) -> CollapsibleFormCard {
    let form_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    form_box.add_css_class("card");
    form_box.set_margin_top(4);
    form_box.set_margin_bottom(4);
    form_box.set_margin_start(4);
    form_box.set_margin_end(4);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_margin_top(12);
    header.set_margin_bottom(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

    let drag_handle = ui::icon_button("list-drag-handle-symbolic", "Move item");
    drag_handle.add_css_class("flat");
    drag_handle.add_css_class("drag-handle");
    drag_handle.set_visible(false);
    drag_handle.set_focus_on_click(false);
    drag_handle.set_focusable(false);

    let summary = gtk::Box::new(gtk::Orientation::Vertical, 2);
    summary.set_hexpand(true);
    let title_label = ui::wrapped_label(title);
    title_label.add_css_class("title-4");
    let subtitle_label = ui::wrapped_label(subtitle);
    subtitle_label.add_css_class("dim-label");
    summary.append(&title_label);
    summary.append(&subtitle_label);

    let edit_button = ui::icon_button(EDIT_DETAILS_ICON, "Edit details");
    edit_button.add_css_class("flat");
    let revert_button = ui::icon_button("document-revert-symbolic", "Revert details");
    revert_button.add_css_class("flat");
    revert_button.set_sensitive(false);
    let delete_button = ui::icon_button("user-trash-symbolic", delete_tooltip);
    delete_button.add_css_class("destructive-action");

    let actions = ui::linked_button_group();
    actions.append(&edit_button);
    actions.append(&revert_button);
    actions.append(&delete_button);

    header.append(&drag_handle);
    header.append(&summary);
    header.append(&actions);
    form_box.append(&header);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 0);
    details.set_visible(false);
    details.set_sensitive(false);
    form_box.append(&details);

    let details_for_edit = details.clone();
    edit_button.connect_clicked(move |button| {
        let expanded = !details_for_edit.is_visible();
        details_for_edit.set_visible(expanded);
        details_for_edit.set_sensitive(expanded);
        button.set_icon_name(if expanded {
            COLLAPSE_DETAILS_ICON
        } else {
            EDIT_DETAILS_ICON
        });
        button.set_tooltip_text(Some(&tr(if expanded {
            "Collapse details"
        } else {
            "Edit details"
        })));
    });

    CollapsibleFormCard {
        form_box,
        drag_handle,
        title: title_label,
        subtitle: subtitle_label,
        details,
        revert_button,
        delete_button,
    }
}

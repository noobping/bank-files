use super::super::*;

pub(super) struct CollapsibleFormCard {
    pub(super) form_box: gtk::Box,
    pub(super) row: adw::ExpanderRow,
    pub(super) drag_handle: gtk::Button,
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

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_margin_top(4);
    list.set_margin_bottom(4);
    list.set_margin_start(4);
    list.set_margin_end(4);

    let row = adw::ExpanderRow::builder()
        .title(tr(title))
        .subtitle(tr(subtitle))
        .enable_expansion(true)
        .expanded(false)
        .build();

    let drag_handle = ui::icon_button("list-drag-handle-symbolic", "Move item");
    drag_handle.add_css_class("flat");
    drag_handle.add_css_class("drag-handle");
    drag_handle.set_visible(false);
    drag_handle.set_focus_on_click(false);
    drag_handle.set_focusable(false);
    row.add_prefix(&drag_handle);

    let revert_button = ui::icon_button("document-revert-symbolic", "Revert details");
    revert_button.add_css_class("flat");
    revert_button.set_sensitive(false);
    let delete_button = ui::icon_button("user-trash-symbolic", delete_tooltip);
    delete_button.add_css_class("destructive-action");

    row.add_suffix(&revert_button);
    row.add_suffix(&delete_button);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 0);
    row.add_row(&details);
    list.append(&row);
    form_box.append(&list);

    CollapsibleFormCard {
        form_box,
        row,
        drag_handle,
        details,
        revert_button,
        delete_button,
    }
}

pub(super) fn enable_budget_card_reorder(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    card: &CollapsibleFormCard,
    advanced_features: bool,
) {
    card.drag_handle
        .set_tooltip_text(Some(&tr(if advanced_features {
            "Move budget"
        } else {
            "Move category"
        })));
    card.drag_handle.set_visible(true);
    connect_budget_form_reorder(container, forms, &card.drag_handle, &card.form_box);
}

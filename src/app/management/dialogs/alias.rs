use super::shared::{new_record_dialog, scroll_to_bottom};
use super::*;

pub(in crate::app) fn show_new_alias_dialog(
    parent: &adw::Dialog,
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<AliasForm>>>,
    scrolled_window: &gtk::ScrolledWindow,
    status: &gtk::Label,
    filter_entry: &gtk::SearchEntry,
) {
    let alias = EditableAlias::new_default();
    let (dialog, page, add_button, dialog_status) = new_record_dialog(
        "New Field Name",
        "Map a bank column to a fixed field. It is only saved when you press Save.",
        "Add",
    );

    let grid = form_grid();
    let canonical = field_alias_combo(&alias.canonical);
    let alias_entry = entry("", "Column name from bank CSV");
    add_labeled(&grid, 0, "Fixed field", &canonical);
    add_labeled(&grid, 1, "Bank column", &alias_entry);
    page.append(&grid);
    page.append(&dialog_status);
    dialog.set_focus(Some(&alias_entry));

    let container_for_add = container.clone();
    let forms_for_add = Rc::clone(forms);
    let scrolled_window_for_add = scrolled_window.clone();
    let status_for_add = status.clone();
    let dialog_for_add = dialog.clone();
    let filter_entry_for_add = filter_entry.clone();
    add_button.connect_clicked(move |_| {
        let alias_text = alias_entry.text().trim().to_string();
        if alias_text.is_empty() {
            dialog_status.set_text(&tr("Enter the bank column first."));
            alias_entry.grab_focus();
            return;
        }

        let alias = EditableAlias {
            canonical: combo_active_id(&canonical),
            alias: alias_text,
        };
        append_alias_form(&container_for_add, &forms_for_add, alias);
        filter_alias_forms(&filter_entry_for_add.text(), &forms_for_add.borrow());
        status_for_add.set_text(&tr("New field name added. Press Save to keep it."));
        scroll_to_bottom(&scrolled_window_for_add);
        dialog_for_add.close();
    });

    dialog.present(Some(parent));
}

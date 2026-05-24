use super::*;

pub(super) fn connect_alias_and_search_actions(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog;
    let add_alias_button = actions.add_alias_button;
    let filter_entry = actions.filter_entry;
    let filter_search_bar = actions.filter_search_bar;
    let rules_forms = actions.rules_forms;
    let budgets_forms = actions.budgets_forms;
    let aliases_list = actions.aliases_list;
    let aliases_forms = actions.aliases_forms;
    let aliases_scroll = actions.aliases_scroll;
    let status = actions.status;

    let management_dialog_for_alias_add = management_dialog.clone();
    let aliases_list_for_alias_add = aliases_list.clone();
    let aliases_forms_for_alias_add = Rc::clone(aliases_forms);
    let aliases_scroll_for_alias_add = aliases_scroll.clone();
    let status_for_alias_add = status.clone();
    let filter_entry_for_alias_add = filter_entry.clone();
    add_alias_button.connect_clicked(move |_| {
        show_new_alias_dialog(
            &management_dialog_for_alias_add,
            &aliases_list_for_alias_add,
            &aliases_forms_for_alias_add,
            &aliases_scroll_for_alias_add,
            &status_for_alias_add,
            &filter_entry_for_alias_add,
        );
    });

    let rules_forms_for_filter = Rc::clone(rules_forms);
    let budgets_forms_for_filter = Rc::clone(budgets_forms);
    let aliases_forms_for_filter = Rc::clone(aliases_forms);
    let status_for_filter = status.clone();
    filter_entry.connect_search_changed(move |entry| {
        apply_management_filter(
            &entry.text(),
            &rules_forms_for_filter.borrow(),
            &budgets_forms_for_filter.borrow(),
            &aliases_forms_for_filter.borrow(),
            &status_for_filter,
        );
    });

    let filter_search_bar_for_stop = filter_search_bar.clone();
    filter_entry.connect_stop_search(move |entry| {
        entry.set_text("");
        filter_search_bar_for_stop.set_search_mode(false);
    });

    let filter_entry_for_close = filter_entry.clone();
    filter_search_bar.connect_search_mode_enabled_notify(move |search_bar| {
        if !search_bar.is_search_mode() && !filter_entry_for_close.text().is_empty() {
            filter_entry_for_close.set_text("");
        }
    });

    ui::connect_search_shortcut(management_dialog, filter_search_bar, filter_entry);
}

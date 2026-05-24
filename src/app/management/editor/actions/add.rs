use super::*;

pub(super) fn connect_add_actions(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog;
    let add_button = actions.add_button;
    let stack = actions.stack;
    let filter_entry = actions.filter_entry;
    let rules_list = actions.rules_list;
    let rules_forms = actions.rules_forms;
    let rules_scroll = actions.rules_scroll;
    let budgets_list = actions.budgets_list;
    let budgets_forms = actions.budgets_forms;
    let budgets_scroll = actions.budgets_scroll;
    let aliases_list = actions.aliases_list;
    let aliases_forms = actions.aliases_forms;
    let aliases_scroll = actions.aliases_scroll;
    let status = actions.status;
    let ui_handles = actions.ui_handles;

    let management_dialog_for_header_add = management_dialog.clone();
    let rules_list_for_header_add = rules_list.clone();
    let rules_forms_for_header_add = Rc::clone(rules_forms);
    let rules_scroll_for_header_add = rules_scroll.clone();
    let budgets_list_for_header_add = budgets_list.clone();
    let budgets_forms_for_header_add = Rc::clone(budgets_forms);
    let budgets_scroll_for_header_add = budgets_scroll.clone();
    let aliases_list_for_header_add = aliases_list.clone();
    let aliases_forms_for_header_add = Rc::clone(aliases_forms);
    let aliases_scroll_for_header_add = aliases_scroll.clone();
    let status_for_header_add = status.clone();
    let stack_for_add = stack.clone();
    let filter_entry_for_header_add = filter_entry.clone();
    let advanced_autofill_for_header_add = Rc::clone(&ui_handles.advanced_autofill);
    let ui_for_header_add = Rc::clone(ui_handles);
    add_button.connect_clicked(
        move |_| match stack_for_add.visible_child_name().as_deref() {
            Some("budgets") => show_new_budget_dialog(NewBudgetDialogRequest {
                parent: &management_dialog_for_header_add,
                container: &budgets_list_for_header_add,
                forms: &budgets_forms_for_header_add,
                scrolled_window: &budgets_scroll_for_header_add,
                status: &status_for_header_add,
                filter_entry: &filter_entry_for_header_add,
                advanced_autofill: &advanced_autofill_for_header_add,
                advanced_features: ui_for_header_add.advanced_features.get(),
            }),
            Some("aliases") => show_new_alias_dialog(
                &management_dialog_for_header_add,
                &aliases_list_for_header_add,
                &aliases_forms_for_header_add,
                &aliases_scroll_for_header_add,
                &status_for_header_add,
                &filter_entry_for_header_add,
            ),
            _ => show_new_rule_dialog(
                &management_dialog_for_header_add,
                &rules_list_for_header_add,
                &rules_forms_for_header_add,
                &rules_scroll_for_header_add,
                &status_for_header_add,
                &filter_entry_for_header_add,
                &advanced_autofill_for_header_add,
            ),
        },
    );
}

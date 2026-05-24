use super::*;

pub(super) fn connect_add_actions(actions: &ManagementDialogActions<'_>) {
    connect_header_add_action(actions);
    connect_rule_add_action(actions);
    connect_budget_add_action(actions);
    connect_alias_add_action(actions);
}

fn connect_header_add_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog.clone();
    let rules_list = actions.rules_list.clone();
    let rules_forms = Rc::clone(actions.rules_forms);
    let rules_scroll = actions.rules_scroll.clone();
    let budgets_list = actions.budgets_list.clone();
    let budgets_forms = Rc::clone(actions.budgets_forms);
    let budgets_scroll = actions.budgets_scroll.clone();
    let aliases_list = actions.aliases_list.clone();
    let aliases_forms = Rc::clone(actions.aliases_forms);
    let aliases_scroll = actions.aliases_scroll.clone();
    let status = actions.status.clone();
    let stack = actions.stack.clone();
    let filter_entry = actions.filter_entry.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let ui_handles = Rc::clone(actions.ui_handles);

    actions
        .add_button
        .connect_clicked(move |_| match stack.visible_child_name().as_deref() {
            Some("budgets") => show_new_budget_dialog(NewBudgetDialogRequest {
                parent: &management_dialog,
                container: &budgets_list,
                forms: &budgets_forms,
                scrolled_window: &budgets_scroll,
                status: &status,
                filter_entry: &filter_entry,
                advanced_autofill: &advanced_autofill,
                advanced_features: ui_handles.advanced_features.get(),
            }),
            Some("aliases") => show_new_alias_dialog(
                &management_dialog,
                &aliases_list,
                &aliases_forms,
                &aliases_scroll,
                &status,
                &filter_entry,
            ),
            _ => show_new_rule_dialog(
                &management_dialog,
                &rules_list,
                &rules_forms,
                &rules_scroll,
                &status,
                &filter_entry,
                &advanced_autofill,
            ),
        });
}

fn connect_rule_add_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog.clone();
    let rules_list = actions.rules_list.clone();
    let rules_forms = Rc::clone(actions.rules_forms);
    let rules_scroll = actions.rules_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);

    actions.add_rule_button.connect_clicked(move |_| {
        show_new_rule_dialog(
            &management_dialog,
            &rules_list,
            &rules_forms,
            &rules_scroll,
            &status,
            &filter_entry,
            &advanced_autofill,
        );
    });
}

fn connect_budget_add_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog.clone();
    let budgets_list = actions.budgets_list.clone();
    let budgets_forms = Rc::clone(actions.budgets_forms);
    let budgets_scroll = actions.budgets_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let ui_handles = Rc::clone(actions.ui_handles);

    actions.add_budget_button.connect_clicked(move |_| {
        show_new_budget_dialog(NewBudgetDialogRequest {
            parent: &management_dialog,
            container: &budgets_list,
            forms: &budgets_forms,
            scrolled_window: &budgets_scroll,
            status: &status,
            filter_entry: &filter_entry,
            advanced_autofill: &advanced_autofill,
            advanced_features: ui_handles.advanced_features.get(),
        });
    });
}

fn connect_alias_add_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog.clone();
    let aliases_list = actions.aliases_list.clone();
    let aliases_forms = Rc::clone(actions.aliases_forms);
    let aliases_scroll = actions.aliases_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();

    actions.add_alias_button.connect_clicked(move |_| {
        show_new_alias_dialog(
            &management_dialog,
            &aliases_list,
            &aliases_forms,
            &aliases_scroll,
            &status,
            &filter_entry,
        );
    });
}

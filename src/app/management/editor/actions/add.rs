use super::*;

pub(super) fn connect_add_actions(actions: &ManagementDialogActions<'_>) {
    connect_header_add_action(actions);
    connect_rule_add_action(actions);
    connect_budget_add_action(actions);
    connect_transfer_budget_add_action(actions);
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
            _ => show_new_rule_dialog(NewRuleDialogRequest {
                parent: &management_dialog,
                container: &rules_list,
                forms: &rules_forms,
                scrolled_window: &rules_scroll,
                status: &status,
                filter_entry: &filter_entry,
                advanced_autofill: &advanced_autofill,
                advanced_features: ui_handles.advanced_features.get(),
            }),
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
    let ui_handles = Rc::clone(actions.ui_handles);

    actions.add_rule_row.connect_activated(move |_| {
        show_new_rule_dialog(NewRuleDialogRequest {
            parent: &management_dialog,
            container: &rules_list,
            forms: &rules_forms,
            scrolled_window: &rules_scroll,
            status: &status,
            filter_entry: &filter_entry,
            advanced_autofill: &advanced_autofill,
            advanced_features: ui_handles.advanced_features.get(),
        });
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

    actions.add_budget_row.connect_activated(move |_| {
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

fn connect_transfer_budget_add_action(actions: &ManagementDialogActions<'_>) {
    let budgets_list = actions.budgets_list.clone();
    let budgets_forms = Rc::clone(actions.budgets_forms);
    let budgets_scroll = actions.budgets_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let ui_handles = Rc::clone(actions.ui_handles);

    actions
        .add_transfer_budget_action
        .connect_activate(move |action, _| {
            if !action.is_enabled() {
                return;
            }
            if transfer_budget_form_exists(&budgets_forms.borrow()) {
                status.set_text(&tr(
                    "TRANSFER budget already exists. Review existing budget, then Save.",
                ));
                return;
            }

            append_budget_form(
                &budgets_list,
                &budgets_forms,
                transfer_budget::editable_budget(String::new()),
                false,
                &advanced_autofill,
                ui_handles.advanced_features.get(),
            );
            filter_budget_forms(&filter_entry.text(), &budgets_forms.borrow());
            status.set_text(&tr("TRANSFER budget added. Press Save to keep it."));
            scroll_budget_forms_to_bottom(&budgets_scroll);
        });
}

fn transfer_budget_form_exists(forms: &[BudgetForm]) -> bool {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .any(|form| transfer_budget::is_budget_code(&ui::combo_text(&form.code)))
}

fn scroll_budget_forms_to_bottom(scrolled_window: &gtk::ScrolledWindow) {
    let scrolled_window = scrolled_window.clone();
    gtk::glib::idle_add_local_once(move || {
        let adjustment = scrolled_window.vadjustment();
        let bottom = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
        adjustment.set_value(bottom);
    });
}

fn connect_alias_add_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog.clone();
    let aliases_list = actions.aliases_list.clone();
    let aliases_forms = Rc::clone(actions.aliases_forms);
    let aliases_scroll = actions.aliases_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();

    actions.add_alias_row.connect_activated(move |_| {
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

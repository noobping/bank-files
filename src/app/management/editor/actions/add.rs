use super::*;

pub(super) fn connect_add_actions(actions: &ManagementDialogActions<'_>) {
    connect_header_add_action(actions);
    connect_rule_add_action(actions);
    connect_budget_add_action(actions);
    connect_special_budget_add_action(
        actions,
        actions.add_planned_income_budget_action,
        SpecialBudgetAdd::PlannedIncome,
    );
    connect_special_budget_add_action(
        actions,
        actions.add_transfer_budget_action,
        SpecialBudgetAdd::Transfer,
    );
    connect_special_budget_add_action(
        actions,
        actions.add_refunding_budget_action,
        SpecialBudgetAdd::Refunding,
    );
    connect_special_budget_add_action(
        actions,
        actions.add_refunded_budget_action,
        SpecialBudgetAdd::Refunded,
    );
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

#[derive(Debug, Clone, Copy)]
enum SpecialBudgetAdd {
    PlannedIncome,
    Transfer,
    Refunding,
    Refunded,
}

impl SpecialBudgetAdd {
    fn kind(self) -> crate::model::BudgetSpecialKind {
        match self {
            Self::PlannedIncome => crate::model::BudgetSpecialKind::PlannedIncome,
            Self::Transfer => crate::model::BudgetSpecialKind::Transfer,
            Self::Refunding => crate::model::BudgetSpecialKind::Refunding,
            Self::Refunded => crate::model::BudgetSpecialKind::Refunded,
        }
    }

    fn editable_budget(self) -> EditableBudget {
        match self {
            Self::PlannedIncome => planned_income::editable_budget(
                tr("Income"),
                "0".to_string(),
                String::new(),
                String::new(),
            ),
            Self::Transfer => transfer_budget::editable_budget(String::new()),
            Self::Refunding => refund_budget::editable_budget(
                refund_budget::RefundBudgetKind::Refunding,
                String::new(),
            ),
            Self::Refunded => refund_budget::editable_budget(
                refund_budget::RefundBudgetKind::Refunded,
                String::new(),
            ),
        }
    }

    fn display_name(self) -> String {
        tr(match self {
            Self::PlannedIncome => "Planned income",
            Self::Transfer => "Transfers",
            Self::Refunding => "Outgoing refunds",
            Self::Refunded => "Incoming refunds",
        })
    }

    fn already_exists_message(self) -> String {
        trf(
            "{name} already exists. Review the existing budget, then Save.",
            &[("name", self.display_name())],
        )
    }

    fn added_message(self) -> String {
        trf(
            "{name} added. Press Save to keep it.",
            &[("name", self.display_name())],
        )
    }
}

fn connect_special_budget_add_action(
    actions: &ManagementDialogActions<'_>,
    special_action: &gtk::gio::SimpleAction,
    special: SpecialBudgetAdd,
) {
    let budgets_list = actions.budgets_list.clone();
    let budgets_forms = Rc::clone(actions.budgets_forms);
    let budgets_scroll = actions.budgets_scroll.clone();
    let status = actions.status.clone();
    let filter_entry = actions.filter_entry.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let ui_handles = Rc::clone(actions.ui_handles);

    special_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        if special_budget_form_exists(&budgets_forms.borrow(), special.kind()) {
            status.set_text(&special.already_exists_message());
            action.set_enabled(false);
            return;
        }

        append_special_budget_form(
            &budgets_list,
            &budgets_forms,
            special,
            &advanced_autofill,
            ui_handles.advanced_features.get(),
        );
        filter_budget_forms(&filter_entry.text(), &budgets_forms.borrow());
        status.set_text(&special.added_message());
        action.set_enabled(false);
        scroll_budget_forms_to_bottom(&budgets_scroll);
    });
}

fn append_special_budget_form(
    budgets_list: &gtk::Box,
    budgets_forms: &Rc<RefCell<Vec<BudgetForm>>>,
    special: SpecialBudgetAdd,
    advanced_autofill: &Rc<Cell<bool>>,
    advanced_features: bool,
) {
    let budget = special.editable_budget();
    if matches!(special, SpecialBudgetAdd::PlannedIncome) {
        append_planned_income_budget_form(budgets_list, budgets_forms, budget, advanced_features);
    } else {
        append_budget_form(
            budgets_list,
            budgets_forms,
            budget,
            false,
            advanced_autofill,
            advanced_features,
        );
    }
}

fn special_budget_form_exists(
    forms: &[BudgetForm],
    special: crate::model::BudgetSpecialKind,
) -> bool {
    forms.iter().filter(|form| !form.deleted.get()).any(|form| {
        crate::model::budget_special_kind_for_config(&form.special, &ui::combo_text(&form.code))
            == special
    })
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

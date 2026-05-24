use super::budget_bulk::{
    set_budget_bulk_status, set_budget_forms_income_basis, set_budget_forms_value_period,
    BudgetValuePeriod,
};
use super::budget_move::show_move_budget_code_dialog;
use super::*;

pub(super) fn connect_budget_actions(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog;
    let add_budget_button = actions.add_budget_button;
    let move_budget_code_button = actions.move_budget_code_button;
    let use_real_income_button = actions.use_real_income_button;
    let use_planned_income_button = actions.use_planned_income_button;
    let use_monthly_values_button = actions.use_monthly_values_button;
    let use_yearly_values_button = actions.use_yearly_values_button;
    let filter_entry = actions.filter_entry;
    let rules_forms = actions.rules_forms;
    let budgets_list = actions.budgets_list;
    let budgets_forms = actions.budgets_forms;
    let budgets_scroll = actions.budgets_scroll;
    let status = actions.status;
    let ui_handles = actions.ui_handles;

    let management_dialog_for_budget_add = management_dialog.clone();
    let budgets_list_for_budget_add = budgets_list.clone();
    let budgets_forms_for_budget_add = Rc::clone(budgets_forms);
    let budgets_scroll_for_budget_add = budgets_scroll.clone();
    let status_for_budget_add = status.clone();
    let filter_entry_for_budget_add = filter_entry.clone();
    let advanced_autofill_for_budget_add = Rc::clone(&ui_handles.advanced_autofill);
    let ui_for_budget_add = Rc::clone(ui_handles);
    add_budget_button.connect_clicked(move |_| {
        show_new_budget_dialog(NewBudgetDialogRequest {
            parent: &management_dialog_for_budget_add,
            container: &budgets_list_for_budget_add,
            forms: &budgets_forms_for_budget_add,
            scrolled_window: &budgets_scroll_for_budget_add,
            status: &status_for_budget_add,
            filter_entry: &filter_entry_for_budget_add,
            advanced_autofill: &advanced_autofill_for_budget_add,
            advanced_features: ui_for_budget_add.advanced_features.get(),
        });
    });

    let management_dialog_for_budget_move = management_dialog.clone();
    let rules_forms_for_budget_move = Rc::clone(rules_forms);
    let budgets_forms_for_budget_move = Rc::clone(budgets_forms);
    let filter_entry_for_budget_move = filter_entry.clone();
    let status_for_budget_move = status.clone();
    let ui_for_budget_move = Rc::clone(ui_handles);
    move_budget_code_button.connect_clicked(move |_| {
        show_move_budget_code_dialog(
            &management_dialog_for_budget_move,
            &rules_forms_for_budget_move,
            &budgets_forms_for_budget_move,
            &filter_entry_for_budget_move,
            &status_for_budget_move,
            ui_for_budget_move.advanced_features.get(),
        );
    });

    let budgets_forms_for_real_income = Rc::clone(budgets_forms);
    let status_for_real_income = status.clone();
    use_real_income_button.connect_clicked(move |_| {
        let changed =
            set_budget_forms_income_basis(&budgets_forms_for_real_income.borrow(), "real");
        set_budget_bulk_status(
            &status_for_real_income,
            changed,
            0,
            "budget(s) set to real income basis",
        );
    });

    let budgets_forms_for_planned_income = Rc::clone(budgets_forms);
    let status_for_planned_income = status.clone();
    use_planned_income_button.connect_clicked(move |_| {
        let changed =
            set_budget_forms_income_basis(&budgets_forms_for_planned_income.borrow(), "planned");
        set_budget_bulk_status(
            &status_for_planned_income,
            changed,
            0,
            "budget(s) set to planned income basis",
        );
    });

    let budgets_forms_for_monthly_values = Rc::clone(budgets_forms);
    let status_for_monthly_values = status.clone();
    use_monthly_values_button.connect_clicked(move |_| {
        let result = set_budget_forms_value_period(
            &budgets_forms_for_monthly_values.borrow(),
            BudgetValuePeriod::Monthly,
        );
        set_budget_bulk_status(
            &status_for_monthly_values,
            result.changed,
            result.skipped,
            "budget(s) converted to monthly values",
        );
    });

    let budgets_forms_for_yearly_values = Rc::clone(budgets_forms);
    let status_for_yearly_values = status.clone();
    use_yearly_values_button.connect_clicked(move |_| {
        let result = set_budget_forms_value_period(
            &budgets_forms_for_yearly_values.borrow(),
            BudgetValuePeriod::Yearly,
        );
        set_budget_bulk_status(
            &status_for_yearly_values,
            result.changed,
            result.skipped,
            "budget(s) converted to yearly values",
        );
    });
}

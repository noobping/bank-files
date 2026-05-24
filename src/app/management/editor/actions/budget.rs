use super::budget_bulk::{
    set_budget_bulk_status, set_budget_forms_income_basis, set_budget_forms_value_period,
    BudgetValuePeriod,
};
use super::budget_move::show_move_budget_code_dialog;
use super::*;

pub(super) fn connect_budget_actions(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog;
    let move_budget_code_action = actions.move_budget_code_action.clone();
    let use_real_income_action = actions.use_real_income_action.clone();
    let use_planned_income_action = actions.use_planned_income_action.clone();
    let use_monthly_values_action = actions.use_monthly_values_action.clone();
    let use_yearly_values_action = actions.use_yearly_values_action.clone();
    let filter_entry = actions.filter_entry;
    let rules_forms = actions.rules_forms;
    let budgets_forms = actions.budgets_forms;
    let status = actions.status;
    let ui_handles = actions.ui_handles;

    let management_dialog_for_budget_move = management_dialog.clone();
    let rules_forms_for_budget_move = Rc::clone(rules_forms);
    let budgets_forms_for_budget_move = Rc::clone(budgets_forms);
    let filter_entry_for_budget_move = filter_entry.clone();
    let status_for_budget_move = status.clone();
    let ui_for_budget_move = Rc::clone(ui_handles);
    move_budget_code_action.connect_activate(move |_, _| {
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
    use_real_income_action.connect_activate(move |_, _| {
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
    use_planned_income_action.connect_activate(move |_, _| {
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
    use_monthly_values_action.connect_activate(move |_, _| {
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
    use_yearly_values_action.connect_activate(move |_, _| {
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

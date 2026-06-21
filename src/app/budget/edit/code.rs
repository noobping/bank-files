use super::*;

pub(in crate::app::budget::edit) struct BudgetCodeSaveRequest<'a> {
    pub(in crate::app::budget::edit) creating_budget: bool,
    pub(in crate::app::budget::edit) advanced_features: bool,
    pub(in crate::app::budget::edit) configured_code: &'a str,
    pub(in crate::app::budget::edit) category: &'a str,
    pub(in crate::app::budget::edit) direction: &'a str,
    pub(in crate::app::budget::edit) code_input: Option<&'a gtk::ComboBoxText>,
    pub(in crate::app::budget::edit) state: &'a Rc<RefCell<AppData>>,
}

pub(in crate::app::budget::edit) fn budget_code_for_save(
    request: BudgetCodeSaveRequest<'_>,
) -> Option<String> {
    if !request.creating_budget {
        return Some(request.configured_code.to_string());
    }

    if request.advanced_features {
        return request
            .code_input
            .map(ui::combo_text)
            .filter(|code| !code.trim().is_empty());
    }

    let existing_codes = request
        .state
        .borrow()
        .budgets
        .iter()
        .map(|budget| budget.code.clone())
        .collect::<Vec<_>>();
    Some(transfer_budget::code_for_new_budget(
        request.category,
        request.direction,
        &existing_codes,
    ))
}

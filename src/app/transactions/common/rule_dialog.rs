use super::rule_helpers::editable_rule_for_transaction;
use super::*;

pub(super) fn show_transaction_rule_dialog(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    direction_override: Option<&str>,
) {
    let initial = {
        let data = state.borrow();
        editable_rule_for_transaction(tx, direction_override, &data.budgets)
    };
    show_rule_enqueue_dialog(
        initial,
        RuleDialogSpec {
            subtitle: "Create a categorization rule from this transaction.",
            content_width: 680,
            field_options: TRANSACTION_RULE_FIELD_OPTIONS,
            search_values: transaction_rule_search_values(tx),
        },
        state,
        ui_handles,
    );
}

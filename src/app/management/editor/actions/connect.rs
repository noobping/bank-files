use super::*;

pub(in crate::app::management::editor) fn connect_management_dialog_actions(
    actions: ManagementDialogActions<'_>,
) {
    page_actions::connect_management_page_actions(
        actions.page_actions_button,
        actions.stack,
        actions.rules_forms,
        actions.budgets_forms,
        actions.aliases_forms,
        actions.status,
        actions.ui_handles,
    );
    add::connect_add_actions(&actions);
    rule_bulk::connect_rule_bulk_actions(&actions);
    budget::connect_budget_actions(&actions);
    alias_search::connect_alias_and_search_actions(&actions);
    save::connect_save_action(&actions);
}

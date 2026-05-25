use super::*;

mod add;
mod alias_search;
mod budget;
mod budget_bulk;
mod budget_move;
mod config_backup;
mod connect;
mod page_actions;
mod rule_bulk;
mod save;

#[cfg(test)]
mod tests;

const MANAGEMENT_FORM_DIALOG_WIDTH: i32 = 620;

pub(in crate::app::management::editor) use connect::connect_management_dialog_actions;

pub(in crate::app::management::editor) struct ManagementDialogActions<'a> {
    pub(in crate::app::management::editor) management_dialog: &'a adw::Dialog,
    pub(in crate::app::management::editor) add_button: &'a gtk::Button,
    pub(in crate::app::management::editor) add_rule_row: &'a adw::ActionRow,
    pub(in crate::app::management::editor) add_budget_row: &'a adw::ActionRow,
    pub(in crate::app::management::editor) add_alias_row: &'a adw::ActionRow,
    pub(in crate::app::management::editor) group_rules_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) combine_rules_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) clean_orphaned_rules_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) move_budget_code_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) add_planned_income_budget_action:
        &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) add_transfer_budget_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) add_refunding_budget_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) add_refunded_budget_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) use_real_income_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) use_planned_income_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) use_monthly_values_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) use_yearly_values_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) back_up_configuration_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) restore_latest_backup_action: &'a gtk::gio::SimpleAction,
    pub(in crate::app::management::editor) save_button: &'a gtk::Button,
    pub(in crate::app::management::editor) page_actions_button: &'a gtk::MenuButton,
    pub(in crate::app::management::editor) stack: &'a adw::ViewStack,
    pub(in crate::app::management::editor) filter_entry: &'a gtk::SearchEntry,
    pub(in crate::app::management::editor) filter_search_bar: &'a gtk::SearchBar,
    pub(in crate::app::management::editor) rules_list: &'a gtk::Box,
    pub(in crate::app::management::editor) rules_forms: &'a Rc<RefCell<Vec<RuleForm>>>,
    pub(in crate::app::management::editor) rules_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) budgets_list: &'a gtk::Box,
    pub(in crate::app::management::editor) budgets_forms: &'a Rc<RefCell<Vec<BudgetForm>>>,
    pub(in crate::app::management::editor) budgets_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) aliases_list: &'a gtk::Box,
    pub(in crate::app::management::editor) aliases_forms: &'a Rc<RefCell<Vec<AliasForm>>>,
    pub(in crate::app::management::editor) aliases_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) status: &'a gtk::Label,
    pub(in crate::app::management::editor) dialog_closed: Rc<Cell<bool>>,
    pub(in crate::app::management::editor) save_running: Rc<Cell<bool>>,
    pub(in crate::app::management::editor) finish_management_dialog: Rc<dyn Fn()>,
    pub(in crate::app::management::editor) state: &'a Rc<RefCell<AppData>>,
    pub(in crate::app::management::editor) ui_handles: &'a Rc<UiHandles>,
}

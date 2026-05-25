use super::super::*;
use super::setup::{finish_management_dialog_setup, ManagementDialogSetup};
use super::shell::{build_management_dialog_shell, ManagementDialogShell};
use super::sizing::management_dialog_content_size;
use super::*;

pub(in crate::app) fn show_management_dialog(
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    initial_tab: &str,
) -> bool {
    if !try_begin_config_operation(ui_handles, "Rules, budgets, and fields is already open.") {
        return false;
    }

    let finish_called = Rc::new(Cell::new(false));
    let ui_for_finish = Rc::clone(ui_handles);
    let finish_management_dialog: Rc<dyn Fn()> = Rc::new(move || {
        if finish_called.replace(true) {
            return;
        }
        finish_config_operation(&ui_for_finish);
    });
    let dialog_closed = Rc::new(Cell::new(false));
    let save_running = Rc::new(Cell::new(false));
    let advanced_features = ui_handles.advanced_features.get();

    let filter_placeholder = "Filter rules, budgets, and field names";
    let ManagementDialogShell {
        root,
        add_button,
        add_rule_row,
        add_budget_row,
        add_alias_row,
        group_rules_action,
        combine_rules_action,
        clean_orphaned_rules_action,
        rule_bulk_menu_button,
        move_budget_code_action,
        add_planned_income_budget_action,
        add_transfer_budget_action,
        add_refunding_budget_action,
        add_refunded_budget_action,
        use_real_income_action,
        use_planned_income_action,
        use_monthly_values_action,
        use_yearly_values_action,
        budget_bulk_menu_button,
        back_up_configuration_action,
        restore_latest_backup_action,
        save_button,
        filter_entry,
        filter_search_bar,
        stack,
        switcher,
        switcher_bar,
        rules_list,
        rules_scroll,
        budgets_list,
        budgets_scroll,
        aliases_list,
        aliases_scroll,
    } = build_management_dialog_shell(filter_placeholder, advanced_features);

    let rules_forms: Rc<RefCell<Vec<RuleForm>>> = Rc::new(RefCell::new(Vec::new()));
    let budgets_forms: Rc<RefCell<Vec<BudgetForm>>> = Rc::new(RefCell::new(Vec::new()));
    let aliases_forms: Rc<RefCell<Vec<AliasForm>>> = Rc::new(RefCell::new(Vec::new()));

    let (initial_tab, initial_filter) = management_initial_tab(initial_tab);
    stack.set_visible_child_name(initial_tab);

    let status_bar = build_status_bar();
    connect_embedded_status_bar(window, &status_bar, Rc::clone(&ui_handles.status_autohide));
    status_bar.page_actions_button.set_sensitive(false);
    let status_handle = StatusHandle::from_status_bar(&status_bar);
    status_handle.set_text(&tr("Loading management data..."));
    status_handle.set_loading(true);
    root.append(&status_bar.container);
    let status = status_bar.label.clone();

    let (content_width, content_height) = management_dialog_content_size(window);
    let management_title = "Rules, budgets, and fields";
    let management_dialog = ui::content_dialog(tr(management_title), &root)
        .width_request(MANAGEMENT_DIALOG_MIN_WIDTH)
        .height_request(MANAGEMENT_DIALOG_MIN_HEIGHT)
        .content_width(content_width)
        .content_height(content_height)
        .build();
    let dialog_closed_for_closed = Rc::clone(&dialog_closed);
    let save_running_for_closed = Rc::clone(&save_running);
    let finish_for_closed = Rc::clone(&finish_management_dialog);
    let ui_for_closed = Rc::clone(ui_handles);
    management_dialog.connect_closed(move |_| {
        dialog_closed_for_closed.set(true);
        ui_for_closed.management_search.borrow_mut().take();
        if !save_running_for_closed.get() {
            finish_for_closed();
        }
    });
    *ui_handles.management_search.borrow_mut() = Some(SearchToggleHandle {
        search_bar: filter_search_bar.clone(),
        search_entry: filter_entry.clone(),
    });
    ui::bind_search_bar(
        &management_dialog,
        &management_dialog,
        &filter_search_bar,
        &filter_entry,
    );
    add_responsive_switcher_for_dialog(&management_dialog, &switcher, &switcher_bar);

    finish_management_dialog_setup(ManagementDialogSetup {
        window,
        management_dialog: &management_dialog,
        add_button: &add_button,
        add_rule_row: &add_rule_row,
        add_budget_row: &add_budget_row,
        add_alias_row: &add_alias_row,
        group_rules_action: &group_rules_action,
        combine_rules_action: &combine_rules_action,
        clean_orphaned_rules_action: &clean_orphaned_rules_action,
        move_budget_code_action: &move_budget_code_action,
        add_planned_income_budget_action: &add_planned_income_budget_action,
        add_transfer_budget_action: &add_transfer_budget_action,
        add_refunding_budget_action: &add_refunding_budget_action,
        add_refunded_budget_action: &add_refunded_budget_action,
        use_real_income_action: &use_real_income_action,
        use_planned_income_action: &use_planned_income_action,
        use_monthly_values_action: &use_monthly_values_action,
        use_yearly_values_action: &use_yearly_values_action,
        back_up_configuration_action: &back_up_configuration_action,
        restore_latest_backup_action: &restore_latest_backup_action,
        rule_bulk_menu_button: &rule_bulk_menu_button,
        budget_bulk_menu_button: &budget_bulk_menu_button,
        save_button: &save_button,
        page_actions_button: &status_bar.page_actions_button,
        stack: &stack,
        filter_entry: &filter_entry,
        filter_search_bar: &filter_search_bar,
        rules_list,
        rules_forms,
        rules_scroll: &rules_scroll,
        budgets_list,
        budgets_forms,
        budgets_scroll: &budgets_scroll,
        aliases_list,
        aliases_forms,
        aliases_scroll: &aliases_scroll,
        status,
        dialog_closed,
        save_running,
        finish_management_dialog,
        initial_filter,
        advanced_features,
        state,
        ui_handles,
        status_handle,
    });
    true
}

fn management_initial_tab(initial_tab: &str) -> (&'static str, Option<&'static str>) {
    match initial_tab {
        "active-rules" => ("rules", Some("active")),
        "rules" => ("rules", None),
        "budgets" => ("budgets", None),
        "aliases" => ("aliases", None),
        _ => ("budgets", None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_rules_initial_tab_opens_rules_with_active_filter() {
        assert_eq!(
            management_initial_tab("active-rules"),
            ("rules", Some("active"))
        );
        assert_eq!(management_initial_tab("rules"), ("rules", None));
        assert_eq!(management_initial_tab("unknown"), ("budgets", None));
    }
}

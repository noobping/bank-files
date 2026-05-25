use super::super::*;
use super::header::{
    connect_header_action_visibility, insert_menu_actions, HeaderActionWidgets,
    BUDGET_ACTION_NAMESPACE, CONFIG_ACTION_NAMESPACE, RULE_ACTION_NAMESPACE,
};

pub(super) struct ManagementDialogShell {
    pub(super) root: gtk::Box,
    pub(super) add_button: gtk::Button,
    pub(super) add_rule_row: adw::ActionRow,
    pub(super) add_budget_row: adw::ActionRow,
    pub(super) add_alias_row: adw::ActionRow,
    pub(super) group_rules_action: gtk::gio::SimpleAction,
    pub(super) combine_rules_action: gtk::gio::SimpleAction,
    pub(super) clean_orphaned_rules_action: gtk::gio::SimpleAction,
    pub(super) rule_bulk_menu_button: gtk::MenuButton,
    pub(super) move_budget_code_action: gtk::gio::SimpleAction,
    pub(super) use_real_income_action: gtk::gio::SimpleAction,
    pub(super) use_planned_income_action: gtk::gio::SimpleAction,
    pub(super) use_monthly_values_action: gtk::gio::SimpleAction,
    pub(super) use_yearly_values_action: gtk::gio::SimpleAction,
    pub(super) budget_bulk_menu_button: gtk::MenuButton,
    pub(super) back_up_configuration_action: gtk::gio::SimpleAction,
    pub(super) restore_latest_backup_action: gtk::gio::SimpleAction,
    pub(super) save_button: gtk::Button,
    pub(super) filter_entry: gtk::SearchEntry,
    pub(super) filter_search_bar: gtk::SearchBar,
    pub(super) stack: adw::ViewStack,
    pub(super) switcher: adw::ViewSwitcher,
    pub(super) switcher_bar: adw::ViewSwitcherBar,
    pub(super) rules_list: gtk::Box,
    pub(super) rules_scroll: gtk::ScrolledWindow,
    pub(super) budgets_list: gtk::Box,
    pub(super) budgets_scroll: gtk::ScrolledWindow,
    pub(super) aliases_list: gtk::Box,
    pub(super) aliases_scroll: gtk::ScrolledWindow,
}

pub(super) fn build_management_dialog_shell(
    filter_placeholder: &str,
    advanced_features: bool,
) -> ManagementDialogShell {
    const RESOURCE: &str = "management-dialog.ui";

    let builder = ui::builder_from_resource(RESOURCE);
    let root = ui::builder_object::<gtk::Box>(&builder, "management_root", RESOURCE);
    let header = ui::builder_object::<adw::HeaderBar>(&builder, "management_header", RESOURCE);
    let switcher =
        ui::builder_object::<adw::ViewSwitcher>(&builder, "management_switcher", RESOURCE);
    let switcher_bar =
        ui::builder_object::<adw::ViewSwitcherBar>(&builder, "management_switcher_bar", RESOURCE);
    let stack = ui::builder_object::<adw::ViewStack>(&builder, "management_stack", RESOURCE);
    let add_button = ui::builder_object::<gtk::Button>(&builder, "management_add_button", RESOURCE);
    let save_button =
        ui::builder_object::<gtk::Button>(&builder, "management_save_button", RESOURCE);
    let filter_search_bar =
        ui::builder_object::<gtk::SearchBar>(&builder, "management_filter_search_bar", RESOURCE);
    let filter_entry =
        ui::builder_object::<gtk::SearchEntry>(&builder, "management_filter_entry", RESOURCE);

    let add_rule_row =
        ui::builder_object::<adw::ActionRow>(&builder, "management_add_rule_row", RESOURCE);
    let rules_list = ui::builder_object::<gtk::Box>(&builder, "management_rules_list", RESOURCE);
    let rules_scroll =
        ui::builder_object::<gtk::ScrolledWindow>(&builder, "management_rules_scroll", RESOURCE);
    let rule_bulk_menu_button = ui::builder_object::<gtk::MenuButton>(
        &builder,
        "management_rule_bulk_menu_button",
        RESOURCE,
    );
    let rule_bulk_menu =
        ui::builder_object::<gtk::gio::Menu>(&builder, "management_rule_bulk_menu", RESOURCE);

    let add_budget_row =
        ui::builder_object::<adw::ActionRow>(&builder, "management_add_budget_row", RESOURCE);
    let budgets_title =
        ui::builder_object::<gtk::Label>(&builder, "management_budgets_title", RESOURCE);
    let budgets_subtitle =
        ui::builder_object::<gtk::Label>(&builder, "management_budgets_subtitle", RESOURCE);
    let budgets_loading_label =
        ui::builder_object::<gtk::Label>(&builder, "management_budgets_loading_label", RESOURCE);
    let budgets_list =
        ui::builder_object::<gtk::Box>(&builder, "management_budgets_list", RESOURCE);
    let budgets_scroll =
        ui::builder_object::<gtk::ScrolledWindow>(&builder, "management_budgets_scroll", RESOURCE);
    let budget_bulk_menu_button = ui::builder_object::<gtk::MenuButton>(
        &builder,
        "management_budget_bulk_menu_button",
        RESOURCE,
    );
    let budget_bulk_menu =
        ui::builder_object::<gtk::gio::Menu>(&builder, "management_budget_bulk_menu", RESOURCE);
    let simple_budget_bulk_menu = ui::builder_object::<gtk::gio::Menu>(
        &builder,
        "management_simple_budget_bulk_menu",
        RESOURCE,
    );
    let config_menu =
        ui::builder_object::<gtk::gio::Menu>(&builder, "management_config_menu", RESOURCE);

    let add_alias_row =
        ui::builder_object::<adw::ActionRow>(&builder, "management_add_alias_row", RESOURCE);
    let aliases_list =
        ui::builder_object::<gtk::Box>(&builder, "management_aliases_list", RESOURCE);
    let aliases_scroll =
        ui::builder_object::<gtk::ScrolledWindow>(&builder, "management_aliases_scroll", RESOURCE);

    let group_rules_action = gtk::gio::SimpleAction::new("group-rules", None);
    let combine_rules_action = gtk::gio::SimpleAction::new("combine-rules", None);
    let clean_orphaned_rules_action = gtk::gio::SimpleAction::new("clean-orphaned-rules", None);
    let move_budget_code_action = gtk::gio::SimpleAction::new("move-budget-code", None);
    let use_real_income_action = gtk::gio::SimpleAction::new("use-real-income", None);
    let use_planned_income_action = gtk::gio::SimpleAction::new("use-planned-income", None);
    let use_monthly_values_action = gtk::gio::SimpleAction::new("use-monthly-values", None);
    let use_yearly_values_action = gtk::gio::SimpleAction::new("use-yearly-values", None);
    let back_up_configuration_action = gtk::gio::SimpleAction::new("back-up-configuration", None);
    let restore_latest_backup_action = gtk::gio::SimpleAction::new("restore-latest-backup", None);
    restore_latest_backup_action.set_enabled(data::configuration_archive_exists().unwrap_or(false));

    let config_action_group = gtk::gio::SimpleActionGroup::new();
    config_action_group.add_action(&back_up_configuration_action);
    config_action_group.add_action(&restore_latest_backup_action);
    root.insert_action_group(CONFIG_ACTION_NAMESPACE, Some(&config_action_group));

    insert_menu_actions(
        &rule_bulk_menu_button,
        RULE_ACTION_NAMESPACE,
        &[
            &group_rules_action,
            &combine_rules_action,
            &clean_orphaned_rules_action,
        ],
    );
    rule_bulk_menu_button.set_menu_model(Some(&rule_bulk_menu));
    insert_menu_actions(
        &budget_bulk_menu_button,
        BUDGET_ACTION_NAMESPACE,
        &[
            &move_budget_code_action,
            &use_real_income_action,
            &use_planned_income_action,
            &use_monthly_values_action,
            &use_yearly_values_action,
        ],
    );
    budget_bulk_menu_button.set_menu_model(Some(&budget_bulk_menu));

    switcher.set_stack(Some(&stack));
    switcher_bar.set_stack(Some(&stack));
    header.set_title_widget(Some(&switcher));

    save_button.set_tooltip_text(Some(&tr("Save rules, budgets, and field names")));
    filter_entry.set_placeholder_text(Some(&tr(filter_placeholder)));

    configure_management_page_text(
        advanced_features,
        &budgets_title,
        &budgets_subtitle,
        &budgets_loading_label,
        &add_budget_row,
    );
    rule_bulk_menu_button.set_tooltip_text(Some(&tr("Rule actions")));
    budget_bulk_menu_button.set_tooltip_text(Some(&tr("Budget actions")));
    connect_header_action_visibility(
        HeaderActionWidgets {
            stack: stack.clone(),
            add_button: add_button.clone(),
            rule_bulk_menu_button: rule_bulk_menu_button.clone(),
            budget_bulk_menu_button: budget_bulk_menu_button.clone(),
            rule_bulk_menu: rule_bulk_menu.clone(),
            budget_bulk_menu: budget_bulk_menu.clone(),
            simple_budget_bulk_menu,
            config_menu,
        },
        advanced_features,
    );

    ManagementDialogShell {
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
    }
}

fn configure_management_page_text(
    advanced_features: bool,
    budgets_title: &gtk::Label,
    budgets_subtitle: &gtk::Label,
    budgets_loading_label: &gtk::Label,
    add_budget_row: &adw::ActionRow,
) {
    let (title, description, add_label, add_tooltip) = if advanced_features {
        (
            "Budget Codes",
            "Use fixed amounts or percentages; choose real or planned income for percentages.",
            "New Budget",
            "Create a new budget",
        )
    } else {
        (
            "Budgets",
            "Use categories with fixed amounts or percentages.",
            "New Category",
            "Create a new category with monthly or yearly amounts",
        )
    };

    budgets_title.set_text(&tr(title));
    budgets_subtitle.set_text(&tr(description));
    budgets_loading_label.set_text(&tr("Loading budgets..."));
    add_budget_row.set_title(&tr(add_label));
    add_budget_row.set_tooltip_text(Some(&tr(add_tooltip)));
}

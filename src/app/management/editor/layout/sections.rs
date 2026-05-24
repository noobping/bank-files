use super::super::*;

const BUDGET_ACTION_NAMESPACE: &str = "management-budgets";
const RULE_ACTION_NAMESPACE: &str = "management-rules";

pub(super) struct RuleActionSection {
    pub(super) container: gtk::Box,
    pub(super) add_rule_button: gtk::Button,
    pub(super) group_rules_action: gtk::gio::SimpleAction,
    pub(super) combine_rules_action: gtk::gio::SimpleAction,
    pub(super) bulk_menu_button: gtk::MenuButton,
}

pub(super) struct BudgetActionSection {
    pub(super) container: gtk::Box,
    pub(super) add_budget_button: gtk::Button,
    pub(super) move_budget_code_action: gtk::gio::SimpleAction,
    pub(super) use_real_income_action: gtk::gio::SimpleAction,
    pub(super) use_planned_income_action: gtk::gio::SimpleAction,
    pub(super) use_monthly_values_action: gtk::gio::SimpleAction,
    pub(super) use_yearly_values_action: gtk::gio::SimpleAction,
    pub(super) bulk_menu_button: gtk::MenuButton,
}

pub(super) fn build_rule_action_section() -> RuleActionSection {
    let add_rule_button =
        ui::plain_text_icon_button("list-add-symbolic", "New Rule", "Create a new rule");
    let group_rules_action = gtk::gio::SimpleAction::new("group-rules", None);
    let combine_rules_action = gtk::gio::SimpleAction::new("combine-rules", None);
    let bulk_menu_button = bulk_menu_button(
        RULE_ACTION_NAMESPACE,
        "Rule actions",
        &[
            ("group-rules", "Group Compatible Rules", &group_rules_action),
            (
                "combine-rules",
                "Combine Compatible Rules",
                &combine_rules_action,
            ),
        ],
    );

    let container = ui::linked_button_group();
    container.append(&add_rule_button);
    container.append(&bulk_menu_button);

    RuleActionSection {
        container,
        add_rule_button,
        group_rules_action,
        combine_rules_action,
        bulk_menu_button,
    }
}

pub(super) fn build_budget_action_section(advanced_features: bool) -> BudgetActionSection {
    let add_budget_button = ui::plain_text_icon_button(
        "list-add-symbolic",
        if advanced_features {
            "New Budget"
        } else {
            "New Category"
        },
        if advanced_features {
            "Create a new budget"
        } else {
            "Create a new category with monthly or yearly amounts"
        },
    );
    let move_budget_code_action = gtk::gio::SimpleAction::new("move-budget-code", None);
    let use_real_income_action = gtk::gio::SimpleAction::new("use-real-income", None);
    let use_planned_income_action = gtk::gio::SimpleAction::new("use-planned-income", None);
    let use_monthly_values_action = gtk::gio::SimpleAction::new("use-monthly-values", None);
    let use_yearly_values_action = gtk::gio::SimpleAction::new("use-yearly-values", None);
    let bulk_menu_button = bulk_menu_button(
        BUDGET_ACTION_NAMESPACE,
        "Budget actions",
        &[
            (
                "move-budget-code",
                "Move Budget Code",
                &move_budget_code_action,
            ),
            (
                "use-real-income",
                "Use Real Income",
                &use_real_income_action,
            ),
            (
                "use-planned-income",
                "Use Planned Income",
                &use_planned_income_action,
            ),
            (
                "use-monthly-values",
                "Use Monthly Values",
                &use_monthly_values_action,
            ),
            (
                "use-yearly-values",
                "Use Yearly Values",
                &use_yearly_values_action,
            ),
        ],
    );
    bulk_menu_button.set_visible(advanced_features);

    let container = ui::linked_button_group();
    container.append(&add_budget_button);
    container.append(&bulk_menu_button);

    BudgetActionSection {
        container,
        add_budget_button,
        move_budget_code_action,
        use_real_income_action,
        use_planned_income_action,
        use_monthly_values_action,
        use_yearly_values_action,
        bulk_menu_button,
    }
}

fn bulk_menu_button(
    action_namespace: &str,
    tooltip: &str,
    actions: &[(&str, &str, &gtk::gio::SimpleAction)],
) -> gtk::MenuButton {
    let menu = gtk::gio::Menu::new();
    let action_group = gtk::gio::SimpleActionGroup::new();
    for (name, label, action) in actions {
        action_group.add_action(*action);
        menu.append(
            Some(&tr(label)),
            Some(&format!("{action_namespace}.{name}")),
        );
    }

    let button = gtk::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text(tr(tooltip))
        .build();
    button.insert_action_group(action_namespace, Some(&action_group));
    button.set_menu_model(Some(&menu));
    button
}

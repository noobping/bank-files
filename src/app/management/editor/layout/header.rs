use super::super::*;

pub(super) const RULE_ACTION_NAMESPACE: &str = "management-rules";
pub(super) const BUDGET_ACTION_NAMESPACE: &str = "management-budgets";
pub(super) const CONFIG_ACTION_NAMESPACE: &str = "management-config";

#[derive(Clone)]
pub(super) struct HeaderActionWidgets {
    pub(super) stack: adw::ViewStack,
    pub(super) add_button: gtk::Button,
    pub(super) rule_bulk_menu_button: gtk::MenuButton,
    pub(super) budget_bulk_menu_button: gtk::MenuButton,
    pub(super) rule_bulk_menu: gtk::gio::Menu,
    pub(super) budget_bulk_menu: gtk::gio::Menu,
    pub(super) config_menu: gtk::gio::Menu,
}

pub(super) fn connect_header_action_visibility(
    widgets: HeaderActionWidgets,
    advanced_features: bool,
) {
    update_header_action_visibility(&widgets, advanced_features);

    let widgets_for_stack = widgets.clone();
    widgets.stack.connect_visible_child_name_notify(move |_| {
        update_header_action_visibility(&widgets_for_stack, advanced_features);
    });
}

pub(super) fn insert_menu_actions(
    menu_button: &gtk::MenuButton,
    namespace: &str,
    actions: &[&gtk::gio::SimpleAction],
) {
    let action_group = gtk::gio::SimpleActionGroup::new();
    for action in actions {
        action_group.add_action(*action);
    }
    menu_button.insert_action_group(namespace, Some(&action_group));
}

fn update_header_action_visibility(widgets: &HeaderActionWidgets, advanced_features: bool) {
    match widgets.stack.visible_child_name().as_deref() {
        Some("rules") => show_rule_menu(widgets),
        Some("aliases") => show_config_menu(widgets),
        _ => show_budget_menu(widgets, advanced_features),
    }
}

fn show_rule_menu(widgets: &HeaderActionWidgets) {
    widgets
        .add_button
        .set_tooltip_text(Some(&tr("Create a new rule")));
    widgets
        .rule_bulk_menu_button
        .set_menu_model(Some(&widgets.rule_bulk_menu));
    widgets
        .rule_bulk_menu_button
        .set_tooltip_text(Some(&tr("Rule actions")));
    widgets.rule_bulk_menu_button.set_visible(true);
    widgets.budget_bulk_menu_button.set_visible(false);
}

fn show_config_menu(widgets: &HeaderActionWidgets) {
    widgets
        .add_button
        .set_tooltip_text(Some(&tr("Create a new field name")));
    widgets.rule_bulk_menu_button.set_visible(false);
    widgets
        .budget_bulk_menu_button
        .set_menu_model(Some(&widgets.config_menu));
    widgets
        .budget_bulk_menu_button
        .set_tooltip_text(Some(&tr("Configuration actions")));
    widgets.budget_bulk_menu_button.set_visible(true);
}

fn show_budget_menu(widgets: &HeaderActionWidgets, advanced_features: bool) {
    widgets
        .add_button
        .set_tooltip_text(Some(&tr(if advanced_features {
            "Create a new budget"
        } else {
            "Create a new category with monthly or yearly amounts"
        })));
    widgets.rule_bulk_menu_button.set_visible(false);

    if advanced_features {
        widgets
            .budget_bulk_menu_button
            .set_menu_model(Some(&widgets.budget_bulk_menu));
        widgets
            .budget_bulk_menu_button
            .set_tooltip_text(Some(&tr("Budget actions")));
    } else {
        widgets
            .budget_bulk_menu_button
            .set_menu_model(Some(&widgets.config_menu));
        widgets
            .budget_bulk_menu_button
            .set_tooltip_text(Some(&tr("Configuration actions")));
    }
    widgets.budget_bulk_menu_button.set_visible(true);
}

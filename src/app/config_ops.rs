use super::*;

#[derive(Clone)]
pub(in crate::app) struct ConfigWidget {
    widget: gtk::Widget,
    base_sensitive: bool,
    base_visible: bool,
    was_rooted: Rc<Cell<bool>>,
}

pub(in crate::app) fn config_operation_is_active(
    ui_handles: &Rc<UiHandles>,
    busy_message: &str,
) -> bool {
    if ui_handles.management_dialog_active.get() {
        show_status(ui_handles, busy_message);
        true
    } else {
        false
    }
}

pub(in crate::app) fn try_begin_config_operation(
    ui_handles: &Rc<UiHandles>,
    busy_message: &str,
) -> bool {
    if !ui_handles.storage_capabilities.borrow().config_writable {
        show_status(
            ui_handles,
            ui_handles
                .storage_capabilities
                .borrow()
                .config_write_reason(),
        );
        return false;
    }
    if ui_handles.management_dialog_active.get() {
        show_status(ui_handles, busy_message);
        return false;
    }

    ui_handles.management_dialog_active.set(true);
    refresh_write_actions(ui_handles.as_ref());
    true
}

pub(in crate::app) fn finish_config_operation(ui_handles: &Rc<UiHandles>) {
    ui_handles.management_dialog_active.set(false);
    refresh_write_actions(ui_handles.as_ref());
}

pub(in crate::app) fn save_rule_in_background(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    rule: EditableRule,
    ensure_budget: bool,
) -> bool {
    if !try_begin_config_operation(ui_handles, "Another edit or save is already running.") {
        return false;
    }

    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui_handles.as_ref());
    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    show_status(ui_handles, "Saving rule...");
    begin_background_operation(ui_handles.as_ref());
    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            apply_rule_config_change(rule, ensure_budget)?;
            let new_data = data::load_app_data_with_config_cleanup(mode, auto_clean_config, scope)?;
            anyhow::Ok(new_data)
        });

        match task.await {
            Ok(Ok(new_data)) => {
                *state_for_save.borrow_mut() = new_data;
                render_views(&state_for_save.borrow(), &ui_for_save, &state_for_save);
                show_status(&ui_for_save, "Rule saved");
            }
            Ok(Err(error)) => show_status(
                &ui_for_save,
                &trf(
                    "Could not save rule: {error}",
                    &[("error", format!("{error:#}"))],
                ),
            ),
            Err(_) => show_status(
                &ui_for_save,
                "Rule save canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_save.as_ref());
        finish_config_operation(&ui_for_save);
    });
    true
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) struct RuleConfigChange {
    pub(in crate::app) rule_replaced: bool,
    pub(in crate::app) budget_added: bool,
}

pub(in crate::app) fn apply_rule_config_change(
    rule: EditableRule,
    ensure_budget: bool,
) -> anyhow::Result<RuleConfigChange> {
    let mut rules = data::load_editable_rules()?;
    let mut budgets = if ensure_budget {
        data::load_editable_budgets()?
    } else {
        Vec::new()
    };
    let change = apply_rule_to_editable_config(&mut rules, &mut budgets, rule, ensure_budget);
    if change.budget_added {
        data::write_editable_budgets(&budgets)?;
    }
    data::write_editable_rules(&rules)?;
    Ok(change)
}

pub(in crate::app) fn apply_rule_to_editable_config(
    rules: &mut Vec<EditableRule>,
    budgets: &mut Vec<EditableBudget>,
    rule: EditableRule,
    ensure_budget: bool,
) -> RuleConfigChange {
    let budget_added = ensure_budget && ensure_budget_for_rule_in(budgets, &rule);
    let rule_replaced = upsert_rule_in(rules, rule);
    RuleConfigChange {
        rule_replaced,
        budget_added,
    }
}

fn upsert_rule_in(rules: &mut Vec<EditableRule>, rule: EditableRule) -> bool {
    if let Some(existing) = rules
        .iter_mut()
        .find(|existing| rule_matches_existing(existing, &rule))
    {
        *existing = rule;
        return true;
    }

    rules.push(rule);
    false
}

fn rule_matches_existing(existing: &EditableRule, rule: &EditableRule) -> bool {
    existing.field.trim() == rule.field.trim()
        && existing
            .search
            .trim()
            .eq_ignore_ascii_case(rule.search.trim())
        && existing.direction.trim() == rule.direction.trim()
}

fn ensure_budget_for_rule_in(budgets: &mut Vec<EditableBudget>, rule: &EditableRule) -> bool {
    let code = rule.budget_code.trim();
    if code.is_empty()
        || budgets
            .iter()
            .any(|budget| budget.code.trim().eq_ignore_ascii_case(code))
    {
        return false;
    }

    let direction = crate::model::BudgetDirection::parse(&rule.direction, code, &rule.category);
    budgets.push(EditableBudget {
        code: code.to_string(),
        category: rule.category.trim().to_string(),
        monthly_budget: "0".to_string(),
        yearly_budget: String::new(),
        direction: direction.as_str().to_string(),
        income_basis: "real".to_string(),
        notes: tr("Created from rule."),
    });
    true
}

pub(in crate::app) fn register_exclusive_config_widget<W: IsA<gtk::Widget>>(
    ui_handles: &Rc<UiHandles>,
    widget: &W,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    let base_sensitive = widget.is_sensitive();
    let base_visible = widget.is_visible();
    let was_rooted = widget.root().is_some();
    let item = ConfigWidget {
        widget,
        base_sensitive,
        base_visible,
        was_rooted: Rc::new(Cell::new(was_rooted)),
    };
    apply_registered_widget_state(
        ui_handles.as_ref(),
        &item,
        !config_actions_are_busy(ui_handles.as_ref()),
    );
    ui_handles.config_widgets.borrow_mut().push(item);
}

pub(in crate::app) fn register_config_widget<W: IsA<gtk::Widget>>(
    ui_handles: &Rc<UiHandles>,
    widget: &W,
) {
    register_exclusive_config_widget(ui_handles, widget);
}

pub(in crate::app) fn set_config_widget_base_sensitive<W: IsA<gtk::Widget>>(
    ui_handles: &Rc<UiHandles>,
    widget: &W,
    sensitive: bool,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    for item in ui_handles.config_widgets.borrow_mut().iter_mut() {
        if item.widget == widget {
            item.base_sensitive = sensitive;
        }
    }
    update_config_action_widgets(ui_handles.as_ref());
}

fn config_actions_are_busy(ui_handles: &UiHandles) -> bool {
    config_actions_are_busy_for(
        ui_handles.management_dialog_active.get(),
        ui_handles.loading_count.get(),
    )
}

fn config_actions_are_busy_for(config_operation_active: bool, loading_count: u32) -> bool {
    config_operation_active || loading_count > 0
}

pub(in crate::app) fn update_config_action_widgets(ui_handles: &UiHandles) {
    set_registered_widgets_sensitive(ui_handles, !config_actions_are_busy(ui_handles));
}

fn set_registered_widgets_sensitive(ui_handles: &UiHandles, sensitive: bool) {
    let mut widgets = ui_handles.config_widgets.borrow_mut();
    widgets.retain(config_widget_should_remain_registered);
    for item in widgets.iter() {
        apply_registered_widget_state(ui_handles, item, sensitive);
    }
}

fn config_widget_should_remain_registered(item: &ConfigWidget) -> bool {
    let rooted = item.widget.root().is_some();
    if rooted {
        item.was_rooted.set(true);
    }
    config_widget_registration_is_live(rooted, item.was_rooted.get())
}

fn config_widget_registration_is_live(rooted: bool, was_rooted: bool) -> bool {
    rooted || !was_rooted
}

fn apply_registered_widget_state(ui_handles: &UiHandles, item: &ConfigWidget, sensitive: bool) {
    let availability = config_write_availability(ui_handles);
    match &availability {
        ActionAvailability::Available => {
            item.widget.set_visible(item.base_visible);
            item.widget.set_sensitive(item.base_sensitive && sensitive);
            item.widget.set_tooltip_text(None);
        }
        ActionAvailability::Hidden => {
            apply_action_availability(&item.widget, &availability);
        }
        ActionAvailability::Disabled(_) => {
            apply_action_availability(&item.widget, &availability);
            item.widget.set_visible(item.base_visible);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(search: &str, code: &str) -> EditableRule {
        EditableRule {
            field: "counterparty".to_string(),
            search: search.to_string(),
            category: "Household".to_string(),
            budget_code: code.to_string(),
            direction: "expense".to_string(),
            ..EditableRule::new_default()
        }
    }

    fn budget(code: &str) -> EditableBudget {
        EditableBudget {
            code: code.to_string(),
            category: "Household".to_string(),
            direction: "expense".to_string(),
            ..EditableBudget::new_default()
        }
    }

    #[test]
    fn config_actions_are_busy_during_config_operation_or_loading() {
        assert!(!config_actions_are_busy_for(false, 0));
        assert!(config_actions_are_busy_for(true, 0));
        assert!(config_actions_are_busy_for(false, 1));
        assert!(config_actions_are_busy_for(true, 2));
    }

    #[test]
    fn unrooted_config_widgets_stay_registered_until_first_root() {
        assert!(config_widget_registration_is_live(false, false));
        assert!(config_widget_registration_is_live(true, false));
        assert!(!config_widget_registration_is_live(false, true));
    }

    #[test]
    fn queued_rule_upserts_existing_rule() {
        let mut rules = vec![rule("Market", "FOOD")];
        let mut budgets = vec![budget("FOOD")];
        let replacement = EditableRule {
            priority: 300,
            search: " market ".to_string(),
            notes: "replacement".to_string(),
            ..rule("market", "FOOD")
        };

        let change = apply_rule_to_editable_config(&mut rules, &mut budgets, replacement, true);

        assert!(change.rule_replaced);
        assert!(!change.budget_added);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].priority, 300);
        assert_eq!(rules[0].notes, "replacement");
    }

    #[test]
    fn queued_rule_creates_missing_budget_when_requested() {
        let mut rules = Vec::new();
        let mut budgets = Vec::new();

        let change =
            apply_rule_to_editable_config(&mut rules, &mut budgets, rule("Market", "FOOD"), true);

        assert!(!change.rule_replaced);
        assert!(change.budget_added);
        assert_eq!(rules.len(), 1);
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].code, "FOOD");
        assert_eq!(budgets[0].category, "Household");
        assert_eq!(budgets[0].direction, "expense");
    }

    #[test]
    fn queued_rule_does_not_create_budget_without_ensure_flag() {
        let mut rules = Vec::new();
        let mut budgets = Vec::new();

        let change =
            apply_rule_to_editable_config(&mut rules, &mut budgets, rule("Market", "FOOD"), false);

        assert!(!change.rule_replaced);
        assert!(!change.budget_added);
        assert_eq!(rules.len(), 1);
        assert!(budgets.is_empty());
    }
}

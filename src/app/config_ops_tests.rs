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
        special: String::new(),
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
fn queued_transfer_rule_creates_canonical_transfer_budget() {
    let mut rules = Vec::new();
    let mut budgets = Vec::new();
    let transfer_rule = EditableRule {
        category: "Internal transfers".to_string(),
        budget_code: "transfer".to_string(),
        direction: "expense".to_string(),
        ..rule("Savings", "transfer")
    };

    let change = apply_rule_to_editable_config(&mut rules, &mut budgets, transfer_rule, true);

    assert!(change.budget_added);
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].code, transfer_budget::BUDGET_CODE);
    assert_eq!(budgets[0].category, "Internal transfers");
    assert_eq!(budgets[0].direction, "transfer");
    assert_eq!(budgets[0].income_basis, "real");
}

#[test]
fn queued_refund_rule_creates_canonical_refund_budget() {
    let mut rules = Vec::new();
    let mut budgets = Vec::new();
    let refund_rule = EditableRule {
        category: "Refunded".to_string(),
        budget_code: "refunded".to_string(),
        direction: "income".to_string(),
        ..rule("Return", "refunded")
    };

    let change = apply_rule_to_editable_config(&mut rules, &mut budgets, refund_rule, true);

    assert!(change.budget_added);
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].code, crate::model::REFUNDED_BUDGET_CODE);
    assert_eq!(budgets[0].category, "Refunded");
    assert_eq!(budgets[0].direction, "income");
    assert_eq!(budgets[0].income_basis, "real");
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
